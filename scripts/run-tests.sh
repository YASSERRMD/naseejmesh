#!/bin/bash
# NaseejMesh Test Runner Script
# Usage: ./scripts/run-tests.sh [unit|integration|all]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

echo_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

echo_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

run_unit_tests() {
    echo_info "Running unit tests..."
    cargo test --all --lib -- --nocapture
}

run_integration_tests() {
    echo_info "Starting test infrastructure..."
    docker-compose -f docker-compose.test.yml up -d surrealdb mock-backend mqtt-broker

    echo_info "Waiting for services to be ready..."
    sleep 5

    echo_info "Running integration tests..."
    SURREAL_URL=ws://localhost:8000 \
    MOCK_BACKEND_URL=http://localhost:9080 \
    MQTT_BROKER_URL=mqtt://localhost:1883 \
    cargo test --all -- --nocapture --test-threads=1

    echo_info "Stopping test infrastructure..."
    docker-compose -f docker-compose.test.yml down
}

run_docker_tests() {
    echo_info "Building test image..."
    docker-compose -f docker-compose.test.yml build test-runner

    echo_info "Running tests in Docker..."
    docker-compose -f docker-compose.test.yml up --abort-on-container-exit test-runner

    echo_info "Cleaning up..."
    docker-compose -f docker-compose.test.yml down
}

run_security_tests() {
    echo_info "Running security-focused tests..."
    cargo test -p naseej-security -- --nocapture
    cargo test -p naseej-test-harness scenarios -- --nocapture
}

run_performance_tests() {
    echo_info "Running performance benchmarks..."
    cargo test -p naseej-test-harness test_waf_performance -- --nocapture
    cargo test -p naseej-test-harness test_rate_limiter_performance -- --nocapture
}

show_help() {
    echo "NaseejMesh Test Runner"
    echo ""
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  unit          Run unit tests only"
    echo "  integration   Run integration tests with Docker services"
    echo "  docker        Run all tests in Docker container"
    echo "  security      Run security-focused tests"
    echo "  performance   Run performance benchmarks"
    echo "  all           Run all tests"
    echo "  help          Show this help message"
}

case "${1:-all}" in
    unit)
        run_unit_tests
        ;;
    integration)
        run_integration_tests
        ;;
    docker)
        run_docker_tests
        ;;
    security)
        run_security_tests
        ;;
    performance)
        run_performance_tests
        ;;
    all)
        run_unit_tests
        run_security_tests
        run_performance_tests
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        echo_error "Unknown command: $1"
        show_help
        exit 1
        ;;
esac

echo_info "Tests completed!"
