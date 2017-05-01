use std::ops::Deref;

pub trait FullDeref {
	type Target: ?Sized;

	fn full_deref(&self) -> &Self::Target;
}

impl<T> FullDeref for [T] {
	type Target = Self;

	#[inline]
	fn full_deref(&self) -> &Self { self }
}

impl<T> FullDeref for T
	where T: Deref,
	      T::Target: FullDeref
{
	type Target = <T::Target as FullDeref>::Target;

	#[inline]
	fn full_deref(&self) -> &Self::Target { (*self).full_deref() }
}
