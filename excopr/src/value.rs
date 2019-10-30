use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct Value {
    feeder: String,
    value: String,
}

impl Value {
    pub fn feeder(&self) -> &str {
        &self.feeder
    }

    pub fn value<V>(&self) -> Result<V, V::Err>
    where
        V: FromStr,
    {
        V::from_str(&self.value)
    }

    pub const fn new(feeder: String, value: String) -> Self {
        Self { feeder, value }
    }
}
