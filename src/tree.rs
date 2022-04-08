use crate::{
	Set,
	ReversibleColoring
};

pub struct Node<S: Set> {
	path: Vec<S::Item>, // TODO: Replace with a more memory efficient type.
	coloring: ReversibleColoring<S>
}

impl<S: Set> Node<S> {
	/// Create a new root cell.
	pub fn root(coloring: ReversibleColoring<S>) -> Self {
		Self {
			path: Vec::new(),
			coloring
		}
	}

	pub fn is_leaf(&self) -> bool {
		self.coloring.is_discrete()
	}

	/// Target cell selection.
	pub fn target_cell_index(&self) -> Option<usize> {
		if self.is_leaf() {
			None
		} else {
			todo!()
		}
	}

	pub fn target_cell(&self) -> Option<&[S::Item]> {
		self.coloring.get(self.target_cell_index()?)
	}

	pub fn children(&self) -> Children<S> {
		match self.target_cell() {
			Some(target_cell) => {
				Children::Node {
					path: &self.path,
					coloring: &self.coloring,
					target_cell,
					i: 0
				}
			}
			None => Children::Leaf
		}
	}
}

pub enum Children<'a, S: Set> {
	Leaf,
	Node {
		path: &'a Vec<S::Item>,
		coloring: &'a ReversibleColoring<S>,
		target_cell: &'a [S::Item],

		/// Index of the next child item in the target cell.
		i: usize
	}
}

impl<'a, S: Set> Iterator for Children<'a, S> where S::Map<usize>: Clone {
	type Item = Node<S>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Leaf => None,
			Self::Node { path, coloring, target_cell, i } => {
				if *i < target_cell.len() {
					let item = &target_cell[*i];

					let mut child_path = path.clone();
					child_path.push(item.clone());

					let mut child_coloring = coloring.clone();
					child_coloring.individualize(item);

					*i += 1;
					Some(Node {
						path: child_path,
						coloring: child_coloring
					})
				} else {
					None
				}
			}
		}
	}
}