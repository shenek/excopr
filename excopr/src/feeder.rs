use std::sync::{Arc, Mutex};

use crate::{
    error::Config as ConfigError,
    tree::{Element, ElementConverter},
};

/// Trait which will identify underlying Feeder structure
/// It should be placed inside configuration node
pub trait Match {
    /// Number which can be used to identify linked internal structure inside feeder
    fn id_in_feeder(&self) -> usize;
    /// Human readable name
    fn repr(&self) -> String;
}

impl Match for Arc<Mutex<dyn Match>> {
    fn id_in_feeder(&self) -> usize {
        self.lock().unwrap().id_in_feeder()
    }
    fn repr(&self) -> String {
        self.lock().unwrap().repr()
    }
}

/// Represents matches inside configuration node
pub trait Matches {
    /// Hint which will be shown in help
    fn repr(&self) -> String;
    /// All matches
    fn matches(&self) -> Vec<Arc<Mutex<dyn Match>>>;
    /// Add new match
    fn add_match(&mut self, new_match: Arc<Mutex<dyn Match>>);
}

impl Matches for Arc<Mutex<dyn Matches>> {
    fn repr(&self) -> String {
        self.lock().unwrap().repr()
    }

    fn matches(&self) -> Vec<Arc<Mutex<dyn Match>>> {
        self.lock().unwrap().matches()
    }

    fn add_match(&mut self, new_match: Arc<Mutex<dyn Match>>) {
        self.lock().unwrap().add_match(new_match)
    }
}

pub trait Feeder {
    /// Default processing of configuration node
    fn process(&mut self, element: Arc<Mutex<Element>>) -> Result<(), ConfigError> {
        self.process_matches(element.clone());
        self.dfs(element)?;
        Ok(())
    }

    /// A feeder is supposed to have a unique name
    fn name(&self) -> &str;

    /// DFS
    fn dfs(&mut self, element: Arc<Mutex<Element>>) -> Result<(), ConfigError> {
        if let Some(conf) = element.clone().as_config() {
            for subelement in conf.lock().unwrap().elements() {
                self.process(subelement.clone())?;
                self.dfs(subelement.clone())?;
            }
        }
        Ok(())
    }

    /// Checks feeder matches of the feeder and appends
    /// value(s) if match passes
    fn process_matches(&mut self, element: Arc<Mutex<Element>>);
}
