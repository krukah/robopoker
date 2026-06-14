//! Generic ID wrapper providing compile-time type safety over uuid::Uuid.
use std::cmp::Ordering;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::hash::Hash;
use std::hash::Hasher;
use std::marker::PhantomData;

/// Generic ID wrapper providing compile-time type safety over uuid::Uuid.
pub struct ID<T> {
    inner: uuid::Uuid,
    marker: PhantomData<T>,
}

impl<T> ID<T> {
    pub fn inner(&self) -> uuid::Uuid {
        self.inner
    }
    /// Cast `ID<T>` to `ID<U>` while preserving the underlying UUID.
    /// Useful for converting between marker types.
    pub fn cast<U>(self) -> ID<U> {
        ID {
            inner: self.inner,
            marker: PhantomData,
        }
    }
}

impl<T> From<ID<T>> for uuid::Uuid {
    fn from(id: ID<T>) -> Self {
        id.inner()
    }
}
impl<T> From<uuid::Uuid> for ID<T> {
    fn from(inner: uuid::Uuid) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }
}

impl<T> Default for ID<T> {
    fn default() -> Self {
        Self {
            inner: uuid::Uuid::now_v7(),
            marker: PhantomData,
        }
    }
}

impl<T> Copy for ID<T> {}
impl<T> Clone for ID<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Eq for ID<T> {}
impl<T> PartialEq for ID<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T> Ord for ID<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}
impl<T> PartialOrd for ID<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Hash for ID<T> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.inner.hash(state);
    }
}

impl<T> Debug for ID<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ID").field(&self.inner).finish()
    }
}
impl<T> Display for ID<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.inner, f)
    }
}
