//! Deterministic signal test injection utilities.

use std::time::Duration;

use super::{SignalManager, SignalManagerError};

/// Test helper that injects synthetic signals into a [`SignalManager`].
#[derive(Clone)]
pub struct SignalInjector {
    manager: SignalManager,
}

impl SignalInjector {
    /// Create a new injector bound to a manager.
    pub fn new(manager: &SignalManager) -> Self {
        Self {
            manager: manager.clone(),
        }
    }

    /// Inject a synthetic signal into the manager queue.
    pub fn inject(&self, signal: i32) -> Result<(), SignalManagerError> {
        self.manager.inject(signal)
    }

    /// Wait until [`SignalManager::listen`] enters its active loop.
    pub fn wait_for_listen(&self, timeout: Duration) -> Result<(), SignalManagerError> {
        self.manager.wait_for_listen(timeout)
    }

    /// Schedule [`SignalManager::stop`] to run after a delay.
    pub fn stop_after(&self, delay: Duration) {
        let manager = self.manager.clone();
        std::thread::spawn(move || {
            std::thread::sleep(delay);
            manager.stop();
        });
    }
}
