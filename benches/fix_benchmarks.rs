use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use fix_learning::{FixMessage, MsgType, OrdStatus, Side};
use std::{hint::black_box, str::FromStr};
use time::macros::datetime;

// Sample FIX message data for benchmarks
const SAMPLE_FIX_STRINGS: &[&str] = &[
	// Simple Heartbeat
	"8=FIX.4.2\x019=40\x0135=0\x0149=SENDER\x0156=TARGET\x0134=1\x0152=20241201-12:00:00.000\x0110=123\x01",
	// New Order Single
	"8=FIX.4.2\x019=163\x0135=D\x0134=972\x0149=TESTBUY3\x0152=20190206-16:25:10.403\x0156=TESTSELL3\x0111=14163685067084226997921\x0121=2\x0138=100\x0140=1\x0154=1\x0155=AAPL\x0160=20190206-16:25:08.968\x01207=TO\x016000=TEST1234\x0110=106\x01",
	// Execution Report
	"8=FIX.4.2\x019=200\x0135=8\x0149=BROKER\x0156=CLIENT\x0134=100\x0152=20241201-12:00:00.000\x0137=ORDER001\x0111=CLIENT001\x0117=EXEC001\x01150=F\x0139=2\x0155=MSFT\x0154=1\x0138=500\x0131=155.75\x0132=500\x0114=500\x01151=0\x016=155.75\x0110=123\x01",
	// Order Cancel Request
	"8=FIX.4.2\x019=120\x0135=F\x0149=CLIENT\x0156=BROKER\x0134=50\x0152=20241201-12:00:00.000\x0137=ORDER001\x0111=CANCEL001\x0141=CLIENT001\x0155=GOOGL\x0154=2\x0138=100\x0110=123\x01",
	// Market Data Request
	"8=FIX.4.2\x019=150\x0135=V\x0149=CLIENT\x0156=MARKET\x0134=25\x0152=20241201-12:00:00.000\x01262=MDREQ001\x01263=1\x01264=20\x01267=2\x01269=0\x01269=1\x01146=1\x0155=AAPL\x0110=123\x01",
];

fn create_sample_messages() -> Vec<FixMessage> {
	vec![
		// Heartbeat
		FixMessage::builder(MsgType::Heartbeat, "SENDER", "TARGET", 1).build(),
		// New Order Single
		FixMessage::builder(MsgType::NewOrderSingle, "TESTBUY3", "TESTSELL3", 972)
			.sending_time(datetime!(2019-02-06 16:25:10.403 UTC))
			.cl_ord_id("14163685067084226997921")
			.symbol("AAPL")
			.side(Side::Buy)
			.order_qty(100.0)
			.field(207, "TO")
			.field(6000, "TEST1234")
			.build(),
		// Execution Report
		FixMessage::builder(MsgType::ExecutionReport, "BROKER", "CLIENT", 100)
			.order_id("ORDER001")
			.cl_ord_id("CLIENT001")
			.exec_id("EXEC001")
			.exec_type("F")
			.ord_status(OrdStatus::Filled)
			.symbol("MSFT")
			.side(Side::Buy)
			.order_qty(500.0)
			.last_px(155.75)
			.last_qty(500.0)
			.cum_qty(500.0)
			.leaves_qty(0.0)
			.avg_px(155.75)
			.build(),
		// Order Cancel Request
		FixMessage::builder(MsgType::OrderCancelRequest, "CLIENT", "BROKER", 50)
			.order_id("ORDER001")
			.cl_ord_id("CANCEL001")
			.symbol("GOOGL")
			.side(Side::Sell)
			.order_qty(100.0)
			.build(),
		// Market Data Request
		FixMessage::builder(MsgType::MarketDataRequest, "CLIENT", "MARKET", 25)
			.field(262, "MDREQ001") // MDReqID
			.field(263, "1") // SubscriptionRequestType
			.field(264, "20") // MarketDepth
			.field(267, "2") // NoMDEntryTypes
			.field(269, "0") // MDEntryType - Bid
			.field(269, "1") // MDEntryType - Offer
			.field(146, "1") // NoRelatedSym
			.symbol("AAPL")
			.build(),
	]
}

// Benchmark message creation using builder pattern
fn bench_message_creation(c: &mut Criterion) {
	let mut group = c.benchmark_group("message_creation");

	group.bench_function("heartbeat", |b| {
		b.iter(|| {
			black_box(
				FixMessage::builder(
					black_box(MsgType::Heartbeat),
					black_box("SENDER"),
					black_box("TARGET"),
					black_box(1),
				)
				.build(),
			)
		})
	});

	group.bench_function("simple_new_order", |b| {
		b.iter(|| {
			black_box(
				FixMessage::builder(
					black_box(MsgType::NewOrderSingle),
					black_box("CLIENT"),
					black_box("BROKER"),
					black_box(100),
				)
				.cl_ord_id(black_box("ORDER123"))
				.symbol(black_box("AAPL"))
				.side(black_box(Side::Buy))
				.order_qty(black_box(100.0))
				.price(black_box(150.25))
				.build(),
			)
		})
	});

	group.bench_function("complex_new_order", |b| {
		b.iter(|| {
			black_box(
				FixMessage::builder(
					black_box(MsgType::NewOrderSingle),
					black_box("TESTBUY3"),
					black_box("TESTSELL3"),
					black_box(972),
				)
				.sending_time(black_box(datetime!(2019-02-06 16:25:10.403 UTC)))
				.cl_ord_id(black_box("14163685067084226997921"))
				.symbol(black_box("AAPL"))
				.side(black_box(Side::Buy))
				.order_qty(black_box(100.0))
				.field(black_box(207), black_box("TO"))
				.field(black_box(6000), black_box("TEST1234"))
				.build(),
			)
		})
	});

	group.bench_function("execution_report", |b| {
		b.iter(|| {
			black_box(
				FixMessage::builder(
					black_box(MsgType::ExecutionReport),
					black_box("BROKER"),
					black_box("CLIENT"),
					black_box(100),
				)
				.order_id(black_box("ORDER001"))
				.cl_ord_id(black_box("CLIENT001"))
				.exec_id(black_box("EXEC001"))
				.exec_type(black_box("F"))
				.ord_status(black_box(OrdStatus::Filled))
				.symbol(black_box("MSFT"))
				.side(black_box(Side::Buy))
				.order_qty(black_box(500.0))
				.last_px(black_box(155.75))
				.last_qty(black_box(500.0))
				.cum_qty(black_box(500.0))
				.leaves_qty(black_box(0.0))
				.avg_px(black_box(155.75))
				.build(),
			)
		})
	});

	group.finish();
}

// Benchmark serialization to FIX string
fn bench_serialization(c: &mut Criterion) {
	let messages = create_sample_messages();
	let mut group = c.benchmark_group("serialization");

	for (i, message) in messages.iter().enumerate() {
		group.bench_with_input(BenchmarkId::new("to_fix_string", i), message, |b, msg| {
			b.iter(|| black_box(msg.to_fix_string()))
		});
	}

	group.finish();
}

// Benchmark parsing from FIX string
fn bench_parsing(c: &mut Criterion) {
	let mut group = c.benchmark_group("parsing");

	for (i, fix_string) in SAMPLE_FIX_STRINGS.iter().enumerate() {
		group.bench_with_input(BenchmarkId::new("from_fix_string", i), fix_string, |b, fix_str| {
			b.iter(|| black_box(FixMessage::from_fix_string(black_box(fix_str))))
		});
	}

	group.finish();
}

// Benchmark round-trip (serialize + parse)
fn bench_round_trip(c: &mut Criterion) {
	let messages = create_sample_messages();
	let mut group = c.benchmark_group("round_trip");

	for (i, message) in messages.iter().enumerate() {
		group.bench_with_input(BenchmarkId::new("serialize_parse", i), message, |b, msg| {
			b.iter(|| {
				let fix_string = black_box(msg.to_fix_string());
				black_box(FixMessage::from_fix_string(&fix_string))
			})
		});
	}

	group.finish();
}

// Benchmark enum parsing performance
fn bench_enum_parsing(c: &mut Criterion) {
	let mut group = c.benchmark_group("enum_parsing");

	let msg_types = ["0", "1", "D", "8", "F", "G", "V", "W"];
	let sides = ["1", "2"];
	let ord_statuses = ["0", "1", "2", "4", "8"];

	group.bench_function("msg_type_parsing", |b| {
		b.iter(|| {
			for msg_type_str in &msg_types {
				let _ = black_box(MsgType::from_str(black_box(msg_type_str)));
			}
		})
	});

	group.bench_function("side_parsing", |b| {
		b.iter(|| {
			for side_str in &sides {
				let _ = black_box(Side::from_str(black_box(side_str)));
			}
		})
	});

	group.bench_function("ord_status_parsing", |b| {
		b.iter(|| {
			for status_str in &ord_statuses {
				let _ = black_box(OrdStatus::from_str(black_box(status_str)));
			}
		})
	});

	group.finish();
}

// Benchmark field access and manipulation
fn bench_field_operations(c: &mut Criterion) {
	let mut group = c.benchmark_group("field_operations");

	let mut message = FixMessage::builder(MsgType::NewOrderSingle, "CLIENT", "BROKER", 100)
		.cl_ord_id("ORDER123")
		.symbol("AAPL")
		.side(Side::Buy)
		.order_qty(100.0)
		.price(150.25)
		.build();

	group.bench_function("set_custom_field", |b| {
		b.iter(|| {
			black_box(message.set_field(black_box(9999), black_box("TEST_VALUE")));
		})
	});

	group.bench_function("get_custom_field", |b| {
		message.set_field(9999, "TEST_VALUE");
		b.iter(|| {
			black_box(message.get_field(black_box(9999)));
		})
	});

	group.bench_function("validation", |b| {
		b.iter(|| {
			black_box(message.is_valid());
		})
	});

	group.finish();
}

// Benchmark different message sizes
fn bench_message_sizes(c: &mut Criterion) {
	let mut group = c.benchmark_group("message_sizes");

	// Small message (Heartbeat)
	let small_msg = FixMessage::builder(MsgType::Heartbeat, "A", "B", 1).build();

	// Medium message (New Order Single)
	let medium_msg = FixMessage::builder(MsgType::NewOrderSingle, "CLIENT", "BROKER", 100)
		.cl_ord_id("ORDER123")
		.symbol("AAPL")
		.side(Side::Buy)
		.order_qty(100.0)
		.price(150.25)
		.build();

	// Large message (with many custom fields)
	let mut large_msg = FixMessage::builder(MsgType::ExecutionReport, "BROKER", "CLIENT", 100)
		.order_id("ORDER001")
		.cl_ord_id("CLIENT001")
		.exec_id("EXEC001")
		.exec_type("F")
		.ord_status(OrdStatus::Filled)
		.symbol("MSFT")
		.side(Side::Buy)
		.order_qty(500.0)
		.last_px(155.75)
		.last_qty(500.0)
		.cum_qty(500.0)
		.leaves_qty(0.0)
		.avg_px(155.75)
		.build();

	// Add many custom fields to make it large
	for i in 5000..5050 {
		large_msg.set_field(i, format!("CUSTOM_FIELD_{}", i));
	}

	group.bench_function("small_message_serialization", |b| b.iter(|| black_box(small_msg.to_fix_string())));

	group.bench_function("medium_message_serialization", |b| b.iter(|| black_box(medium_msg.to_fix_string())));

	group.bench_function("large_message_serialization", |b| b.iter(|| black_box(large_msg.to_fix_string())));

	let small_fix = small_msg.to_fix_string();
	let medium_fix = medium_msg.to_fix_string();
	let large_fix = large_msg.to_fix_string();

	group.bench_function("small_message_parsing", |b| {
		b.iter(|| black_box(FixMessage::from_fix_string(black_box(&small_fix))))
	});

	group.bench_function("medium_message_parsing", |b| {
		b.iter(|| black_box(FixMessage::from_fix_string(black_box(&medium_fix))))
	});

	group.bench_function("large_message_parsing", |b| {
		b.iter(|| black_box(FixMessage::from_fix_string(black_box(&large_fix))))
	});

	group.finish();
}

// Benchmark memory allocation patterns
fn bench_memory_patterns(c: &mut Criterion) {
	let mut group = c.benchmark_group("memory_patterns");

	// Benchmark creating many messages in sequence
	group.bench_function("batch_message_creation", |b| {
		b.iter(|| {
			let mut messages = Vec::new();
			for i in 0..100 {
				let msg = FixMessage::builder(
					black_box(MsgType::NewOrderSingle),
					black_box("CLIENT"),
					black_box("BROKER"),
					black_box(i),
				)
				.cl_ord_id(black_box(format!("ORDER_{}", i)))
				.symbol(black_box("AAPL"))
				.side(black_box(Side::Buy))
				.order_qty(black_box(100.0))
				.build();
				messages.push(msg);
			}
			black_box(messages);
		})
	});

	// Benchmark reusing builder
	group.bench_function("builder_reuse_pattern", |b| {
		b.iter(|| {
			let mut messages = Vec::new();
			for i in 0..100 {
				let msg = FixMessage::builder(
					black_box(MsgType::NewOrderSingle),
					black_box("CLIENT"),
					black_box("BROKER"),
					black_box(i),
				)
				.cl_ord_id(black_box(format!("ORDER_{}", i)))
				.symbol(black_box("AAPL"))
				.side(black_box(Side::Buy))
				.order_qty(black_box(100.0))
				.build();
				messages.push(msg);
			}
			black_box(messages);
		})
	});

	group.finish();
}

criterion_group!(
	benches,
	bench_message_creation,
	bench_serialization,
	bench_parsing,
	bench_round_trip,
	bench_enum_parsing,
	bench_field_operations,
	bench_message_sizes,
	bench_memory_patterns
);

criterion_main!(benches);
