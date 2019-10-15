pub use excopr::{
    configuration::{
        Config, Configuration, Element, Field, FieldContainer, Group, Members, Named, Node, Values,
    },
    error::Config as ConfigError,
    feeder::{self, Match},
    value::Value,
};
use std::{collections::HashMap, rc::Rc};

pub struct FakeConfig {
    pub name: String,
    pub elements: Vec<Element>,
    pub groups: Vec<Box<dyn Group>>,
    pub values: Vec<Value>,
    pub feeder_matches: HashMap<String, Rc<dyn feeder::Matches>>,
}

pub struct FakeGroup {
    pub name: String,
    pub members: Vec<String>,
}

pub struct FakeField {
    pub name: String,
    pub values: Vec<Value>,
    pub feeder_matches: HashMap<String, Rc<dyn feeder::Matches>>,
}

pub struct FakeFeeder {
    pub name: String,
    pub map: HashMap<String, String>,
    matches: Vec<FakeMatchFactory>,
}

#[derive(Clone)]
pub struct FakeMatch {
    id_in_feeder: usize,
    repr: String,
}

impl feeder::Match for FakeMatch {
    fn id_in_feeder(&self) -> usize {
        self.id_in_feeder
    }

    fn repr(&self) -> &str {
        &self.repr
    }
}

pub struct FakeMatchFactory {
    /// There can be any generic value to be matched
    value: String,
    id_in_feeder: usize,
}

impl FakeMatchFactory {
    fn new(id_in_feeder: usize, value: String) -> Self {
        Self {
            id_in_feeder,
            value,
        }
    }
}

pub struct FakeMatches {
    matches: Vec<Rc<dyn feeder::Match>>,
}

impl feeder::Matches for FakeMatches {
    fn repr(&self) -> String {
        self.matches
            .iter()
            .map(|e| e.repr())
            .collect::<Vec<&str>>()
            .join(",")
    }

    fn matches(&self) -> Vec<Rc<dyn feeder::Match>> {
        self.matches
            .iter()
            .map(|e| e.clone() as Rc<dyn feeder::Match>)
            .collect()
    }

    fn add_match(&mut self, new_match: Rc<dyn feeder::Match>) {
        self.matches.push(new_match);
    }
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

    fn add_feeder_matches(
        &mut self,
        feeder_name: &str,
        feeder_matches: Rc<dyn feeder::Matches>,
    ) -> Result<(), ConfigError> {
        self.feeder_matches
            .insert(feeder_name.to_string(), feeder_matches);
        Ok(())
    }

    fn feeder_matches(&mut self, feeder_name: &str) -> Option<Rc<dyn feeder::Matches>> {
        self.feeder_matches.get(feeder_name).map(|e| e.clone())
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

    fn add_feeder_matches(
        &mut self,
        feeder_name: &str,
        feeder_matches: Rc<dyn feeder::Matches>,
    ) -> Result<(), ConfigError> {
        self.feeder_matches
            .insert(feeder_name.to_string(), feeder_matches);
        Ok(())
    }

    fn feeder_matches(&mut self, feeder_name: &str) -> Option<Rc<dyn feeder::Matches>> {
        self.feeder_matches.get(feeder_name).cloned()
    }
}

impl Named for FakeField {
    fn name(&self) -> &str {
        &self.name
    }
}

impl feeder::Feeder for FakeFeeder {
    fn name(&self) -> &str {
        &self.name
    }

    fn process_matches(&mut self, element: &mut Element) {
        if let Some(matches) = element.feeder_matches(self.name()) {
            for idx in matches.matches().iter().map(|e| e.id_in_feeder()) {
                if let Some(val) = self.map.get(&self.matches[idx].value) {
                    element.append(self.name(), val.to_string());
                }
            }
        }
    }
}
