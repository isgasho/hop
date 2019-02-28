// wengwengweng

use std::fs;
use std::path::PathBuf;

use dirty::*;
use input::Key;

use crate::Act;
use crate::Browser;

pub struct Buffer {

	mode: Mode,
	selections: Vec<(Pos, Pos)>,
	path: String,
	content: String,

}

struct Pos {
	line: u32,
	col: u32,
}

enum Mode {
	Normal,
	Insert,
}

impl Buffer {

	pub fn new(path: &str) -> Self {

		let mut buf = Self {
			mode: Mode::Normal,
			selections: Vec::new(),
			path: path.to_owned(),
			content: String::new(),
		};

		buf.read();

		return buf;

	}

	pub fn read(&mut self) {

		if let Ok(content) = fs::read_to_string(&self.path) {
			self.content = content;
		} else {
			unimplemented!("dialog error (failed to write file)");
		}

	}

	pub fn write(&self) {

		if let Ok(_) = fs::write(&self.path, &self.content) {
			// ...
		} else {
			unimplemented!("dialog error (failed to read file)");
		}

	}

	pub fn start_browser(&self) {

		let path = PathBuf::from(&self.path);

		if let Some(parent) = path.parent() {

			let mut browser = Browser::new(parent.to_path_buf());

			browser.select_item(&path);
			crate::start(browser);

		}

	}

}

impl Act for Buffer {

	fn update(&mut self) {

		if input::key_pressed(Key::Tab) {
			self.start_browser();
		}

	}

	fn draw(&self) {

		g2d::translate(vec2!(16));
		g2d::text(&format!("{}", self.content));

	}

}

