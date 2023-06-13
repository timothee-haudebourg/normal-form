use super::{Map, Set};

macro_rules! natural_set {
	($($ty:ident),*) => {
		$(
			impl Set for $ty {
				type Item = $ty;

				/// Map type, binding each item of the graph to a value `V`.
				///
				/// ## Example
				///
				/// `Vec<V>`.
				type Map<V> = Vec<V>;

				/// Items iterator.
				type Iter<'a> = std::ops::Range<$ty>;

				/// The number of elements in the set.
				fn len(&self) -> usize {
					*self as usize
				}

				/// Returns an iterator over the items of the set.
				fn iter(&self) -> Self::Iter<'_> {
					0..*self
				}

				fn map<V: Clone, F>(&self, f: F) -> Self::Map<V>
				where
					F: Fn(&Self::Item) -> V,
				{
					let mut map = Vec::with_capacity(*self as usize);
					for i in 0..*self {
						map.push(f(&i))
					}
					map
				}
			}

			impl<T> Map<$ty, T> for Vec<T> {
				fn len(&self) -> usize {
					self.len()
				}

				fn get(&self, key: &$ty) -> Option<&T> {
					self.as_slice().get(*key as usize)
				}

				fn set(&mut self, key: &$ty, value: T) {
					self[*key as usize] = value
				}

				fn map<F>(&mut self, f: F)
				where
					F: Fn(&$ty, T) -> T,
				{
					for (i, v) in self.iter_mut().enumerate() {
						unsafe {
							let t = std::ptr::read(v);
							std::ptr::write(v, f(&(i as $ty), t));
						}
					}
				}
			}
		)*
	};
}

natural_set!(u32, u64, usize);
