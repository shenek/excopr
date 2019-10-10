pub use excopr::{
    configuration::{
        Config, Configuration, Element, Field, FieldContainer, Group, Members, Named, Node, Values,
    },
    error::Config as ConfigError,
    feeder::Feeder,
    value::Value,
};
use std::collections::HashMap;

pub struct FakeConfig {
    pub name: String,
    pub elements: Vec<Element>,
    pub groups: Vec<Box<dyn Group>>,
    pub values: Vec<Value>,
    pub feeder_matches: HashMap<String, Vec<String>>,
}

pub struct FakeGroup {
    pub name: String,
    pub members: Vec<String>,
}

pub struct FakeField {
    pub name: String,
    pub values: Vec<Value>,
    pub feeder_matches: HashMap<String, Vec<String>>,
}

pub struct FakeFeeder {
    pub name: String,
    pub map: HashMap<String, String>,
}

impl Named for FakeConfig {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Node for FakeConfig {
    fn elements(&self) -> &[Element] {
        self.elements.as_ref()
    }
    fn groups(&self) -> &[Box<dyn Group>] {
        &self.groups[..]
    }
    fn elements_mut(&mut self) -> &mut Vec<Element> {
        &mut self.elements
    }
}

impl Values for FakeConfig {
    fn as_values(&mut self) -> &mut dyn Values {
        self
    }

    fn values(&self) -> &[Value] {
        &self.values
    }

    fn append(&mut self, feeder: &str, value: String) {
        self.values.push(Value::new(feeder.to_string(), value));
    }

    fn add_feeder_match(&mut self, feeder: &str, key: String) -> Result<(), ConfigError> {
        self.feeder_matches
            .entry(feeder.to_string())
            .or_default()
            .push(key);
        Ok(())
    }

    fn feeder_matches(&self, feeder: &str) -> Option<&[String]> {
        let res = self.feeder_matches.get(feeder)?;
        Some(res)
    }
}

impl Config for FakeConfig {
    fn add_config(mut self, config: Box<dyn Config>) -> Result<Self, ConfigError>
    where
        Self: Sized,
    {
        self.elements.push(Element::Config(config));
        Ok(self)
    }
    fn add_group(mut self, group: Box<dyn Group>) -> Result<Self, ConfigError>
    where
        Self: Sized,
    {
        self.groups.push(group);
        Ok(self)
    }
}

impl FieldContainer for FakeConfig {
    fn add_field(mut self, field: Box<dyn Field>) -> Result<Self, ConfigError>
    where
        Self: Sized,
    {
        self.elements.push(Element::Field(field));
        Ok(self)
    }
}

impl Members for FakeGroup {
    fn members(&self) -> &[String] {
        &self.members
    }
}

impl Group for FakeGroup {}

impl Named for FakeGroup {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Field for FakeField {}

impl Values for FakeField {
    fn as_values(&mut self) -> &mut dyn Values {
        self
    }

    fn values(&self) -> &[Value] {
        &self.values
    }

    fn append(&mut self, feeder: &str, value: String) {
        self.values.push(Value::new(feeder.to_string(), value));
    }

    fn add_feeder_match(&mut self, feeder: &str, key: String) -> Result<(), ConfigError> {
        self.feeder_matches
            .entry(feeder.to_string())
            .or_default()
            .push(key);
        Ok(())
    }

    fn feeder_matches(&self, feeder: &str) -> Option<&[String]> {
        let res = self.feeder_matches.get(feeder)?;
        Some(res)
    }
}

impl Named for FakeField {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Feeder for FakeFeeder {
    fn name(&self) -> &str {
        &self.name
    }

    fn process(&mut self, element: &mut Element) -> Result<(), ConfigError> {
        match element {
            Element::Config(config) => {
                for m in config.feeder_matches(self.name()).unwrap_or(&[]).to_vec() {
                    if let Some(val) = self.map.get(&m) {
                        config.append(self.name(), val.to_string());
                    }
                }
            }
            Element::Field(field) => {
                for m in field.feeder_matches(self.name()).unwrap_or(&[]).to_vec() {
                    if let Some(val) = self.map.get(&m) {
                        (*field).append(self.name(), val.to_string());
                    }
                }
            }
        };
        self.dfs(element)?;

        Ok(())
    }
}
