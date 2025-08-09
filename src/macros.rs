/// Defines a FIX-style enum with `FromStr` and `Display` implementations.
///
/// # Modes
///
/// The first argument is the enum **mode**:
///
/// - `Strict` → Unknown values cause `from_str` to return `Err(())`.
/// - `Loose`  → Unknown values are preserved in an `Other(String)` variant.
///
/// # Parameters
///
/// - `Mode`   : Either `Strict` or `Loose` (controls parsing behavior).
/// - `Name`   : The name of the generated enum.
/// - Variants : A list of `Variant => "Code"` mappings, where `"Code"` is the FIX string representation.
///
/// # Examples
///
/// Strict mode (reject unknown values):
/// ```rust
/// use fix_learning::fix_enum;
///
/// fix_enum!(Strict Side {
///     Buy  => "1",
///     Sell => "2",
/// });
///
/// assert!("1".parse::<Side>().is_ok());
/// assert!("X".parse::<Side>().is_err());
/// ```
///
/// Loose mode (store unknown values in `Other(String)`):
/// ```rust
/// use fix_learning::fix_enum;
///
/// fix_enum!(Loose MsgType {
///     Heartbeat   => "0",
///     TestRequest => "1",
/// });
///
/// assert!("1".parse::<MsgType>().is_ok());
/// assert!(matches!("CUSTOM".parse::<MsgType>().unwrap(), MsgType::Other(s) if s == "CUSTOM"));
/// ```
#[macro_export]
macro_rules! fix_enum {
    // Strict mode: unknown values cause Err(())
    (Strict $name:ident { $($variant:ident => $code:expr,)* }) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum $name {
            $($variant,)*
        }

        impl std::str::FromStr for $name {
            type Err = ();
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $( $code => Ok(Self::$variant), )*
                    _ => Err(()),
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $( Self::$variant => f.write_str($code), )*
                }
            }
        }
    };

    // Loose mode: unknown values stored in Other(String)
    (Loose $name:ident { $($variant:ident => $code:expr,)* }) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum $name {
            $($variant,)*
            Other(String),
        }

        impl std::str::FromStr for $name {
            type Err = ();
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $( $code => Ok(Self::$variant), )*
                    other => Ok(Self::Other(other.into())),
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $( Self::$variant => f.write_str($code), )*
                    Self::Other(s) => f.write_str(s),
                }
            }
        }
    };
}

pub use fix_enum;
