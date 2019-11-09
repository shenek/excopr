use std::{
    fmt,
    sync::{Arc, Mutex, RwLock},
};

use crate::{config::Config, error, field::Field, tree::ElementConverter};

/// Trait which will identify underlying Feeder structure
/// It should be placed inside configuration node
pub trait Match: fmt::Debug {
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
pub trait Matches: fmt::Debug {
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
    /// A feeder is supposed to have a unique name
    fn name(&self) -> &str;

    /// populates the tree with values
    fn populate(
        &mut self,
        root: Arc<RwLock<dyn Config>>,
    ) -> Result<(), Arc<Mutex<dyn error::Run>>> {
        self.dfs(root, vec![])?;
        Ok(())
    }

    /// Default processing of configuration node
    /// using DFS
    fn dfs(
        &mut self,
        config: Arc<RwLock<dyn Config>>,
        mut parents: Vec<Arc<RwLock<dyn Config>>>,
    ) -> Result<Vec<Arc<RwLock<dyn Config>>>, Arc<Mutex<dyn error::Run>>> {
        {
            // should accquire write lock
            self.process_config(config.clone())?;
        }

        parents.push(config.clone());
        for subelement in config.read().unwrap().elements() {
            //subelement.xx;
            if let Some(conf) = subelement.as_config() {
                // TODO document read lock only parent access
                parents = self.dfs(conf.clone(), parents).map_err(|e| {
                    e.lock().unwrap().add_parent(conf);
                    e
                })?;
            } else if let Some(field) = subelement.as_field() {
                self.process_field(field)?;
            }
        }
        parents.pop();

        Ok(parents)
    }

    /// Checks feeder matches of the field and appends
    /// value(s) if match passes
    fn process_field(
        &mut self,
        field: Arc<RwLock<dyn Field>>,
    ) -> Result<(), Arc<Mutex<dyn error::Run>>>;

    /// Checks feeder matches of the field and appends
    /// value(s) if match passes
    fn process_config(
        &mut self,
        config: Arc<RwLock<dyn Config>>,
    ) -> Result<(), Arc<Mutex<dyn error::Run>>>;
}
