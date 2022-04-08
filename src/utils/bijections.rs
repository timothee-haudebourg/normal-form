use super::Matching;
use std::collections::HashMap;
use std::borrow::Cow;

#[derive(Default, Clone)]
pub struct Bijection {
	forward: HashMap<usize, usize>,
	backward: HashMap<usize, usize>
}

impl Bijection {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_capacity(capacity: usize) -> Self {
		Self {
			forward: HashMap::with_capacity(capacity),
			backward: HashMap::with_capacity(capacity)
		}
	}

	pub fn get(&self, x: usize) -> Option<usize> {
		self.forward.get(&x).cloned()
	}

	pub fn get_inverse(&self, y: usize) -> Option<usize> {
		self.backward.get(&y).cloned()
	}

	pub fn set(this: Cow<Self>, x: usize, y: usize) -> Option<Cow<Self>> {
		match this.get(x) {
			Some(z) => {
				if z == y {
					Some(this)
				} else {
					None
				}
			}
			None => {
				if this.get_inverse(y).is_some() {
					None
				} else {
					let mut new = this.into_owned();
					new.forward.insert(x, y);
					new.backward.insert(y, x);
					Some(Cow::Owned(new))
				}
			}
		}
	}
}

pub struct Bijections<'t, A, B, F> {
	inner: Matching<std::slice::Iter<'t, A>, Builder<'t, A, B, F>, Box<dyn 't + Fn(&'t A, Bijection) -> Builder<'t, A, B, F>>>
}

impl<'t, A, B, F> Bijections<'t, A, B, F>
where
	F: 't + Clone + Fn(&'t A, &'t B, &Bijection) -> Option<Bijection>
{
	pub fn new(a_list: &'t [A], b_list: &'t [B], start_bijection: Bijection, f: F) -> Self {
		Self {
			inner: Matching::new(a_list.iter(), start_bijection, Box::new(
				move |a, current| {
					Builder {
						a,
						b_list,
						f: f.clone(),
						current
					}
				}
			))
		}
	}
}

impl<'t, A, B, F> Iterator for Bijections<'t, A, B, F>
where
	F: Fn(&'t A, &'t B, &Bijection) -> Option<Bijection>
{
	type Item = Bijection;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next()
	}
}

struct Builder<'t, A, B, F> {
	a: &'t A,
	b_list: &'t [B],
	f: F,
	current: Bijection
}

impl<'t, A, B, F> Iterator for Builder<'t, A, B, F>
where
	F: Fn(&'t A, &'t B, &Bijection) -> Option<Bijection>
{
	type Item = Bijection;

	fn next(&mut self) -> Option<Self::Item> {
		for b in self.b_list.iter() {
			if let Some(new) = (self.f)(self.a, b, &self.current) {
				return Some(new)
			}
		}

		None
	}
}