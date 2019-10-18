use std::{
    collections::HashMap,
    env,
    sync::{Arc, Mutex},
};

use excopr::{
    configuration::{Element, Values},
    feeder::{self, Feeder, Match, Matches},
};

#[derive(Clone)]
pub struct EnvMatch {
    id_in_feeder: usize,
    env_variable_name: String,
}

impl feeder::Match for EnvMatch {
    fn repr(&self) -> String {
        self.env_variable_name.clone()
    }

    fn id_in_feeder(&self) -> usize {
        self.id_in_feeder
    }
}

pub struct EnvMatches {
    matches: Vec<Arc<Mutex<dyn feeder::Match>>>,
}

impl EnvMatches {
    pub fn new(matches: Vec<Arc<Mutex<dyn feeder::Match>>>) -> Self {
        Self { matches }
    }
}

impl feeder::Matches for EnvMatches {
    fn repr(&self) -> String {
        let matches: Vec<String> = self.matches.iter().map(|e| e.repr()).collect();
        format!("[env {}]", matches.join(", "))
    }

    fn add_match(&mut self, new_match: Arc<Mutex<dyn feeder::Match>>) {
        self.matches.push(new_match);
    }

    fn matches(&self) -> Vec<Arc<Mutex<dyn feeder::Match>>> {
        self.matches.clone()
    }
}

pub struct EnvFeeder {
    name: String,
    env_vars: HashMap<String, String>,
    matches: Vec<Arc<Mutex<EnvMatch>>>,
}

impl Default for EnvFeeder {
    fn default() -> Self {
        Self::new("env")
    }
}

impl EnvFeeder {
    fn read_env() -> HashMap<String, String> {
        env::vars().collect()
    }

    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            env_vars: Self::read_env(),
            matches: Vec::new(),
        }
    }

    pub fn add_match(&mut self, env_variable_name: &str) -> Arc<Mutex<dyn feeder::Match>> {
        let new_match = Arc::new(Mutex::new(EnvMatch {
            id_in_feeder: self.matches.len(),
            env_variable_name: env_variable_name.to_string(),
        }));
        self.matches.push(new_match.clone());
        new_match
    }
}

impl Feeder for EnvFeeder {
    fn name(&self) -> &str {
        &self.name
    }

    fn process_matches(&mut self, element: Arc<Mutex<Element>>) {
        // TODO several strategies can be use here:
        // * add value only if no prev value is set
        // * add value only if no prev values from this feeder is set
        // * ...
        let mut unlocked = element.lock().unwrap();
        if let Some(matches) = unlocked.feeder_matches(self.name()) {
            for idx in matches.matches().iter().map(|e| e.id_in_feeder()) {
                if let Some(value) = self
                    .env_vars
                    .get(&self.matches[idx].lock().unwrap().env_variable_name)
                {
                    unlocked.append(self.name(), value.to_string())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use excopr_tests::{
        Config, Configuration, Element, ElementConverter, FakeConfig, FakeField, Node,
    };
    use std::{
        collections::HashMap,
        env,
        sync::{Arc, Mutex},
    };

    use super::{EnvFeeder, EnvMatches, Values};

    #[test]
    fn env_feeder_test() {
        env::set_var("TEST1", "test1");
        env::set_var("TEST2", "test2");
        env::set_var("TEST3", "test3");

        let mut feeder = EnvFeeder::new("env_test");

        let builder = Configuration::builder();
        let element = Arc::new(Mutex::new(Element::Field(Arc::new(Mutex::new(
            FakeField {
                name: "second".to_string(),
                values: vec![],
                feeder_matches: HashMap::new(),
            },
        )))));
        let root = Arc::new(Mutex::new(FakeConfig {
            name: "first".to_string(),
            elements: vec![element],
            groups: vec![],
            values: vec![],
            feeder_matches: HashMap::new(),
        }));

        (root.clone() as Arc<Mutex<dyn Config>>)
            .add_feeder_matches(
                "env_test",
                Arc::new(Mutex::new(EnvMatches::new(vec![feeder.add_match("TEST2")]))),
            )
            .unwrap();

        if let Some(mut field) = (root.clone() as Arc<Mutex<dyn Config>>).elements()[0].as_field() {
            field
                .add_feeder_matches(
                    "env_test",
                    Arc::new(Mutex::new(EnvMatches::new(vec![
                        feeder.add_match("TEST3"),
                        feeder.add_match("TEST1"),
                        feeder.add_match("TEST4"),
                    ]))),
                )
                .unwrap();
        }

        let res = builder
            .add_feeder(feeder)
            .unwrap()
            .set_root(Element::Config(root))
            .build()
            .unwrap();

        if let Some(cfg) = res.root.clone().as_config() {
            assert_eq!(cfg.values()[0].feeder(), "env_test");
            assert_eq!(
                cfg.values()[0].value::<String>().unwrap(),
                "test2".to_string()
            );

            if let Some(fld) = &cfg.elements()[0].as_field() {
                assert_eq!(fld.values().len(), 2);
                assert_eq!(fld.values()[0].feeder(), "env_test");
                assert_eq!(fld.values()[0].value::<String>().unwrap(), "test3");
                assert_eq!(fld.values()[1].feeder(), "env_test");
                assert_eq!(fld.values()[1].value::<String>().unwrap(), "test1");
            }
        } else {
            panic!();
        }
    }
}
