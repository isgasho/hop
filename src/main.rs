// wengwengweng

use std::path::PathBuf;
use std::any::Any;
use std::collections::BTreeMap;

use gctx::*;
use dirty::*;

pub mod browser;
pub mod buffer;

use suite::browser::Browser;

trait Act: Any {
	fn update(&mut self);
	fn draw(&self);
}

type Id = usize;

ctx!(HOP: HoP);

struct HoP {
	acts: BTreeMap<Id, Box<Act>>,
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
				unimplemented!("go to next available act");
			}
		}

	}

	fn update(&mut self) {
		if let Some(id) = self.current_act {
			if let Some(act) = self.acts.get_mut(&id) {
				act.update();
			}
		}
	}

	fn draw(&self) {
		if let Some(id) = self.current_act {
			if let Some(act) = self.acts.get(&id) {
				act.draw();
			}
		}
	}

}

fn start<A: Act>(act: A) -> Id {
	return ctx_mut!(HOP).start(act);
}

fn close(id: Id) {
	return ctx_mut!(HOP).close(id);
}

fn update() {
	return ctx_mut!(HOP).update();
}

fn draw() {
	return ctx_get!(HOP).draw();
}

fn main() {

	app::init();
	window::init("HoP", 960, 640);

	ctx_init!(HOP, HoP::new());
	start(browser::View::new(Browser::new(PathBuf::from("/Users/t/Things/hop"))));

	app::run(|| {

		update();
		draw();

	});

}
