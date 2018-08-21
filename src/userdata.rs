use std::ops::Deref;

/// Trait for marking userdata.
///
/// Default impl for all types that implement `std::ops::Deref`
/// (`&T`, `&mut T`, `Box<T>`, `Rc<T>`, `Arc<T>`, etc.)
pub trait Userdata {}

impl<T> Userdata for T where T: Deref {}