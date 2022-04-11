use super::{Map, Set};

impl Set for usize {
	type Item = usize;

	/// Map type, binding each item of the graph to a value `V`.
	///
	/// ## Example
	///
	/// `Vec<V>`.
	type Map<V> = Vec<V>;

	/// Items iterator.
	type Iter<'a> = std::ops::Range<usize>;

	/// The number of elements in the set.
	fn len(&self) -> usize {
		*self
	}

	/// Returns an iterator over the items of the set.
	fn iter(&self) -> Self::Iter<'_> {
		0..*self
	}

	fn map<V: Clone, F>(&self, f: F) -> Self::Map<V>
	where
		F: Fn(&Self::Item) -> V,
	{
		let mut map = Vec::with_capacity(*self);
		for i in 0..*self {
			map.push(f(&i))
		}
		map
	}
}

impl<T> Map<usize, T> for Vec<T> {
	fn get(&self, key: &usize) -> Option<&T> {
		self.as_slice().get(*key)
	}

	fn set(&mut self, key: &usize, value: T) {
		self[*key] = value
	}

	fn map<F>(&mut self, f: F)
	where
		F: Fn(&usize, T) -> T,
	{
		for (i, v) in self.iter_mut().enumerate() {
			unsafe {
				let t = std::ptr::read(v);
				std::ptr::write(v, f(&i, t));
			}
		}
	}
}
