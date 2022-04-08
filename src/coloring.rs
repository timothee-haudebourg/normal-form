use crate::{
	Set,
	Map
};
use std::ops::Deref;
use derivative::Derivative;

/// Coloring.
/// 
/// A coloring for the set `S` is a list `(W_1, W_2, ..., W_m)` such
/// that `{ W_1, W_2, ..., W_m }` is a coloring of `S`.
/// Each `W_i` is called a *cell* of the coloring.
#[derive(Derivative)]
#[derivative(Clone(bound=""))]
pub struct Coloring<S: Set> {
	/// Elements of the partitioned set,
	/// ordered by cell index first, `S::Item::cmp` second (in each cell).
	elements: Vec<S::Item>,

	/// Bounds of the coloring.
	/// 
	/// If the partitioned set is { e_1, ..., e_n } then
	/// the list of bounds [ i_1, ..., i_k ] produces the coloring
	/// { { e_0, ..., e_{i_0} }, { e_{i_0 + 1}, .., e_{i_1}  }, ..., { e_{i_k + 1}, .. e_n } }
	///
	/// The resulting coloring contains `k` members, the length of the list.
	bounds: Vec<usize>,
}

impl<S: Set> Coloring<S> {
	/// Creates a new unit coloring of the input `set`.
	pub fn new(set: &S) -> Self {
		let mut elements: Vec<_> = set.iter().collect();
		elements.sort_unstable();
		Self {
			elements,
			bounds: Vec::new()
		}
	}

	/// Creates a new coloring from a map associating each item to an initial
	/// color of type `C`.
	pub fn from_map<C: Ord>(set: &S, map: &S::Map<C>) -> Self {
		// Sort elements by color.
		let mut elements: Vec<_> = set.iter().collect();
		elements.sort_unstable_by_key(|e| map.get(e).unwrap());
		
		// Create colors.
		let mut bounds = Vec::new();
		for (i, w) in elements.windows(2).enumerate() {
			if w[0] != w[1] {
				bounds.push(i+1)
			}
		}

		// Sort the elements in each cell.
		let mut start = 0;
		for &end in &bounds {
			elements[start..=end].sort_unstable();
			start = end + 1
		}
		if !bounds.is_empty() {
			elements[start..].sort_unstable()
		}
		
		Self {
			elements,
			bounds
		}
	}

	pub fn colors(&self) -> Colors<S> {
		Colors {
			coloring: self,
			i: 0
		}
	}

	/// Returns the number of colors in the coloring.
	pub fn len(&self) -> usize {
		self.bounds.len() + 1
	}

	/// Checks if the coloring is a unit coloring.
	pub fn is_unit(&self) -> bool {
		self.bounds.is_empty()
	}

	pub fn is_discrete(&self) -> bool {
		self.len() == self.elements.len()
	}

	/// Returns the `i`th cell of the coloring, partitioning the given
	/// input `set`.
	pub fn get(&self, i: usize) -> Option<&[S::Item]> {
		let start = if i == 0 {
			0
		} else {
			self.bounds.get(i - 1)? + 1
		};

		match self.bounds.get(i) {
			Some(end) => Some(&self.elements[start..=*end]),
			None => Some(&self.elements[start..])
		}
	}
}

pub struct Colors<'a, S: Set> {
	coloring: &'a Coloring<S>,
	i: usize
}

impl<'a, S: Set> Iterator for Colors<'a, S> {
	type Item = &'a [S::Item];

	fn next(&mut self) -> Option<Self::Item> {
		if self.i < self.coloring.len() {
			let cell = self.coloring.get(self.i).unwrap();
			self.i += 1;
			Some(cell)
		} else {
			None
		}
	}
}

/// A reversible ordered coloring.
#[derive(Derivative)]
#[derivative(Clone(bound="S::Map<usize>: Clone"))]
pub struct ReversibleColoring<S: Set> {
	/// Coloring.
	coloring: Coloring<S>,

	/// Associates each element to the index of its cell in the coloring.
	reverse: S::Map<usize>
}

impl<S: Set> ReversibleColoring<S> {
	/// Creates a new reversible coloring.
	pub fn new(set: &S) -> Self {
		Self {
			coloring: Coloring::new(set),
			reverse: set.map(|_| 0)
		}
	}

	pub fn cell_index_of(&self, item: &S::Item) -> Option<usize> {
		self.reverse.get(item).cloned()
	}

	/// Returns the permutation represented by this coloring, if any.
	/// 
	/// If the coloring *discrete* then it defines a mapping
	/// `p : x -> self.reverse(x)` that is a permutation of the partitioned set.
	/// In that case, `Some(p)` is returned.
	/// Otherwise, if the coloring is not discrete, `None` is returned.
	pub fn as_permutation(&self) -> Option<S::Map<usize>> where S::Map<usize>: Clone {
		if self.coloring.is_discrete() {
			Some(self.reverse.clone())
		} else {
			None
		}
	}

	/// Refine the coloring such that `item` is in its own cell of size 1.
	pub fn individualize(&mut self, item: &S::Item) {
		todo!()
	}

	/// Checks if `self` is a finer than or equal to `other`, assuming they
	/// target the same set.
	pub fn is_finer_or_equal_to(&self, other: &Self) -> bool {
		let mut last_index = 0;
		
		for cell in self.coloring.colors() {
			let (first_item, cell_rest) = cell.split_first().unwrap();
			let index = other.cell_index_of(first_item).unwrap();

			if index < last_index {
				return false
			}

			if !cell_rest.iter().all(|item| other.cell_index_of(item).unwrap() == index) {
				return false
			}

			last_index = index
		}

		true
	}
}

impl<S: Set> Deref for ReversibleColoring<S> {
	type Target = Coloring<S>;

	fn deref(&self) -> &Self::Target {
		&self.coloring
	}
}