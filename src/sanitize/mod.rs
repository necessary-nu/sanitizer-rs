use crate::config::SanitizerConfig;
use crate::error::SanitizeError;
use crate::output::SanitizedOutput;

mod baseline;
mod core;
mod urls;

// s[impl sanitize.entry.safe]
// s[impl sanitize.entry.unsafe]
pub(crate) fn sanitize_with(
    cfg: &SanitizerConfig,
    html: &str,
    safe: bool,
) -> Result<SanitizedOutput, SanitizeError> {
    let mut working = cfg.clone();
    if safe {
        baseline::remove_unsafe(&mut working);
    }
    core::run(&working, html, safe)
}
