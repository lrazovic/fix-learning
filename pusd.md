# Technical Design Doc: `pallet-vaults` (part of the pUSD Protocol)

**Authors:** Leonardo Razovic, ⁨Raffael H⁩uber, ⁨Luca Von Wyttenbach⁩
**Status:** Writing it
**Related Links:** [Polkadot SDK PR TBA]

---

## 1. Purpose
The purpose of this document is to outline the technical implementation of `pallet-vaults`. This pallet serves as the "Collateralized Debt Position" (CDP) engine for the pUSD protocol, enabling users to lock native currency (DOT) to mint stablecoins (pUSD) while ensuring the system remains over-collateralized.

## 2. Background
This implementation mimics the **MakerDAO v1 (Single-Collateral SAI)** model but adapts the accounting for a Polkadot SDK native environment.

Unlike the standard Maker model where stability fees increase the debt (pUSD) owed, this implementation calculates fees in pUSD but **deducts them from the locked collateral (DOT)**. This design choice ensures that the pUSD supply remains strictly pegged to user actions (mint/burn) rather than inflating automatically via interest.

## 3. Goals
**Success Metrics:**
1.  **Solvency:** Total Debt < Total Collateral Value (adjusted for LTV).
2.  **Safety Buffer:** The system enforces a higher collateral ratio for opening positions (`Initial`) vs maintaining them (`Minimum`) to prevent instant liquidations due to minor price volatility.
3.  **Atomic Liquidation:** Liquidations resolve debt and distribute penalties in a single atomic transaction flow.

## 4. Assumptions
*   **Currency Traits:** The generic `Currency` type implements `InspectHold` and `MutateHold` from the `fungible` trait. We assume collateral is "held" (reserved) in the user's account rather than transferred to a pallet account, using the `VaultDeposit` hold reason.
*   **Asset Traits:** The `Asset` type implements `InspectFungibles` and `MutateFungibles` (for minting/burning pUSD), from the `fungibles` trait.
*   **Oracle Data Model:** The `Oracle` must provide a **Normalized Price** (`FixedU128`).
    *   *Definition:* The price represents `Smallest_Unit_pUSD / Smallest_Unit_Collateral`.
    *   *Why:* This allows the pallet to perform decimal-agnostic math.
*   **Treasury:** A valid account exists to receive protocol revenue (Fees + Penalties).

---

## 5. Detailed Design

### 5.1 Architecture
The pallet does not hold funds directly. Instead, it places a **Hold** (reason: `VaultDeposit`) on the user's balance in `pallet-balances`.

1.  **User** calls `create_vault` -> `pallet-vaults` instructs `pallet-balances` to **Hold** DOT.
2.  **User** calls `mint` -> `pallet-vaults` instructs `pallet-assets` to **Mint** pUSD.
3.  **Fee Update** -> `pallet-vaults` calculates owed fees, converts them to DOT value, and updates the `accrued_interest` field in the Vault struct.

### 5.2 Data Model (Storage)

**Struct: `Vault`**
```rust
struct Vault<T> {
    debt: T::Balance,                // pUSD owed
    accrued_interest: BalanceOf<T>,  // Accumulated Interest in DOT
    last_fee_update: BlockNumber,    // Last block the fee was calculated
}
```

**Storage Maps:**
*   **`Vaults`**: `StorageMap<AccountId, Vault>` (1:1 mapping, one vault per user).


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

**Rationale**

* The Balances pallet already maintains authoritative, auditable accounting for held funds.
* The vault only needs to track *accrued interest*, not raw collateral.
* Using saturating subtraction ensures safe behavior in edge cases where accrued interest temporarily equals or exceeds held collateral (which will force the collateralization ratio to zero and make the vault eligible for liquidation).



**Parameters**
*   `MinimumCollateralizationRatio`: `FixedU128`, the minimum ratio of collateral value to debt value before a Vault is considered unsafe (e.g., 130%).
* `InitialCollateralizationRatio`:  `FixedU128`, the ratio of collateral value to debt value upon opening a Vault. This should be equal or higher (e.g., 150%) than the `MinimumCollateralizationRatio` and help prevent cases where users get liquidated very shortly after opening the Vault.
*   `StabilityFee`:  `FixedU128`, the annual interest rate charged on outstanding pUSD debt.
*   `LiquidationPenalty`: `Permill`, a penalty fee applied to a Vault's debt when it is liquidated.
*   `MaximumDebt`: `Balance`, the maximum amount of pUSD debt that can be issued by all Vaults together.
*   `MinimumDeposit`: `Balance`, the minimum amount of collateral required to create a Vault.

### 5.3 Internal Logic & Math

#### A. Fee Calculation (`update_vault_fees`)
Fees are calculated based on time elapsed since `last_fee_update`.
1.  Calculate `Interest_pUSD = Debt * StabilityFee * (DeltaBlocks / BlocksPerYear)`.
2.  Get `Price` from Oracle (Normalized).
3.  Convert `Interest_pUSD` to `Interest_DOT`:
    *   `Interest_DOT = Interest_pUSD / Price`.
4.  Add `Interest_DOT` to `vault.accrued_interest`.
    *   *Note:* This reduces the user's "Available Collateral" but does not immediately transfer funds. Funds are transferred to Treasury only upon `repay` or `liquidate`.

#### B. Collateralization Ratio
*   **Formula:** `Ratio = (HeldCollateral - AccruedInterest) * Price / Debt`.
*   This uses `FixedU128` to handle precision.

### 5.4 Workflow (Extrinsics)

#### User Flows
1.  **`create_vault(deposit)`**
    *   Ensures user has no existing vault.
    *   Calls `Currency::hold` to lock `deposit`.
    *   Initializes `Vault` with 0 debt.
2.  **`deposit_collateral(amount)`**
    *   Triggers `update_vault_fees`.
    *   Increases the held amount via `Currency::hold`.
3.  **`withdraw_collateral(amount)`**
    *   Triggers `update_vault_fees`.
    *   Calculates `AvailableCollateral = Held - AccruedInterest`.
    *   Checks if `(Available - Amount) / Debt >= MinimumCollateralizationRatio`.
    *   Calls `Currency::release`.
4.  **`mint(amount)`**
    *   Triggers `update_vault_fees`.
    *   Checks global `MaximumDebt`.
    *   Enforces `InitialCollateralizationRatio` (e.g., 150%) to ensure safety buffer.
    *   Calls `Asset::mint_into`.
5.  **`repay(amount)`**
    *   Triggers `update_vault_fees`.
    *   Burns `amount` (or max debt) of pUSD.
    *   **Interest Payment:** If `accrued_interest > 0`, it transfers that amount of DOT from the held balance to the **Treasury**.
6.  **`close_vault()`**
    *   Can only be called if `Debt == 0`.
    *   Pay remaining `accrued_interest` to Treasury.
    *   Releases all remaining held collateral to User.
    *   Removes Vault from storage.

#### System Flows

1.  **`liquidate_vault(target)`**
    *   Triggers `update_vault_fees`.
    *   Checks if `Ratio < MinimumCollateralizationRatio`.
    *   **Penalty Calculation:**
        *   `Penalty_pUSD = Debt * LiquidationPenalty`.
        *   `Penalty_DOT = Penalty_pUSD / Price`.
    *   **Distribution:**
        1.  Transfer `Penalty_DOT` from Held Collateral -> Treasury.
        2.  Transfer `AccruedInterest` from Held Collateral -> Treasury.
        3.  Release remaining DOT to User (to be seized by Auction Logic).
    *   **Debt Resolution via Auction:** If the liquidated Vault's remaining collateral is insufficient to cover the outstanding debt, the system does not write off the loss immediately. Instead, following the MakerDAO model, the collateral is handed off to pallet-auctions which conducts a Collateral Auction. Bidders compete to purchase the seized collateral in exchange for pUSD, which is then burned to reduce the system's total debt. This mechanism ensures that bad debt is socialised through market dynamics rather than absorbed by the protocol treasury.
    *   Closes Vault.

---

## 6. Impact
*   **Economic:** The protocol revenue (Fees + Penalties) is collected in **DOT**, not pUSD. This creates a continuous buy-pressure/accumulation of DOT for the Treasury.
*   **UX:** Users do not need to mint extra pUSD to pay interest; they effectively "pay" with the appreciation of their collateral or by forfeiting a portion of it.
*   **Storage:** The `Vaults` map grows linearly. {TBD: Check how the on_idle can be used}
*   **Governance:** All protocol financial parameters (`StabilityFee`, `LiquidationPenalty`, `MinimumCollateralizationRatio`, `InitialCollateralizationRatio`, and `MaximumDebt`) are controlled via the Polkadot Governance.