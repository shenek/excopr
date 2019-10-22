use std::sync::{Arc, Mutex};

use crate::{
    common::{Help, Named, Node, Values},
    error, feeder,
    group::Group,
    tree::Element,
    value::Value,
};

pub trait Config: Named + Help + Node + Values {
    /// Adds mutually exclusive configs
    fn add_config(self, configs: Arc<Mutex<dyn Config>>) -> Result<Self, error::Config>
    where
        Self: Sized;
    fn add_group(self, group: Arc<Mutex<dyn Group>>) -> Result<Self, error::Config>
    where
        Self: Sized;
}

impl Values for Arc<Mutex<dyn Config>> {
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

impl Named for Arc<Mutex<dyn Config>> {
    fn name(&self) -> String {
        self.lock().unwrap().name()
    }
}

impl Help for Arc<Mutex<dyn Config>> {
    fn help(&self) -> String {
        self.lock().unwrap().help()
    }
}

impl Node for Arc<Mutex<dyn Config>> {
    fn elements(&self) -> Vec<Arc<Mutex<Element>>> {
        self.lock().unwrap().elements()
    }

    fn groups(&self) -> Vec<Arc<Mutex<dyn Group>>> {
        self.lock().unwrap().groups()
    }
}
