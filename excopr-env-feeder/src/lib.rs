use std::{collections::HashMap, env};

use excopr::{
    configuration::{Element, Values},
    error,
    feeder::Feeder,
};

struct EnvFeeder {
    name: String,
    env_vars: HashMap<String, String>,
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
        }
    }
}

impl Feeder for EnvFeeder {
    fn name(&self) -> &str {
        &self.name
    }

    fn process(&mut self, element: &mut Element) -> Result<(), error::Config> {
        let values: &mut dyn Values = match element {
            Element::Config(config) => config.as_values(),
            Element::Field(field) => field.as_values(),
        };

        let matches: Vec<String> = if let Some(matches) = values.feeder_matches(self.name()) {
            matches.to_vec()
        } else {
            vec![]
        };

        for env_match in matches.iter().cloned() {
            if let Some(value) = self.env_vars.get(&env_match) {
                values.append(self.name(), value.to_string());
            }
        }

        self.dfs(element)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use excopr_tests::{Configuration, Element, FakeConfig, FakeField, Node, Values};
    use std::{collections::HashMap, env};

    use super::EnvFeeder;

    #[test]
    fn env_feeder_test() {
        env::set_var("TEST1", "test1");
        env::set_var("TEST2", "test2");
        env::set_var("TEST3", "test3");

        let feeder = EnvFeeder::new("env_test");

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

        root.add_feeder_match("env_test", "TEST2".to_string())
            .unwrap();
        if let Element::Field(f) = &mut root.elements_mut()[0] {
            f.add_feeder_match("env_test", "TEST3".to_string()).unwrap();
            f.add_feeder_match("env_test", "TEST1".to_string()).unwrap();
            f.add_feeder_match("env_test", "TEST4".to_string()).unwrap();
        }

        let res = builder
            .add_feeder(Box::new(feeder))
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
