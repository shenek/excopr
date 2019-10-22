use std::sync::{Arc, Mutex};

use crate::{
    config::Config,
    error,
    feeder::{Feeder, Matches as FeederMatches},
    field::Field,
    group::Group,
    value::Value,
};

pub struct Builder {
    feeders: Vec<Box<dyn Feeder>>,
    root: Option<Arc<Mutex<Element>>>,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            feeders: Vec::default(),
            root: None,
        }
    }
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_feeder<F>(mut self, feeder: F) -> Result<Self, error::Config>
    where
        F: 'static + Feeder,
    {
        if self.feeders.iter().any(|f| f.name() == feeder.name()) {
            Err(error::Config::new(&format!(
                "Feeder '{}' already exists",
                feeder.name()
            )))
        } else {
            self.feeders.push(Box::new(feeder));
            Ok(self)
        }
    }

    pub fn set_root(mut self, root: Element) -> Self {
        self.root = Some(Arc::new(Mutex::new(root)));
        self
    }

    pub fn build(self) -> Result<Configuration, error::Config> {
        let root = self
            .root
            .ok_or_else(|| error::Config::new("No Configuration set"))?;
        for mut feeder in self.feeders {
            feeder.process(root.clone())?;
        }
        // TODO remove empty programs
        Ok(Configuration { root })
    }
}

pub struct Configuration {
    pub root: Arc<Mutex<Element>>,
}

impl Configuration {
    pub fn builder() -> Builder {
        Builder::new()
    }
}

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
        feeder_match: Arc<Mutex<dyn FeederMatches>>,
    ) -> Result<(), error::Config>;
    fn feeder_matches(&mut self, feeder_name: &str) -> Option<Arc<Mutex<dyn FeederMatches>>>;
}

pub trait FieldContainer {
    fn add_field(self, field: Arc<Mutex<dyn Field>>) -> Result<Self, error::Config>
    where
        Self: Sized;
}

pub enum Element {
    Config(Arc<Mutex<dyn Config>>),
    Field(Arc<Mutex<dyn Field>>),
}

impl Values for Element {
    fn values(&self) -> Vec<Value> {
        match self {
            Self::Config(config) => config.values(),
            Self::Field(field) => field.values(),
        }
    }

    fn append(&mut self, feeder: &str, value: String) {
        match self {
            Self::Config(config) => config.append(feeder, value),
            Self::Field(field) => field.append(feeder, value),
        }
    }

    fn add_feeder_matches(
        &mut self,
        feeder_name: &str,
        feeder_match: Arc<Mutex<dyn FeederMatches>>,
    ) -> Result<(), error::Config> {
        match self {
            Self::Config(config) => config.add_feeder_matches(feeder_name, feeder_match),
            Self::Field(field) => field.add_feeder_matches(feeder_name, feeder_match),
        }
    }

    fn feeder_matches(&mut self, feeder_name: &str) -> Option<Arc<Mutex<dyn FeederMatches>>> {
        match self {
            Self::Config(config) => config.feeder_matches(feeder_name),
            Self::Field(field) => field.feeder_matches(feeder_name),
        }
    }
}

impl Named for Element {
    fn name(&self) -> String {
        match self {
            Self::Config(config) => config.name(),
            Self::Field(field) => field.name(),
        }
    }

    fn help(&self, indentation: usize, expand: bool) -> String {
        match self {
            Self::Config(config) => config.help(indentation, expand),
            Self::Field(field) => field.help(indentation, expand),
        }
    }
}

impl Named for Arc<Mutex<Element>> {
    fn name(&self) -> String {
        self.lock().unwrap().name()
    }

    fn help(&self, indentation: usize, expand: bool) -> String {
        self.lock().unwrap().help(indentation, expand)
    }
}

pub trait ElementConverter {
    fn as_config(&mut self) -> Option<Arc<Mutex<dyn Config>>>;
    fn as_field(&mut self) -> Option<Arc<Mutex<dyn Field>>>;
}

impl ElementConverter for Element {
    fn as_config(&mut self) -> Option<Arc<Mutex<dyn Config>>> {
        match self {
            Self::Config(config) => Some(config.clone()),
            _ => None,
        }
    }
    fn as_field(&mut self) -> Option<Arc<Mutex<dyn Field>>> {
        match self {
            Self::Field(field) => Some(field.clone()),
            _ => None,
        }
    }
}

impl ElementConverter for Arc<Mutex<Element>> {
    fn as_config(&mut self) -> Option<Arc<Mutex<dyn Config>>> {
        self.lock().unwrap().as_config()
    }
    fn as_field(&mut self) -> Option<Arc<Mutex<dyn Field>>> {
        self.lock().unwrap().as_field()
    }
}

#[cfg(test)]
mod tests {
    use excopr_tests::{
        Config, Configuration, Element, ElementConverter, FakeConfig, FakeFeeder, FakeField,
        FakeGroup, FakeMatches, Named, Node, Values,
    };
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    #[test]
    fn impl_test() {
        let builder = Configuration::builder();
        let root = FakeConfig {
            name: "root".to_string(),
            elements: vec![],
            groups: vec![],
            values: vec![],
            feeder_matches: HashMap::new(),
        };
        let element = Arc::new(Mutex::new(Element::Field(Arc::new(Mutex::new(
            FakeField {
                name: "Fld".to_string(),
                values: vec![],
                feeder_matches: HashMap::new(),
            },
        )))));
        let subconfig = FakeConfig {
            name: "sub".to_string(),
            elements: vec![element.clone()],
            groups: vec![],
            values: vec![],
            feeder_matches: HashMap::new(),
        };
        let group = FakeGroup {
            name: "Grp".to_string(),
            members: vec![element],
        };
        let subconfig = subconfig.add_group(Arc::new(Mutex::new(group))).unwrap();
        let root = root.add_config(Arc::new(Mutex::new(subconfig))).unwrap();
        let configuration = builder
            .set_root(Element::Config(Arc::new(Mutex::new(root))))
            .build()
            .unwrap();
        if let Some(conf) = configuration.root.clone().as_config() {
            if let Some(subconf) = conf.elements()[0].as_config() {
                assert_eq!(subconf.name(), "sub");
                let group = subconf.groups()[0].clone();
                assert_eq!(group.name(), "Grp");
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn adding_feeders() {
        let builder = Configuration::builder();
        let builder = builder
            .add_feeder(FakeFeeder::new("test", HashMap::new()))
            .unwrap();
        assert!(builder
            .add_feeder(FakeFeeder::new("test", HashMap::new()))
            .is_err());
    }

    #[test]
    fn empty_builder() {
        assert!(Configuration::builder().build().is_err())
    }

    #[test]
    fn values() {
        let builder = Configuration::builder();
        let element = Arc::new(Mutex::new(Element::Field(Arc::new(Mutex::new(
            FakeField {
                name: "second".to_string(),
                values: vec![],
                feeder_matches: HashMap::new(),
            },
        )))));
        let mut root = FakeConfig {
            name: "first".to_string(),
            elements: vec![element],
            groups: vec![],
            values: vec![],
            feeder_matches: HashMap::new(),
        };

        let mut map = HashMap::new();
        map.insert("feeder_id_1".to_string(), "11111".to_string());
        map.insert("feeder_id_2".to_string(), "22222".to_string());

        let mut feeder = FakeFeeder::new("testing_feeder", map);

        root.add_feeder_matches(
            "testing_feeder",
            Arc::new(Mutex::new(FakeMatches::new(vec![
                feeder.add_match("feeder_id_1")
            ]))),
        )
        .unwrap();

        if let Some(fld) = &mut root.elements()[0].as_field() {
            fld.add_feeder_matches(
                "testing_feeder",
                Arc::new(Mutex::new(FakeMatches::new(vec![
                    feeder.add_match("feeder_id_2")
                ]))),
            )
            .unwrap();
        }

        let res = builder
            .add_feeder(feeder)
            .unwrap()
            .set_root(Element::Config(Arc::new(Mutex::new(root))))
            .build()
            .unwrap();

        if let Some(cfg) = res.root.clone().as_config() {
            assert_eq!(cfg.values()[0].feeder(), "testing_feeder");
            assert_eq!(cfg.values()[0].value::<u32>().unwrap(), 11111);

            if let Some(fld) = &cfg.elements()[0].as_field() {
                assert_eq!(fld.values()[0].feeder(), "testing_feeder");
                assert_eq!(fld.values()[0].value::<u16>().unwrap(), 22222);
                assert!(fld.values()[0].value::<u8>().is_err());
            }
        } else {
            panic!();
        }
    }
}
