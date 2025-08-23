//! Execution Report message implementation (MsgType=8)
//!
//! This module implements a minimal subset of the FIX 4.2 Execution Report message.
//! It focuses on the core required fields to acknowledge order state and fills.
//! The full specification is extensive; this struct can be extended incrementally.

use crate::{
	OrdStatus, SOH, Side,
	common::{
		Validate, ValidationError,
		enums::{ExecTransType, ExecType},
		parse_fix_timestamp,
		validation::{FixFieldHandler, WriteTo},
		write_tag_timestamp,
	},
};
use std::fmt::Write;
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionReportBody {
	// (Tag 37) Required
	pub order_id: String,
	// (Tag 17) Required
	pub exec_id: String,
	// (Tag 20) Required
	pub exec_trans_type: ExecTransType,
	// (Tag 150) Required
	pub exec_type: ExecType,
	// (Tag 39) Required
	pub ord_status: OrdStatus,
	// (Tag 55) Required
	pub symbol: String,
	// (Tag 54) Required
	pub side: Side,
	// (Tag 151) Required
	pub leaves_qty: f64,
	// (Tag 14) Required
	pub cum_qty: f64,
	// (Tag 6) Required
	pub avg_px: f64,
	// (Tag 32) Optional (Qty of last fill)
	pub last_shares: Option<f64>,
	// (Tag 31) Optional (Px of last fill)
	pub last_px: Option<f64>,
	// (Tag 60) Optional TransactTime
	pub transact_time: Option<OffsetDateTime>,
	// (Tag 11) Optional ClOrdID for linkage
	pub cl_ord_id: Option<String>,
	// (Tag 41) Optional OrigClOrdID
	pub orig_cl_ord_id: Option<String>,
	// (Tag 103) Optional OrdRejReason when Rejected
	pub ord_rej_reason: Option<u32>,
}

impl Default for ExecutionReportBody {
	fn default() -> Self {
		Self {
			order_id: String::new(),
			exec_id: String::new(),
			exec_trans_type: ExecTransType::New,
			exec_type: ExecType::New,
			ord_status: OrdStatus::New,
			symbol: String::new(),
			side: Side::Buy,
			leaves_qty: 0.0,
			cum_qty: 0.0,
			avg_px: 0.0,
			last_shares: None,
			last_px: None,
			transact_time: None,
			cl_ord_id: None,
			orig_cl_ord_id: None,
			ord_rej_reason: None,
		}
	}
}

impl Validate for ExecutionReportBody {
	fn validate(&self) -> Result<(), ValidationError> {
		if self.order_id.is_empty() {
			return Err(ValidationError::MissingRequiredField("OrderID".into()));
		}
		if self.exec_id.is_empty() {
			return Err(ValidationError::MissingRequiredField("ExecID".into()));
		}
		if self.symbol.is_empty() {
			return Err(ValidationError::MissingRequiredField("Symbol".into()));
		}
		// LeavesQty + CumQty consistency (cannot be negative; allow zero for terminal states)
		if self.leaves_qty < 0.0 {
			return Err(ValidationError::InvalidFieldValue("LeavesQty".into(), self.leaves_qty.to_string()));
		}
		if self.cum_qty < 0.0 {
			return Err(ValidationError::InvalidFieldValue("CumQty".into(), self.cum_qty.to_string()));
		}
		if self.avg_px < 0.0 {
			return Err(ValidationError::InvalidFieldValue("AvgPx".into(), self.avg_px.to_string()));
		}
		Ok(())
	}
}

impl WriteTo for ExecutionReportBody {
	fn write_to(&self, buffer: &mut String) {
		write!(buffer, "37={}{}", self.order_id, SOH).unwrap();
		write!(buffer, "17={}{}", self.exec_id, SOH).unwrap();
		write!(buffer, "20={}{}", self.exec_trans_type, SOH).unwrap();
		write!(buffer, "150={}{}", self.exec_type, SOH).unwrap();
		write!(buffer, "39={}{}", self.ord_status, SOH).unwrap();
		if let Some(ref cl) = self.cl_ord_id {
			write!(buffer, "11={}{}", cl, SOH).unwrap();
		}
		if let Some(ref orig) = self.orig_cl_ord_id {
			write!(buffer, "41={}{}", orig, SOH).unwrap();
		}
		write!(buffer, "55={}{}", self.symbol, SOH).unwrap();
		write!(buffer, "54={}{}", self.side, SOH).unwrap();
		if let Some(ts) = self.transact_time {
			write_tag_timestamp(buffer, 60, ts);
		}
		if let Some(qty) = self.last_shares {
			write!(buffer, "32={}{}", qty, SOH).unwrap();
		}
		if let Some(px) = self.last_px {
			write!(buffer, "31={}{}", px, SOH).unwrap();
		}
		write!(buffer, "151={}{}", self.leaves_qty, SOH).unwrap();
		write!(buffer, "14={}{}", self.cum_qty, SOH).unwrap();
		write!(buffer, "6={}{}", self.avg_px, SOH).unwrap();
		if let Some(reason) = self.ord_rej_reason {
			write!(buffer, "103={}{}", reason, SOH).unwrap();
		}
	}
}

impl ExecutionReportBody {
	pub fn new(order_id: impl Into<String>, exec_id: impl Into<String>) -> Self {
		Self { order_id: order_id.into(), exec_id: exec_id.into(), ..Default::default() }
	}
}

impl FixFieldHandler for ExecutionReportBody {
	fn parse_field(&mut self, tag: u32, value: &str) -> Result<(), String> {
		match tag {
			37 => self.order_id = value.to_string(),
			17 => self.exec_id = value.to_string(),
			20 => self.exec_trans_type = value.parse().map_err(|_| "Invalid ExecTransType")?,
			150 => self.exec_type = value.parse().map_err(|_| "Invalid ExecType")?,
			39 => self.ord_status = value.parse().map_err(|_| "Invalid OrdStatus")?,
			55 => self.symbol = value.to_string(),
			54 => self.side = value.parse().map_err(|_| "Invalid Side")?,
			151 => self.leaves_qty = value.parse().map_err(|_| "Invalid LeavesQty")?,
			14 => self.cum_qty = value.parse().map_err(|_| "Invalid CumQty")?,
			6 => self.avg_px = value.parse().map_err(|_| "Invalid AvgPx")?,
			32 => self.last_shares = Some(value.parse().map_err(|_| "Invalid LastShares")?),
			31 => self.last_px = Some(value.parse().map_err(|_| "Invalid LastPx")?),
			60 => self.transact_time = Some(parse_fix_timestamp(value)?),
			11 => self.cl_ord_id = Some(value.to_string()),
			41 => self.orig_cl_ord_id = Some(value.to_string()),
			103 => self.ord_rej_reason = Some(value.parse().map_err(|_| "Invalid OrdRejReason")?),
			_ => return Err(format!("Unknown execution report field: {}", tag)),
		}
		Ok(())
	}

	fn write_body_fields(&self, buffer: &mut String) {
		self.write_to(buffer);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_basic_execution_report_validation() {
		let body = ExecutionReportBody {
			order_id: "OID".into(),
			exec_id: "EID".into(),
			symbol: "AAPL".into(),
			side: Side::Buy,
			leaves_qty: 50.0,
			cum_qty: 50.0,
			avg_px: 150.25,
			..Default::default()
		};
		assert!(body.validate().is_ok());
	}

	#[test]
	fn test_missing_required_fields() {
		let body = ExecutionReportBody::default();
		assert!(body.validate().is_err());
	}

	#[test]
	fn test_parse_and_write_roundtrip() {
		let mut body = ExecutionReportBody::default();
		assert!(body.parse_field(37, "OID1").is_ok());
		assert!(body.parse_field(17, "EID1").is_ok());
		assert!(body.parse_field(20, "0").is_ok());
		assert!(body.parse_field(150, "0").is_ok());
		assert!(body.parse_field(39, "0").is_ok());
		assert!(body.parse_field(55, "MSFT").is_ok());
		assert!(body.parse_field(54, "1").is_ok());
		assert!(body.parse_field(151, "100").is_ok());
		assert!(body.parse_field(14, "0").is_ok());
		assert!(body.parse_field(6, "0").is_ok());
		assert!(body.validate().is_ok());
		let mut s = String::new();
		body.write_to(&mut s);
		assert!(s.contains("37=OID1"));
		assert!(s.contains("150=0"));
	}
}
