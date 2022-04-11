#![feature(slice_group_by)]
#![feature(generic_associated_types)]
#![feature(extend_one)]
use canonical::Canonize;
use std::collections::BTreeSet;
use std::fmt;
use std::hash::Hash;

pub trait Value: Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash + fmt::Debug {}

impl<T: Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash + fmt::Debug> Value for T {}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Term<T> {
	Value(T),
	Var(usize),
}

impl<T: Value> Term<T> {
	fn apply_morphism<F>(&self, f: F) -> Self
	where
		F: Fn(&usize) -> usize,
	{
		match self {
			Self::Value(v) => Self::Value(*v),
			Self::Var(x) => Self::Var(f(x)),
		}
	}
}

pub type Triple<T> = rdf_types::Triple<Term<T>, Term<T>, Term<T>>;

/// Scoped gRDF graph.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Graph<T: Value> {
	/// Actual gRDF graph.
	graph: grdf::BTreeGraph<Term<T>>,

	/// Number of variables in the graph.
	///
	/// Variables **must** be indexed from 0 to `variable_count`.
	variable_count: usize,
}

impl<T: Value> fmt::Debug for Graph<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.graph.fmt(f)
	}
}

impl<T: Value> Canonize for Graph<T> {
	type Elements = usize;
	type Color = Vec<Color<T>>;
	type Cache = Cache;

	fn elements(&self) -> &usize {
		&self.variable_count
	}

	fn initialize_cache(&self) -> Cache {
		let mut neighbors = Vec::new();
		neighbors.resize_with(self.variable_count, BTreeSet::new);

		for rdf_types::Triple(s, p, o) in &self.graph {
			match s {
				Term::Var(x) => match p {
					Term::Var(y) => match o {
						Term::Var(z) => {
							neighbors[*x].insert(*y);
							neighbors[*x].insert(*z);
							neighbors[*y].insert(*x);
							neighbors[*y].insert(*z);
							neighbors[*z].insert(*x);
							neighbors[*z].insert(*y);
						}
						Term::Value(_) => {
							neighbors[*x].insert(*y);
							neighbors[*y].insert(*x);
						}
					},
					Term::Value(_) => {
						if let Term::Var(z) = o {
							neighbors[*x].insert(*z);
							neighbors[*z].insert(*x);
						}
					}
				},
				Term::Value(_) => {
					if let Term::Var(y) = p {
						if let Term::Var(z) = o {
							neighbors[*y].insert(*z);
							neighbors[*z].insert(*y);
						}
					}
				}
			}
		}

		Cache {
			stack: Vec::new(),
			map: Vec::new(),
			neighbors,
		}
	}

	fn initial_coloring(&self) -> Vec<Vec<Color<T>>> {
		let mut colors: Vec<Vec<Color<T>>> = Vec::new();
		colors.resize_with(self.variable_count, Vec::new);

		for triple in &self.graph {
			use rdf_types::Triple;
			use Term::*;
			match triple {
				// No values, one variable.
				Triple(Var(x), Var(y), Var(z)) if x == y && y == z => colors[*x].push(Color::XXX),
				// No values, two variables.
				Triple(Var(x), Var(y), Var(z)) if x == y => {
					colors[*x].push(Color::XXY);
					colors[*z].push(Color::YYX)
				}
				Triple(Var(x), Var(y), Var(z)) if y == z => {
					colors[*x].push(Color::XYY);
					colors[*z].push(Color::YXX)
				}
				Triple(Var(x), Var(y), Var(z)) if x == z => {
					colors[*x].push(Color::XYX);
					colors[*y].push(Color::YXY)
				}
				// No values, three variables.
				Triple(Var(x), Var(y), Var(z)) => {
					colors[*x].push(Color::XYZ);
					colors[*y].push(Color::YXZ);
					colors[*z].push(Color::YZX);
				}
				// One value, one variable.
				Triple(Var(x), Var(y), Value(a)) if x == y => colors[*x].push(Color::XXV(*a)),
				Triple(Var(x), Value(a), Var(y)) if x == y => colors[*x].push(Color::XVX(*a)),
				Triple(Value(a), Var(x), Var(y)) if x == y => colors[*x].push(Color::VXX(*a)),
				// One value, two variables.
				Triple(Var(x), Var(y), Value(a)) => {
					colors[*x].push(Color::XYV(*a));
					colors[*y].push(Color::YXV(*a))
				}
				Triple(Var(x), Value(a), Var(y)) => {
					colors[*x].push(Color::XVY(*a));
					colors[*y].push(Color::YVX(*a))
				}
				Triple(Value(a), Var(x), Var(y)) => {
					colors[*x].push(Color::VXY(*a));
					colors[*y].push(Color::VYX(*a))
				}
				// Two values, one variable.
				Triple(Var(x), Value(a), Value(b)) => colors[*x].push(Color::XVV(*a, *b)),
				Triple(Value(a), Var(x), Value(b)) => colors[*x].push(Color::VXV(*a, *b)),
				Triple(Value(a), Value(b), Var(x)) => colors[*x].push(Color::VVX(*a, *b)),
				// Three values.
				Triple(Value(_), Value(_), Value(_)) => (),
			}
		}

		for color in &mut colors {
			color.sort_unstable();
		}

		colors
	}

	fn refine_coloring(
		&self,
		cache: &mut Self::Cache,
		coloring: &mut canonical::ReversibleColoring<usize>,
	) {
		coloring.make_equitable_with(&mut cache.stack, &mut cache.map, |i| &cache.neighbors[*i])
	}

	fn apply_morphism<F>(&self, f: F) -> Self
	where
		F: Fn(&usize) -> usize,
	{
		let mut morphed_graph = grdf::BTreeGraph::new();
		for rdf_types::Triple(s, p, o) in &self.graph {
			morphed_graph.insert(rdf_types::Triple(
				s.apply_morphism(&f),
				p.apply_morphism(&f),
				o.apply_morphism(&f),
			))
		}

		Self {
			graph: morphed_graph,
			variable_count: self.variable_count,
		}
	}
}

pub struct Cache {
	stack: Vec<usize>,
	map: Vec<usize>,
	neighbors: Vec<BTreeSet<usize>>,
}

/// Variable color.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Color<T> {
	// No values, one variable.
	XXX,

	// No values, two variables.
	XYY,
	YXY,
	YYX,
	XXY,
	XYX,
	YXX,

	// No values, three variables.
	XYZ,
	YXZ,
	YZX,

	// One value, one variable.
	XXV(T),
	XVX(T),
	VXX(T),

	// One value, two variables.
	XYV(T),
	XVY(T),
	YXV(T),
	VXY(T),
	YVX(T),
	VYX(T),

	// Two values, one variable.
	XVV(T, T),
	VXV(T, T),
	VVX(T, T),
}

fn make_graph<T: Value, I: IntoIterator<Item = Triple<T>>>(
	variable_count: usize,
	triples: I,
) -> Graph<T> {
	Graph {
		graph: triples.into_iter().collect(),
		variable_count,
	}
}

fn make_random_graph(variable_count: usize, max_len: usize) -> Graph<bool> {
	use rand::Rng;

	fn random_term(variable_count: usize) -> Term<bool> {
		if rand::random::<bool>() {
			Term::Var(rand::thread_rng().gen_range(0..variable_count))
		} else {
			Term::Value(rand::random::<bool>())
		}
	}

	make_graph(
		variable_count,
		(0..max_len).map(|_| {
			rdf_types::Triple(
				random_term(variable_count),
				random_term(variable_count),
				random_term(variable_count),
			)
		}),
	)
}

fn random_morphism<T: Value>(graph: &Graph<T>) -> Graph<T> {
	use rand::seq::SliceRandom;
	let mut morphism: Vec<_> = (0..graph.variable_count).collect();
	morphism.shuffle(&mut rand::thread_rng());
	graph.apply_morphism(|i| morphism[*i])
}

#[test]
fn simple_no_automorphism() {
	use rdf_types::Triple;
	use Term::*;

	let a: Graph<bool> = make_graph(3, [Triple(Var(0), Var(1), Var(2))]);

	let b: Graph<bool> = make_graph(3, [Triple(Var(2), Var(1), Var(0))]);

	assert_eq!(a.canonize(), b.canonize())
}

#[test]
fn simple_automorphism() {
	use rdf_types::Triple;
	use Term::*;

	let a: Graph<bool> = make_graph(
		3,
		[
			Triple(Var(0), Var(1), Var(2)),
			Triple(Var(1), Var(0), Var(2)),
		],
	);

	let b: Graph<bool> = make_graph(
		3,
		[
			Triple(Var(2), Var(1), Var(0)),
			Triple(Var(1), Var(2), Var(0)),
		],
	);

	assert_eq!(a.canonize(), b.canonize())
}

fn test_random(variable_count: usize, max_len: usize) {
	for _ in 0..100 {
		let a = make_random_graph(variable_count, max_len);
		let canonized_a = a.canonize();

		for _ in 0..10 {
			let b = random_morphism(&a);
			assert_eq!(canonized_a, b.canonize())
		}
	}
}

/// Test that two random graphs do not have the same normal form in general.
///
/// In theory, this test may fail even if the code is correct,
/// but with a very low probability.
fn test_random_negative(variable_count: usize, max_len: usize) {
	for _ in 0..100 {
		let a = make_random_graph(variable_count, max_len);
		let b = make_random_graph(variable_count, max_len);

		if a.canonize() != b.canonize() {
			return; // success
		}
	}

	assert!(false)
}

#[test]
fn random_3_10() {
	test_random(3, 10)
}

#[test]
fn random_3_10_neg() {
	test_random_negative(3, 10)
}

#[test]
fn random_5_10() {
	test_random(5, 10)
}

#[test]
fn random_5_10_neg() {
	test_random_negative(5, 10)
}

#[test]
fn random_5_100() {
	test_random(5, 100)
}

#[test]
fn random_5_100_neg() {
	test_random_negative(5, 100)
}

#[test]
fn random_10_100() {
	test_random(10, 100)
}

#[test]
fn random_10_100_neg() {
	test_random_negative(10, 100)
}

// This one can be super slow.
#[test]
fn random_50_100() {
	test_random(50, 100)
}

#[test]
fn random_50_100_neg() {
	test_random_negative(50, 100)
}
