//! Common types and utilities for the FIX library
//!
//! This module contains shared types, validation traits, and utilities
//! that are used across different FIX message types.

pub mod enums;
pub mod header;
pub mod trailer;
pub mod validation;

// Re-export commonly used types
pub use enums::{EncryptMethod, MsgType, OrdStatus, Side};
pub use header::{FixHeader, parse_fix_timestamp};
use time::{format_description::BorrowedFormatItem, macros::format_description};
pub use trailer::FixTrailer;
pub use validation::{Validate, ValidationError};

/// The Start of Heading control character, value 0x01, used for field termination.
pub const SOH: &str = "\x01";

/// Time/date combination format for FIX timestamps
pub const FORMAT_TIME: &[BorrowedFormatItem<'_>] =
	format_description!("[year][month][day]-[hour]:[minute]:[second].[subsecond digits:3]");
