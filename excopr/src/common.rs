use std::sync::{Arc, Mutex};

use crate::{error, feeder, field::Field, group::Group, tree::Element, value::Value};
/// Common traits

pub trait Named {
    fn name(&self) -> String;
    fn help(&self, indentation: usize, expand: bool) -> String;
}

pub trait Members {
    fn members(&self) -> &[Arc<Mutex<Element>>];
}

pub trait Node {
    fn elements(&self) -> Vec<Arc<Mutex<Element>>>;
    fn groups(&self) -> Vec<Arc<Mutex<dyn Group>>>;
}

pub trait Values {
    fn values(&self) -> Vec<Value>;
    fn append(&mut self, feeder: &str, value: String);
    fn add_feeder_matches(
        &mut self,
        feeder_name: &str,
        feeder_match: Arc<Mutex<dyn feeder::Matches>>,
    ) -> Result<(), error::Config>;
    fn feeder_matches(&mut self, feeder_name: &str) -> Option<Arc<Mutex<dyn feeder::Matches>>>;
}

pub trait FieldContainer {
    fn add_field(self, field: Arc<Mutex<dyn Field>>) -> Result<Self, error::Config>
    where
        Self: Sized;
}
