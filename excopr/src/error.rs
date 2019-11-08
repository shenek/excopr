use std::{
    error::Error,
    sync::{Arc, RwLock},
};

use crate::{config::Config, field::Field};

/// Setup error should occur only when
/// an incorrect config configuration is detected
///
/// e.g. user tries to insert a feeder which already exists
pub trait Setup: Error {
    fn msg(&self) -> String;
}

/// non-dynamic constructor trait
pub trait NewSetup: Setup {
    fn new(msg: String) -> Self;
}

/// Error which happens during the feeder run
///
/// e.g. cmdline parameters are not parsed correctly
pub trait Run: Error {
    fn config(&self) -> Option<Arc<RwLock<dyn Config>>>;
    fn field(&self) -> Option<Arc<RwLock<dyn Field>>>;
    fn parents(&self) -> Vec<Arc<RwLock<dyn Config>>>;
    fn msg(&self) -> Option<String>;
    fn add_parent(&mut self, parent: Arc<RwLock<dyn Config>>);
}

/// non-dynamic constructor trait
pub trait NewRun: Run {
    fn new(
        config: Option<Arc<RwLock<dyn Config>>>,
        field: Option<Arc<RwLock<dyn Field>>>,
        parents: Vec<Arc<RwLock<dyn Config>>>,
        msg: Option<String>,
    ) -> Self;
}

/// Error which happens during the validation
///
/// e.g. expected i64 and string was passed
pub trait Validation: Error {}

/// non-dynamic constructor trait
pub trait NewValidation {
    fn new() -> Self;
}
