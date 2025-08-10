# FIX Learning Benchmarks

This directory contains comprehensive Criterion.rs benchmarks for the FIX Learning library, designed to measure performance and identify optimization opportunities.

## Quick Start

```bash
# Run all benchmarks
./run_benchmarks.sh

# Run specific benchmark suite
cargo bench --bench fix_benchmarks
cargo bench --bench optimization_benchmarks

# View HTML reports
open target/criterion/report/index.html
```

## Benchmark Suites

### Core FIX Benchmarks (`fix_benchmarks.rs`)

**Message Creation**
- `heartbeat` - Simple message creation overhead
- `simple_new_order` - Basic order creation with required fields
- `complex_new_order` - Order with custom fields and timestamps
- `execution_report` - Full execution report with all fields

**Serialization Performance**
- `to_fix_string` - Converting messages to FIX wire format
- Tests across different message types and complexities

**Parsing Performance**
- `from_fix_string` - Parsing FIX strings back to message objects
- Includes validation and field extraction overhead

**Round-trip Performance**
- `serialize_parse` - Complete serialize → parse cycle
- Critical for high-frequency trading scenarios

**Enum Operations**
- `msg_type_parsing` - MsgType enum FromStr performance
- `side_parsing` - Side enum conversion
- `ord_status_parsing` - Order status enum handling

**Field Operations**
- `set_custom_field` - Adding custom fields to messages
- `get_custom_field` - Retrieving custom field values
- `validation` - Message validation overhead

**Message Size Impact**
- Small/Medium/Large message handling
- Memory usage scaling with message complexity

**Memory Patterns**
- `batch_message_creation` - Creating many messages in sequence
- `builder_reuse_pattern` - Builder pattern efficiency

## Key Metrics to Monitor

### Performance Indicators
- **Throughput**: Messages processed per second
- **Latency**: Time per message operation
- **Memory**: Allocation patterns and peak usage

### Critical Benchmarks for Trading Systems
1. **`simple_new_order`** - Core order creation latency
2. **`serialization`** - Wire format conversion speed
3. **`parsing`** - Incoming message processing speed
4. **`batch_creation`** - High-frequency trading scenarios

### Optimization Opportunities
1. **Memory allocation** - Compare current vs optimized approaches
2. **Field storage** - BTreeMap vs SmallVec trade-offs
3. **String handling** - Stack vs heap allocation patterns

## Understanding Results

### Interpreting Criterion Output
```
message_creation/heartbeat  time: [1.2345 µs 1.2567 µs 1.2789 µs]
                           change: [+2.34% +3.45% +4.56%] (p = 0.00 < 0.05)
```

- **First line**: [lower_bound mean upper_bound] confidence interval
- **Second line**: Performance change vs previous run
- **p-value**: Statistical significance (< 0.05 is significant)
