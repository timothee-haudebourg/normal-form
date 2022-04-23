#![feature(generic_associated_types)]
#![feature(is_sorted)]
use std::collections::BTreeMap;

mod coloring;
pub mod set;
mod tree;

pub use coloring::{Coloring, ReversibleColoring};
use set::Map;
pub use set::Set;

/// Type for which a canonical form can be found.
pub trait Canonize: Ord + Sized {
	/// Set of elements that can be permuted in order to find the canonical form.
	type Elements: Set;

	/// Initial coloring of the permutable elements.
	type Color: Ord;

	/// Cached data used to refine the coloring at each step.
	/// 
	/// You can put in there the result of preliminary computations and allocations.
	type Cache;

	type Morphed: Ord;

	/// Initialize the cache.
	/// 
	/// This is the place to perform preliminary computations and allocations that will be
	/// useful every time the coloring must be refined.
	fn initialize_cache(&self) -> Self::Cache;

	/// Returns a reference to the permutable elements.
	fn elements(&self) -> &Self::Elements;

	/// Returns the initial coloring of the permutable elements.
	fn initial_coloring(&self) -> <Self::Elements as Set>::Map<Self::Color>;

	/// Refine the current coloring.
	fn refine_coloring(
		&self,
		_cache: &mut Self::Cache,
		_coloring: &mut ReversibleColoring<Self::Elements>,
	) {
		// nothing by default.
	}

	/// Apply the given morphism.
	fn apply_morphism<F>(&self, morphism: F) -> Self::Morphed
	where
		F: Fn(&<Self::Elements as Set>::Item) -> usize;

	/// Computes the canonical form of this object.
	fn canonical_form(&self) -> Self::Morphed
	where
		<Self::Elements as Set>::Map<usize>: Clone,
	{
		self.canonize().0	
	}

	/// Computes the canonical permutation of this object.
	fn canonical_permutation(&self) -> <Self::Elements as Set>::Map<usize>
	where
		<Self::Elements as Set>::Map<usize>: Clone,
	{
		self.canonize().1
	}

	/// Computes the canonical form of this object, with the associated permutation.
	fn canonize(&self) -> (Self::Morphed, <Self::Elements as Set>::Map<usize>)
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

		pub struct Automorphism<T: Canonize> {
			path: Vec<<T::Elements as Set>::Item>,
			permutation: <T::Elements as Set>::Map<usize>
		}

		let mut automorphisms: BTreeMap<Self::Morphed, Automorphism<Self>> = BTreeMap::new();

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
					let prefix_len = longest_common_prefix_len(n.path(), &entry.get().path);

					// Step 2: We skip the other nodes in this branch and directly
					// go back to the parent node of depth `prefix_len`.
					// More precisely, we go back to the parent node of depth
					// `prefix_len + 1` (just after the divergence), and let the
					// call to `into_next_leaf` below move up to the parent and to the
					// next leaf node.
					n.restore(len - prefix_len - 1); // prune the search tree.
				}
				Entry::Vacant(entry) => {
					entry.insert(Automorphism {
						path: n.path().clone(),
						permutation: permutation.clone()
					});
				}
			}

			node = n.into_next_leaf(|coloring| self.refine_coloring(&mut cache, coloring));
		}

		let (normal_form, data) = automorphisms.into_iter().next().unwrap();
		(normal_form, data.permutation)
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
