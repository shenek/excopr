pub use excopr::{
    error, Config, Configuration, Description, Element, ElementConverter, Feeder, FeederMatch,
    FeederMatches, Field, FieldContainer, Group, Members, Named, Node, Value, Values,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct FakeConfig {
    pub name: String,
    pub elements: Vec<Arc<Mutex<Element>>>,
    pub groups: Vec<Arc<Mutex<dyn Group>>>,
    pub values: Vec<Value>,
    pub feeder_matches: HashMap<String, Arc<Mutex<dyn FeederMatches>>>,
    pub description: Option<String>,
}

pub struct FakeGroup {
    pub name: String,
    pub members: Vec<Arc<Mutex<Element>>>,
    pub description: Option<String>,
}

pub struct FakeField {
    pub name: String,
    pub values: Vec<Value>,
    pub feeder_matches: HashMap<String, Arc<Mutex<dyn FeederMatches>>>,
    pub description: Option<String>,
}

pub struct FakeFeeder {
    pub name: String,
    pub map: HashMap<String, String>,
    matches: Vec<Arc<Mutex<FakeMatch>>>,
}

#[derive(Clone)]
pub struct FakeMatch {
    id_in_feeder: usize,
    repr: String,
}

impl FeederMatch for FakeMatch {
    fn id_in_feeder(&self) -> usize {
        self.id_in_feeder
    }

    fn repr(&self) -> String {
        self.repr.clone()
    }
}

pub struct FakeMatches {
    matches: Vec<Arc<Mutex<dyn FeederMatch>>>,
}

impl FakeMatches {
    pub fn new(matches: Vec<Arc<Mutex<dyn FeederMatch>>>) -> Self {
        Self { matches }
    }
}

impl FeederMatches for FakeMatches {
    fn repr(&self) -> String {
        self.matches
            .iter()
            .map(|e| e.repr())
            .collect::<Vec<String>>()
            .join(",")
    }

    fn matches(&self) -> Vec<Arc<Mutex<dyn FeederMatch>>> {
        self.matches.clone()
    }

    fn add_match(&mut self, new_match: Arc<Mutex<dyn FeederMatch>>) {
        self.matches.push(new_match);
    }
}

impl Named for FakeConfig {
    fn name(&self) -> String {
        self.name.clone()
    }
}

impl Node for FakeConfig {
    fn elements(&self) -> Vec<Arc<Mutex<Element>>> {
        self.elements.clone()
    }
    fn groups(&self) -> Vec<Arc<Mutex<dyn Group>>> {
        self.groups.clone()
    }
}

impl Values for FakeConfig {
    fn values(&self) -> Vec<Value> {
        self.values.clone()
    }

    fn append(&mut self, feeder: &str, value: String) {
        self.values.push(Value::new(feeder.to_string(), value));
    }

    fn add_feeder_matches(
        &mut self,
        feeder_name: &str,
        feeder_matches: Arc<Mutex<dyn FeederMatches>>,
    ) -> Result<(), error::Config> {
        self.feeder_matches
            .insert(feeder_name.to_string(), feeder_matches);
        Ok(())
    }

    fn feeder_matches(&mut self, feeder_name: &str) -> Option<Arc<Mutex<dyn FeederMatches>>> {
        if let Some(matches) = self.feeder_matches.get(feeder_name) {
            Some(matches.clone())
        } else {
            None
        }
    }
}

impl Config for FakeConfig {
    fn add_config(mut self, config: Arc<Mutex<dyn Config>>) -> Result<Self, error::Config>
    where
        Self: Sized,
    {
        self.elements
            .push(Arc::new(Mutex::new(Element::Config(config))));
        Ok(self)
    }
    fn add_group(mut self, group: Arc<Mutex<dyn Group>>) -> Result<Self, error::Config>
    where
        Self: Sized,
    {
        self.groups.push(group);
        Ok(self)
    }
}

impl Description for FakeConfig {
    fn description(&self) -> Option<String> {
        self.description.clone()
    }
}

impl FieldContainer for FakeConfig {
    fn add_field(mut self, field: Arc<Mutex<dyn Field>>) -> Result<Self, error::Config>
    where
        Self: Sized,
    {
        self.elements
            .push(Arc::new(Mutex::new(Element::Field(field))));
        Ok(self)
    }
}

impl Members for FakeGroup {
    fn members(&self) -> &[Arc<Mutex<Element>>] {
        &self.members[..]
    }
}

impl Group for FakeGroup {}

impl Description for FakeGroup {
    fn description(&self) -> Option<String> {
        self.description.clone()
    }
}

impl Named for FakeGroup {
    fn name(&self) -> String {
        self.name.clone()
    }
}

impl Field for FakeField {}

impl Description for FakeField {
    fn description(&self) -> Option<String> {
        self.description.clone()
    }
}

impl Values for FakeField {
    fn values(&self) -> Vec<Value> {
        self.values.clone()
    }

    fn append(&mut self, feeder: &str, value: String) {
        self.values.push(Value::new(feeder.to_string(), value));
    }

    fn add_feeder_matches(
        &mut self,
        feeder_name: &str,
        feeder_matches: Arc<Mutex<dyn FeederMatches>>,
    ) -> Result<(), error::Config> {
        self.feeder_matches
            .insert(feeder_name.to_string(), feeder_matches);
        Ok(())
    }

    fn feeder_matches(&mut self, feeder_name: &str) -> Option<Arc<Mutex<dyn FeederMatches>>> {
        if let Some(matches) = self.feeder_matches.get(feeder_name) {
            Some(matches.clone())
        } else {
            None
        }
    }
}

impl Named for FakeField {
    fn name(&self) -> String {
        self.name.clone()
    }
}

impl Feeder for FakeFeeder {
    fn name(&self) -> &str {
        &self.name
    }

    fn process_matches(&mut self, element: Arc<Mutex<Element>>) {
        let mut unlocked = element.lock().unwrap();
        if let Some(matches) = unlocked.feeder_matches(self.name()) {
            for idx in matches.matches().iter().map(|e| e.id_in_feeder()) {
                if let Some(val) = self.map.get(&self.matches[idx].lock().unwrap().repr) {
                    unlocked.append(self.name(), val.to_string());
                }
            }
        }
    }
}

impl FakeFeeder {
    pub fn add_match(&mut self, match_name: &str) -> Arc<Mutex<dyn FeederMatch>> {
        let new_match = Arc::new(Mutex::new(FakeMatch {
            id_in_feeder: self.matches.len(),
            repr: match_name.to_string(),
        }));
        self.matches.push(new_match.clone());
        new_match
    }

    pub fn new(name: &str, map: HashMap<String, String>) -> Self {
        Self {
            name: name.to_string(),
            map,
            matches: vec![],
        }
    }
}
