use std::ops::{Deref, DerefMut};

pub struct Invalidator<T> {
    inner: T,
    invalidated: bool
}

impl<T> Invalidator<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            invalidated: false
        }
    }
    pub fn reset(&mut self) {
        self.invalidated = false;
    }
    pub fn invalidated(&self) -> bool {
        self.invalidated
    }
}

impl<T> Deref for Invalidator<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Invalidator<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.invalidated = true;
        &mut self.inner
    }
}

