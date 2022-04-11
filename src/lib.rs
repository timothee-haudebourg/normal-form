#![feature(generic_associated_types)]
#![feature(is_sorted)]
use std::collections::BTreeMap;

mod coloring;
pub mod set;
mod tree;

pub use coloring::{Coloring, ReversibleColoring};
use set::Map;
pub use set::Set;

pub trait Canonize: Ord + Sized {
	type Elements: Set;
	type Color: Ord;
	type Cache;

	fn initialize_cache(&self) -> Self::Cache;

	fn elements(&self) -> &Self::Elements;

	fn initial_coloring(&self) -> <Self::Elements as Set>::Map<Self::Color>;

	fn refine_coloring(
		&self,
		_cache: &mut Self::Cache,
		_coloring: &mut ReversibleColoring<Self::Elements>,
	) {
		// nothing by default.
	}

	/// Apply the given morphism.
	fn apply_morphism<F>(&self, morphism: F) -> Self
	where
		F: Fn(&<Self::Elements as Set>::Item) -> usize;

	/// Computes the canonical form of this object.
	fn canonize(&self) -> Self
	where
		<Self::Elements as Set>::Map<usize>: Clone,
	{
		use std::collections::btree_map::Entry;
		let mut cache = self.initialize_cache();
		let elements = self.elements();
		let mut node = Some(
			tree::Node::root(ReversibleColoring::from_coloring(
				elements,
				Coloring::from_map(elements, &self.initial_coloring()),
			))
			.into_first_child_leaf(|coloring| self.refine_coloring(&mut cache, coloring)),
		);
		let mut automorphisms: BTreeMap<Self, Vec<<Self::Elements as Set>::Item>> = BTreeMap::new();

		while let Some(mut n) = node {
			let permutation = n.coloring().as_permutation().unwrap();
			let morphed = self.apply_morphism(|i| *permutation.get(i).unwrap());
			match automorphisms.entry(morphed) {
				Entry::Occupied(entry) => {
					// We found an automorphism with a previous branch, we can prune the search tree!
					// We can prune up to the parent node sharing the longest prefix path.
					// Why: because the first different choice lead to an automorphism.
					// Any other leaf node morphism in this branch will be an automorphism with
					// one of the leaves in the previous branch.
					let len = n.path().len();

					// Step 1: We find the longest common prefix path length.
					let prefix_len = longest_common_prefix_len(n.path(), entry.get());

					// Step 2: We skip the other nodes in this branch and directly
					// go back to the parent node of depth `prefix_len`.
					// More precisely, we go back to the parent node of depth
					// `prefix_len + 1` (just after the divergence), and let the
					// call to `into_next_leaf` below move up to the parent and to the
					// next leaf node.
					n.restore(len - prefix_len - 1); // prune the search tree.
				}
				Entry::Vacant(entry) => {
					entry.insert(n.path().clone());
				}
			}

			node = n.into_next_leaf(|coloring| self.refine_coloring(&mut cache, coloring));
		}

		automorphisms.into_keys().next().unwrap()
	}
}

fn longest_common_prefix_len<T: PartialEq>(a: &[T], b: &[T]) -> usize {
	let mut n = 0;

	for (a, b) in a.iter().zip(b) {
		if a == b {
			n += 1
		} else {
			break;
		}
	}

	n
}
