use std::sync::{Arc, Mutex};

use crate::{
    common::{Description, Named, Values},
    error, feeder,
    value::Value,
};

pub trait Field: Named + Values + Description {}

impl Values for Arc<Mutex<dyn Field>> {
    fn values(&self) -> Vec<Value> {
        self.lock().unwrap().values()
    }

    fn append(&mut self, feeder: &str, value: String) {
        self.lock().unwrap().append(feeder, value)
    }

    fn add_feeder_matches(
        &mut self,
        feeder_name: &str,
        feeder_match: Arc<Mutex<dyn feeder::Matches>>,
    ) -> Result<(), error::Config> {
        self.lock()
            .unwrap()
            .add_feeder_matches(feeder_name, feeder_match)
    }

    fn feeder_matches(&mut self, feeder_name: &str) -> Option<Arc<Mutex<dyn feeder::Matches>>> {
        self.lock().unwrap().feeder_matches(feeder_name)
    }
}

impl Named for Arc<Mutex<dyn Field>> {
    fn name(&self) -> String {
        self.lock().unwrap().name()
    }
}

impl Description for Arc<Mutex<dyn Field>> {
    fn description(&self) -> Option<String> {
        self.lock().unwrap().description()
    }
}
