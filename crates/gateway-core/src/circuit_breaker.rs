//! Circuit Breaker Implementation
//!
//! Provides fault tolerance by stopping requests to failing upstreams.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;

/// Circuit Breaker State
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Closed,   // Normal operation
    Open,     // Failing, reject requests
    HalfOpen, // Testing recovery
}

/// Configuration for the circuit breaker
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures to trip the breaker
    pub failure_threshold: u64,
    /// Time window for counting failures
    pub failure_window: Duration,
    /// Time to wait before attempting recovery (Open -> HalfOpen)
    pub recovery_timeout: Duration,
    /// Number of successful requests to close the breaker
    pub success_threshold: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            failure_window: Duration::from_secs(60),
            recovery_timeout: Duration::from_secs(30),
            success_threshold: 3,
        }
    }
}

/// Circuit Breaker for a specific upstream
#[derive(Debug)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: RwLock<State>,
    failures: AtomicU64,
    successes: AtomicU64,
    last_failure_time: RwLock<Option<Instant>>,
    last_state_change: RwLock<Instant>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: RwLock::new(State::Closed),
            failures: AtomicU64::new(0),
            successes: AtomicU64::new(0),
            last_failure_time: RwLock::new(None),
            last_state_change: RwLock::new(Instant::now()),
        }
    }

    /// Check if a request is allowed
    pub fn allow_request(&self) -> bool {
        let state = *self.state.read();
        match state {
            State::Closed => true,
            State::Open => {
                // Check if recovery timeout has passed
                let last_change = *self.last_state_change.read();
                if last_change.elapsed() >= self.config.recovery_timeout {
                    // Transition to HalfOpen
                    self.transition(State::HalfOpen);
                    true
                } else {
                    false
                }
            }
            State::HalfOpen => {
                // Only allow one request at a time in HalfOpen?
                // For simplicity, we allow requests, but failure trips immediately
                true
            }
        }
    }

    /// Record a successful request
    pub fn record_success(&self) {
        let state = *self.state.read();
        match state {
            State::Closed => {
                // Reset failure count if window passed
                let last_failure = *self.last_failure_time.read();
                if let Some(time) = last_failure {
                    if time.elapsed() > self.config.failure_window {
                        self.failures.store(0, Ordering::Relaxed);
                    }
                }
            }
            State::HalfOpen => {
                let count = self.successes.fetch_add(1, Ordering::Relaxed) + 1;
                if count >= self.config.success_threshold {
                    self.transition(State::Closed);
                    self.failures.store(0, Ordering::Relaxed);
                    self.successes.store(0, Ordering::Relaxed);
                }
            }
            State::Open => {
                // Should not happen unless race condition, ignore
            }
        }
    }

    /// Record a failed request
    pub fn record_failure(&self) {
        let state = *self.state.read();
        match state {
            State::Closed => {
                let count = self.failures.fetch_add(1, Ordering::Relaxed) + 1;
                let mut last_fail = self.last_failure_time.write();
                *last_fail = Some(Instant::now());

                if count >= self.config.failure_threshold {
                    self.transition(State::Open);
                }
            }
            State::HalfOpen => {
                // Immediate fail back to Open
                self.transition(State::Open);
            }
            State::Open => {
                // Reset timer? Usually extend it
                let mut last_change = self.last_state_change.write();
                *last_change = Instant::now();
            }
        }
    }

    fn transition(&self, new_state: State) {
        let mut state = self.state.write();
        if *state != new_state {
            *state = new_state;
            *self.last_state_change.write() = Instant::now();
            
            // Reset counters on transition
            if new_state == State::Open {
                self.successes.store(0, Ordering::Relaxed);
            } else if new_state == State::Closed {
                self.failures.store(0, Ordering::Relaxed);
            }
        }
    }

    pub fn state(&self) -> State {
        *self.state.read()
    }
}

/// Registry for circuit breakers per route/upstream
pub struct CircuitBreakerRegistry {
    breakers: RwLock<std::collections::HashMap<String, Arc<CircuitBreaker>>>,
    default_config: CircuitBreakerConfig,
}

impl CircuitBreakerRegistry {
    pub fn new(default_config: CircuitBreakerConfig) -> Self {
        Self {
            breakers: RwLock::new(std::collections::HashMap::new()),
            default_config,
        }
    }

    pub fn get(&self, key: &str) -> Arc<CircuitBreaker> {
        // Fast path check
        {
            let breakers = self.breakers.read();
            if let Some(cb) = breakers.get(key) {
                return cb.clone();
            }
        }

        // Slow path create
        let mut breakers = self.breakers.write();
        // Double check
        if let Some(cb) = breakers.get(key) {
            return cb.clone();
        }

        let cb = Arc::new(CircuitBreaker::new(self.default_config.clone()));
        breakers.insert(key.to_string(), cb.clone());
        cb
    }
}
