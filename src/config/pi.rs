use serde::{Deserialize, Serialize};

// s[impl config.types.pi]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SanitizerPI {
    pub target: String,
}

impl SanitizerPI {
    pub fn new(target: impl Into<String>) -> Self {
        Self {
            target: target.into(),
        }
    }
}
