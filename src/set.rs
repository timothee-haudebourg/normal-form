mod r#usize;

#[allow(clippy::len_without_is_empty)]
/// Ordered set.
pub trait Set {
	/// Type of the items of the set.
	type Item: Clone + Ord;

	/// Map type, binding each item of the graph to a value `V`.
	///
	/// ## Example
	///
	/// `Vec<V>`.
	type Map<V>: Map<Self::Item, V>;

	/// Items iterator.
	type Iter<'a>: 'a + Iterator<Item = Self::Item>
	where
		Self: 'a;

	/// The number of elements in the set.
	fn len(&self) -> usize;

	/// Returns an iterator over the items of the set.
	fn iter(&self) -> Self::Iter<'_>;

	fn map<V: Clone, F>(&self, f: F) -> Self::Map<V>
	where
		F: Fn(&Self::Item) -> V;
}

pub trait Map<K, T> {
	fn len(&self) -> usize;

	fn is_empty(&self) -> bool {
		self.len() == 0
	}

	fn get(&self, key: &K) -> Option<&T>;

	fn set(&mut self, key: &K, value: T);

	fn map<F>(&mut self, f: F)
	where
		F: Fn(&K, T) -> T;
}
