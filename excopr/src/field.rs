use std::sync::{Arc, Mutex, RwLock};

use crate::{
    common::{AsValues, Description, Named, Values},
    error, feeder,
    value::Value,
};

pub trait Field: Named + Values + Description + AsValues {}

impl Values for Arc<RwLock<dyn Field>> {
    fn values(&self) -> Vec<Value> {
        self.read().unwrap().values()
    }

    fn append(&mut self, feeder: &str, value: String) {
        self.write().unwrap().append(feeder, value)
    }

    fn add_feeder_matches(
        &mut self,
        feeder_name: &str,
        feeder_match: Arc<Mutex<dyn feeder::Matches>>,
    ) -> Result<(), error::Config> {
        self.write()
            .unwrap()
            .add_feeder_matches(feeder_name, feeder_match)
    }

    fn feeder_matches(&mut self, feeder_name: &str) -> Option<Arc<Mutex<dyn feeder::Matches>>> {
        self.write().unwrap().feeder_matches(feeder_name)
    }
}

impl Named for Arc<RwLock<dyn Field>> {
    fn name(&self) -> String {
        self.read().unwrap().name()
    }
}

impl Description for Arc<RwLock<dyn Field>> {
    fn description(&self) -> Option<String> {
        self.read().unwrap().description()
    }
}
