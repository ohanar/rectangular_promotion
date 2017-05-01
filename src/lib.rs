#![feature(fused)]
#![feature(loop_break_value)]

#[macro_use] extern crate cpython;
extern crate seahash;

mod full_deref;
mod lattice_word;
mod lattice_words;
mod pairs;
mod python;

pub use lattice_word::{LatticeWord, ScentIter, TableauCyclicDescentIter};
pub use lattice_words::{LatticeWords, LatticeWordsStreamingIter, LatticeWordsIter};

py_module_initializer!(rectangular_promotion, initrectangular_promotion, PyInit_rectangular_promotion, |py, m| {
	m.add(py, "LatticeWord", py.get_type::<python::LatticeWord>())?;
	m.add(py, "LatticeWords", py.get_type::<python::LatticeWords>())?;
	Ok(())
});