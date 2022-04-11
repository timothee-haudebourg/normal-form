use crate::{set::Map, Set};
use derivative::Derivative;
use std::fmt;
use std::ops::Deref;

#[derive(Clone, Copy)]
pub struct Bound {
	pub offset: usize,
	pub depth: usize,
}

impl PartialEq for Bound {
	fn eq(&self, other: &Self) -> bool {
		self.offset == other.offset
	}
}

impl Eq for Bound {}

impl Bound {
	fn new(offset: usize) -> Self {
		Self { offset, depth: 0 }
	}
}

/// Coloring.
///
/// A coloring for the set `S` is a list `(W_1, W_2, ..., W_m)` such
/// that `{ W_1, W_2, ..., W_m }` is a coloring of `S`.
/// Each `W_i` is called a *cell* of the coloring.
#[derive(Derivative)]
#[derivative(Clone(bound = ""), PartialEq(bound = ""), Eq(bound = ""))]
pub struct Coloring<S: Set + ?Sized> {
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
	bounds: Vec<Bound>,
}

impl<S: Set + ?Sized> Coloring<S> {
	/// Creates a new unit coloring of the input `set`.
	pub fn new(set: &S) -> Self {
		let mut elements: Vec<_> = set.iter().collect();
		elements.sort_unstable();
		Self {
			elements,
			bounds: Vec::new(),
		}
	}

	#[cfg(test)]
	fn from_parts(elements: Vec<S::Item>, bounds: Vec<Bound>) -> Self {
		Self { elements, bounds }
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
			if map.get(&w[0]) != map.get(&w[1]) {
				bounds.push(Bound::new(i + 1))
			}
		}

		let mut result = Self { elements, bounds };

		result.sort_cells();

		result
	}

	fn reset_bounds(&mut self) {
		for b in &mut self.bounds {
			b.depth = 0
		}
	}

	/// Sort the elements in each cell.
	///
	/// This invariant should always be preserved,
	/// so this function is called to enforce it whenever an operation
	/// may have broken it.
	fn sort_cells(&mut self) {
		let mut start = 0;
		for &end in &self.bounds {
			self.elements[start..end.offset].sort_unstable();
			start = end.offset
		}
		self.elements[start..].sort_unstable()
	}

	pub fn color_index_of(&self, item: &S::Item) -> Option<usize> {
		for (i, color) in self.colors().enumerate() {
			if color.contains(item) {
				return Some(i);
			}
		}

		None
	}

	pub fn colors(&self) -> Colors<S> {
		Colors {
			coloring: self,
			i: 0,
		}
	}

	#[allow(clippy::len_without_is_empty)]
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

	pub fn color_start(&self, i: usize) -> Option<usize> {
		if i == 0 {
			Some(0)
		} else {
			Some(self.bounds.get(&(i - 1))?.offset)
		}
	}

	/// Returns the `i`th cell of the coloring, partitioning the given
	/// input `set`.
	pub fn get(&self, i: usize) -> Option<&[S::Item]> {
		let start = if i == 0 {
			0
		} else {
			self.bounds.get(&(i - 1))?.offset
		};

		match self.bounds.get(&i) {
			Some(end) => Some(&self.elements[start..end.offset]),
			None => Some(&self.elements[start..]),
		}
	}

	/// Returns the `i`th cell of the coloring, partitioning the given
	/// input `set`.
	fn get_mut(&mut self, i: usize) -> Option<&mut [S::Item]> {
		let start = if i == 0 {
			0
		} else {
			self.bounds.get(&(i - 1))?.offset
		};

		match self.bounds.get(&i) {
			Some(end) => Some(&mut self.elements[start..end.offset]),
			None => Some(&mut self.elements[start..]),
		}
	}
}

impl<S: Set + ?Sized> fmt::Debug for Coloring<S>
where
	S::Item: fmt::Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{{")?;

		for (i, color) in self.colors().enumerate() {
			if i > 0 {
				write!(f, ", ")?;
			}

			write!(f, "{:?}", color)?;
		}

		write!(f, "}}")
	}
}

pub struct Colors<'a, S: Set + ?Sized> {
	coloring: &'a Coloring<S>,
	i: usize,
}

impl<'a, S: Set + ?Sized> Iterator for Colors<'a, S> {
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

trait StartedRange {
	fn start(&self) -> usize;
}

impl StartedRange for std::ops::Range<usize> {
	fn start(&self) -> usize {
		self.start
	}
}

impl StartedRange for std::ops::RangeInclusive<usize> {
	fn start(&self) -> usize {
		*self.start()
	}
}

impl StartedRange for std::ops::RangeFrom<usize> {
	fn start(&self) -> usize {
		self.start
	}
}

/// A reversible ordered coloring.
#[derive(Derivative)]
#[derivative(Clone(bound = "S::Map<usize>: Clone"))]
pub struct ReversibleColoring<S: Set + ?Sized> {
	/// Coloring.
	coloring: Coloring<S>,

	/// Associates each element to the index of its cell in the coloring.
	reverse: S::Map<usize>,

	depth: usize,
}

impl<S: Set + ?Sized> PartialEq for ReversibleColoring<S> {
	fn eq(&self, other: &Self) -> bool {
		self.coloring == other.coloring
	}
}

impl<S: Set + ?Sized> Eq for ReversibleColoring<S> {}

impl<S: Set + ?Sized> fmt::Debug for ReversibleColoring<S>
where
	S::Item: fmt::Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.coloring.fmt(f)
	}
}

impl<S: Set + ?Sized> ReversibleColoring<S> {
	/// Creates a new reversible coloring.
	pub fn new(set: &S) -> Self {
		Self {
			coloring: Coloring::new(set),
			reverse: set.map(|_| 0),
			depth: 0,
		}
	}

	pub fn from_coloring(set: &S, mut coloring: Coloring<S>) -> Self {
		coloring.reset_bounds();
		let reverse = set.map(|item| coloring.color_index_of(item).unwrap());
		Self {
			coloring,
			reverse,
			depth: 0,
		}
	}

	pub fn color_index_of(&self, item: &S::Item) -> Option<usize> {
		self.reverse.get(item).cloned()
	}

	fn check(&self) -> bool {
		for (i, color) in self.colors().enumerate() {
			if !color.is_sorted() {
				return false;
			}

			for t in color {
				let reverse = self.color_index_of(t);
				if reverse != Some(i) {
					return false;
				}
			}
		}

		true
	}

	/// Returns the permutation represented by this coloring, if any.
	///
	/// If the coloring *discrete* then it defines a mapping
	/// `p : x -> self.reverse(x)` that is a permutation of the partitioned set.
	/// In that case, `Some(p)` is returned.
	/// Otherwise, if the coloring is not discrete, `None` is returned.
	pub fn as_permutation(&self) -> Option<&S::Map<usize>> {
		if self.coloring.is_discrete() {
			Some(&self.reverse)
		} else {
			None
		}
	}

	/// Refine the coloring such that `item` is in its own cell of size 1.
	///
	/// Returns `true` if the individualization succeeded,
	/// or `false` if `item` was already individualized.
	pub fn individualize(&mut self, item: &S::Item) -> bool {
		let color_index = self.color_index_of(item).unwrap();
		let color_start = self.color_start(color_index).unwrap();
		let color = self.coloring.get_mut(color_index).unwrap();

		if color.len() > 1 {
			let i = color.binary_search(item).unwrap();
			color.swap(0, i);
			color[1..].sort_unstable();
			self.coloring.bounds.insert(
				color_index,
				Bound {
					offset: color_start + 1,
					depth: self.depth,
				},
			);
			self.reverse.map(|t, c| {
				if c < color_index || t == item {
					c
				} else {
					c + 1
				}
			});

			debug_assert!(self.check());
			true
		} else {
			false
		}
	}

	pub fn deindividualize(&mut self, item: &S::Item) -> bool {
		let color_index = self.color_index_of(item).unwrap();
		let color = self.coloring.get_mut(color_index).unwrap();

		if color.len() == 1 && color_index < self.coloring.bounds.len() {
			self.coloring.bounds.remove(color_index);
			self.coloring.get_mut(color_index).unwrap().sort_unstable();
			self.reverse
				.map(|_, c| if c <= color_index { c } else { c - 1 });
			true
		} else {
			false
		}
	}

	pub(crate) fn begin(&mut self) {
		self.depth += 1;
	}

	pub(crate) fn restore(&mut self, n: usize) {
		let restored_depth = self.depth - n;
		self.retain_bounds(|b| b.depth <= restored_depth);
		self.depth = restored_depth;
	}

	pub(crate) fn depth(&self) -> usize {
		self.depth
	}

	fn retain_bounds<P>(&mut self, predicate: P) -> bool
	where
		P: FnMut(&Bound) -> bool,
	{
		let len = self.len();
		self.coloring.bounds.retain(predicate);
		if self.len() != len {
			self.coloring.sort_cells();
			for (c, color) in self.coloring.colors().enumerate() {
				for i in color {
					self.reverse.set(i, c);
				}
			}

			debug_assert!(self.check());
			true
		} else {
			false
		}
	}

	/// Checks if `self` is a finer than or equal to `other`, assuming they
	/// target the same set.
	pub fn is_finer_or_equal_to(&self, other: &Self) -> bool {
		let mut last_index = 0;

		for cell in self.coloring.colors() {
			let (first_item, cell_rest) = cell.split_first().unwrap();
			let index = other.color_index_of(first_item).unwrap();

			if index < last_index {
				return false;
			}

			if !cell_rest
				.iter()
				.all(|item| other.color_index_of(item).unwrap() == index)
			{
				return false;
			}

			last_index = index
		}

		true
	}

	pub fn refine<F, C: Ord>(&mut self, f: F) -> bool
	where
		F: Fn(&S::Item) -> C,
	{
		self.refine_with(&mut Vec::new(), f)
	}

	/// Refine this coloring using the given sub-coloring.
	///
	/// The `refined_colors` array will be expended to include the newly added colors indexes.
	/// Colors that have not been refined are not added to the array, even if their index changes.
	/// If the array already contains old color indexes, they will be updated in place to
	/// the new color index.
	pub fn refine_with<F, C: Ord>(&mut self, refined_colors: &mut Vec<usize>, f: F) -> bool
	where
		F: Fn(&S::Item) -> C,
	{
		#[allow(clippy::too_many_arguments)]
		fn refine_color<S: Set + ?Sized, F, C: Ord, R>(
			coloring: &mut Coloring<S>,
			reverse: &mut S::Map<usize>,
			depth: usize,
			refined_colors: &mut Vec<usize>,
			already_refined_len: usize,
			f: F,
			range: R,
			old_color_index: usize,
			mut new_color_index: usize,
		) -> usize
		where
			F: Fn(&S::Item) -> C,
			R: StartedRange + Clone,
			Vec<S::Item>: std::ops::IndexMut<R, Output = [S::Item]>
				+ std::ops::IndexMut<usize, Output = S::Item>,
		{
			coloring.elements[range.clone()].sort_unstable_by_key(|i| f(i));
			reverse.set(&coloring.elements[range.start()], new_color_index);
			for (i, w) in coloring.elements[range.clone()].windows(2).enumerate() {
				if f(&w[0]) != f(&w[1]) {
					refined_colors.push(new_color_index);
					new_color_index += 1;
					coloring.bounds.push(Bound {
						offset: range.start() + i + 1,
						depth,
					})
				}
				reverse.set(&w[1], new_color_index);
			}

			if old_color_index != new_color_index {
				let mut present = false;
				for c in &mut refined_colors[..already_refined_len] {
					if *c == old_color_index {
						*c = new_color_index;
						present = true;
						break;
					}
				}

				if !present {
					refined_colors.push(new_color_index);
				}
			}

			new_color_index
		}

		let already_refined_len = refined_colors.len();
		let len = self.len();
		let mut old_bounds = Vec::new();
		std::mem::swap(&mut old_bounds, &mut self.coloring.bounds);
		let mut start = 0;
		let mut old_color_index = 0;
		let mut new_color_index = 0;
		for end in old_bounds {
			new_color_index = refine_color(
				&mut self.coloring,
				&mut self.reverse,
				self.depth,
				refined_colors,
				already_refined_len,
				&f,
				start..end.offset,
				old_color_index,
				new_color_index,
			);

			self.coloring.bounds.push(end);
			old_color_index += 1;
			new_color_index += 1;
			start = end.offset
		}

		refine_color(
			&mut self.coloring,
			&mut self.reverse,
			self.depth,
			refined_colors,
			already_refined_len,
			&f,
			start..,
			old_color_index,
			new_color_index,
		);

		self.coloring.sort_cells();
		debug_assert!(self.check());
		self.len() != len
	}

	pub fn make_equitable<'i, F, I>(&mut self, set: &S, neighbors: F)
	where
		F: Fn(&S::Item) -> I,
		I: IntoIterator<Item = &'i S::Item>,
		S::Item: 'i,
	{
		let mut stack = Vec::new();
		let mut map = set.map(|_| 0);

		self.make_equitable_with(&mut stack, &mut map, neighbors)
	}

	/// Make this coloring equitable.
	///
	/// The stack is empty when this function returns.
	pub fn make_equitable_with<'i, F, I>(
		&mut self,
		stack: &mut Vec<usize>,
		map: &mut S::Map<usize>,
		neighbors: F,
	) where
		F: Fn(&S::Item) -> I,
		I: IntoIterator<Item = &'i S::Item>,
		S::Item: 'i,
	{
		stack.clear();
		stack.extend(0..self.len());

		while !stack.is_empty() && !self.is_discrete() {
			let color = stack.pop().unwrap();

			// For the given color, we associate for each element
			// the number of edges that connects to a element of
			// this color.
			map.map(|i, _| {
				let mut count = 0;
				for j in neighbors(i) {
					let j_color = self.color_index_of(j).unwrap();
					if j_color == color {
						count += 1
					}
				}
				count
			});

			// Refine the partition using the computed information.
			// This will also update the stack with the new colors.
			self.refine_with(stack, |i| map.get(i));
		}

		stack.clear()
	}
}

impl<S: Set + ?Sized> Deref for ReversibleColoring<S> {
	type Target = Coloring<S>;

	fn deref(&self) -> &Self::Target {
		&self.coloring
	}
}

#[cfg(test)]
mod tests {
	// macro_rules! coloring {
	// 	{ $([ $($i:expr),* ]),* } => {
	// 		{
	// 			let mut elements = Vec::new();
	// 			let mut bounds = Vec::new();

	// 			$(
	// 				if !elements.is_empty() {
	// 					bounds.push($crate::coloring::Bound::new(elements.len()));
	// 				}

	// 				$(
	// 					elements.push($i);
	// 				)*
	// 			)*

	// 			$crate::coloring::Coloring::from_parts(
	// 				elements,
	// 				bounds
	// 			)
	// 		}
	// 	};
	// }

	macro_rules! rcoloring {
		{ $set:tt : $([ $($i:expr),* ]),* } => {
			{
				let mut elements = Vec::new();
				let mut bounds = Vec::new();

				$(
					if !elements.is_empty() {
						bounds.push($crate::coloring::Bound::new(elements.len()));
					}

					$(
						elements.push($i);
					)*
				)*

				$crate::coloring::ReversibleColoring::from_coloring(&$set, $crate::coloring::Coloring::from_parts(
					elements,
					bounds
				))
			}
		};
	}

	#[test]
	fn individualize_01() {
		let mut coloring = rcoloring! { 1 : [ 0 ] };
		coloring.individualize(&0);
		assert_eq!(coloring, rcoloring! { 1 : [ 0 ] })
	}

	#[test]
	fn deindividualize_01() {
		let mut coloring = rcoloring! { 1 : [ 0 ] };
		coloring.deindividualize(&0);
		assert_eq!(coloring, rcoloring! { 1 : [ 0 ] })
	}

	#[test]
	fn individualize_02() {
		let mut coloring = rcoloring! { 2 : [ 0, 1 ] };
		coloring.individualize(&0);
		assert_eq!(coloring, rcoloring! { 2 : [ 0 ], [ 1 ] })
	}

	#[test]
	fn deindividualize_02() {
		let mut coloring = rcoloring! { 2 : [ 0 ], [ 1 ] };
		coloring.deindividualize(&0);
		assert_eq!(coloring, rcoloring! { 2 : [ 0, 1 ] })
	}

	#[test]
	fn individualize_03() {
		let mut coloring = rcoloring! { 2 : [ 0, 1 ] };
		coloring.individualize(&1);
		assert_eq!(coloring, rcoloring! { 2 : [ 1 ], [ 0 ] })
	}

	#[test]
	fn deindividualize_03() {
		let mut coloring = rcoloring! { 2 : [ 1 ], [ 0 ] };
		coloring.deindividualize(&1);
		assert_eq!(coloring, rcoloring! { 2 : [ 0, 1 ] })
	}

	#[test]
	fn individualize_04() {
		let mut coloring = rcoloring! { 3 : [ 0, 1, 2 ] };
		coloring.individualize(&0);
		assert_eq!(coloring, rcoloring! { 3 : [ 0 ], [ 1, 2 ] })
	}

	#[test]
	fn deindividualize_04() {
		let mut coloring = rcoloring! { 3 : [ 0 ], [ 1, 2 ] };
		coloring.deindividualize(&0);
		assert_eq!(coloring, rcoloring! { 3 : [ 0, 1, 2 ] })
	}

	#[test]
	fn individualize_05() {
		let mut coloring = rcoloring! { 3 : [ 0, 1, 2 ] };
		coloring.individualize(&1);
		assert_eq!(coloring, rcoloring! { 3 : [ 1 ], [ 0, 2 ] })
	}

	#[test]
	fn deindividualize_05() {
		let mut coloring = rcoloring! { 3 : [ 1 ], [ 0, 2 ] };
		coloring.deindividualize(&1);
		assert_eq!(coloring, rcoloring! { 3 : [ 0, 1, 2 ] })
	}

	#[test]
	fn individualize_06() {
		let mut coloring = rcoloring! { 3 : [ 0, 1, 2 ] };
		coloring.individualize(&2);
		assert_eq!(coloring, rcoloring! { 3 : [ 2 ], [ 0, 1 ] })
	}

	#[test]
	fn deindividualize_06() {
		let mut coloring = rcoloring! { 3 : [ 2 ], [ 0, 1 ] };
		coloring.deindividualize(&2);
		assert_eq!(coloring, rcoloring! { 3 : [ 0, 1, 2 ] })
	}

	#[test]
	fn refine_01() {
		let mut coloring = rcoloring! { 3 : [ 0, 1, 2 ] };
		coloring.refine(|i| match i {
			0 => 0,
			1 => 1,
			2 => 2,
			_ => unreachable!(),
		});

		assert_eq!(coloring, rcoloring! { 3 : [ 0 ], [ 1 ], [ 2 ] })
	}

	#[test]
	fn refine_02() {
		let mut coloring = rcoloring! { 3 : [ 0, 1, 2 ] };
		coloring.refine(|i| match i {
			0 => 0,
			1 => 0,
			2 => 1,
			_ => unreachable!(),
		});

		assert_eq!(coloring, rcoloring! { 3 : [ 0, 1 ], [ 2 ] })
	}

	#[test]
	fn refine_03() {
		let mut coloring = rcoloring! { 3 : [ 0, 1, 2 ] };
		coloring.refine(|i| match i {
			0 => 1,
			1 => 1,
			2 => 0,
			_ => unreachable!(),
		});

		assert_eq!(coloring, rcoloring! { 3 : [ 2 ], [ 0, 1 ] })
	}

	#[test]
	fn refine_04() {
		let mut coloring = rcoloring! { 4 : [ 0, 1 ], [ 2, 3 ] };
		coloring.refine(|i| match i {
			0 => 0,
			1 => 1,
			2 => 0,
			3 => 1,
			_ => unreachable!(),
		});

		assert_eq!(coloring, rcoloring! { 4 : [ 0 ], [ 1 ], [ 2 ], [ 3 ] })
	}

	#[test]
	fn refine_05() {
		let mut coloring = rcoloring! { 4 : [ 0, 1 ], [ 2, 3 ] };
		coloring.refine(|i| match i {
			0 => 0,
			1 => 1,
			2 => 2,
			3 => 2,
			_ => unreachable!(),
		});

		assert_eq!(coloring, rcoloring! { 4 : [ 0 ], [ 1 ], [ 2, 3 ] })
	}

	#[test]
	fn make_equitable_01() {
		let mut coloring = rcoloring! { 3 : [ 0 ], [ 1, 2 ] };
		coloring.make_equitable(&3, |i| match i {
			0 => (&[1usize] as &[_]),
			1 => &[0],
			2 => &[],
			_ => unreachable!(),
		});

		assert_eq!(coloring, rcoloring! { 3 : [ 0 ], [ 2 ], [ 1 ] })
	}
}
