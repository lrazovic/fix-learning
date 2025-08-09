use thiserror::Error;

#[derive(Error, Debug)]
pub enum FixError {
	#[error("Invalid timestamp: {0}")]
	InvalidTimestamp(String),
	#[error("Missing required field: {0}")]
	MissingField(String),
	#[error("Invalid value for {0}: {1}")]
	InvalidValue(String, String),
	#[error("Checksum mismatch: expected {0}, got {1}")]
	ChecksumMismatch(u32, u32),
	#[error("Body length mismatch: expected {0}, calculated {1}")]
	BodyLengthMismatch(u32, usize),
	#[error("Unsupported BeginString: {0}")]
	UnsupportedBeginString(String),
	// Add more as needed
}
