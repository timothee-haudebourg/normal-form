use crate::{ReversibleColoring, Set};

pub struct Node<S: Set + ?Sized> {
	path: Vec<S::Item>, // TODO: Replace with a more memory efficient type.
	coloring: ReversibleColoring<S>,
}

impl<S: Set + ?Sized> Node<S> {
	/// Create a new root cell.
	pub fn root(coloring: ReversibleColoring<S>) -> Self {
		Self {
			path: Vec::new(),
			coloring,
		}
	}

	pub fn path(&self) -> &Vec<S::Item> {
		&self.path
	}

	pub fn coloring(&self) -> &ReversibleColoring<S> {
		&self.coloring
	}

	pub fn restore(&mut self, n: usize) {
		debug_assert_eq!(self.path.len(), self.coloring.depth());
		self.coloring.restore(n);
		self.path.truncate(self.path.len() - n);
		debug_assert_eq!(self.path.len(), self.coloring.depth());
	}

	pub fn children_color(&self) -> Option<&[S::Item]> {
		self.coloring.colors().find(|&color| color.len() > 1)
	}

	// pub fn children(&self) -> Children<S> {
	// 	match self.target_cell() {
	// 		Some(target_cell) => {
	// 			Children::Node {
	// 				path: &self.path,
	// 				coloring: &self.coloring,
	// 				target_cell,
	// 				i: 0
	// 			}
	// 		}
	// 		None => Children::Leaf
	// 	}
	// }

	// pub fn revert(&mut self, n: usize) -> bool {
	// 	// We use `n + 1` because at depth 0 we may have an initial coloring with bounds of tag 0.
	// 	// Bound tags of the search tree below the root node start at 1.
	// 	self.coloring.retain_bounds(|bound| bound.tag <= n + 1)
	// }

	fn individualize<F>(&mut self, child: S::Item, mut refine: F)
	where
		F: FnMut(&mut ReversibleColoring<S>),
	{
		debug_assert_eq!(self.path.len(), self.coloring.depth());
		debug_assert!(!self.path.contains(&child));

		self.coloring.begin();
		self.coloring.individualize(&child);
		refine(&mut self.coloring);
		self.path.push(child);

		debug_assert_eq!(self.path.len(), self.coloring.depth());
	}

	pub fn into_first_child_leaf<F>(mut self, mut refine: F) -> Self
	where
		F: FnMut(&mut ReversibleColoring<S>),
	{
		while let Some(color) = self.children_color() {
			let child = color[0].clone();
			self.individualize(child, &mut refine)
		}

		self
	}

	pub fn into_next_leaf<F>(mut self, mut refine: F) -> Option<Self>
	where
		F: FnMut(&mut ReversibleColoring<S>),
	{
		debug_assert_eq!(self.path.len(), self.coloring.depth());
		let last = self.path.pop()?;
		self.coloring.restore(1); // undo individualization & refinement.

		let color_index = self.coloring.color_index_of(&last).unwrap();
		let color = self.coloring.get(color_index).unwrap();
		let next_sibling_index = color.binary_search(&last).unwrap() + 1;
		match color.get(next_sibling_index) {
			Some(next_sibling) => {
				// move to next sibling...
				let next_sibling = next_sibling.clone();
				self.individualize(next_sibling, &mut refine);

				while let Some(color) = self.children_color() {
					// ...then move to leaf
					let child = color[0].clone();
					self.individualize(child, &mut refine)
				}

				Some(self)
			}
			None => {
				// move to parent node...
				debug_assert_eq!(self.path.len(), self.coloring.depth());
				self.into_next_leaf(refine) // ...then move to sibling leaf
			}
		}
	}
}

// pub enum Children<'a, S: Set + ?Sized> {
// 	Leaf,
// 	Node {
// 		path: &'a Vec<S::Item>,
// 		coloring: &'a ReversibleColoring<S>,
// 		target_cell: &'a [S::Item],

// 		/// Index of the next child item in the target cell.
// 		i: usize
// 	}
// }

// impl<'a, S: Set + ?Sized> Iterator for Children<'a, S> where S::Map<usize>: Clone {
// 	type Item = Node<S>;

// 	fn next(&mut self) -> Option<Self::Item> {
// 		match self {
// 			Self::Leaf => None,
// 			Self::Node { path, coloring, target_cell, i } => {
// 				if *i < target_cell.len() {
// 					let item = &target_cell[*i];

// 					let mut child_path = path.clone();
// 					child_path.push(item.clone());

// 					let mut child_coloring = coloring.clone();
// 					child_coloring.individualize(item);

// 					*i += 1;
// 					Some(Node {
// 						path: child_path,
// 						coloring: child_coloring
// 					})
// 				} else {
// 					None
// 				}
// 			}
// 		}
// 	}
// }
