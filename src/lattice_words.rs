use std::iter::FusedIterator;
//use std::ops::Deref;

use full_deref::FullDeref;
use lattice_word::LatticeWord;
use pairs::IntoPairs;

#[derive(Clone, Debug)]
pub struct LatticeWords {
	weight: Box<[u8]>,
}

#[derive(Clone, Debug)]
pub struct LatticeWordsStreamingIter<T> {
	weight: T,
	first_pass: bool,
	current: Box<[u8]>,
	subweight: Box<[u8]>,
}

#[derive(Clone, Debug)]
pub struct LatticeWordsIter<T> {
	inner: LatticeWordsStreamingIter<T>,
}

impl LatticeWords {
	#[inline]
	pub fn new(mut weight: Vec<u8>) -> Result<Self, &'static str> {
		for pair in weight.windows(2) {
			if pair[1] > pair[0] {
				return Err("weight is not a partition");
			}
		}
		while weight.last().map(|x| *x == 0).unwrap_or(false) {
			weight.pop();
		}
		Ok(LatticeWords {
			weight: weight.into_boxed_slice(),
		})
	}

	#[inline]
	pub fn weight(&self) -> &[u8] {
		&*self.weight
	}

	#[inline]
	pub fn streaming_iter(&self) -> LatticeWordsStreamingIter<&[u8]> {
		LatticeWordsStreamingIter::new(&*self.weight)
	}

	#[inline]
	pub fn into_streaming_iter(self) -> LatticeWordsStreamingIter<Box<[u8]>> {
		LatticeWordsStreamingIter::new(self.weight)
	}

	#[inline]
	pub fn iter(&self) -> LatticeWordsIter<&[u8]> {
		LatticeWordsIter {
			inner: self.streaming_iter(),
		}
	}
}

impl IntoIterator for LatticeWords {
	type Item = LatticeWord<Box<[u8]>>;
	type IntoIter = LatticeWordsIter<Box<[u8]>>;

	#[inline]
	fn into_iter(self) -> Self::IntoIter {
		LatticeWordsIter {
			inner: self.into_streaming_iter(),
		}
	}
}

#[inline]
fn init_starting_word(word: &mut [u8], weight: &[u8]) {
	let mut last_row = 0;
	let mut height = weight.len();
	let mut word_iter = word.iter_mut();

	for row in weight.iter().rev() {
		let width = *row - last_row;
		for _ in 0..width {
			for row_index in 0..height {
				*word_iter.next().unwrap() = row_index as u8;
			}
		}
		height -= 1;
		last_row = *row;
	}
}

impl<T> LatticeWordsStreamingIter<T> where T: FullDeref<Target=[u8]> {
	fn new(weight: T) -> Self {
		let size = weight.full_deref().iter().fold(0, |partial, entry| partial + usize::from(*entry));
		LatticeWordsStreamingIter {
			weight: weight,
			first_pass: true,
			current: vec![0; size].into_boxed_slice(),
			subweight: vec![0; size].into_boxed_slice(),
		}
	}

	pub fn next(&mut self) -> Option<LatticeWord<&[u8]>> {
		if self.first_pass {
			self.first_pass = false;

			init_starting_word(&mut *self.current, self.weight.full_deref());

			return Some(LatticeWord::unchecked_new(&*self.current));
		}

		for row in &mut *self.subweight {
			*row = 0;
		}

		let first_descent = {
			let mut iter = (&*self.current).into_pairs().enumerate();
			loop {
				if let Some((index, (first, second))) = iter.next() {
					self.subweight[usize::from(first)] += 1;
					if second < first {
						break index + 1;
					}
				} else {
					return None;
				}
			}
		};

		let new_row_index = {
			let mut iter = self.subweight.iter().enumerate().rev();
			let target = self.subweight[usize::from(self.current[first_descent]) + 1];
			loop {
				let (index, row) = iter.next().unwrap();
				if *row == target {
					break index;
				}
			}
		};

		// move the first descent into the new row
		self.subweight[usize::from(self.current[first_descent])] += 1;
		self.subweight[new_row_index] -= 1;
		self.current[first_descent] = new_row_index as u8;

		init_starting_word(&mut self.current[..first_descent], &*self.subweight);

		Some(LatticeWord::unchecked_new(&*self.current))
	}
}

impl<T> Iterator for LatticeWordsIter<T> where T: FullDeref<Target=[u8]> {
	type Item = LatticeWord<Box<[u8]>>;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next().map(|x| Self::Item::from(&x))
	}
}

impl<T> FusedIterator for LatticeWordsIter<T> where Self: Iterator {}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn basic_case() {
		let mut iter = LatticeWords::new(vec![3, 2]).unwrap().into_iter();

		assert_eq!(&*iter.next().unwrap(), &[0, 1, 0, 1, 0]);
		assert_eq!(&*iter.next().unwrap(), &[0, 0, 1, 1, 0]);
		assert_eq!(&*iter.next().unwrap(), &[0, 1, 0, 0, 1]);
		assert_eq!(&*iter.next().unwrap(), &[0, 0, 1, 0, 1]);
		assert_eq!(&*iter.next().unwrap(), &[0, 0, 0, 1, 1]);
		assert!(iter.next().is_none());
	}

	#[test]
	fn empty_case() {
		let mut iter = LatticeWords::new(vec![]).unwrap().into_iter();

		assert_eq!(&*iter.next().unwrap(), &[]);
		assert!(iter.next().is_none());
	}

	#[test]
	fn large_count() {
		let mut iter = LatticeWords::new(vec![4, 4, 4, 4]).unwrap().into_streaming_iter();
		let mut n = 0;

		while let Some(_) = iter.next() {
			n += 1;
		}

		assert_eq!(n, 24024);
	}

	#[cfg(feature = "long_tests")]
	#[test]
	fn very_large_count() {
		let mut iter = LatticeWords::new(vec![5, 5, 5, 5, 5]).unwrap().into_streaming_iter();
		let mut n = 0;

		while let Some(_) = iter.next() {
			n += 1;
		}

		assert_eq!(n, 701149020);
	}
}