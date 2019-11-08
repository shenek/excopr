use std::sync::{Arc, Mutex, RwLock};

use crate::{
    common::{AsValues, Description, Help, Named, Values},
    config::Config,
    error, feeder,
    value::Value,
};

pub trait Field: Named + Values + Description + AsValues {}

impl Help for Arc<RwLock<dyn Field>> {
    fn help(&self, parents: Vec<Arc<RwLock<dyn Config>>>) -> String {
        self.read().unwrap().help(parents)
    }
}

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
    ) -> Result<(), Arc<Mutex<dyn error::Setup>>> {
        self.write()
            .unwrap()
            .add_feeder_matches(feeder_name, feeder_match)
    }

    fn get_feeder_matches(&mut self, feeder_name: &str) -> Option<Arc<Mutex<dyn feeder::Matches>>> {
        self.write().unwrap().get_feeder_matches(feeder_name)
    }

    fn all_feeder_matches(&mut self) -> Vec<Arc<Mutex<dyn feeder::Matches>>> {
        self.write().unwrap().all_feeder_matches()
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
