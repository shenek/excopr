use std::sync::{Arc, RwLock};

use crate::common::{Description, Members, Named};

pub trait Group: Named + Members + Description {}

impl Named for Arc<RwLock<dyn Group>> {
    fn name(&self) -> String {
        self.read().unwrap().name()
    }
}

impl Description for Arc<RwLock<dyn Group>> {
    fn description(&self) -> Option<String> {
        self.read().unwrap().description()
    }
}
