#!/bin/bash

# FIX Benchmarks Runner
# This script runs all benchmarks and generates reports

set -e

echo "🚀 Running FIX Benchmarks"
echo "=================================="

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust and Cargo."
    exit 1
fi

# Create benchmarks output directory
mkdir -p target/criterion

echo "📊 Running core FIX message benchmarks..."
cargo bench --bench fix_benchmarks

echo ""
echo "📈 Benchmark reports generated!"
echo "HTML reports available at:"
echo "  - target/criterion/report/index.html"
echo ""

# Check if we can open the report automatically, Linux first
if command -v xdg-open &> /dev/null; then
    echo "🌐 Opening benchmark report in browser..."
    xdg-open target/criterion/report/index.html
# macOS branch
elif command -v open &> /dev/null; then
    echo "🌐 Opening benchmark report in browser..."
    open target/criterion/report/index.html
else
    echo "💡 To view the HTML report, open target/criterion/report/index.html in your browser"
fi

echo ""
echo "✅ Benchmarks complete!"
echo ""
echo "📝 Quick analysis tips:"
echo "  - Look for 'message_creation' benchmarks to see constructor overhead"
echo "  - Check 'serialization' vs 'parsing' performance"
echo "  - Monitor 'memory_allocation' benchmarks for potential optimizations"
