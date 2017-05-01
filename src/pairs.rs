use std::iter::FusedIterator;
use full_deref::FullDeref;

#[derive(Clone, Copy, Debug)]
pub struct Pairs<T> {
	inner: T,
	index: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct EnumeratedPairs<T> {
	inner: Pairs<T>,
}

impl<T> Pairs<T> {
	#[inline]
	pub fn enumerate(self) -> EnumeratedPairs<T> {
		EnumeratedPairs {
			inner: self
		}
	}
}

impl<T, U> Iterator for Pairs<T> where T: FullDeref<Target=[U]>, U: Copy {
	type Item = (U, U);

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		if let Some(first) = self.inner.full_deref().get(self.index) {
			self.index += 1;
			if let Some(second) = self.inner.full_deref().get(self.index) {
				return Some((*first, *second));
			}
		}
		None
	}
}

impl<T> FusedIterator for Pairs<T> where Self: Iterator {}

impl<T> Iterator for EnumeratedPairs<T> where Pairs<T>: Iterator {
	type Item = (usize, <Pairs<T> as Iterator>::Item);

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		let index = self.inner.index;
		self.inner.next().map(|x| (index, x))
	}
}

impl<T> FusedIterator for EnumeratedPairs<T> where Pairs<T>: FusedIterator {} 

pub trait IntoPairs: Sized {
	fn into_pairs(self) -> Pairs<Self>;
}

impl<T> IntoPairs for T {
	#[inline]
	fn into_pairs(self) -> Pairs<Self> {
		Pairs {
			inner: self,
			index: 0,
		}
	}
}