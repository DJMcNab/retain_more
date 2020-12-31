#![no_std]
extern crate alloc;

mod string;

use core::ops::{Deref, DerefMut};

pub use string::RetainMoreString;

/// A wrapper type which implements the traits safely
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct SafeImpl<T>(pub T);

impl<T> From<T> for SafeImpl<T> {
    fn from(it: T) -> Self {
        Self(it)
    }
}

impl<T> DerefMut for SafeImpl<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Deref for SafeImpl<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
