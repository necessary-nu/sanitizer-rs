use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    MixedElementsLists,
    MixedAttributesLists,
    MixedProcessingInstructionsLists,
    DataAttributesWithoutAllowList,
    DuplicateElements,
    DuplicateRemoveElements,
    DuplicateReplaceWithChildren,
    DuplicateAttributes,
    DuplicateRemoveAttributes,
    DuplicateProcessingInstructionTargets,
    DuplicateRemoveProcessingInstructionTargets,
    ReplaceWithChildrenContainsNonReplaceable(String),
    ReplaceWithChildrenOverlapsElements,
    ReplaceWithChildrenOverlapsRemoveElements,
    PerElementAttributesBothListsPresent(String),
    PerElementAttributesDuplicate(String),
    PerElementRemoveAttributesDuplicate(String),
    PerElementAttributesOverlapGlobalAllow(String),
    PerElementRemoveAttributesNotSubsetOfGlobalAllow(String),
    PerElementAttributesContainsDataAttribute(String),
    GlobalAttributesContainsDataAttribute,
    PerElementAttributesOverlapGlobalRemove(String),
    PerElementRemoveAttributesOverlapGlobalRemove(String),
    DataAttributesRequiresAttributesAllowList,
    JsonError(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MixedElementsLists => {
                write!(f, "elements and removeElements cannot both be present")
            }
            Self::MixedAttributesLists => {
                write!(f, "attributes and removeAttributes cannot both be present")
            }
            Self::MixedProcessingInstructionsLists => write!(
                f,
                "processingInstructions and removeProcessingInstructions cannot both be present"
            ),
            Self::DataAttributesWithoutAllowList => write!(
                f,
                "dataAttributes is only allowed when an attributes allow-list is present"
            ),
            Self::DuplicateElements => write!(f, "duplicate entries in elements"),
            Self::DuplicateRemoveElements => write!(f, "duplicate entries in removeElements"),
            Self::DuplicateReplaceWithChildren => {
                write!(f, "duplicate entries in replaceWithChildrenElements")
            }
            Self::DuplicateAttributes => write!(f, "duplicate entries in attributes"),
            Self::DuplicateRemoveAttributes => write!(f, "duplicate entries in removeAttributes"),
            Self::DuplicateProcessingInstructionTargets => {
                write!(f, "duplicate entries in processingInstructions")
            }
            Self::DuplicateRemoveProcessingInstructionTargets => {
                write!(f, "duplicate entries in removeProcessingInstructions")
            }
            Self::ReplaceWithChildrenContainsNonReplaceable(el) => write!(
                f,
                "replaceWithChildrenElements contains non-replaceable element: {el}"
            ),
            Self::ReplaceWithChildrenOverlapsElements => {
                write!(f, "replaceWithChildrenElements overlaps with elements")
            }
            Self::ReplaceWithChildrenOverlapsRemoveElements => write!(
                f,
                "replaceWithChildrenElements overlaps with removeElements"
            ),
            Self::PerElementAttributesBothListsPresent(el) => write!(
                f,
                "element {el} has both attributes and removeAttributes under a global removeAttributes list"
            ),
            Self::PerElementAttributesDuplicate(el) => {
                write!(f, "duplicate entries in per-element attributes for {el}")
            }
            Self::PerElementRemoveAttributesDuplicate(el) => write!(
                f,
                "duplicate entries in per-element removeAttributes for {el}"
            ),
            Self::PerElementAttributesOverlapGlobalAllow(el) => write!(
                f,
                "per-element attributes on {el} overlap with global attributes"
            ),
            Self::PerElementRemoveAttributesNotSubsetOfGlobalAllow(el) => write!(
                f,
                "per-element removeAttributes on {el} is not a subset of global attributes"
            ),
            Self::PerElementAttributesContainsDataAttribute(el) => write!(
                f,
                "per-element attributes on {el} contains data-* while dataAttributes is true"
            ),
            Self::GlobalAttributesContainsDataAttribute => write!(
                f,
                "global attributes list contains data-* while dataAttributes is true"
            ),
            Self::PerElementAttributesOverlapGlobalRemove(el) => write!(
                f,
                "per-element attributes on {el} overlap with global removeAttributes"
            ),
            Self::PerElementRemoveAttributesOverlapGlobalRemove(el) => write!(
                f,
                "per-element removeAttributes on {el} overlap with global removeAttributes"
            ),
            Self::DataAttributesRequiresAttributesAllowList => {
                write!(f, "dataAttributes requires a global attributes allow-list")
            }
            Self::JsonError(s) => write!(f, "JSON deserialization error: {s}"),
        }
    }
}

impl std::error::Error for ConfigError {}

#[derive(Debug)]
pub enum SanitizeError {
    Config(ConfigError),
    Serialize(std::io::Error),
    AllocationLimit,
}

impl fmt::Display for SanitizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(e) => write!(f, "config error: {e}"),
            Self::Serialize(e) => write!(f, "serialize error: {e}"),
            Self::AllocationLimit => write!(f, "arena allocation limit exceeded"),
        }
    }
}

impl std::error::Error for SanitizeError {}

impl From<ConfigError> for SanitizeError {
    fn from(e: ConfigError) -> Self {
        Self::Config(e)
    }
}
