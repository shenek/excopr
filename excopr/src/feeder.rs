use crate::{configuration::Element, error::Config as ConfigError};

pub trait Feeder {
    /// Processes entire configuration tree
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
}
