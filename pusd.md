# Technical Design Doc: `pallet-vaults` (part of the pUSD Protocol)

**Authors:** Leonardo Razovic, Raffael Huber, Luca Von Wyttenbach

**Status:** Writing it

**Related Links:** [Polkadot SDK PR TBA]


---

## 1. Purpose
The purpose of this document is to outline the technical implementation of `pallet-vaults`. This pallet serves as the "Collateralized Debt Position" (CDP) engine for the pUSD protocol, enabling users to lock native currency (DOT) to mint stablecoins (pUSD) while ensuring the system remains over-collateralized.

## 2. Background
This implementation mimics the **MakerDAO v1 (Single-Collateral SAI)** model but adapts the accounting for a Polkadot SDK native environment.

Unlike the standard Maker model where stability fees increase the debt (pUSD) owed, this implementation calculates fees in pUSD but **deducts them from the locked collateral (DOT)**. This design choice ensures that the pUSD supply remains strictly pegged to user actions (mint/burn) rather than inflating automatically via interest.

## 3. Terminology

* **Accrued Interest** — Accumulated stability fees owed by a vault, denominated in DOT. Stored in `vault.accrued_interest`. Reduces available collateral until paid.
* **Asset** — A fungible token managed by `pallet-assets`. In this protocol, refers to pUSD.
* **Available Collateral** — Collateral that can be withdrawn or used to satisfy collateralization requirements: `Available = HeldCollateral – AccruedInterest`.
* **Auction** — Mechanism for selling seized collateral for pUSD. Auction proceeds are burned to reduce system debt.
* **Bad Debt** — Unbacked pUSD recorded at the system level when interest or liquidation shortfalls exceed collateral. Always denominated in pUSD.
* **Collateral** — DOT locked under the `VaultDeposit` hold reason, backing a vault’s pUSD debt.
* **Collateral Asset** — The token accepted as collateral. Configured via `CollateralAssetId` (DOT).
* **Collateral Value** — Value of available collateral in USD: `CollateralValue = AvailableCollateral × Price`.
* **Collateralization Ratio (CR)** — Safety measure: `CR = (HeldCollateral – AccruedInterest) × Price / Debt`. Infinite if `Debt == 0`.
* **Debt** — pUSD owed by a vault (principal). Stored in `vault.debt`.
* **Debt Asset** — The stablecoin issued by the system, pUSD.
* **Held Collateral** — Total DOT locked under the `VaultDeposit` hold reason. Queried via `balance_on_hold`.
* **Hold Reason** — Enum describing why funds are locked:
  * `VaultDeposit` — collateral backing an active vault
  * `Seized` — collateral locked after liquidation and controlled by Auctions
* **Initial Collateralization Ratio** — Collateral ratio required when minting new debt. Must exceed the minimum ratio.
* **Interest (Stability Fee)** — Linear fee charged on outstanding debt. Calculated in pUSD first, converted to DOT using the USD price from the Oracle.
* **Last Fee Update** — Block at which interest was last accrued for a vault.
* **Liquidation** — Forced closure of an unsafe vault whose CR < minimum requirement.
* **Liquidation Penalty** — pUSD fee charged on vault debt during liquidation, converted to DOT and deducted from collateral.
* **MaximumDebt** — System-wide cap on total pUSD issuance (debt ceiling).
* **Minimum Collateralization Ratio (MCR)** — Minimum CR required to keep a vault safe. Falling below triggers liquidation.
* **MinimumDeposit** — Minimum DOT required to create a vault. Prevents dust vaults.
* **Principal** — The pUSD debt excluding interest. Equal to `vault.debt`.
* **Protocol Revenue** — DOT collected by the system from interest and liquidation penalties.
* **Repay** — Operation that burns pUSD (reducing debt) and transfers accrued interest in DOT to the Treasury.
* **Seized Collateral** — Remaining vault collateral after liquidation penalties and interest are deducted, held under the `Seized` reason.
* **Stability Fee** — See **Interest**.
* **System Debt** — Total pUSD in circulation: `total_issuance(StablecoinAssetId)`.
* **Vault** — Per-account structure tracking collateralized debt:
  `{ debt, accrued_interest, last_fee_update }`.
* **VaultDeposit** — Hold reason used for locking active collateral.
* **Withdrawal** — Operation releasing DOT from the `VaultDeposit` hold reason, subject to collateralization constraints.


## 4. Assumptions
*   **Currency Traits:** The generic `Currency` type implements `InspectHold` and `MutateHold` from the `fungible` trait. We assume collateral is "held" (reserved) in the user's account rather than transferred to a pallet account, using the `VaultDeposit` hold reason.
*   **Asset Traits:** The `Asset` type implements `InspectFungibles` and `MutateFungibles` (for minting/burning pUSD), from the `fungibles` trait.
*   **Oracle Data Model:** The `Oracle` must provide a **Normalized Price** (`FixedU128`).
    *   *Definition:* The price represents `Smallest_Unit_pUSD / Smallest_Unit_Collateral`.
    *   *Why:* This allows the pallet to perform decimal-agnostic math.
*   **Treasury:** A valid account exists to receive protocol revenue (Fees + Penalties).
*   **Block Time:** The pallet assumes ~6 second block times (5,256,000 blocks/year) for interest calculations.

---

## 5. Detailed Design

### 5.1 Architecture
The pallet does not hold funds directly. Instead, it places a **Hold** on the user's balance in `pallet-balances`.

**Hold Reasons:**
*   `VaultDeposit` — Collateral locked in an active vault.
*   `Seized` — Collateral seized during liquidation, pending auction. The auction pallet operates on funds held with this reason.

**Flow:**
1.  **User** calls `create_vault` → `pallet-vaults` instructs `pallet-balances` to **Hold** DOT with reason `VaultDeposit`.
2.  **User** calls `mint` → `pallet-vaults` instructs `pallet-assets` to **Mint** pUSD.
3.  **Fee Update** → `pallet-vaults` calculates owed fees, converts them to DOT value, and updates the `accrued_interest` field in the Vault struct.
4.  **Liquidation** → Hold reason changes from `VaultDeposit` to `Seized`, then `pallet-auctions` takes over.

### 5.2 Data Model (Storage)

**Struct: `Vault`**
```rust
struct Vault<T: Config> {
    debt: BalanceOf<T>,             // pUSD owed
    accrued_interest: BalanceOf<T>, // Accumulated interest in DOT
    last_fee_update: BlockNumberFor<T>,
}
```

**Storage Maps:**
*   `Vaults`: `StorageMap<AccountId, Vault>` (1:1 mapping, one vault per user).

**Governance Parameters (StorageValues):**
*   `MinimumCollateralizationRatio`: `FixedU128`, the minimum ratio of collateral value to debt value before a Vault is considered unsafe (e.g., 1.3 for 130%).
*   `InitialCollateralizationRatio`: `FixedU128`, the ratio of collateral value to debt value required when minting. Should be higher than `MinimumCollateralizationRatio` (e.g., 1.5 for 150%) to prevent immediate liquidation after minting.
*   `StabilityFee`: `Permill`, the annual interest rate charged on outstanding pUSD debt.
*   `LiquidationPenalty`: `Permill`, a penalty fee applied to a Vault's debt when it is liquidated.
*   `MaximumDebt`: `Balance`, the maximum amount of pUSD debt that can be issued by all Vaults together.
*   `BadDebt`: `Balance`, accumulated bad debt in pUSD (interest that exceeded collateral).

**Config Constants (require runtime upgrade to change):**
*   `MinimumDeposit`: `Balance`, the minimum amount of collateral required to create a Vault (prevents dust attacks).
*   `StablecoinAssetId`: The AssetId for pUSD.
*   `CollateralAssetId`: The AssetId for DOT.
*   `TreasuryAccount`: Account that receives protocol revenue.

## **5.2.1 Collateral Accounting Helpers**

The pallet does **not** track collateral inside the `Vault` struct directly. Instead, all collateral is held via the Balances pallet using the hold reason `VaultDeposit`. The canonical source of truth for collateral is therefore the *held balance* maintained by `T::Currency`.

To facilitate this, the `Vault` struct provides two helper methods:

```rust
impl<T: Config> Vault<T> {
    /// Returns the total amount of collateral (DOT) currently held for this vault.
    /// This queries the Balances pallet for the balance locked under the VaultDeposit hold reason.
    pub fn get_held_collateral(&self, who: &T::AccountId) -> CollateralBalanceOf<T> {
        T::Currency::balance_on_hold(&HoldReason::VaultDeposit.into(), who)
    }

    /// Returns the effective collateral available for collateralization and withdrawals.
    /// This is defined as: held_collateral - accrued_interest.
    pub fn get_available_collateral(&self, who: &T::AccountId) -> CollateralBalanceOf<T> {
        self.get_held_collateral(who)
            .saturating_sub(self.accrued_interest)
    }
}
```

### 5.3 Internal Logic & Math

#### A. Fee Calculation (`update_vault_fees`)
Fees are calculated based on time elapsed since `last_fee_update`.
1.  Calculate `Interest_pUSD = Debt × StabilityFee × (DeltaBlocks / BlocksPerYear)`.
2.  Get `Price` from Oracle (Normalized).
3.  Convert `Interest_pUSD` to `Interest_DOT`:
    *   `Interest_DOT = Interest_pUSD / Price`.
4.  Cap `Interest_DOT` to available collateral. If capped, the excess is recorded as `BadDebt` (in pUSD).
5.  Add capped `Interest_DOT` to `vault.accrued_interest`.
    *   *Note:* This reduces the user's "Available Collateral" but does not immediately transfer funds. Funds are transferred to Treasury only upon `repay`, `close_vault`, or `liquidate`.

#### B. Collateralization Ratio
*   **Formula:** `Ratio = (HeldCollateral - AccruedInterest) × Price / Debt`.
*   This uses `FixedU128` to handle precision.

#### C. Bad Debt Handling
If accrued interest exceeds a vault's collateral (e.g., during prolonged price decline without liquidation), the excess is recorded as `BadDebt` in pUSD. Governance can call `repay_bad_debt` to burn pUSD from the Treasury and restore protocol health.

### 5.4 External Interfaces

**Auctions Trait:**
```rust
pub trait Auctions<AccountId, AssetId, Balance> {
    fn start_auction(
        vault_owner: AccountId,
        collateral_asset: AssetId,
        collateral_amount: Balance,
        debt_asset: AssetId,
        debt_amount: Balance,
    ) -> DispatchResult;
}
```

The auction pallet must operate on funds held with `HoldReason::Seized`. When liquidation occurs, the vaults pallet transfers the hold reason from `VaultDeposit` to `Seized`, then calls `start_auction`. The auction pallet is responsible for releasing or transferring the seized collateral.

### 5.5 Workflow (Extrinsics)

#### User Flows
1.  **`create_vault(deposit)`**
    *   Ensures `deposit >= MinimumDeposit`.
    *   Ensures user has no existing vault.
    *   Calls `Currency::hold` with reason `VaultDeposit` to lock `deposit`.
    *   Initializes `Vault` with 0 pUSD debt.

2.  **`deposit_collateral(amount)`**
    *   Triggers `update_vault_fees`.
    *   Increases the held amount via `Currency::hold`.

3.  **`withdraw_collateral(amount)`**
    *   Triggers `update_vault_fees`.
    *   Calculates `AvailableCollateral = Held - AccruedInterest`.
    *   Checks if `(Available - Amount) × Price / Debt >= MinimumCollateralizationRatio`.
    *   Calls `Currency::release` to release the DOT.

4.  **`mint(amount)`**
    *   Triggers `update_vault_fees`.
    *   Checks global `MaximumDebt` (based on total pUSD issuance).
    *   Enforces `InitialCollateralizationRatio` (e.g., 150%) to ensure safety buffer.
    *   Calls `Asset::mint_into`.

5.  **`repay(amount)`**
    *   Triggers `update_vault_fees`.
    *   Burns `min(amount, debt)` of pUSD.
    *   **Interest Payment:** If `accrued_interest > 0`, transfers that amount of DOT from the held balance to the Treasury.

6.  **`close_vault()`**
    *   Can only be called if `Debt == 0`.
    *   Triggers `update_vault_fees`.
    *   Transfers remaining `accrued_interest` to Treasury.
    *   Releases all remaining held collateral to user.
    *   Removes Vault from storage.

#### System Flows

1.  **`liquidate_vault(target)`**
    *   Triggers `update_vault_fees`.
    *   Checks if `Ratio < MinimumCollateralizationRatio`. Fails if vault is safe.
    *   **Penalty Calculation:**
        *   `Penalty_pUSD = Debt × LiquidationPenalty`.
        *   `Penalty_DOT = Penalty_pUSD / Price` (capped to available collateral).
    *   **Distribution:**
        1.  Transfer `Penalty_DOT` from held collateral → Treasury.
        2.  Transfer `AccruedInterest` from held collateral → Treasury.
    *   **Seizure:**
        1.  Release remaining collateral from `VaultDeposit` hold.
        2.  Re-hold with `Seized` reason.
    *   **Debt Resolution via Auction:** Calls `Auctions::start_auction` with the seized collateral and outstanding debt. Bidders compete to purchase the collateral in exchange for pUSD, which is burned to reduce system debt. This mechanism ensures bad debt is socialised through market dynamics rather than absorbed by the protocol treasury.
    *   Closes Vault.

#### Governance Flows (Root Only)

1.  **`set_minimum_collateralization_ratio(ratio)`** — Update minimum CR.
2.  **`set_initial_collateralization_ratio(ratio)`** — Update initial CR.
3.  **`set_stability_fee(fee)`** — Update annual interest rate.
4.  **`set_liquidation_penalty(penalty)`** — Update liquidation penalty.
5.  **`repay_bad_debt(amount)`** — Burn pUSD from Treasury to reduce accumulated bad debt.

### 5.6 Events
*   `VaultCreated { owner }`
*   `CollateralDeposited { owner, amount }`
*   `CollateralWithdrawn { owner, amount }`
*   `Minted { owner, amount }`
*   `Repaid { owner, amount }`
*   `Liquidated { owner, debt, collateral_seized }`
*   `VaultClosed { owner }`
*   `InterestCollected { owner, amount }`
*   `LiquidationPenaltyCollected { owner, amount }`
*   `ParametersUpdated`
*   `BadDebtAccrued { owner, amount }` — Interest exceeded collateral.
*   `BadDebtRepaid { amount }` — Treasury repaid bad debt.

### 5.7 Errors
*   `VaultNotFound` — No vault exists for the account.
*   `VaultAlreadyExists` — Account already has a vault.
*   `VaultHasDebt` — Cannot close vault with outstanding debt.
*   `VaultIsSafe` — Cannot liquidate a healthy vault.
*   `InsufficientCollateral` — Not enough collateral for the operation.
*   `InsufficientDebt` — Repay amount exceeds debt.
*   `ExceedsMaxDebt` — Minting would exceed system debt ceiling.
*   `UnsafeCollateralizationRatio` — Operation would breach required CR.
*   `BelowMinimumDeposit` — Initial deposit too small.
*   `PriceNotAvailable` — Oracle returned no price.
*   `ArithmeticOverflow` — Calculation overflow.

---

## 6. Impact
*   **Economic:** Protocol revenue (Fees + Penalties) is collected in **DOT**, not pUSD. This creates continuous buy-pressure/accumulation of DOT for the Treasury.
*   **UX:** Users do not need to mint extra pUSD to pay interest; they effectively "pay" with the appreciation of their collateral or by forfeiting a portion of it.
*   **Storage:** The `Vaults` map grows linearly. Fee updates are performed lazily (triggered only when a Vault is interacted with). A future optimisation may use `on_idle` to batch-update stale Vaults, subject to defining an incentivisation mechanism.
*   **Governance:** All protocol financial parameters (`StabilityFee`, `LiquidationPenalty`, `MinimumCollateralizationRatio`, `InitialCollateralizationRatio`, `MaximumDebt`) are controlled via the **Polkadot Governance**. The `MinimumDeposit` constant requires a runtime upgrade to change.
