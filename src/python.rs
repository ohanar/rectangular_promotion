use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Write;
use std::{cmp, hash};
use std::sync::Arc;

use cpython::{CompareOp, PyClone, PyErr, PyObject, PyResult, PythonObject, ToPyObject};
use cpython::exc::{IndexError, NotImplementedError, ValueError};

use seahash::SeaHasher;

pub struct SeaHashBuilder;

impl hash::BuildHasher for SeaHashBuilder {
	type Hasher = SeaHasher;

	#[inline]
	fn build_hasher(&self) -> Self::Hasher { SeaHasher::new() }
}

fn generating_function<F, K>(lattice_words: &super::LatticeWords, mut f: F) -> HashMap<K, usize, SeaHashBuilder>
	where F: FnMut(super::LatticeWord<&[u8]>) -> K,
	      K: cmp::Eq + hash::Hash + ToPyObject,
{
	let mut map = HashMap::with_hasher(SeaHashBuilder);

	let mut iter = lattice_words.streaming_iter();
	while let Some(word) = iter.next() {
		*map.entry(f(word)).or_insert(0) += 1;
	}

	map
}

py_class!(pub class LatticeWords |py| {
	data lattice_words: super::LatticeWords;

	def __new__(_cls, weight: Vec<u8>) -> PyResult<Self> {
		match super::LatticeWords::new(weight) {
			Ok(lattice_words) =>
				Self::create_instance(
					py,
					lattice_words,
				),
			Err(s) => Err(PyErr::new_lazy_init(
				py.get_type::<ValueError>(),
				Some(s.to_py_object(py).into_object()),
			)),
		}
	}

	def maj_cdes_dict(&self) -> PyResult<HashMap<(usize, usize), usize, SeaHashBuilder>> {
		let lattice_words = self.lattice_words(py);

		{
			let mut iter = lattice_words.weight().iter();
			let first = iter.next();
			for entry in iter {
				if Some(entry) != first {
					return Err(
						PyErr::new_lazy_init(
							py.get_type::<NotImplementedError>(),
							Some("only implemented for rectangular shapes".to_py_object(py).into_object()),
						)
					)
				}
			}
		}

		let mut tracking_shape = Vec::with_capacity(lattice_words.weight().len());
		unsafe {
			tracking_shape.set_len(lattice_words.weight().len());
		}

		Ok(generating_function(
			lattice_words,
			|word|
				(
					word.major_index(),
					word.tableau_cyclic_descents_with_tracking_shape(&mut *tracking_shape).count(),
				),
		))
	}

	def maj_des_dict(&self) -> PyResult<HashMap<(usize, usize), usize, SeaHashBuilder>> {
		let lattice_words = self.lattice_words(py);

		Ok(generating_function(
			lattice_words,
			|word|
				(
					word.major_index(),
					word.ascents().count(),
				),
		))
	}

	def __iter__(&self) -> PyResult<LatticeWordsIter> {
		LatticeWordsIter::create_instance(
			py,
			RefCell::new(
				self.lattice_words(py).clone().into_iter()
			)
		)
	}

	def __repr__(&self) -> PyResult<String> {
		let mut iter = self.lattice_words(py).weight().iter();

		Ok(if let Some(first_elt) = iter.next() {
			let (lower_hint, _) = iter.size_hint();

			let mut res = String::with_capacity(25 + 3*lower_hint);

			res.push_str("lattice words of weight ");

			write!(&mut res, "{}", first_elt).unwrap();

			for elt in iter {
				res.push_str(", ");
				write!(&mut res, "{}", elt).unwrap();
			}

			res
		} else {
			"lattice words of weight 0".to_owned()
		})
	}
});

py_class!(pub class LatticeWordsIter |py| {
	data iter: RefCell<super::LatticeWordsIter<Box<[u8]>>>;

	def __iter__(&self) -> PyResult<PyObject> {
		Ok(self.as_object().clone_ref(py))
	}

	def __next__(&self) -> PyResult<Option<LatticeWord>> {
		Ok(match self.iter(py).borrow_mut().next() {
			Some(x) => Some(LatticeWord::create_instance(py, x.into())?),
			None => None
		})
	}
});

py_class!(pub class LatticeWord |py| {
	data lattice_word: super::LatticeWord<Arc<Box<[u8]>>>;

	def __new__(_cls, word: Vec<u8>) -> PyResult<Self> {
		let word = Arc::new(word.into_boxed_slice());
		match super::LatticeWord::new(word) {
			Ok(lattice_word) => Self::create_instance(py, lattice_word),
			Err(s) => Err(PyErr::new_lazy_init(
				py.get_type::<ValueError>(),
				Some(s.to_py_object(py).into_object()),
			)),
		}
	}

	def __richcmp__(&self, other: LatticeWord, op: CompareOp) -> PyResult<bool> {
		let this = self.lattice_word(py);
		let other = other.lattice_word(py);
		Ok(match op {
			CompareOp::Lt => this <  other,
			CompareOp::Le => this <= other,
			CompareOp::Eq => this == other,
			CompareOp::Ne => this != other,
			CompareOp::Ge => this >= other,
			CompareOp::Gt => this >  other,
		})
	}

	def __len__(&self) -> PyResult<usize> {
		Ok(self.lattice_word(py).len())
	}

	def __getitem__(&self, index: usize) -> PyResult<u8> {
		match self.lattice_word(py).get(index) {
			Some(value) => Ok(*value),
			None => Err(PyErr::new_lazy_init(
				py.get_type::<IndexError>(),
				Some("index out of range".to_py_object(py).into_object()),
			))
			}
		}

	def __repr__(&self) -> PyResult<String> {
		let mut iter = self.lattice_word(py).iter();

		Ok(if let Some(first_elt) = iter.next() {
			let (lower_hint, _) = iter.size_hint();

			if *self.lattice_word(py).iter().max().unwrap() < 10 {
				let mut res = String::with_capacity(14 + lower_hint);

				res.push_str("lattice word ");

				write!(&mut res, "{}", first_elt).unwrap();

				for elt in iter {
					write!(&mut res, "{}", elt).unwrap();
				}

				res
			} else {
				let mut res = String::with_capacity(14 + 3*lower_hint);

				res.push_str("lattice_word ");

				write!(&mut res, "{}", first_elt).unwrap();

				for elt in iter {
					res.push_str(",");
					write!(&mut res, "{}", elt).unwrap();
				}

				res
			}
		} else {
			"empty lattice word".to_owned()
		})
	}

	def __iter__(&self) -> PyResult<LatticeWordIter> {
		LatticeWordIter::create_instance(py, RefCell::new(self.lattice_word(py).clone().into_iter()))
	}

	def descents(&self) -> PyResult<ScentIter> {
		ScentIter::create_instance(py, RefCell::new(self.lattice_word(py).clone().into_descents()))
	}

	def ascents(&self) -> PyResult<ScentIter> {
		ScentIter::create_instance(py, RefCell::new(self.lattice_word(py).clone().into_ascents()))
	}

	def major_index(&self) -> PyResult<usize> {
		Ok(self.lattice_word(py).major_index())
	}

	def promotion(&self, count: usize = 1) -> PyResult<Self> {
		match self.lattice_word(py).promotion(Some(count)) {
			Ok(word) => Self::create_instance(py, word.into()),
			Err(s) => Err(PyErr::new_lazy_init(
				py.get_type::<NotImplementedError>(),
				Some(s.to_py_object(py).into_object()),
			)),
		}
	}

	def promotion_order(&self) -> PyResult<usize> {
		match self.lattice_word(py).promotion_order() {
			Ok(order) => Ok(order),
			Err(s) => Err(PyErr::new_lazy_init(
				py.get_type::<NotImplementedError>(),
				Some(s.to_py_object(py).into_object()),
			)),
		}
	}

	def tableau_cyclic_descents(&self) -> PyResult<TableauCyclicDescentIter> {
		match self.lattice_word(py).clone().into_tableau_cyclic_descents() {
			Ok(iter) => TableauCyclicDescentIter::create_instance(py, RefCell::new(iter)),
			Err(s) => Err(PyErr::new_lazy_init(
				py.get_type::<NotImplementedError>(),
				Some(s.to_py_object(py).into_object()),
			)),
		}
	}
});

py_class!(pub class LatticeWordIter |py| {
	data iter: RefCell<<super::LatticeWord<Arc<Box<[u8]>>> as IntoIterator>::IntoIter>;

	def __iter__(&self) -> PyResult<PyObject> {
		Ok(self.as_object().clone_ref(py))
	}

	def __next__(&self) -> PyResult<Option<u8>> {
		Ok(self.iter(py).borrow_mut().next())
	}
});

py_class!(pub class ScentIter |py| {
	data iter: RefCell<super::ScentIter<Arc<Box<[u8]>>>>;

	def __iter__(&self) -> PyResult<PyObject> {
		Ok(self.as_object().clone_ref(py))
	}

	def __next__(&self) -> PyResult<Option<usize>> {
		Ok(self.iter(py).borrow_mut().next())
	}
});

py_class!(pub class TableauCyclicDescentIter |py| {
	data iter: RefCell<super::TableauCyclicDescentIter<Arc<Box<[u8]>>, Box<[u8]>>>;

	def __iter__(&self) -> PyResult<PyObject> {
		Ok(self.as_object().clone_ref(py))
	}

	def __next__(&self) -> PyResult<Option<usize>> {
		Ok(self.iter(py).borrow_mut().next())
	}
});
