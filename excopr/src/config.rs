use std::sync::{Arc, Mutex, RwLock};

use crate::{
    common::{AsValues, Description, Help, Named, Node, Values},
    error, feeder,
    group::Group,
    tree::Element,
    value::Value,
};

pub trait Config: Named + Node + Values + Description + AsValues {
    /// Adds mutually exclusive configs
    fn add_config(self, configs: Arc<RwLock<dyn Config>>) -> Result<Self, error::Config>
    where
        Self: Sized;
    fn add_group(self, group: Arc<RwLock<dyn Group>>) -> Result<Self, error::Config>
    where
        Self: Sized;
}

impl Values for Arc<RwLock<dyn Config>> {
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

    fn get_feeder_matches(&mut self, feeder_name: &str) -> Option<Arc<Mutex<dyn feeder::Matches>>> {
        self.write().unwrap().get_feeder_matches(feeder_name)
    }

    fn all_feeder_matches(&mut self) -> Vec<Arc<Mutex<dyn feeder::Matches>>> {
        self.write().unwrap().all_feeder_matches()
    }
}

impl Named for Arc<RwLock<dyn Config>> {
    fn name(&self) -> String {
        self.read().unwrap().name()
    }
}

impl Help for dyn Config {
    fn help(&self) -> String {
        unimplemented!();
    }
}

impl Help for Arc<RwLock<dyn Config>> {
    fn help(&self) -> String {
        self.read().unwrap().help()
    }
}

impl Description for Arc<RwLock<dyn Config>> {
    fn description(&self) -> Option<String> {
        self.read().unwrap().description()
    }
}

impl Node for Arc<RwLock<dyn Config>> {
    fn elements(&self) -> Vec<Arc<Mutex<Element>>> {
        self.read().unwrap().elements()
    }

    fn groups(&self) -> Vec<Arc<RwLock<dyn Group>>> {
        self.read().unwrap().groups()
    }
}
