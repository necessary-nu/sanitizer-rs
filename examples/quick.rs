fn main() {
    let examples = [
        "<p>hello <b>world</b></p>",
        "<p>hello</p><script>alert(1)</script>",
        "<a href=\"javascript:alert(1)\">click</a>",
        "<img src=\"x\" onerror=\"alert(1)\">",
        "<p onclick=\"alert(1)\">text</p>",
        "<iframe src=\"https://evil\"></iframe>",
        "<!-- secret --><p>hi</p>",
    ];

    for input in examples {
        let safe = sanitizer::sanitize(input).unwrap();
        let unsafe_ = sanitizer::sanitize_unsafe(input).unwrap();
        println!("in   : {input}");
        println!("safe : {safe}");
        println!("unsaf: {unsafe_}");
        println!();
    }
}
