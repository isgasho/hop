// wengwengweng

use std::path::PathBuf;
use std::any::Any;
use std::collections::BTreeMap;

use gctx::ctx;
use dirty::*;

mod buffer;
mod browser;

use buffer::Buffer;
use browser::Browser;

include!("res/font.rs");

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
	return ctx_mut().start(act);
}

fn close(id: Id) {
	return ctx_mut().close(id);
}

fn update() {
	return ctx_mut().update();
}

fn draw() {
	return ctx_get().draw();
}

fn main() {

	app::init();
	window::init("HoP", 960, 640);

	ctx_init(HoP::new());
	start(browser::view::View::new(Browser::new(PathBuf::from("/Users/t/Things/hop"))));

	app::run(|| {

		update();
		draw();
		g2d::reset();
		g2d::translate(vec2!(12));
		g2d::text(&format!("{}", app::fps()));

	});

}
