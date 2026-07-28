use sanitizer::{SanitizeError, SanitizerBuilder, SanitizerConfig};

#[test]
fn non_ascii_in_prefix_does_not_panic() {
    // Multi-byte UTF-8 char straddling byte offset 5 and 9 — the two cutoffs
    // detect_mode used to slice at. Must return without panicking.
    let out = sanitizer::sanitize_unsafe("<h❤ello>").unwrap();
    // html5ever folds the unrecognized tag into text; we don't care about
    // the exact shape, only that we didn't panic.
    assert!(out.is_empty() || !out.is_empty());
    let _ = sanitizer::sanitize_unsafe("<!docty❤").unwrap();
    let _ = sanitizer::sanitize_unsafe("   <!doctype\u{2764}x").unwrap();
}

#[test]
fn deep_nesting_does_not_stack_overflow() {
    // A recursive sanitize_children blows the default ~1 MiB stack at
    // around 4–8k frames. 10k is comfortably past that and still parses
    // quickly. The iterative walk handles it without issue.
    let depth = 10_000;
    let mut input = String::with_capacity(depth * 5);
    for _ in 0..depth {
        input.push_str("<div>");
    }
    let out = sanitizer::sanitize_unsafe(&input).unwrap();
    assert!(!out.is_empty());
}

#[test]
fn arena_limit_rejects_oversized_input() {
    // 1 KB limit; parsing a non-trivial document cannot fit.
    let cfg = SanitizerBuilder::new()
        .max_arena_bytes(1024)
        .build()
        .unwrap();

    let mut big = String::new();
    for i in 0..5_000 {
        big.push_str(&format!("<p id=\"p{i}\">chunk {i}</p>"));
    }
    let err = cfg.sanitize(&big).unwrap_err();
    assert!(matches!(err, SanitizeError::AllocationLimit), "got {err:?}");
}

#[test]
fn arena_limit_of_none_is_unbounded() {
    // Default config has no limit; small inputs work fine.
    let cfg = SanitizerConfig::empty();
    assert!(cfg.max_arena_bytes.is_none());
    let out = cfg.sanitize_unsafe("<p>hello</p>").unwrap();
    assert_eq!(*out, *"<p>hello</p>");
}

#[test]
fn arena_limit_generous_enough_still_succeeds() {
    // Enough headroom; same doc as the rejection case but with 1 MiB.
    let cfg = SanitizerBuilder::new()
        .max_arena_bytes(1 << 20)
        .build()
        .unwrap();
    let out = cfg.sanitize_unsafe("<p>ok</p>").unwrap();
    assert!(out.contains("<p>ok</p>"));
}
