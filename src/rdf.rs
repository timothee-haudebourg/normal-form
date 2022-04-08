#![feature(slice_group_by)]
#![feature(generic_associated_types)]
#![feature(extend_one)]
use std::collections::{
	HashMap,
	HashSet,
	BTreeMap,
	BTreeSet
};
use std::hash::Hash;
use std::fmt::Debug;
use std::borrow::Cow;

mod utils;

pub trait Value: Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash + Debug {}

#[derive(Clone, Default)]
pub struct Context(HashMap<u32, u32>);

impl Context {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn insert(&mut self, x: u32) -> u32 {
		let len = self.0.len();
		*self.0.entry(x).or_insert_with(|| len as u32)
	}

	pub fn with(&self, x: u32) -> Self {
		let mut result = self.clone();
		result.insert(x);
		result
	}
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Term<T> {
	Value(T),
	Var(u32)
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Triple<T>(Term<T>, Term<T>, Term<T>);

impl<T> Triple<T> {
	pub fn normalize(self, x: u32) -> NormalForm<T> {
		todo!()
	}
}

/// Normal form of a triple relative to a variable.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum NormalForm<T> {
	// One occurrence of `X`, no other variables.
	XVV(T, T),
	VXV(T, T),
	VVX(T, T),
	
	// One occurrence of `X`, one other variable.
	XYV(u32, T),
	XVY(u32, T),
	YXV(u32, T),
	VXY(u32, T),
	YVX(u32, T),
	VYX(u32, T),

	// One occurrence of `X`, two other variables.
	XYZ(u32, u32),
	YXZ(u32, u32),
	YZX(u32, u32),

	// Two occurrences of `X`, no other variables.
	XXV(T),
	XVX(T),
	VXX(T),
	
	// Two occurrences of `X`, one other variable.
	XXY(u32),
	XYX(u32),
	YXX(u32),
}

impl<T: Value> NormalForm<T> {
	pub fn variables(&self) -> NormalFormVariables {
		todo!()
	}
}

/// Normal form values.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum NormalFormVariables {
	Two(u32, u32),
	One(u32)
}

impl NormalFormVariables {
	fn bind<'b>(&self, other: &Self, bijection: Cow<'b, utils::Bijection>) -> Option<Cow<'b, utils::Bijection>> {
		match (*self, *other) {
			(Self::One(x), Self::One(y)) => {
				utils::Bijection::set(bijection, x as usize, y as usize)
			}
			(Self::Two(x, z), Self::Two(y, w)) => {
				utils::Bijection::set(bijection, x as usize, y as usize).and_then(|bijection|
					utils::Bijection::set(bijection, z as usize, w as usize)
				)
			}
			_ => None
		}
	}
}

pub struct NormalizedTriple<T> {
	triple: Triple<T>,
	normal_form: NormalForm<T>
}

pub struct Graph<T>(HashSet<Triple<T>>);

impl<T: Value> Graph<T> {
	/// Groups each triple in the graph by normalized form, relative to the
	/// given `context`.
	pub fn group(&self, x: u32) -> BTreeMap<NormalForm<T>, Vec<NormalFormVariables>> {
		let locally_normalized_triples = self.0.iter().map(|triple| {
			let normal_form = triple.normalize(x);
			NormalizedTriple {
				triple: *triple,
				normal_form
			}
		});
		
		let mut result: BTreeMap<NormalForm<T>, Vec<NormalFormVariables>> = BTreeMap::new();
		for lnt in locally_normalized_triples {
			result.entry(lnt.normal_form).or_default().push(lnt.normal_form.variables())
		}
		result
	}

	/// Find the set possible conditions under which the variables `x` and `y`
	/// are equivalent.
	pub fn equiv_conditions(&self, x: u32, y: u32) -> Vec<utils::Bijection> {
		let occurences: HashMap<u32, Graph<T>> = HashMap::new();

		let x_groups = occurences.get(&x).unwrap().group(x);
		let y_groups = occurences.get(&y).unwrap().group(y);

		if x_groups.len() == y_groups.len() {
			if x_groups.iter().zip(&y_groups).all(|((a, ga), (b, gb))| a == b && ga.len() == gb.len()) {
				let bijections = utils::Matching::new(x_groups.iter(), utils::Bijection::new(), move |(group, x_group), bijection| {
					let y_group = y_groups.get(&group).unwrap();

					utils::Bijections::new(x_group, y_group, bijection, move |a, b, bijection| {
						a.bind(b, Cow::Borrowed(bijection)).map(Cow::into_owned)
					})
				});

				return bijections.collect()
			}
		}

		Vec::new()
	}
}