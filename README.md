# urlchecker

A simple url checker for finding fraud url(s) or nearest url while being fast (threading)

A spell-checker based on the statistical algorithm described by Peter Norvig
in <http://norvig.com/spell-correct.html>.

Usage requires a two-step process:

1) Call `url.train()` one or more times with a large text to train the language model
2) Call `url.correct(word)` to retrieve the correction for the specified URL if it exists

![Crates.io](https://img.shields.io/crates/v/urlchecker?style=flat-square)
![docs.rs](https://img.shields.io/docsrs/urlchecker?style=flat-square)

Eg:-

```rust
use std::collections::HashMap;
use urlchecker::URL;

fn main() {
    let mut url = URL {
        letters: "1234567890._-@abcdefghijklmnopqrstuvwxyz".to_string(),
        url_counts: HashMap::new(),
    };
    url.train(
        "https://docs.rs/regex/latest/regex/ \
    https://norvig.com/spell-correct.html \
    https://doc.rust-lang.org/stable/std/thread/fn.scope.html\
    https://docs.rs/urlchecker/latest/urlchecker/index.html",
    );

    println!("{:#?}", url);

    println!("{:#?}", url.correct("doks.rs"));
}
```

Output:-

```
URL {
    letters: "1234567890._-@abcdefghijklmnopqrstuvwxyz",
    url_counts: {
        "docs.rs": 2,
        "doc.rust-lang.org": 1,
        "norvig.com": 1,
    },
}
Some(
    "docs.rs",
)

```

Inspired from:

- https://norvig.com/spell-correct.html
- https://github.com/past/spellcheck