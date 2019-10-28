use std::sync::{Arc, Mutex, RwLock};

use crate::{
    common::{Description, Help, Named, Values},
    config::Config,
    error, feeder,
    field::Field,
    value::Value,
};

pub enum Element {
    Config(Arc<RwLock<dyn Config>>),
    Field(Arc<RwLock<dyn Field>>),
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
        feeder_match: Arc<Mutex<dyn feeder::Matches>>,
    ) -> Result<(), error::Config> {
        match self {
            Self::Config(config) => config.add_feeder_matches(feeder_name, feeder_match),
            Self::Field(field) => field.add_feeder_matches(feeder_name, feeder_match),
        }
    }

    fn feeder_matches(&mut self, feeder_name: &str) -> Option<Arc<Mutex<dyn feeder::Matches>>> {
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
}

impl Help for Element {
    fn help(&self) -> String {
        match self {
            Self::Config(config) => config.help(),
            _ => panic!("help can be called only on configs"),
        }
    }
}

impl Description for Element {
    fn description(&self) -> Option<String> {
        match self {
            Self::Config(config) => config.description(),
            Self::Field(field) => field.description(),
        }
    }
}

impl Named for Arc<Mutex<Element>> {
    fn name(&self) -> String {
        self.lock().unwrap().name()
    }
}

impl Help for Arc<Mutex<Element>> {
    fn help(&self) -> String {
        self.lock().unwrap().help()
    }
}

pub trait ElementConverter {
    fn as_config(&self) -> Option<Arc<RwLock<dyn Config>>>;
    fn as_field(&self) -> Option<Arc<RwLock<dyn Field>>>;
}

impl ElementConverter for Element {
    fn as_config(&self) -> Option<Arc<RwLock<dyn Config>>> {
        match self {
            Self::Config(config) => Some(config.clone()),
            _ => None,
        }
    }
    fn as_field(&self) -> Option<Arc<RwLock<dyn Field>>> {
        match self {
            Self::Field(field) => Some(field.clone()),
            _ => None,
        }
    }
}

impl ElementConverter for Arc<Mutex<Element>> {
    fn as_config(&self) -> Option<Arc<RwLock<dyn Config>>> {
        self.lock().unwrap().as_config()
    }
    fn as_field(&self) -> Option<Arc<RwLock<dyn Field>>> {
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
        sync::{Arc, Mutex, RwLock},
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
            description: None,
        };
        let element = Arc::new(Mutex::new(Element::Field(Arc::new(RwLock::new(
            FakeField {
                name: "Fld".to_string(),
                values: vec![],
                feeder_matches: HashMap::new(),
                description: None,
            },
        )))));
        let subconfig = FakeConfig {
            name: "sub".to_string(),
            elements: vec![element.clone()],
            groups: vec![],
            values: vec![],
            feeder_matches: HashMap::new(),
            description: None,
        };
        let group = FakeGroup {
            name: "Grp".to_string(),
            members: vec![element],
            description: None,
        };
        let subconfig = subconfig.add_group(Arc::new(RwLock::new(group))).unwrap();
        let root = root.add_config(Arc::new(RwLock::new(subconfig))).unwrap();
        let configuration = builder
            .set_root(Arc::new(RwLock::new(root)))
            .build()
            .unwrap();
        let conf = configuration.root.read().unwrap();
        let subconf = conf.elements()[0].as_config().unwrap();
        assert_eq!(subconf.name(), "sub");
        let group = subconf.groups()[0].clone();
        assert_eq!(group.name(), "Grp");
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
        let element = Arc::new(Mutex::new(Element::Field(Arc::new(RwLock::new(
            FakeField {
                name: "second".to_string(),
                values: vec![],
                feeder_matches: HashMap::new(),
                description: None,
            },
        )))));
        let mut root = FakeConfig {
            name: "first".to_string(),
            elements: vec![element],
            groups: vec![],
            values: vec![],
            feeder_matches: HashMap::new(),
            description: None,
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
            .set_root(Arc::new(RwLock::new(root)))
            .build()
            .unwrap();

        let cfg = res.root.read().unwrap();
        assert_eq!(cfg.values()[0].feeder(), "testing_feeder");
        assert_eq!(cfg.values()[0].value::<u32>().unwrap(), 11111);

        let fld = &cfg.elements()[0].as_field().unwrap();
        assert_eq!(fld.values()[0].feeder(), "testing_feeder");
        assert_eq!(fld.values()[0].value::<u16>().unwrap(), 22222);
        assert!(fld.values()[0].value::<u8>().is_err());
    }
}
