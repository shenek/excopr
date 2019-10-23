mod common;
mod config;
pub mod error;
mod feeder;
mod field;
mod group;
mod tree;
mod value;

use std::sync::{Arc, Mutex};

pub use crate::{
    common::{Description, FieldContainer, Members, Named, Node, Values},
    config::Config,
    feeder::{Feeder, Match as FeederMatch, Matches as FeederMatches},
    field::Field,
    group::Group,
    tree::{Element, ElementConverter},
    value::Value,
};

pub struct Builder {
    feeders: Vec<Box<dyn Feeder>>,
    root: Option<Arc<Mutex<Element>>>,
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

    pub fn add_feeder<F>(mut self, feeder: F) -> Result<Self, error::Config>
    where
        F: 'static + Feeder,
    {
        if self.feeders.iter().any(|f| f.name() == feeder.name()) {
            Err(error::Config::new(&format!(
                "Feeder '{}' already exists",
                feeder.name()
            )))
        } else {
            self.feeders.push(Box::new(feeder));
            Ok(self)
        }
    }

    pub fn set_root(mut self, root: Element) -> Self {
        self.root = Some(Arc::new(Mutex::new(root)));
        self
    }

    pub fn build(self) -> Result<Configuration, error::Config> {
        let root = self
            .root
            .ok_or_else(|| error::Config::new("No Configuration set"))?;
        for mut feeder in self.feeders {
            feeder.process(root.clone())?;
        }
        // TODO remove empty programs
        Ok(Configuration { root })
    }
}

pub struct Configuration {
    pub root: Arc<Mutex<Element>>,
}

impl Configuration {
    pub fn builder() -> Builder {
        Builder::new()
    }
}
