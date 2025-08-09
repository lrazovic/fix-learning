# FIX 4.2 Learning Project

A Rust implementation of Financial Information eXchange (FIX) 4.2 protocol messages.

## Overview

This project provides a comprehensive struct-based representation of FIX 4.2 messages, including:
- Standard message types (Heartbeat, ExecutionReport, NewOrderSingle, etc.)
- **Builder pattern** for fluent message construction
- **FIX serialization** to/from wire format with automatic checksum calculation
- Extensible field storage for custom tags
- Complete validation and helper methods

## Project Structure

```
fix-learning/
├── src/
│   ├── lib.rs          # Main library with FIX message structures
│   └── macros.rs       # FIX enum generation macros
├── tests/
│   ├── fix_message_tests.rs    # Unit tests for core functionality
│   ├── integration_tests.rs    # Real-world trading workflow tests
│   └── builder_tests.rs        # Builder pattern and serialization tests
├── examples/
│   ├── user_message_builder.rs # Example recreating user's exact message
│   └── clean_api_demo.rs       # Demonstration of idiomatic Rust API
├── Cargo.toml
└── README.md
```

## Quick Start

### Running Examples

```bash
# Run the clean API demonstration
cargo run --example clean_api_demo

# Run example showing builder pattern
cargo run --example user_message_builder
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test fix_message_tests

# Run integration tests (including real message parsing)
cargo test integration_tests

# Run tests for a specific functionality
cargo test test_parse_real_fix_message
```

## Usage Examples

### Using the Builder Pattern with Idiomatic Rust (Recommended)

```rust
use fix_learning::{FixMessage, MsgType, Side};

// Create a New Order Single using FromStr trait (idiomatic Rust)
let new_order = FixMessage::builder(
    "D".parse().unwrap(),               // MsgType::NewOrderSingle using FromStr
    "TRADER".to_string(),
    "EXCHANGE".to_string(),
    100,
    "20241201-09:30:00.000".to_string(),
)
.cl_ord_id("ORDER_001".to_string())
.symbol("AAPL".to_string())
.side("1".parse().unwrap())             // Side::Buy using FromStr
.order_qty(100.0)
.ord_type("2".to_string()) // Limit order
.price(150.25)
.time_in_force("0".to_string()) // Day order
.build();

// Serialize to FIX wire format
let fix_string = new_order.to_fix_string();
println!("{}", fix_string);
// Output: 8=FIX.4.2^A9=120^A35=D^A34=100^A49=TRADER^A...

// Clean enum conversions - no to_str() methods needed!
let msg_type: MsgType = "8".parse().unwrap();  // ExecutionReport
let side: Side = "2".parse().unwrap();         // Sell
println!("Message: {}, Side: {}", msg_type, side); // Uses Display trait automatically
```

### Recreating Your Original Message

Based on your original FIX string, here's how to build it with the builder pattern:

```rust
let user_message = FixMessage::builder(
    "D".parse().unwrap(),               // MsgType::NewOrderSingle using FromStr
    "TESTBUY3".to_string(),
    "TESTSELL3".to_string(),
    972,
    "20190206-16:25:10.403".to_string(),
)
.cl_ord_id("14163685067084226997921".to_string())
.order_qty(100.0)
.ord_type("1".to_string()) // Market order
.side("1".parse().unwrap())                      // Side::Buy using FromStr
.symbol("AAPL".to_string())
.field(21, "2".to_string())                      // HandlInst
.field(60, "20190206-16:25:08.968".to_string())  // TransactTime
.field(207, "TO".to_string())                    // SecurityExchange
.field(6000, "TEST1234".to_string())             // Custom field
.build();

let fix_string = user_message.to_fix_string();
// Produces properly formatted FIX message with checksum
```

### Creating a Basic Message (Alternative Method)

```rust
use fix_learning::{FixMessage, MsgType};

let heartbeat = FixMessage::new(
    MsgType::Heartbeat,
    "CLIENT".to_string(),
    "BROKER".to_string(),
    1,
    "20241201-12:00:00.000".to_string(),
);
```

### Working with Custom Fields

```rust
let mut message = FixMessage::default();

// Set custom fields using tag numbers
message.set_field(9999, "custom_value".to_string());
message.set_field(8888, "another_value".to_string());

// Retrieve custom fields
if let Some(value) = message.get_field(9999) {
    println!("Custom field value: {}", value);
}
```

### FIX Message Serialization

```rust
// Build a message using idiomatic FromStr
let message = FixMessage::builder("8".parse().unwrap(), ...) // ExecutionReport
    .symbol("MSFT".to_string())
    .side("1".parse().unwrap()) // Buy
    .ord_status("2".parse().unwrap()) // Filled
    .build();

// Serialize to FIX wire format (with SOH separators, uses Display trait)
let fix_string = message.to_fix_string();

// Parse from FIX wire format
let parsed = FixMessage::from_fix_string(&fix_string)?;

// Clean enum usage - Display trait provides automatic string formatting
let msg_type: MsgType = "D".parse()?;     // NewOrderSingle
let side: Side = "1".parse()?;            // Buy
let status: OrdStatus = "0".parse()?;     // New
println!("Type: {}, Side: {}, Status: {}", msg_type, side, status); // No to_str() needed!
```

## FIX 4.2 Message Structure

### Standard Header Fields
- `begin_string` (Tag 8): Protocol version "FIX.4.2"
- `body_length` (Tag 9): Message body length
- `msg_type` (Tag 35): Message type
- `sender_comp_id` (Tag 49): Sender company ID
- `target_comp_id` (Tag 56): Target company ID
- `msg_seq_num` (Tag 34): Message sequence number
- `sending_time` (Tag 52): Message transmission time

### Common Body Fields
- Order fields: `cl_ord_id`, `order_id`, `symbol`, `side`, `order_qty`
- Execution fields: `exec_id`, `exec_type`, `ord_status`, `last_qty`, `last_px`
- Price fields: `price`, `cum_qty`, `leaves_qty`, `avg_px`

### Standard Trailer
- `checksum` (Tag 10): Message checksum

## Example Trading Workflows

The integration tests demonstrate several real-world scenarios:

1. **Complete Order Lifecycle**: New → Partial Fill → Full Fill
2. **Order Cancellation**: New → Cancel Request → Canceled
3. **Order Replacement**: Original → Replace Request → Replaced
4. **Market Data**: Subscription → Snapshot → Updates
5. **Heartbeat Sequence**: Regular keepalive messages
6. **Real Message Parsing**: Parse actual FIX messages from production environments

## Message Types Supported

- `Heartbeat` (0)
- `TestRequest` (1)
- `ExecutionReport` (8)
- `NewOrderSingle` (D)
- `OrderCancelRequest` (F)
- `OrderCancelReplaceRequest` (G)
- `MarketDataRequest` (V)
- `MarketDataSnapshot` (W)
- And many more...

## API Reference

### Main Types

- **`FixMessage`**: The main FIX message structure
- **`FixMessageBuilder`**: Builder for fluent message construction
- **`MsgType`**: Enum for FIX message types
- **`Side`**: Enum for order side (Buy/Sell)
- **`OrdStatus`**: Enum for order status values

### Key Methods

- **`FixMessage::builder(...)`**: Create a new builder
- **`FixMessage::to_fix_string()`**: Serialize to FIX wire format
- **`FixMessage::from_fix_string()`**: Parse from FIX wire format
- **`builder.field(tag, value)`**: Set custom field
- **`message.get_field(tag)`**: Retrieve custom field
- **`"D".parse::<MsgType>()`**: Parse enum from string using `FromStr`
- **`format!("{}", msg_type)`**: Format enum to string using `Display`
- **`msg_type.to_string()`**: Automatic string conversion via `Display` trait
- **No custom `to_str()` methods needed!**

## License

This project is for educational purposes in learning the FIX protocol and Rust development with TDD principles.

## References

- [FIX 4.2 Specification](https://www.fixtrading.org/standards/)
