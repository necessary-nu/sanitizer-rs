use std::fmt;
use std::ops::Deref;

/// Opaque wrapper around a sanitized value so callers can distinguish
/// "HTML that has been through the sanitizer" from an arbitrary `String`
/// that happens to contain HTML-looking text. The wrapper defaults to
/// `String` but is generic so callers that build a different
/// representation (e.g. a pre-parsed tree) can reuse the same type.
///
/// Use [`SanitizedOutput::into_inner`] to recover the underlying value,
/// or deref it where a borrow suffices.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SanitizedOutput<T = String>(T);

impl<T> SanitizedOutput<T> {
    pub(crate) fn new(inner: T) -> Self {
        Self(inner)
    }

    pub fn into_inner(self) -> T {
        self.0
    }

    pub fn as_inner(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for SanitizedOutput<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: fmt::Display> fmt::Display for SanitizedOutput<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> AsRef<T> for SanitizedOutput<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}
