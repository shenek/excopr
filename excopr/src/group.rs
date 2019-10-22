use std::sync::{Arc, Mutex};

use crate::common::{Help, Members, Named};

pub trait Group: Named + Help + Members {}

impl Named for Arc<Mutex<dyn Group>> {
    fn name(&self) -> String {
        self.lock().unwrap().name()
    }
}

impl Help for Arc<Mutex<dyn Group>> {
    fn help(&self) -> String {
        self.lock().unwrap().help()
    }
}
