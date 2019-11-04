mod common;
mod config;
pub mod error;
mod feeder;
mod field;
mod group;
mod tree;
mod value;

use std::sync::{Arc, Mutex, RwLock};

pub use crate::{
    common::{AsValues, Description, FieldContainer, Help, Members, Named, Node, Values},
    config::Config,
    feeder::{Feeder, Match as FeederMatch, Matches as FeederMatches},
    field::Field,
    group::Group,
    tree::{Element, ElementConverter},
    value::Value,
};

pub struct Builder {
    feeders: Vec<Box<dyn Feeder>>,
    root: Option<Arc<RwLock<dyn Config>>>,
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

    pub fn add_feeder<F, E>(mut self, feeder: F) -> Result<Self, Arc<Mutex<E>>>
    where
        F: 'static + Feeder,
        E: error::NewSetup,
    {
        if self.feeders.iter().any(|f| f.name() == feeder.name()) {
            Err(Arc::new(Mutex::new(E::new(format!(
                "Feeder '{}' already exists",
                feeder.name()
            )))))
        } else {
            self.feeders.push(Box::new(feeder));
            Ok(self)
        }
    }

    pub fn set_root(mut self, root: Arc<RwLock<dyn Config>>) -> Self {
        self.root = Some(root);
        self
    }

    pub fn build<E>(self) -> Result<Configuration, Arc<Mutex<E>>>
    where
        E: error::NewRun,
        Arc<Mutex<E>>: From<Arc<Mutex<dyn error::Run>>>,
    {
        let root = self.root.ok_or_else(|| {
            Mutex::new(E::new(
                None,
                vec![],
                Some("No Configuration set".to_string()),
            ))
        })?;
        for mut feeder in self.feeders {
            feeder.populate(root.clone())?;
        }
        // TODO remove empty programs
        Ok(Configuration { root })
    }
}

pub struct Configuration {
    pub root: Arc<RwLock<dyn Config>>,
}

impl Configuration {
    pub fn builder() -> Builder {
        Builder::new()
    }
}
