use std::iter::Iterator;

use full_deref::FullDeref;

#[derive(Clone, Copy, Debug)]
pub struct IntoIter<T> {
	inner: T,
	index: usize,
}

impl<T> IntoIter<T> {
	#[inline]
	pub fn new(inner: T) -> Self {
		IntoIter {
			inner,
			index: 0,
		}
	}
}

impl<T, U> Iterator for IntoIter<T>
	where T: FullDeref<Target = [U]>, U: Copy
{
	type Item = U;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		if let Some(res) = self.inner.full_deref().get(self.index) {
			self.index += 1;
			Some(*res)
		} else {
			None
		}
	}
}

