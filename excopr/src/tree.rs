use std::sync::{Arc, Mutex, RwLock};

use crate::{
    common::{Description, Help, Named, Values},
    config::Config,
    error, feeder,
    field::Field,
    value::Value,
};

#[derive(Debug)]
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
    ) -> Result<(), Arc<Mutex<dyn error::Setup>>> {
        match self {
            Self::Config(config) => config.add_feeder_matches(feeder_name, feeder_match),
            Self::Field(field) => field.add_feeder_matches(feeder_name, feeder_match),
        }
    }

    fn get_feeder_matches(&mut self, feeder_name: &str) -> Option<Arc<Mutex<dyn feeder::Matches>>> {
        match self {
            Self::Config(config) => config.get_feeder_matches(feeder_name),
            Self::Field(field) => field.get_feeder_matches(feeder_name),
        }
    }

    fn all_feeder_matches(&mut self) -> Vec<Arc<Mutex<dyn feeder::Matches>>> {
        match self {
            Self::Config(config) => config.all_feeder_matches(),
            Self::Field(field) => field.all_feeder_matches(),
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
    fn help(&self, parents: Vec<Arc<RwLock<dyn Config>>>) -> String {
        match self {
            Self::Config(config) => config.help(parents),
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
    fn help(&self, parents: Vec<Arc<RwLock<dyn Config>>>) -> String {
        self.lock().unwrap().help(parents)
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
        error, AsValues, Config, Configuration, Description, Element, ElementConverter, FakeConfig,
        FakeFeeder, FakeField, FakeGroup, FakeMatches, FakeRunError, FakeSetupError, Feeder,
        FeederMatch, FeederMatches, Field, Group, Named, NewSetup, Node, Value, Values,
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
            feeder_matches: Vec::new(),
            description: None,
        };
        let element = Arc::new(Mutex::new(Element::Field(Arc::new(RwLock::new(
            FakeField {
                name: "Fld".to_string(),
                values: vec![],
                feeder_matches: Vec::new(),
                description: None,
            },
        )))));
        let subconfig = FakeConfig {
            name: "sub".to_string(),
            elements: vec![element.clone()],
            groups: vec![],
            values: vec![],
            feeder_matches: Vec::new(),
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
            .build::<FakeRunError>()
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
            .add_feeder::<FakeFeeder, FakeSetupError>(FakeFeeder::new("test", HashMap::new()))
            .unwrap();
        assert!(builder
            .add_feeder::<FakeFeeder, FakeSetupError>(FakeFeeder::new("test", HashMap::new()))
            .is_err());
    }

    #[test]
    fn empty_builder() {
        assert!(Configuration::builder().build::<FakeRunError>().is_err())
    }

    #[test]
    fn values() {
        let builder = Configuration::builder();
        let element = Arc::new(Mutex::new(Element::Field(Arc::new(RwLock::new(
            FakeField {
                name: "second".to_string(),
                values: vec![],
                feeder_matches: Vec::new(),
                description: None,
            },
        )))));
        let mut root = FakeConfig {
            name: "first".to_string(),
            elements: vec![element],
            groups: vec![],
            values: vec![],
            feeder_matches: Vec::new(),
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
            .add_feeder::<FakeFeeder, FakeSetupError>(feeder)
            .unwrap()
            .set_root(Arc::new(RwLock::new(root)))
            .build::<FakeRunError>()
            .unwrap();

        let cfg = res.root.read().unwrap();
        assert_eq!(cfg.values()[0].feeder(), "testing_feeder");
        assert_eq!(cfg.values()[0].value::<u32>().unwrap(), 11111);

        let fld = &cfg.elements()[0].as_field().unwrap();
        assert_eq!(fld.values()[0].feeder(), "testing_feeder");
        assert_eq!(fld.values()[0].value::<u16>().unwrap(), 22222);
        assert!(fld.values()[0].value::<u8>().is_err());
    }

    #[derive(Debug)]
    struct FailingField {
        description: Option<String>,
        name: String,
        feeder_matches: Vec<(String, Arc<Mutex<dyn FeederMatches>>)>,
    }

    impl Field for FailingField {}

    impl Description for FailingField {
        fn description(&self) -> Option<String> {
            self.description.clone()
        }
    }
    impl Named for FailingField {
        fn name(&self) -> String {
            self.name.clone()
        }
    }
    impl Values for FailingField {
        fn values(&self) -> Vec<Value> {
            vec![]
        }

        fn append(&mut self, _: &str, _: String) {}

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

        fn get_feeder_matches(
            &mut self,
            feeder_name: &str,
        ) -> Option<Arc<Mutex<dyn FeederMatches>>> {
            let name = feeder_name.to_string();
            if let Some((_, matches)) = self.feeder_matches.iter().find(|(n, _)| *n == name) {
                Some(matches.clone())
            } else {
                None
            }
        }

        fn all_feeder_matches(&mut self) -> Vec<Arc<Mutex<dyn FeederMatches>>> {
            self.feeder_matches.iter().map(|(_, m)| m.clone()).collect()
        }
    }

    impl AsValues for FailingField {
        fn as_values(&mut self) -> &mut dyn Values {
            self
        }
    }

    #[derive(Debug)]
    struct FalingMatch(usize);

    impl FeederMatch for FalingMatch {
        fn id_in_feeder(&self) -> usize {
            self.0
        }
        fn repr(&self) -> String {
            self.0.to_string()
        }
    }

    #[derive(Debug)]
    struct FalingMatches(Vec<Arc<Mutex<dyn FeederMatch>>>);

    impl FeederMatches for FalingMatches {
        fn repr(&self) -> String {
            self.0
                .iter()
                .map(|e| e.repr())
                .collect::<Vec<String>>()
                .join(" ")
        }

        fn matches(&self) -> Vec<Arc<Mutex<dyn FeederMatch>>> {
            self.0.clone()
        }

        fn add_match(&mut self, new_match: Arc<Mutex<dyn FeederMatch>>) {
            self.0.push(new_match)
        }
    }

    #[derive(Debug)]
    struct FailingFeeder;

    impl Feeder for FailingFeeder {
        fn name(&self) -> &str {
            "failing"
        }
        fn process_matches(
            &mut self,
            element: &mut dyn Values,
        ) -> Result<(), Arc<Mutex<dyn error::Run>>> {
            element.get_feeder_matches("failing")
        }
    }

    #[derive(Debug)]
    struct FailingConfig {
        description: Option<String>,
        name: String,
        feeder_matches: Vec<(String, Arc<Mutex<dyn FeederMatches>>)>,
        elements: Vec<Arc<Mutex<Element>>>,
        groups: Vec<Arc<RwLock<dyn Group>>>,
    }

    impl Config for FailingConfig {
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

    impl Description for FailingConfig {
        fn description(&self) -> Option<String> {
            self.description.clone()
        }
    }
    impl Named for FailingConfig {
        fn name(&self) -> String {
            self.name.clone()
        }
    }
    impl Values for FailingConfig {
        fn values(&self) -> Vec<Value> {
            vec![]
        }

        fn append(&mut self, _: &str, _: String) {}

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

        fn get_feeder_matches(
            &mut self,
            feeder_name: &str,
        ) -> Option<Arc<Mutex<dyn FeederMatches>>> {
            let name = feeder_name.to_string();
            if let Some((_, matches)) = self.feeder_matches.iter().find(|(n, _)| *n == name) {
                Some(matches.clone())
            } else {
                None
            }
        }

        fn all_feeder_matches(&mut self) -> Vec<Arc<Mutex<dyn FeederMatches>>> {
            self.feeder_matches.iter().map(|(_, m)| m.clone()).collect()
        }
    }

    impl AsValues for FailingConfig {
        fn as_values(&mut self) -> &mut dyn Values {
            self
        }
    }

    impl Node for FailingConfig {
        fn elements(&self) -> Vec<Arc<Mutex<Element>>> {
            self.elements.clone()
        }
        fn groups(&self) -> Vec<Arc<RwLock<dyn Group>>> {
            self.groups.clone()
        }
    }

    #[test]
    fn help_test() {
        let builder = Configuration::builder();
        let root = FakeConfig {
            name: "root".to_string(),
            elements: vec![],
            groups: vec![],
            values: vec![],
            feeder_matches: Vec::new(),
            description: Some("This is testing command".to_string()),
        };
        let element1 = Arc::new(Mutex::new(Element::Field(Arc::new(RwLock::new(
            FakeField {
                name: "Fld".to_string(),
                values: vec![],
                feeder_matches: Vec::new(),
                description: Some("Field description".to_string()),
            },
        )))));
        let element2 = Arc::new(Mutex::new(Element::Field(Arc::new(RwLock::new(
            FakeField {
                name: "Fld".to_string(),
                values: vec![],
                feeder_matches: Vec::new(),
                description: None,
            },
        )))));
        let element3 = Arc::new(Mutex::new(Element::Field(Arc::new(RwLock::new(
            FailingField {
                description: Some("help for failing field".to_string()),
                name: "test_help".to_string(),
                feeder_matches: Vec::new(),
            },
        )))));

        let subconfig = FakeConfig {
            name: "sub".to_string(),
            elements: vec![element1.clone(), element2, element3],
            groups: vec![],
            values: vec![],
            feeder_matches: Vec::new(),
            description: None,
        };
        let group = FakeGroup {
            name: "Grp".to_string(),
            members: vec![element1],
            description: None,
        };
        let subconfig = subconfig.add_group(Arc::new(RwLock::new(group))).unwrap();
        let root = root.add_config(Arc::new(RwLock::new(subconfig))).unwrap();
        let res = builder
            .set_root(Arc::new(RwLock::new(root)))
            .build::<FakeRunError>();

        if let Err(error) = res {
            assert_eq!(format!("{}", error.lock().unwrap()), "XXXXXXXXX");
        } else {
            panic!("not failing")
        }
    }
}
