//! New Order Single message implementation (MsgType=D)
//!
//! This module implements the FIX 4.2 New Order Single message, which is used for
//! session-level communication to maintain connection liveness and respond
//! to test requests.

use crate::{
	FORMAT_TIME, SOH, Side,
	common::{
		Validate, ValidationError, parse_fix_timestamp,
		validation::{FixFieldHandler, WriteTo},
	},
};
use std::fmt::Write;
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq)]
pub struct NewOrderSingleBody {
	/// Unique identifier of the order as assigned by institution (Tag 11) - Required
	pub cl_ord_id: String,
	// (Tag 21) - Required
	pub handl_inst: String,
	// (Tag 55) - Required
	pub symbol: String,
	// (Tag 54) - Required
	pub side: Side,
	// (Tag 38) - Either CashOrderQty or OrderQty is required. Note that either, but not both, CashOrderQty or OrderQty should be specified.
	pub order_qty: Option<f64>,
	// (Tag 152)
	pub cash_order_qty: Option<f64>,
	// (Tag 60) - Required
	pub transact_time: OffsetDateTime,
	// (Tag 40) - Required
	pub ord_type: String,
	// (Tag 44) Price - Optional
	pub price: Option<f64>,
	// (Tag 207) - Optional
	pub security_exchange: Option<String>,
}

impl Validate for NewOrderSingleBody {
	fn validate(&self) -> Result<(), ValidationError> {
		// Rust is enforcing the presence of all the required values, so we should do a sanity check of the values.
		if self.order_qty.is_none() && self.cash_order_qty.is_none() {
			return Err(ValidationError::MissingRequiredField("OrderQty or CashOrderQty".to_string()));
		}
		Ok(())
	}
}

impl WriteTo for NewOrderSingleBody {
	fn write_to(&self, buffer: &mut String) {
		write!(buffer, "11={}{}", self.cl_ord_id, SOH).unwrap();
		write!(buffer, "21={}{}", self.handl_inst, SOH).unwrap();
		write!(buffer, "55={}{}", self.symbol, SOH).unwrap();
		write!(buffer, "54={}{}", self.side, SOH).unwrap();
		write!(buffer, "60={}{}", self.transact_time.format(FORMAT_TIME).unwrap(), SOH).unwrap();
		write!(buffer, "40={}{}", self.ord_type, SOH).unwrap();
		if let Some(order_qty) = self.order_qty {
			write!(buffer, "38={}{}", order_qty, SOH).unwrap();
		}
		if let Some(cash_order_qty) = self.cash_order_qty {
			write!(buffer, "152={}{}", cash_order_qty, SOH).unwrap();
		}
		if let Some(security_exchange) = &self.security_exchange {
			write!(buffer, "207={}{}", security_exchange, SOH).unwrap();
		}
		if let Some(price) = self.price {
			write!(buffer, "44={}{}", price, SOH).unwrap();
		}
	}
}

impl Default for NewOrderSingleBody {
	fn default() -> Self {
		Self::new()
	}
}

impl NewOrderSingleBody {
	/// Create a new empty heartbeat body
	pub fn new() -> Self {
		Self {
			cl_ord_id: String::new(),
			handl_inst: String::new(),
			symbol: String::new(),
			side: Side::Buy,
			order_qty: None,
			price: None,
			cash_order_qty: None,
			transact_time: OffsetDateTime::now_utc(),
			ord_type: String::new(),
			security_exchange: None,
		}
	}
}

impl FixFieldHandler for NewOrderSingleBody {
	fn parse_field(&mut self, tag: u32, value: &str) -> Result<(), String> {
		match tag {
			11 => self.cl_ord_id = value.to_string(),
			21 => self.handl_inst = value.to_string(),
			55 => self.symbol = value.to_string(),
			54 => self.side = value.parse().map_err(|_| "Invalid side")?,
			60 => self.transact_time = parse_fix_timestamp(value)?,
			38 => self.order_qty = Some(value.parse().map_err(|_| "Invalid order quantity")?),
			40 => self.ord_type = value.to_string(),
			207 => self.security_exchange = Some(value.to_string()),
			44 => self.price = Some(value.parse().map_err(|_| "Invalid price")?),
			_ => return Err(format!("Unknown new order single field: {}", tag)),
		}
		Ok(())
	}

	fn write_body_fields(&self, buffer: &mut String) {
		// For new order single, write_body_fields is the same as write_to
		// since all order fields contribute to body length
		self.write_to(buffer);
	}
}
