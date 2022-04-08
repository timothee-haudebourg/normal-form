#![feature(generic_associated_types)]

mod coloring;
mod tree;

pub use coloring::{
	Coloring,
	ReversibleColoring
};

pub trait Set {
	type Item: Clone + Ord;

	/// Map type, binding each item of the graph to a value `V`.
	/// 
	/// ## Example
	/// 
	/// `Vec<V>`.
	type Map<V>: Map<Self::Item, V>;

	type Iter<'a>: 'a + Iterator<Item=Self::Item> where Self: 'a;

	/// The number of elements in the set.
	fn len(&self) -> usize;

	fn iter(&self) -> Self::Iter<'_>;

	fn map<V: Clone, F>(&self, f: F) -> Self::Map<V> where F: Fn(&Self::Item) -> V;
}

pub trait Map<K, T> {
	fn get(&self, key: &K) -> Option<&T>;
}

pub type Vertex = usize;

pub trait Canonize {
	type Color;

	/// Returns the number of vertices.
	fn len(&self) -> usize;

	/// Returns the color for the given `vertex` that is invariant by
	/// isomorphism.
	fn invariant_color(&self, vertex: Vertex) -> Self::Color;

	/// Check if the given morphism is an endomorphism.
	fn is_endomorphism(&self, morphism: &[Vertex]) -> bool;

	/// Computes the canonical form of this object.
	fn canonical_substitution(&self) -> Vec<Vertex> {
		// enumerate all possible morphisms,
		// without swapping vertices that don't have the same color.
		// For each of them, check if we have an endomorphism.

		// ...

		todo!()
	}
}

