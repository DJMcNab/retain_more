//! The tests which require access to [`std::panic::catch_unwind`], which is
//! unavailable in a `no_std` crate

use retain_more::RetainMoreString as _;

#[test]
fn retain_default() {
    // Adapted from https://github.com/rust-lang/rust/blob/2ad5292aea63/library/alloc/tests/string.rs#L364-L396
    let mut s = String::from("α_β_γ");

    s.retain_default(|_| true);
    assert_eq!(s, "α_β_γ");

    s.retain_default(|c| c != '_');
    assert_eq!(s, "αβγ");

    s.retain_default(|c| c != 'β');
    assert_eq!(s, "αγ");

    s.retain_default(|c| c == 'α');
    assert_eq!(s, "α");

    s.retain_default(|_| false);
    assert_eq!(s, "");

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
