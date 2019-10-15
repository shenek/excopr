use std::{collections::HashMap, env, rc::Rc};

use excopr::{
    configuration::{Element, Values},
    feeder::{self, Feeder},
};

#[derive(Clone)]
pub struct EnvMatch {
    id_in_feeder: usize,
    env_variable_name: String,
}

impl feeder::Match for EnvMatch {
    fn repr(&self) -> &str {
        &self.env_variable_name
    }

    fn id_in_feeder(&self) -> usize {
        self.id_in_feeder
    }
}

pub struct EnvMatches {
    matches: Vec<Rc<dyn feeder::Match>>,
}

impl EnvMatches {
    pub fn new(matches: Vec<Rc<dyn feeder::Match>>) -> Self {
        Self { matches }
    }
}

impl feeder::Matches for EnvMatches {
    fn repr(&self) -> String {
        let matches: Vec<&str> = self.matches.iter().map(|e| e.repr()).collect();
        format!("[env {}]", matches.join(", "))
    }
    fn add_match(&mut self, new_match: Rc<dyn feeder::Match>) {
        self.matches.push(new_match);
    }
    fn matches(&self) -> Vec<Rc<dyn feeder::Match>> {
        self.matches.clone()
    }
}

struct EnvFeeder {
    name: String,
    env_vars: HashMap<String, String>,
    matches: Vec<Rc<EnvMatch>>,
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

    pub fn add_match(&mut self, env_variable_name: &str) -> Rc<dyn feeder::Match> {
        let new_match = Rc::new(EnvMatch {
            id_in_feeder: self.matches.len(),
            env_variable_name: env_variable_name.to_string(),
        });
        self.matches.push(new_match.clone());
        new_match as Rc<dyn feeder::Match>
    }
}

impl Feeder for EnvFeeder {
    fn name(&self) -> &str {
        &self.name
    }

    fn process_matches(&mut self, element: &mut Element) {
        if let Some(matches) = element.feeder_matches(self.name()) {
            for idx in matches.matches().iter().map(|e| e.id_in_feeder()) {
                if let Some(value) = self.env_vars.get(&self.matches[idx].env_variable_name) {
                    element.append(self.name(), value.to_string())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use excopr_tests::{Configuration, Element, FakeConfig, FakeField, Node, Values};
    use std::{collections::HashMap, env, rc::Rc};

    use super::{EnvFeeder, EnvMatches};

    #[test]
    fn env_feeder_test() {
        env::set_var("TEST1", "test1");
        env::set_var("TEST2", "test2");
        env::set_var("TEST3", "test3");

        let mut feeder = EnvFeeder::new("env_test");

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

        root.add_feeder_matches(
            "env_test",
            Rc::new(EnvMatches {
                matches: vec![feeder.add_match("TEST2")],
            }),
        )
        .unwrap();

        if let Element::Field(f) = &mut root.elements_mut()[0] {
            f.add_feeder_matches(
                "env_test",
                Rc::new(EnvMatches {
                    matches: vec![feeder.add_match("TEST3")],
                }),
            )
            .unwrap();
            f.add_feeder_matches(
                "env_test",
                Rc::new(EnvMatches {
                    matches: vec![feeder.add_match("TEST1")],
                }),
            )
            .unwrap();
            f.add_feeder_matches(
                "env_test",
                Rc::new(EnvMatches {
                    matches: vec![feeder.add_match("TEST4")],
                }),
            )
            .unwrap();
        }

        let res = builder
            .add_feeder(feeder)
            .unwrap()
            .set_root(Element::Config(Box::new(root)))
            .build()
            .unwrap();

        if let Element::Config(cfg) = res.root {
            assert_eq!(cfg.values()[0].feeder(), "env_test");
            assert_eq!(
                cfg.values()[0].value::<String>().unwrap(),
                "test2".to_string()
            );

            if let Element::Field(fld) = &cfg.elements()[0] {
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
