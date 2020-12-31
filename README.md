# Retain More

A version of `String::retain` which provides access to the rest of the string

```rust
use retain_more::RetainMore as _;

fn redact(input: &mut String) {
    input.retain_after(|current: char, rest: &str| {
        match (current, rest.chars().next()) {
            ('-', Some(c)) => !c.is_ascii_digit(),
            (c, _) => !c.is_ascii_digit(),
            _ => true,
        }
    });
}
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.