use std::fmt::Debug;

pub trait Content
{
    fn count(&self) -> usize;
    fn content<C: ToString + Debug + AsRef<str>>(&self) -> &C;
    fn id<T: ToString + AsRef<str>>(&self) -> T;
}