use std::sync::{Arc, Mutex};

use esp_idf_svc::hal::gpio::Level;

pub struct AppState {
    pub led: Level,
}

impl Default for AppState {
    fn default() -> Self {
        Self { led: Level::Low }
    }
}

impl AppState {
    pub fn shared(self) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(self))
    }
}
