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
    use super::{
        Builder, Config, ConfigError, Element, Feeder, Field, FieldContainer, Group, Members,
        Named, Node, Value, Values,
    };
    use std::collections::HashMap;

    struct Cnf {
        name: String,
        elements: Vec<Element>,
        groups: Vec<Box<dyn Group>>,
        values: Vec<Value>,
        feeder_matches: HashMap<String, Vec<String>>,
    }

    struct Grp {
        name: String,
        members: Vec<String>,
    }

    struct Fld {
        name: String,
        values: Vec<Value>,
        feeder_matches: HashMap<String, Vec<String>>,
    }

    impl Named for Cnf {
        fn name(&self) -> &str {
            &self.name
        }
    }

    impl Node for Cnf {
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

    impl Values for Cnf {
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

    impl Config for Cnf {
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

    impl FieldContainer for Cnf {
        fn add_field(mut self, field: Box<dyn Field>) -> Result<Self, ConfigError>
        where
            Self: Sized,
        {
            self.elements.push(Element::Field(field));
            Ok(self)
        }
    }

    impl Members for Grp {
        fn members(&self) -> &[String] {
            &self.members
        }
    }

    impl Group for Grp {}

    impl Named for Grp {
        fn name(&self) -> &str {
            &self.name
        }
    }

    impl Field for Fld {}

    impl Values for Fld {
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

    impl Named for Fld {
        fn name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn impl_test() {
        let builder = Builder::new();
        let root = Cnf {
            name: "root".to_string(),
            elements: vec![],
            groups: vec![],
            values: vec![],
            feeder_matches: HashMap::new(),
        };
        let subconfig = Cnf {
            name: "sub".to_string(),
            elements: vec![Element::Field(Box::new(Fld {
                name: "Fld".to_string(),
                values: vec![],
                feeder_matches: HashMap::new(),
            }))],
            groups: vec![],
            values: vec![],
            feeder_matches: HashMap::new(),
        };
        let group = Grp {
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

    struct Fdr {
        name: String,
        map: HashMap<String, String>,
    }

    impl Feeder for Fdr {
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

    #[test]
    fn adding_feeders() {
        let builder = Builder::new();
        let builder = builder
            .add_feeder(Box::new(Fdr {
                name: "test".to_string(),
                map: HashMap::new(),
            }))
            .unwrap();
        assert!(builder
            .add_feeder(Box::new(Fdr {
                name: "test".to_string(),
                map: HashMap::new(),
            }))
            .is_err());
    }

    #[test]
    fn empty_builder() {
        assert!(Builder::new().build().is_err())
    }

    #[test]
    fn values() {
        let builder = Builder::new();
        let mut root = Cnf {
            name: "first".to_string(),
            elements: vec![Element::Field(Box::new(Fld {
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

        let feeder = Fdr {
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
