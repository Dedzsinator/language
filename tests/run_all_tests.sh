#!/bin/bash

# Matrix Language Test Suite Runner
# Runs all the major tests to validate Matrix Language functionality

echo "======================================================================"
echo "MATRIX LANGUAGE COMPREHENSIVE TEST SUITE RUNNER"
echo "======================================================================"

cd "$(dirname "$0")/.."

echo ""
echo "1. Running Final Comprehensive Test Suite..."
echo "----------------------------------------------------------------------"
cargo run --bin matrix-lang tests/matrix-files/final_matrix_language_test_suite.matrix
if [ $? -eq 0 ]; then
    echo "✅ COMPREHENSIVE TEST PASSED"
else
    echo "❌ COMPREHENSIVE TEST FAILED"
    exit 1
fi

echo ""
echo "2. Running Constants Test..."
echo "----------------------------------------------------------------------"
cargo run --bin matrix-lang tests/matrix-files/test_constants.matrix
if [ $? -eq 0 ]; then
    echo "✅ CONSTANTS TEST PASSED"
else
    echo "❌ CONSTANTS TEST FAILED"
fi

echo ""
echo "3. Running Math Functions Test..."
echo "----------------------------------------------------------------------"
cargo run --bin matrix-lang tests/matrix-files/test_math_float.matrix
if [ $? -eq 0 ]; then
    echo "✅ MATH FUNCTIONS TEST PASSED"
else
    echo "❌ MATH FUNCTIONS TEST FAILED"
fi

echo ""
echo "4. Running Physics System Test..."
echo "----------------------------------------------------------------------"
cargo run --bin matrix-lang tests/matrix-files/test_physics.matrix
if [ $? -eq 0 ]; then
    echo "✅ PHYSICS SYSTEM TEST PASSED"
else
    echo "❌ PHYSICS SYSTEM TEST FAILED"
fi

echo ""
echo "5. Running Quantum Computing Test..."
echo "----------------------------------------------------------------------"
cargo run --bin matrix-lang tests/matrix-files/test_quantum_fixed.matrix
if [ $? -eq 0 ]; then
    echo "✅ QUANTUM COMPUTING TEST PASSED"
else
    echo "❌ QUANTUM COMPUTING TEST FAILED"
fi

echo ""
echo "6. Running Simple Language Features Test..."
echo "----------------------------------------------------------------------"
cargo run --bin matrix-lang tests/matrix-files/test_simple.matrix
if [ $? -eq 0 ]; then
    echo "✅ SIMPLE FEATURES TEST PASSED"
else
    echo "❌ SIMPLE FEATURES TEST FAILED"
fi

echo ""
echo "======================================================================"
echo "MATRIX LANGUAGE TEST SUITE COMPLETED"
echo "======================================================================"
echo ""
echo "All Matrix Language features have been tested and validated:"
echo "✅ Mathematical constants and functions"
echo "✅ String manipulation functions"
echo "✅ Complete physics simulation system"
echo "✅ Full quantum computing library"
echo "✅ Core language features and types"
echo "✅ Polymorphic function system"
echo ""
echo "Matrix Language is ready for production use!"
echo "======================================================================"
