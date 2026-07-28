use serde::{Deserialize, Serialize};

use crate::namespace::{HTML_NS, MATHML_NS, SVG_NS};

use super::attribute::SanitizerAttribute;

// s[impl config.types.element]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SanitizerElement {
    pub name: String,
    #[serde(default)]
    pub namespace: Option<String>,
}

impl SanitizerElement {
    pub fn html(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            namespace: Some(HTML_NS.into()),
        }
    }

    pub fn svg(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            namespace: Some(SVG_NS.into()),
        }
    }

    pub fn mathml(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            namespace: Some(MATHML_NS.into()),
        }
    }

    pub(crate) fn normalize(&mut self) {
        if matches!(self.namespace.as_deref(), Some("")) {
            self.namespace = None;
        }
    }

    pub(crate) fn display(&self) -> String {
        match &self.namespace {
            Some(ns) => format!("{{{ns}}}{}", self.name),
            None => self.name.clone(),
        }
    }
}

// s[impl config.types.element]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SanitizerElementWithAttributes {
    pub name: String,
    #[serde(default)]
    pub namespace: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Vec<SanitizerAttribute>>,
    #[serde(
        default,
        rename = "removeAttributes",
        skip_serializing_if = "Option::is_none"
    )]
    pub remove_attributes: Option<Vec<SanitizerAttribute>>,
}

impl SanitizerElementWithAttributes {
    pub fn html(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            namespace: Some(HTML_NS.into()),
            attributes: None,
            remove_attributes: None,
        }
    }

    pub fn svg(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            namespace: Some(SVG_NS.into()),
            attributes: None,
            remove_attributes: None,
        }
    }

    pub fn mathml(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            namespace: Some(MATHML_NS.into()),
            attributes: None,
            remove_attributes: None,
        }
    }

    pub fn with_attributes(mut self, attrs: Vec<SanitizerAttribute>) -> Self {
        self.attributes = Some(attrs);
        self
    }

    pub fn with_remove_attributes(mut self, attrs: Vec<SanitizerAttribute>) -> Self {
        self.remove_attributes = Some(attrs);
        self
    }

    pub(crate) fn as_element(&self) -> SanitizerElement {
        SanitizerElement {
            name: self.name.clone(),
            namespace: self.namespace.clone(),
        }
    }

    pub(crate) fn normalize(&mut self) {
        if matches!(self.namespace.as_deref(), Some("")) {
            self.namespace = None;
        }
        if let Some(attrs) = &mut self.attributes {
            for a in attrs {
                a.normalize();
            }
        }
        if let Some(attrs) = &mut self.remove_attributes {
            for a in attrs {
                a.normalize();
            }
        }
    }

    pub(crate) fn display(&self) -> String {
        self.as_element().display()
    }
}
