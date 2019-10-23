use std::sync::{Arc, Mutex};

use crate::common::{Description, Members, Named};

pub trait Group: Named + Members + Description {}

impl Named for Arc<Mutex<dyn Group>> {
    fn name(&self) -> String {
        self.lock().unwrap().name()
    }
}

impl Description for Arc<Mutex<dyn Group>> {
    fn description(&self) -> Option<String> {
        self.lock().unwrap().description()
    }
}
