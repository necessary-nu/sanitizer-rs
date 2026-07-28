use serde::{Deserialize, Serialize};

use crate::namespace::XLINK_NS;

// s[impl config.types.attribute]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SanitizerAttribute {
    pub name: String,
    #[serde(default)]
    pub namespace: Option<String>,
}

impl SanitizerAttribute {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            namespace: None,
        }
    }

    pub fn xlink(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            namespace: Some(XLINK_NS.into()),
        }
    }

    pub(crate) fn normalize(&mut self) {
        if matches!(self.namespace.as_deref(), Some("")) {
            self.namespace = None;
        }
    }

    pub(crate) fn is_data_attribute(&self) -> bool {
        self.namespace.is_none() && self.name.starts_with("data-")
    }
}
