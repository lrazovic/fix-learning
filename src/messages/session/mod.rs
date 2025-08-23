//! Session-level FIX messages
//!
//! This module contains implementations for FIX messages that handle
//! session management, including Heartbeat, Logon, Logout, and TestRequest.
//! These messages are fundamental to maintaining FIX session state and
//! connection liveness.

pub mod heartbeat;
pub mod logon;

// Re-export message body types for convenience
pub use heartbeat::HeartbeatBody;
pub use logon::LogonBody;

#[cfg(test)]
mod tests {
	use super::*;
	use crate::common::{EncryptMethod, Validate};

	#[test]
	fn test_session_message_exports() {
		// Test that we can create session message bodies
		let heartbeat = HeartbeatBody::new();
		assert!(heartbeat.is_valid());

		let logon = LogonBody::new(EncryptMethod::None, 30);
		assert!(logon.is_valid());
	}

	#[test]
	fn test_session_message_validation() {
		// All session messages should implement Validate
		let heartbeat = HeartbeatBody::default();
		let logon = LogonBody::default();

		assert!(heartbeat.validate().is_ok());
		assert!(logon.validate().is_ok());
	}
}
