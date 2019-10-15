use std::rc::Rc;

use crate::{configuration::Element, error::Config as ConfigError};

/// Trait which will identify underlying Feeder structure
/// It should be placed inside configuration node
pub trait Match {
    /// Number which can be used to identify linked internal structure inside feeder
    fn id_in_feeder(&self) -> usize;
    /// Human readable name
    fn repr(&self) -> &str;
}

pub trait MatchFactory {
    /// Prepares match which can be placed into configuration tree
    fn make_match(&self) -> Rc<dyn Match>;
}

/// Represents matches inside configuration node
pub trait Matches {
    /// Hint which will be shown in help
    fn repr(&self) -> String;
    /// All matches
    fn matches(&self) -> Vec<Rc<dyn Match>>;
}

pub trait Feeder {
    /// Processes configuration node
    fn process(&mut self, element: &mut Element) -> Result<(), ConfigError>;

    /// A feeder is supposed to have a unique name
    fn name(&self) -> &str;

    /// DFS
    fn dfs(&mut self, element: &mut Element) -> Result<(), ConfigError> {
        if let Element::Config(conf) = element {
            for subelement in conf.elements_mut().iter_mut() {
                self.process(subelement)?;
                self.dfs(subelement)?;
            }
        }
        Ok(())
    }

    /// Checks feeder matches of the feeder and appends
    /// value(s) if match passes
    fn process_matches(&mut self, element: &mut Element);
}
