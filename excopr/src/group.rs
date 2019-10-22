use std::sync::{Arc, Mutex};

use crate::configuration::{Members, Named};

pub trait Group: Named + /*Help +*/ Members {}

impl Named for Arc<Mutex<dyn Group>> {
    fn name(&self) -> String {
        self.lock().unwrap().name()
    }

    fn help(&self, indentation: usize, expand: bool) -> String {
        self.lock().unwrap().help(indentation, expand)
    }
}
