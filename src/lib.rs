//! ReplaceDrop replaces the drop of a type
//!
//! It wraps ManuallyDrop and instead of not calling drop, it calls a secondary one
//! This allows you to use other crates like ext to override the drop function of a type

use std::{
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

/// # Safety
/// The implemenentor must ensure that they do not remove any drop functionality that is important
/// When using ReplaceDrop, the struct's fields to not automatically get dropped
pub unsafe trait ReplaceDropImpl {
    /// # Safety
    /// The caller must ensure this function is only called once
    unsafe fn drop(&mut self);
}

// SAFETY: Unit type does not have a default drop
unsafe impl ReplaceDropImpl for () {
    unsafe fn drop(&mut self) {}
}

/// A wrapper around ManuallyDrop that instead of removing the drop, replaces it
#[derive(Clone, Debug)]
pub struct ReplaceDrop<T: ReplaceDropImpl>(ManuallyDrop<T>);

impl<T: ReplaceDropImpl> ReplaceDrop<T> {
    pub fn new(val: T) -> Self {
        ReplaceDrop(ManuallyDrop::new(val))
    }

    pub fn new_from_manually_drop(val: ManuallyDrop<T>) -> Self {
        ReplaceDrop(val)
    }

    pub fn into_inner(mut self) -> T {
        // SAFETY: We immediatly mem::forget(self) after this so self.0 cant be used
        let val = unsafe { ManuallyDrop::take(&mut self.0) };
        std::mem::forget(self);
        val
    }
}

impl<T: ReplaceDropImpl> Drop for ReplaceDrop<T> {
    fn drop(&mut self) {
        // SAFETY: This is called in the Drop implementation, so it can't be called multiple times with the same value
        unsafe { ReplaceDropImpl::drop(self.0.deref_mut()) };
    }
}

impl<T: ReplaceDropImpl> Deref for ReplaceDrop<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: ReplaceDropImpl> DerefMut for ReplaceDrop<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        struct MyType<'a>(&'a mut u32);
        impl<'a> Drop for MyType<'a> {
            fn drop(&mut self) {
                *self.0 = 1;
            }
        }

        unsafe impl<'a> ReplaceDropImpl for MyType<'a> {
            unsafe fn drop(&mut self) {
                *self.0 = 5;
            }
        }

        let mut t = 0;
        let thing = MyType(&mut t);

        drop(thing);
        assert_eq!(t, 1);

        let thing2 = ReplaceDrop::new(MyType(&mut t));

        drop(thing2);

        assert_eq!(t, 5);
    }
}
