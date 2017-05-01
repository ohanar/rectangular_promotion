use std::cmp::Ordering;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::Arc;

use full_deref::FullDeref;
use pairs::{EnumeratedPairs, IntoPairs};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct LatticeWord<T> {
	inner: T,
}

#[derive(Clone, Copy, Debug)]
pub struct ScentIter<T> {
	iter: EnumeratedPairs<T>,
	ordering: Ordering,
}

#[derive(Clone, Copy, Debug)]
pub struct TableauCyclicDescentIter<T, U> {
	iter: EnumeratedPairs<T>,
	tracking_shape: U,
	cyclic_descent: usize,
	base: u8,
	hole_row: u8,
	hole_column: u8,
}

fn is_rectangle(word: &[u8]) -> bool {
	let min = word[0];
	let mut max = min;

	let mut min_count = 0;
	let mut max_count = 0;

	for character in word {
		if *character > max {
			max = *character;
			max_count = 0;
		}

		if *character == min {
			min_count += 1;
		}

		if *character == max {
			max_count += 1;
		}
	}

	min_count == max_count
}

impl<T> LatticeWord<T>
	where T: FullDeref<Target = [u8]>
{
	pub fn new(inner: T) -> Result<Self, &'static str> {
		{
			let inner = inner.full_deref();
			if inner.len() > 0 {
				let min = usize::from(*inner.iter().min().unwrap());
				let mut counts = vec![0; usize::from(*inner.iter().max().unwrap()) + 1 - min];
				for entry in inner.iter() {
					let entry = usize::from(*entry) - min;
					counts[entry] += 1;
					if entry > 0 && counts[entry] > counts[entry - 1] {
						return Err("word is not a lattice word");
					}
				}
			}
		}
		Ok(Self::unchecked_new(inner))
	}

	#[inline]
	pub fn unchecked_new(inner: T) -> Self { LatticeWord { inner: inner } }

	#[inline]
	pub fn descents(&self) -> ScentIter<&[u8]> {
		ScentIter::new(self.inner.full_deref(), Ordering::Less)
	}

	#[inline]
	pub fn into_descents(self) -> ScentIter<T> { ScentIter::new(self.inner, Ordering::Less) }

	#[inline]
	pub fn ascents(&self) -> ScentIter<&[u8]> {
		ScentIter::new(self.inner.full_deref(), Ordering::Greater)
	}

	#[inline]
	pub fn into_ascents(self) -> ScentIter<T> { ScentIter::new(self.inner, Ordering::Greater) }

	#[inline]
	pub fn major_index(&self) -> usize { self.ascents().fold(0, |partial, x| partial + x) }

	#[inline]
	pub fn tableau_cyclic_descents(
		&self,
	) -> Result<TableauCyclicDescentIter<&[u8], Box<[u8]>>, &'static str> {
		TableauCyclicDescentIter::new(self.inner.full_deref())
	}

	#[inline]
	pub(crate) fn tableau_cyclic_descents_with_tracking_shape<U>(
		&self,
		tracking_shape: U,
	) -> TableauCyclicDescentIter<&[u8], U>
		where U: Deref<Target = [u8]> + DerefMut
	{
		TableauCyclicDescentIter::with_tracking_shape(self.inner.full_deref(), tracking_shape)
	}

	#[inline]
	pub fn into_tableau_cyclic_descents(
		self,
	) -> Result<TableauCyclicDescentIter<T, Box<[u8]>>, &'static str> {
		TableauCyclicDescentIter::new(self.inner)
	}

	pub fn promotion(&self) -> Result<LatticeWord<Box<[u8]>>, &'static str> {
		if self.is_empty() {
			return Ok(LatticeWord::unchecked_new(Box::new([])));
		}

		if !is_rectangle(&*self) {
			return Err("only implemented for rectangular shapes");
		}

		let first = self.first().unwrap();
		let (last, prefix) = self.split_last().unwrap();

		let mut new_inner = {
			let mut tmp = Vec::with_capacity(self.len());
			tmp.push(*first);
			tmp.extend_from_slice(prefix);
			tmp.into_boxed_slice()
		};

		let mut hole_row = *last;
		let mut hole_column = 1;

		let mut tracking_shape = vec![0; usize::from(last - first) + 1];
		*tracking_shape.last_mut().unwrap() = 1;

		for current_row in new_inner.iter_mut().rev() {
			let current_column = tracking_shape
				.get_mut(usize::from(*current_row - first))
				.unwrap();
			*current_column += 1;

			if *current_row == hole_row {
				hole_column += 1;
			} else if *current_column == hole_column {
				*current_row = hole_row;
				hole_row -= 1;
				if hole_row == *first {
					break;
				}
			}
		}

		Ok(LatticeWord::unchecked_new(new_inner))
	}
}

#[derive(Clone, Copy, Debug)]
pub struct IntoIter<T> {
	inner: T,
	index: usize,
}

impl<T> IntoIterator for LatticeWord<T>
	where T: FullDeref<Target = [u8]>
{
	type Item = u8;
	type IntoIter = IntoIter<T>;

	#[inline]
	fn into_iter(self) -> Self::IntoIter {
		IntoIter {
			inner: self.inner,
			index: 0,
		}
	}
}

impl<T> Iterator for IntoIter<T>
	where T: FullDeref<Target = [u8]>
{
	type Item = u8;

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

impl<T> ScentIter<T>
	where T: FullDeref<Target = [u8]>
{
	#[inline]
	fn new(word: T, ordering: Ordering) -> Self {
		ScentIter {
			iter: word.into_pairs().enumerate(),
			ordering: ordering,
		}
	}
}

impl<'a, T> Iterator for ScentIter<T>
	where T: FullDeref<Target = [u8]>
{
	type Item = usize;

	fn next(&mut self) -> Option<Self::Item> {
		for (index, (first, second)) in &mut self.iter {
			if second.cmp(&first) == self.ordering {
				return Some(index + 1);
			}
		}

		None
	}
}

impl<'a, T> TableauCyclicDescentIter<T, Box<[u8]>>
	where T: FullDeref<Target = [u8]>
{
	#[inline]
	fn new(word: T) -> Result<Self, &'static str> {
		let len = {
			let w = word.full_deref();

			if !is_rectangle(w) {
				return Err("only implemented for rectangular shapes");
			}

			usize::from(w.last().unwrap() - w.first().unwrap()) + 1
		};

		let mut tracking_shape = Vec::with_capacity(len);
		unsafe {
			tracking_shape.set_len(len);
		}

		Ok(Self::with_tracking_shape(word, tracking_shape.into_boxed_slice()))
	}
}

impl<'a, T, U> TableauCyclicDescentIter<T, U>
	where T: FullDeref<Target = [u8]>,
	      U: Deref<Target = [u8]> + DerefMut
{
	fn with_tracking_shape(word: T, mut tracking_shape: U) -> Self {
		tracking_shape[0] = 1;
		for entry in &mut tracking_shape[1..] {
			*entry = 0;
		}
		let base = *word.full_deref().first().unwrap();
		TableauCyclicDescentIter {
			iter: word.into_pairs().enumerate(),
			tracking_shape: tracking_shape,
			cyclic_descent: 0,
			base: base,
			hole_column: 1,
			hole_row: 0,
		}
	}
}

impl<'a, T, U> Iterator for TableauCyclicDescentIter<T, U>
	where T: FullDeref<Target = [u8]>,
	      U: Deref<Target = [u8]> + DerefMut
{
	type Item = usize;

	fn next(&mut self) -> Option<Self::Item> {
		for (index, (first, second)) in &mut self.iter {
			let current_row = second - self.base;
			let current_column = self
				.tracking_shape
				.get_mut(usize::from(current_row))
				.unwrap();

			*current_column += 1;

			if current_row == self.hole_row {
				self.cyclic_descent = 0;
				self.hole_column += 1;
			} else if *current_column == self.hole_column {
				self.cyclic_descent = index + 2;
				self.hole_row += 1;
			}

			if first < second {
				return Some(index + 1);
			}
		}

		if self.cyclic_descent > 0 {
			let res = Some(self.cyclic_descent);
			self.cyclic_descent = 0;

			res
		} else {
			None
		}
	}
}

impl<T> Deref for LatticeWord<T>
	where T: FullDeref<Target = [u8]>
{
	type Target = [u8];

	#[inline]
	fn deref(&self) -> &Self::Target { self.inner.full_deref() }
}


impl<'a, T> From<&'a LatticeWord<T>> for LatticeWord<Box<[u8]>>
	where T: FullDeref<Target = [u8]>
{
	#[inline]
	fn from(x: &'a LatticeWord<T>) -> Self {
		let x = x.full_deref();
		let mut inner = Vec::with_capacity(x.len());
		inner.extend_from_slice(x);
		LatticeWord::unchecked_new(inner.into_boxed_slice())
	}
}

impl<T> From<LatticeWord<T>> for LatticeWord<Rc<T>>
	where T: FullDeref<Target = [u8]>
{
	#[inline]
	fn from(x: LatticeWord<T>) -> Self { LatticeWord::unchecked_new(Rc::new(x.inner)) }
}

impl<T> From<LatticeWord<T>> for LatticeWord<Arc<T>>
	where T: FullDeref<Target = [u8]>
{
	#[inline]
	fn from(x: LatticeWord<T>) -> Self { LatticeWord::unchecked_new(Arc::new(x.inner)) }
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn scents() {
		let raw_lattice_word = [0, 1, 0, 0, 1];
		let lattice_word = LatticeWord::new(&raw_lattice_word[..]).unwrap();

		let descents: Vec<_> = lattice_word.descents().collect();

		assert_eq!(&*descents, &[2]);

		let ascents: Vec<_> = lattice_word.ascents().collect();

		assert_eq!(&*ascents, &[1, 4]);
	}

	#[test]
	fn tableau_cyclic_descents() {
		let raw_lattice_word = [0, 0, 1, 0, 1, 2, 2, 1, 0, 2, 1, 2];
		let tableau_cyclic_descents: Vec<_> = LatticeWord::new(&raw_lattice_word[..])
			.unwrap()
			.tableau_cyclic_descents()
			.unwrap()
			.collect();

		assert_eq!(&*tableau_cyclic_descents, &[2, 4, 5, 9, 11]);

		let raw_lattice_word = [0, 0, 0, 1, 0, 1, 2, 2, 1, 1, 2, 2];
		let tableau_cyclic_descents: Vec<_> = LatticeWord::new(&raw_lattice_word[..])
			.unwrap()
			.tableau_cyclic_descents()
			.unwrap()
			.collect();

		assert_eq!(&*tableau_cyclic_descents, &[3, 5, 6, 10, 12]);

		let raw_lattice_word = [1, 1, 1, 2, 1, 2, 3, 3, 2, 2, 3, 3];
		let tableau_cyclic_descents: Vec<_> = LatticeWord::new(&raw_lattice_word[..])
			.unwrap()
			.tableau_cyclic_descents()
			.unwrap()
			.collect();

		assert_eq!(&*tableau_cyclic_descents, &[3, 5, 6, 10, 12]);
	}

	#[test]
	fn promotion() {
		let raw_lattice_word = [0, 0, 1, 0, 1, 2, 2, 1, 0, 2, 1, 2];
		let lattice_word = LatticeWord::new(&raw_lattice_word[..]).unwrap();

		let first_promotion = lattice_word.promotion().unwrap();

		assert_eq!(&*first_promotion, &[0, 0, 0, 1, 0, 1, 2, 2, 1, 1, 2, 2]);

		let second_promotion = first_promotion.promotion().unwrap();

		assert_eq!(&*second_promotion, &[0, 1, 0, 0, 1, 0, 1, 2, 2, 2, 1, 2]);

		let raw_lattice_word = [1, 1, 2, 1, 2, 3, 3, 2, 1, 3, 2, 3];
		let lattice_word = LatticeWord::new(&raw_lattice_word[..]).unwrap();

		assert_eq!(
			&*lattice_word.promotion().unwrap(),
			&[1, 1, 1, 2, 1, 2, 3, 3, 2, 2, 3, 3]
		);
	}
}
