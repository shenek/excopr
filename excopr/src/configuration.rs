use crate::{error::Config as ConfigError, feeder::Feeder, value::Value};

pub struct Builder {
    feeders: Vec<Box<dyn Feeder>>,
    root: Option<Element>,
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

    pub fn add_feeder(mut self, feeder: Box<dyn Feeder>) -> Result<Self, ConfigError> {
        if self.feeders.iter().any(|f| f.name() == feeder.name()) {
            Err(ConfigError::new(&format!(
                "Feeder '{}' already exists",
                feeder.name()
            )))
        } else {
            self.feeders.push(feeder);
            Ok(self)
        }
    }

    pub fn set_root(mut self, root: Element) -> Self {
        self.root = Some(root);
        self
    }

    pub fn build(self) -> Result<Configuration, ConfigError> {
        let mut root = self
            .root
            .ok_or_else(|| ConfigError::new("No Configuration set"))?;
        for mut feeder in self.feeders {
            feeder.process(&mut root)?;
        }
        // TODO remove empty programs
        Ok(Configuration { root })
    }
}

pub struct Configuration {
    pub root: Element,
}

impl Configuration {
    pub fn builder() -> Builder {
        Builder::new()
    }
}

pub trait Named {
    fn name(&self) -> &str;
}

pub trait Members {
    fn members(&self) -> &[String];
}

pub trait Node {
    fn elements(&self) -> &[Element];
    fn elements_mut(&mut self) -> &mut Vec<Element>;
    fn groups(&self) -> &[Box<dyn Group>];
}

pub trait Values {
    fn as_values(&mut self) -> &mut dyn Values;
    fn values(&self) -> &[Value];
    fn append(&mut self, feeder: &str, value: String);
    fn add_feeder_match(&mut self, feeder: &str, key: String) -> Result<(), ConfigError>;
    fn feeder_matches(&self, feeder: &str) -> Option<&[String]>;
}

pub trait FieldContainer {
    fn add_field(self, field: Box<dyn Field>) -> Result<Self, ConfigError>
    where
        Self: Sized;
}

pub trait Config: Named + Node + Values {
    /// Adds mutually exclusive configs
    fn add_config(self, configs: Box<dyn Config>) -> Result<Self, ConfigError>
    where
        Self: Sized;
    fn add_group(self, group: Box<dyn Group>) -> Result<Self, ConfigError>
    where
        Self: Sized;
}

pub trait Group: Named + Members {}

pub trait Field: Named + Values {}

pub enum Element {
    Config(Box<dyn Config>),
    Field(Box<dyn Field>),
}

#[cfg(test)]
mod tests {
    use excopr_tests::{
        Config, Configuration, Element, FakeConfig, FakeFeeder, FakeField, FakeGroup, Node, Values,
    };
    use std::collections::HashMap;

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
        let subconfig = FakeConfig {
            name: "sub".to_string(),
            elements: vec![Element::Field(Box::new(FakeField {
                name: "Fld".to_string(),
                values: vec![],
                feeder_matches: HashMap::new(),
            }))],
            groups: vec![],
            values: vec![],
            feeder_matches: HashMap::new(),
        };
        let group = FakeGroup {
            name: "Grp".to_string(),
            members: vec!["Fld".to_string()],
        };
        let subconfig = subconfig.add_group(Box::new(group)).unwrap();
        let root = root.add_config(Box::new(subconfig)).unwrap();
        let configuration = builder
            .set_root(Element::Config(Box::new(root)))
            .build()
            .unwrap();
        if let Element::Config(conf) = configuration.root {
            if let Element::Config(subconf) = &conf.elements()[0] {
                assert_eq!(subconf.name(), "sub");
                let group = &subconf.groups()[0];
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
            .add_feeder(Box::new(FakeFeeder {
                name: "test".to_string(),
                map: HashMap::new(),
            }))
            .unwrap();
        assert!(builder
            .add_feeder(Box::new(FakeFeeder {
                name: "test".to_string(),
                map: HashMap::new(),
            }))
            .is_err());
    }

    #[test]
    fn empty_builder() {
        assert!(Configuration::builder().build().is_err())
    }

    #[test]
    fn values() {
        let builder = Configuration::builder();
        let mut root = FakeConfig {
            name: "first".to_string(),
            elements: vec![Element::Field(Box::new(FakeField {
                name: "second".to_string(),
                values: vec![],
                feeder_matches: HashMap::new(),
            }))],
            groups: vec![],
            values: vec![],
            feeder_matches: HashMap::new(),
        };

        root.add_feeder_match("testing_feeder", "feeder_id_1".to_string())
            .unwrap();
        if let Element::Field(f) = &mut root.elements_mut()[0] {
            f.add_feeder_match("testing_feeder", "feeder_id_2".to_string())
                .unwrap();
        }

        let mut map = HashMap::new();
        map.insert("feeder_id_1".to_string(), "11111".to_string());
        map.insert("feeder_id_2".to_string(), "22222".to_string());

        let feeder = FakeFeeder {
            name: "testing_feeder".to_string(),
            map,
        };

        let res = builder
            .add_feeder(Box::new(feeder))
            .unwrap()
            .set_root(Element::Config(Box::new(root)))
            .build()
            .unwrap();

        if let Element::Config(cfg) = res.root {
            assert_eq!(cfg.values()[0].feeder(), "testing_feeder");
            assert_eq!(cfg.values()[0].value::<u32>().unwrap(), 11111);

            if let Element::Field(fld) = &cfg.elements()[0] {
                assert_eq!(fld.values()[0].feeder(), "testing_feeder");
                assert_eq!(fld.values()[0].value::<u16>().unwrap(), 22222);
                assert!(fld.values()[0].value::<u8>().is_err());
            }
        } else {
            panic!();
        }
    }
}
