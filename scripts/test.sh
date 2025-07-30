#!/bin/bash

# Botanika Solana Programs - Test Script

set -e

echo "🧪 Running Botanika Solana Programs Tests..."

# Run all tests
echo "📋 Running unit tests..."
anchor test

# Run specific test suites
echo "🔍 Running enhanced tests..."
cargo test --test staking_enhanced

echo "🔗 Running integration tests..."
cargo test --test integration_tests

echo "✅ All tests completed successfully!" 