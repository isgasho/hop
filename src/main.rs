// wengwengweng

use std::any::Any;
use std::collections::BTreeMap;

use gctx::ctx;
use dirty::*;

mod buffer;
mod browser;

use buffer::*;
use browser::*;

trait Act: Any {
	fn update(&mut self);
	fn draw(&self);
}

type Id = usize;

ctx!(HOP: HoP);

struct HoP {
	acts: BTreeMap<usize, Box<Act>>,
	current_act: Option<Id>,
	last_id: Id,
}

impl HoP {

	fn new() -> Self {
		return Self {
			acts: BTreeMap::new(),
			current_act: None,
			last_id: 0,
		};
	}

	fn start<A: Act>(&mut self, act: A) -> Id {

		let id = self.last_id;

		self.acts.insert(id, Box::new(act));
		self.last_id += 1;
		self.current_act = Some(id);

		return id;

	}

	fn close(&mut self, id: Id) {

		self.acts.remove(&id);

		if let Some(current_id) = self.current_act {
			if current_id == id {
				// ...
			}
		}

	}

	fn update(&mut self) {
		for (id, act) in &mut self.acts {
			act.update();
		}
	}

	fn draw(&self) {
		for (id, act) in &self.acts {
			act.draw();
		}
	}

}

fn main() {

	app::init();
	window::init("HoP", 960, 640);

	let mut hop = HoP::new();

	hop.start(Browser::new("/Users/t/Things/hop"));

	app::run(|| {

		hop.update();
		hop.draw();

	});

}
