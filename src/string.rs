use core::{slice, str::from_utf8_unchecked_mut};

use alloc::string::String;

/// More advanced versions of [`String::retain`], implemented as extension
/// methods on [`String`].
///
/// This trait is sealed and cannot be implemented for types outside of
/// `retain_more`
pub trait RetainMoreString: sealed::AllocMoreSealedString {
    /// Retains only the characters specified by the predicate.
    ///
    /// In other words, remove all characters for which `f` returns false. This
    /// method operates in place, visiting each character exactly once
    /// once in the original order, and preserves the order of the retained
    /// characters.
    ///
    /// This version of [`String::retain`] allows the predicate mutable
    /// access to the valid parts of the full string.
    ///
    /// The arguments of the predicate are:
    ///  - 0: `&mut str`; Contents of `self` which have already been retained,
    ///    i.e. those for which predicate has already returned `true`.
    ///  - 1: [`char`]; The current character being considered.
    ///  - 2: `&mut str`; The parts of `self` yet to be considered.
    ///
    /// # Usage
    ///
    /// ```
    /// # use retain_more::RetainMoreString as _;
    /// let mut my_string = "Super secret code: -100054321-78912EOF\
    ///     Here is some content which shouldn't be seen"
    ///     .to_string();
    /// /// Remove all numbers from the string, including a single leading `'-'` and
    /// /// additionally remove all characters after the first occurence of `"EOF"`
    /// fn cleanup(before: &mut str, it: char, after: &mut str) -> bool {
    ///     if before.ends_with("EOF") {
    ///         false
    ///     } else {
    ///         match (it, after.chars().next()) {
    ///             ('-', Some(c)) => !c.is_ascii_digit(),
    ///             (c, _) => !c.is_ascii_digit(),
    ///         }
    ///     }
    /// }
    /// my_string.retain_all(cleanup);
    /// assert_eq!(&my_string, "Super secret code: EOF");
    /// ```
    fn retain_all<F: FnMut(&mut str, char, &mut str) -> bool>(&mut self, f: F);

    /// A helper for the common case where only access to the parts of the
    /// [`String`] which haven't been considered yet is required, i.e. the
    /// predicate only uses arguments 1 and 2 from [`Self::retain_all`].
    fn retain_after<F: FnMut(char, &mut str) -> bool>(&mut self, mut f: F) {
        self.retain_all(move |_, current, after| f(current, after))
    }

    /// A reimplmentation of [`String::retain`] using
    /// [`retain_all`](`RetainMoreString::retain_all`)
    ///
    /// This is used to demonstrate that
    /// [`retain_all`](`RetainMoreString::retain_all`) is a strictly more
    /// powerful abstraction than [`String::retain`] from [`alloc`].
    ///
    /// The predicate therefore only uses argument 1 from [`Self::retain_all`].
    ///
    /// ## Standard retain docs
    ///
    /// This documentation is taken from [`String::retain`] from [`alloc`].
    ///
    /// Retains only the characters specified by the predicate.
    ///
    /// In other words, remove all characters `c` such that `f(c)` returns
    /// false. This method operates in place, visiting each character exactly
    /// once in the original order, and preserves the order of the retained
    /// characters.
    ///
    /// # Examples
    ///
    /// ```
    /// use retain_more::RetainMoreString as _;
    /// let mut s = String::from("f_o_ob_ar");
    ///
    /// s.retain_default(|c| c != '_');
    ///
    /// assert_eq!(s, "foobar");
    /// ```
    ///
    /// The exact order may be useful for tracking external state, like an
    /// index.
    ///
    /// ```
    /// use retain_more::RetainMoreString as _;
    /// let mut s = String::from("abcde");
    /// let keep = [false, true, true, false, true];
    /// let mut i = 0;
    /// s.retain_default(|_| (keep[i], i += 1).0);
    /// assert_eq!(s, "bce");
    /// ```
    fn retain_default<F: FnMut(char) -> bool>(&mut self, mut f: F) {
        self.retain_all(move |_, current, _| f(current))
    }
}

// Future work - support this for strings with all allocators once/if <https://github.com/rust-lang/rust/pull/79500> lands
impl RetainMoreString for String {
    fn retain_all<F: FnMut(&mut str, char, &mut str) -> bool>(&mut self, mut f: F) {
        let len = self.len();
        // This is required for panic safety, see https://github.com/rust-lang/rust/issues/78498
        // SAFETY: 0..0 is empty and hence that region is valid UTF-8
        // SAFETY: 0 <= self.len(), since self.len() is a usize
        unsafe {
            self.as_mut_vec().set_len(0);
        }
        let mut del_bytes = 0;
        // The index of the start of the region which has not yet been considered.
        // This is always at a UTF-8 character boundary.
        let mut idx = 0;

        while idx < len {
            let ptr = self.as_mut_ptr();
            // The implementation in `alloc` uses `self.get_unchecked(idx..len)` for
            // the equivalent section. <https://github.com/rust-lang/rust/blob/a6bd5246da78/library/alloc/src/string.rs#L1243>
            // This would be unsafe here because the reciever of that method
            // (`DerefMut::deref_mut(&mut self)`) is the empty `str`, since `len` is set to
            // 0 above. However, `get_unchecked` requires that the index is
            // within the bounds of the reciever, not just the allocation of the
            // reciever. This is not a safety issue within `alloc`, because the
            // implementation of `get_unchecked` within `core` expands to the
            // equivalent code as below. However, we cannot make that assumption
            // here, so have to go the long way around.
            let ch = unsafe {
                // SAFETY: `len` came from `self.len()`. Therefore `idx < len` implies `idx` is
                // within the heap allocation owned by self. Therefore the
                // result is within the same allocation as `ptr`.
                let start = ptr.add(idx);
                // SAFETY: The region is not aliased because the method has a mutable reference
                // to self. Additionally, there is no other acess across the
                // loop, and this is the start of the loop body, and no other references exist
                // before this line. We drop the region before any further
                // access later in the loop body.
                let region = slice::from_raw_parts_mut(start, len - idx);

                // `region` is `idx..len` within the original string.
                // idx is on a character boundary, and the rest of this method has not modified
                // this region of bytes (except through the `&mut str` as the third closure
                // parameter, any access through which is required to maintain the UTF-8
                // invariant of that region)
                let ch = from_utf8_unchecked_mut(region).chars().next().unwrap();
                ch
                // region is dropped here, so its access to the region of
            };
            let ch_len = ch.len_utf8();
            let (before, after) = unsafe {
                (
                    // SAFETY: UTF-8 is maintained in the before section by only copying
                    // a full character at a time.
                    from_utf8_unchecked_mut(slice::from_raw_parts_mut(ptr, idx - del_bytes)),
                    // SAFETY: idx + ch_len <= len because self, hence `idx + ch_len` is within the
                    // allocation of self. was valid UTF-8 by invariant, hence
                    // after is valid. This does not alias with `before`,
                    // because `-del_bytes < ch_len`
                    from_utf8_unchecked_mut(slice::from_raw_parts_mut(
                        ptr.add(idx + ch_len),
                        len - idx - ch_len,
                    )),
                )
            };
            if !f(before, ch, after) {
                del_bytes += ch_len;
            } else if del_bytes > 0 {
                // Copy `ch` del_bytes bytes back.
                // Use the version in the allocation of self, which is already UTF-8 encoded.

                // Safety: We copy a region which is a single UTF-8 character.
                // We can't use copy_nonoverlapping here in case del_bytes > ch_len
                unsafe {
                    core::ptr::copy(ptr.add(idx), ptr.add(idx - del_bytes), ch_len);
                }
            }

            // 'Point' idx to the next char
            idx += ch_len;
        }
        // len - del_bytes <= len <= capacity
        unsafe {
            self.as_mut_vec().set_len(len - del_bytes);
        }
    }
}

/// Implementation of the sealed pattern for [`RetainMoreString`]
/// See [C-SEALED] from rust-api-guidelines for explanation
///
/// [C-SEALED]: https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed
mod sealed {
    use alloc::string::String;

    pub trait AllocMoreSealedString {}
    impl AllocMoreSealedString for String {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    fn redact(current: char, rest: &mut str) -> bool {
        match (current, rest.chars().next()) {
            ('-', Some(c)) => !c.is_ascii_digit(),
            (c, _) => !c.is_ascii_digit(),
        }
    }

    fn after_helper<F: FnMut(char, &mut str) -> bool>(input: &str, output: &str, f: F) {
        let mut input = input.to_string();
        input.retain_after(f);

        assert_eq!(&input[..], output);
    }
    #[test]
    fn retain_after() {
        after_helper("this has no numbers", "this has no numbers", redact);
        after_helper("54321", "", redact);
        after_helper("-12345", "", redact);
        after_helper("--12345", "-", redact);
        after_helper("-12-3-45--", "--", redact);
    }
}
