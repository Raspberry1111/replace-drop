use replace_drop::{ReplaceDrop, ReplaceDropImpl};

struct Inner;

impl Drop for Inner {
    fn drop(&mut self) {
        println!("Inner!");
    }
}

struct Outer(Inner);

impl Drop for Outer {
    fn drop(&mut self) {
        println!("Outer!");
    }
}

unsafe impl ReplaceDropImpl for Inner {
    unsafe fn drop(&mut self) {
        println!("Inner 2!");
    }
}

unsafe impl ReplaceDropImpl for Outer {
    unsafe fn drop(&mut self) {
        println!("Outer 2!");

        // The constructor for the fields wont automatically run, so you have to use drop_in_place to call it

        // NOTE: you cant use std::mem::drop(self.0) because that requires ownership (we only have a mut ref)
        // SAFETY: A mut reference is always a valid pointer
        std::ptr::drop_in_place(&mut self.0 as *mut _);

        // You can also call a ReplaceDropImpl directly
        ReplaceDropImpl::drop(&mut self.0);

        // Calling drop on self
        std::ptr::drop_in_place(self as *mut _);
    }
}

fn main() {
    let _ = ReplaceDrop::new(Outer(Inner));
    // Prints:
    // Outer 2!
    // Inner!
    // Inner 2!
    // Outer!
    // Inner!
}
