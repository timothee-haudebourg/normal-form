pub struct Matching<I, Q, F> {
	stack: Vec<State<I, Q>>,
	f: F
}

impl<I: Iterator, Q: Iterator, F> Matching<I, Q, F> {
	pub fn new(mut i: I, context: Q::Item, f: F) -> Self
	where
		F: Fn(I::Item, Q::Item) -> Q
	{
		let mut stack = Vec::new();
		if let Some(next_i) = i.next() {
			stack.push(State {
				user: f(next_i, context),
				i_rest: i
			})
		}

		Self {
			stack,
			f
		}
	}
}

struct State<I, Q> {
	user: Q,
	i_rest: I
}

impl<I: Clone + Iterator, Q: Iterator, F> Iterator for Matching<I, Q, F>
where
	F: Fn(I::Item, Q::Item) -> Q
{
	type Item = Q::Item;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			match self.stack.last_mut() {
				Some(state) => {
					match state.user.next() {
						Some(next_context) => {
							let mut i_rest = state.i_rest.clone();
	
							match i_rest.next() {
								Some(next_i) => {
									self.stack.push(State {
										user: (self.f)(next_i, next_context),
										i_rest
									})
								}
								_ => break Some(next_context)
							}
						}
						None => {
							self.stack.pop();
						}
					}
				}
				None => break None
			}
		}
	}
}