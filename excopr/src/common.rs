use std::{
    fmt,
    sync::{Arc, Mutex, RwLock},
};

use crate::{
    config::Config, error, feeder, field::Field, group::Group, tree::Element, value::Value,
};
/// Common traits

pub trait Help: Named + fmt::Debug {
    fn help(&self, parents: Vec<Arc<RwLock<dyn Config>>>) -> String;
}

pub trait Description {
    fn description(&self) -> Option<String>;
}

pub trait Named: fmt::Debug {
    fn name(&self) -> String;
}

pub trait Members {
    fn members(&self) -> &[Arc<Mutex<Element>>];
}

pub trait Node {
    fn elements(&self) -> Vec<Arc<Mutex<Element>>>;
    fn groups(&self) -> Vec<Arc<RwLock<dyn Group>>>;
}

pub trait Values {
    fn values(&self) -> Vec<Value>;
    fn append(&mut self, feeder: &str, value: String);
    fn add_feeder_matches(
        &mut self,
        feeder_name: &str,
        feeder_match: Arc<Mutex<dyn feeder::Matches>>,
    ) -> Result<(), Arc<Mutex<dyn error::Setup>>>;
    fn get_feeder_matches(&mut self, feeder_name: &str) -> Option<Arc<Mutex<dyn feeder::Matches>>>;
    fn all_feeder_matches(&mut self) -> Vec<Arc<Mutex<dyn feeder::Matches>>>;
}

pub trait AsValues {
    fn as_values(&mut self) -> &mut dyn Values;
}

pub trait FieldContainer {
    fn add_field(self, field: Arc<RwLock<dyn Field>>) -> Result<Self, Arc<Mutex<dyn error::Setup>>>
    where
        Self: Sized;
}
