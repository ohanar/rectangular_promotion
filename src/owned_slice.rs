use std::ops::{Deref, Range};
use std::iter::IntoIterator;

use full_deref::FullDeref;
use into_iter::IntoIter;

#[derive(Clone, Copy, Debug)]
pub struct OwnedSlice<T> {
	inner: T,
	start: usize,
	end: usize,
}

impl<T> OwnedSlice<T> {
	#[inline]
	pub fn new(inner: T, range: Range<usize>) -> Self {
		OwnedSlice { inner, start: range.start, end: range.end }
	}

	#[inline]
	pub fn inner(&self) -> (&T, Range<usize>) {
		(&self.inner, self.start..self.end)
	}
}

impl<T, U> Deref for OwnedSlice<T> where T: FullDeref<Target=[U]> {
	type Target = [U];

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.inner.full_deref()[self.start..self.end]
	}
}

impl<T, U> IntoIterator for OwnedSlice<T> where T: FullDeref<Target=[U]>, U: Copy {
	type Item = U;
	type IntoIter = IntoIter<Self>;

	#[inline]
	fn into_iter(self) -> Self::IntoIter {
		IntoIter::new(self)
	}
}