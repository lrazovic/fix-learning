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
use time::OffsetDateTime;
pub use trailer::FixTrailer;
pub use validation::{Validate, ValidationError};

/// The Start of Heading control character, value 0x01, used for field termination.
pub const SOH: &str = "\x01";

/// Formats a FIX timestamp field with its tag number.
///
/// ### Why not use `time::format()`?
///
/// The `time` crate's formatting functionality (`time::format()`) is designed for
/// flexibility and correctness across many different format strings and edge cases.
/// This generality comes with significant overhead:
///
/// 1. **Dynamic format string parsing** - The format string is parsed at runtime
/// 2. **Heap allocations** - Creates intermediate String allocations
/// 3. **Error handling** - Returns Result<String, Error> requiring unwrap()
/// 4. **Generic abstraction** - Handles many formats we don't need
///
/// For FIX protocol, we have exactly ONE timestamp format that never changes:
/// `YYYYMMDD-HH:MM:SS.sss` (exactly 21 characters, always UTC)
///
/// ### Our approach
///
/// This implementation leverages `itoa::Buffer` for fast integer-to-string conversion.
///
/// The manual padding logic (checking if values < 10 or < 100) is explicit and
/// branch-predictable, making it faster than generic formatting code that handles
/// arbitrary padding widths.
///
/// #### Performance characteristics
///
/// - **Zero heap allocations** - Everything happens on the stack
/// - **Predictable branches** - Month/day/hour/minute/second are usually >= 10
/// - **No error handling** - FIX timestamps are always valid by construction
/// - **Inline-friendly** - Simple enough for the compiler to inline
///
/// ## Example
/// ```
/// use fix_learning::common::write_tag_timestamp;
/// use time::OffsetDateTime;
///
/// let mut buffer = String::with_capacity(256);
/// write_tag_timestamp(&mut buffer, 52, OffsetDateTime::now_utc());
/// // Results in: "52=20240115-14:23:45.678\x01"
/// ```
#[inline(always)]
pub fn write_tag_timestamp(buf: &mut String, tag: u16, time: OffsetDateTime) {
	let mut temp = itoa::Buffer::new();

	buf.push_str(temp.format(tag));
	buf.push('=');

	// Year
	buf.push_str(temp.format(time.year()));

	// Month (pad with 0 if needed)
	let month = time.month() as u8;
	if month < 10 {
		buf.push('0');
	}
	buf.push_str(temp.format(month));

	// Day (pad with 0 if needed)
	let day = time.day();
	if day < 10 {
		buf.push('0');
	}
	buf.push_str(temp.format(day));

	buf.push('-');

	// Hour (pad with 0 if needed)
	let hour = time.hour();
	if hour < 10 {
		buf.push('0');
	}
	buf.push_str(temp.format(hour));
	buf.push(':');

	// Minute (pad with 0 if needed)
	let minute = time.minute();
	if minute < 10 {
		buf.push('0');
	}
	buf.push_str(temp.format(minute));
	buf.push(':');

	// Second (pad with 0 if needed)
	let second = time.second();
	if second < 10 {
		buf.push('0');
	}
	buf.push_str(temp.format(second));
	buf.push('.');

	// Milliseconds (pad with 0s if needed)
	let ms = time.millisecond();
	if ms < 10 {
		buf.push_str("00");
	} else if ms < 100 {
		buf.push('0');
	}
	buf.push_str(temp.format(ms));

	buf.push_str(SOH);
}
