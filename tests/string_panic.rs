//! The tests which require access to [`std::panic::catch_unwind`], which is
//! unavailable in a `no_std` crate

use retain_more::RetainMoreString as _;

#[test]
fn retain_default_safety() {
    let mut s = String::from("0è0");
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut count = 0;
        s.retain_default(|_| {
            count += 1;
            match count {
                1 => false,
                2 => true,
                _ => panic!(),
            }
        });
    }));
    assert!(std::str::from_utf8(s.as_bytes()).is_ok());
}

/// Independently discovered reproduction of
/// https://github.com/rust-lang/rust/issues/78498
#[test]
fn retain_all_safety_78498() {
    let mut index = 0;
    let mut input = "૱uu".to_string();
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        input.retain_all(|_, _, _| {
            let ret = match index {
                0 => false,
                2 => panic!("What happens here"),
                _ => true,
            };
            index += 1;
            return ret;
        })
    }))
    .unwrap_err();
    assert!(std::str::from_utf8(input.as_bytes()).is_ok());
}
