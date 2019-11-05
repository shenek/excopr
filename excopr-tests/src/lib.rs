pub use excopr::{
    error::{self, NewRun, NewSetup},
    AsValues, Config, Configuration, Description, Element, ElementConverter, Feeder, FeederMatch,
    FeederMatches, Field, FieldContainer, Group, Help, Members, Named, Node, Value, Values,
};
use std::{
    collections::HashMap,
    fmt,
    sync::{Arc, Mutex, RwLock},
};

#[derive(Debug)]
pub struct FakeConfig {
    pub name: String,
    pub elements: Vec<Arc<Mutex<Element>>>,
    pub groups: Vec<Arc<RwLock<dyn Group>>>,
    pub values: Vec<Value>,
    pub feeder_matches: Vec<(String, Arc<Mutex<dyn FeederMatches>>)>,
    pub description: Option<String>,
}

#[derive(Debug)]
pub struct FakeGroup {
    pub name: String,
    pub members: Vec<Arc<Mutex<Element>>>,
    pub description: Option<String>,
}

#[derive(Debug)]
pub struct FakeField {
    pub name: String,
    pub values: Vec<Value>,
    pub feeder_matches: Vec<(String, Arc<Mutex<dyn FeederMatches>>)>,
    pub description: Option<String>,
}

#[derive(Debug)]
pub struct FakeFeeder {
    pub name: String,
    pub map: HashMap<String, String>,
    matches: Vec<Arc<Mutex<FakeMatch>>>,
}

#[derive(Clone, Debug)]
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

#[derive(Debug)]
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
    fn groups(&self) -> Vec<Arc<RwLock<dyn Group>>> {
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
    ) -> Result<(), Arc<Mutex<dyn error::Setup>>> {
        let name = feeder_name.to_string();
        if self.feeder_matches.iter().any(|(n, _)| *n == name) {
            Err(Arc::new(Mutex::new(FakeSetupError::new(format!(
                "feeder name '{}' already exists",
                name
            )))))
        } else {
            self.feeder_matches
                .push((feeder_name.to_string(), feeder_matches));
            Ok(())
        }
    }

    fn get_feeder_matches(&mut self, feeder_name: &str) -> Option<Arc<Mutex<dyn FeederMatches>>> {
        let name = feeder_name.to_string();
        if let Some((_, matches)) = self
            .feeder_matches
            .iter()
            .filter(|(n, _)| *n == name)
            .next()
        {
            Some(matches.clone())
        } else {
            None
        }
    }

    fn all_feeder_matches(&mut self) -> Vec<Arc<Mutex<dyn FeederMatches>>> {
        self.feeder_matches.iter().map(|(_, m)| m.clone()).collect()
    }
}

impl AsValues for FakeConfig {
    fn as_values(&mut self) -> &mut dyn Values {
        self
    }
}

impl Config for FakeConfig {
    fn add_config(
        mut self,
        config: Arc<RwLock<dyn Config>>,
    ) -> Result<Self, Arc<Mutex<dyn error::Setup>>>
    where
        Self: Sized,
    {
        self.elements
            .push(Arc::new(Mutex::new(Element::Config(config))));
        Ok(self)
    }
    fn add_group(
        mut self,
        group: Arc<RwLock<dyn Group>>,
    ) -> Result<Self, Arc<Mutex<dyn error::Setup>>>
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
    fn add_field(
        mut self,
        field: Arc<RwLock<dyn Field>>,
    ) -> Result<Self, Arc<Mutex<dyn error::Setup>>>
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
    ) -> Result<(), Arc<Mutex<dyn error::Setup>>> {
        let name = feeder_name.to_string();
        if self.feeder_matches.iter().any(|(n, _)| *n == name) {
            Err(Arc::new(Mutex::new(FakeSetupError::new(format!(
                "feeder name '{}' already exists",
                name
            )))))
        } else {
            self.feeder_matches
                .push((feeder_name.to_string(), feeder_matches));
            Ok(())
        }
    }

    fn get_feeder_matches(&mut self, feeder_name: &str) -> Option<Arc<Mutex<dyn FeederMatches>>> {
        let name = feeder_name.to_string();
        if let Some((_, matches)) = self
            .feeder_matches
            .iter()
            .filter(|(n, _)| *n == name)
            .next()
        {
            Some(matches.clone())
        } else {
            None
        }
    }

    fn all_feeder_matches(&mut self) -> Vec<Arc<Mutex<dyn FeederMatches>>> {
        self.feeder_matches.iter().map(|(_, m)| m.clone()).collect()
    }
}

impl AsValues for FakeField {
    fn as_values(&mut self) -> &mut dyn Values {
        self
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

    fn process_matches(
        &mut self,
        element: &mut dyn Values,
    ) -> Result<(), Arc<Mutex<dyn error::Run>>> {
        if let Some(matches) = element.get_feeder_matches(self.name()) {
            for idx in matches.matches().iter().map(|e| e.id_in_feeder()) {
                if let Some(val) = self.map.get(&self.matches[idx].lock().unwrap().repr) {
                    element.append(self.name(), val.to_string());
                }
            }
        }
        Ok(())
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

#[derive(Debug)]
pub struct FakeSetupError {
    msg: String,
}

impl error::NewSetup for FakeSetupError {
    fn new(msg: String) -> Self {
        Self { msg }
    }
}
impl error::Setup for FakeSetupError {
    fn msg(&self) -> String {
        self.msg.clone()
    }
}
impl std::error::Error for FakeSetupError {}
impl fmt::Display for FakeSetupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct FakeRunError {
    node: Option<Arc<RwLock<dyn Help>>>,
    parents: Vec<Arc<RwLock<dyn Config>>>,
    msg: Option<String>,
}

impl error::NewRun for FakeRunError {
    fn new(
        node: Option<Arc<RwLock<dyn Help>>>,
        parents: Vec<Arc<RwLock<dyn Config>>>,
        msg: Option<String>,
    ) -> Self {
        Self { node, parents, msg }
    }
}

impl error::Run for FakeRunError {
    fn node(&self) -> Option<Arc<RwLock<dyn Help>>> {
        self.node.clone()
    }

    fn parents(&self) -> Vec<Arc<RwLock<dyn Config>>> {
        self.parents.clone()
    }

    fn msg(&self) -> Option<String> {
        self.msg.clone()
    }

    fn add_parent(&mut self, parent: Arc<RwLock<dyn Config>>) {
        self.parents.push(parent);
    }
}

impl std::error::Error for FakeRunError {}

impl fmt::Display for FakeRunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(msg) = self.msg.as_ref() {
            writeln!(f, "{}\n", msg)?;
        }
        if let Some(node) = self.node.clone() {
            writeln!(f, "{}", node.read().unwrap().help(self.parents.clone()))?;
        } else if let Some(node) = self.parents.first() {
            writeln!(
                f,
                "{}",
                node.read().unwrap().help(self.parents[1..].to_vec())
            )?;
        }
        Ok(())
    }
}
