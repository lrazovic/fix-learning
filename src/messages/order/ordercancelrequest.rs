//! Order Cancel Request message implementation (MsgType=F)
//!
//! Minimal FIX 4.2 Order Cancel Request supporting core required fields:
//! OrigClOrdID(41), ClOrdID(11), Symbol(55), Side(54), TransactTime(60)
//! with optional OrderID(37), OrderQty(38)/CashOrderQty(152), Account(1), Text(58).

use crate::{
	SOH, Side,
	common::{
		Validate, ValidationError, parse_fix_timestamp,
		validation::{FixFieldHandler, WriteTo},
		write_tag_timestamp,
	},
};
use std::fmt::Write;
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq)]
pub struct OrderCancelRequestBody {
	pub orig_cl_ord_id: String,        // 41 Required
	pub cl_ord_id: String,             // 11 Required (unique id of this cancel request)
	pub symbol: String,                // 55 Required
	pub side: Side,                    // 54 Required
	pub transact_time: OffsetDateTime, // 60 Required
	pub order_id: Option<String>,      // 37 Optional (most recent order id as assigned by broker)
	pub order_qty: Option<f64>,        // 38 Either this or cash_order_qty required (per spec)
	pub cash_order_qty: Option<f64>,   // 152
	pub account: Option<String>,       // 1 Optional
	pub text: Option<String>,          // 58 Optional
}

impl Default for OrderCancelRequestBody {
	fn default() -> Self {
		Self {
			orig_cl_ord_id: String::new(),
			cl_ord_id: String::new(),
			symbol: String::new(),
			side: Side::Buy,
			transact_time: OffsetDateTime::now_utc(),
			order_id: None,
			order_qty: None,
			cash_order_qty: None,
			account: None,
			text: None,
		}
	}
}

impl Validate for OrderCancelRequestBody {
	fn validate(&self) -> Result<(), ValidationError> {
		if self.orig_cl_ord_id.is_empty() {
			return Err(ValidationError::MissingRequiredField("OrigClOrdID".into()));
		}
		if self.cl_ord_id.is_empty() {
			return Err(ValidationError::MissingRequiredField("ClOrdID".into()));
		}
		if self.symbol.is_empty() {
			return Err(ValidationError::MissingRequiredField("Symbol".into()));
		}
		// OrderQty or CashOrderQty required per spec (treat as required here)
		if self.order_qty.is_none() && self.cash_order_qty.is_none() {
			return Err(ValidationError::MissingRequiredField("OrderQty or CashOrderQty".into()));
		}
		Ok(())
	}
}

impl WriteTo for OrderCancelRequestBody {
	fn write_to(&self, buffer: &mut String) {
		// Required first
		write!(buffer, "41={}{}", self.orig_cl_ord_id, SOH).unwrap();
		if let Some(ref oid) = self.order_id {
			write!(buffer, "37={}{}", oid, SOH).unwrap();
		}
		write!(buffer, "11={}{}", self.cl_ord_id, SOH).unwrap();
		write!(buffer, "55={}{}", self.symbol, SOH).unwrap();
		write!(buffer, "54={}{}", self.side, SOH).unwrap();
		write_tag_timestamp(buffer, 60, self.transact_time);
		if let Some(qty) = self.order_qty {
			write!(buffer, "38={}{}", qty, SOH).unwrap();
		}
		if let Some(cash) = self.cash_order_qty {
			write!(buffer, "152={}{}", cash, SOH).unwrap();
		}
		if let Some(ref acct) = self.account {
			write!(buffer, "1={}{}", acct, SOH).unwrap();
		}
		if let Some(ref txt) = self.text {
			write!(buffer, "58={}{}", txt, SOH).unwrap();
		}
	}
}

impl FixFieldHandler for OrderCancelRequestBody {
	fn parse_field(&mut self, tag: u32, value: &str) -> Result<(), String> {
		match tag {
			41 => self.orig_cl_ord_id = value.to_string(),
			37 => self.order_id = Some(value.to_string()),
			11 => self.cl_ord_id = value.to_string(),
			55 => self.symbol = value.to_string(),
			54 => self.side = value.parse().map_err(|_| "Invalid Side")?,
			60 => self.transact_time = parse_fix_timestamp(value)?,
			38 => self.order_qty = Some(value.parse().map_err(|_| "Invalid OrderQty")?),
			152 => self.cash_order_qty = Some(value.parse().map_err(|_| "Invalid CashOrderQty")?),
			1 => self.account = Some(value.to_string()),
			58 => self.text = Some(value.to_string()),
			_ => return Err(format!("Unknown order cancel request field: {}", tag)),
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
	fn test_validation_missing_required() {
		let body = OrderCancelRequestBody::default();
		assert!(body.validate().is_err());
	}

	#[test]
	fn test_validation_success() {
		let mut body = OrderCancelRequestBody::default();
		body.orig_cl_ord_id = "ORIG1".into();
		body.cl_ord_id = "CXL1".into();
		body.symbol = "AAPL".into();
		body.order_qty = Some(100.0);
		assert!(body.validate().is_ok());
	}

	#[test]
	fn test_parse_and_write() {
		let mut body = OrderCancelRequestBody::default();
		body.parse_field(41, "ORIG1").unwrap();
		body.parse_field(11, "CXL1").unwrap();
		body.parse_field(55, "MSFT").unwrap();
		body.parse_field(54, "1").unwrap();
		body.parse_field(38, "50").unwrap();
		body.parse_field(60, "20240101-12:00:00.000").unwrap();
		assert!(body.validate().is_ok());
		let mut s = String::new();
		body.write_to(&mut s);
		assert!(s.contains("41=ORIG1"));
		assert!(s.contains("11=CXL1"));
		assert!(s.contains("38=50"));
	}
}
