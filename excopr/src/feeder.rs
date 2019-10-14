use crate::{configuration::Element, error::Config as ConfigError};

pub trait Match {
    fn id(&self) -> &str;
}

pub trait Matches {
    /// Hint which will be shown in context
    fn hint(&self) -> String;
    fn matches(&self, feeder: &dyn Feeder) -> bool;
}

pub trait MatchesBuilder<M, Ms>
where
    Ms: Matches,
    M: Match,
{
    fn add_match(self, single_match: M) -> Self;
    fn build(self) -> Ms;
}

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

    /// Current value
    fn current_value(&self) -> &str;
}
