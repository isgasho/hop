// wengwengweng

use std::fs;
use std::path::PathBuf;

use dirty::*;
use input::Key;
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style};
use syntect::util::as_24_bit_terminal_escaped;

use crate::Act;
use crate::Browser;

pub struct Buffer {

	mode: Mode,
	selections: Vec<(Pos, Pos)>,
	path: String,
	content: String,
	rendered: Vec<Vec<(Style, String)>>,

}

pub struct Conf {
	// ...
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
			rendered: Vec::new(),
		};

		buf.read();

		return buf;

	}

	fn get_visible_lines() {
		unimplemented!();
	}

	fn draw_text(&self) {

		for line in &self.rendered {
			// ...
		}

	}

	fn highlight(&mut self) {

		let ps = SyntaxSet::load_defaults_newlines();
		let ts = ThemeSet::load_defaults();

		let syntax = ps.find_syntax_by_extension("rs").unwrap();
		let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

		self.rendered = self.content
			.lines()
			.map(|l| h.highlight(l, &ps))
			.map(|v| v.iter().map(|(style, st)| (*style, String::from(*st))).collect())
			.collect();

	}

	fn read(&mut self) {

		if let Ok(content) = fs::read_to_string(&self.path) {
			self.content = content;
			self.highlight();
		} else {
			unimplemented!("dialog error (failed to read file)");
		}

	}

	fn write(&self) {

		if let Ok(_) = fs::write(&self.path, &self.content) {
			// ...
		} else {
			unimplemented!("dialog error (failed to write file)");
		}

	}

	fn start_browser(&self) {

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

		let keys = input::pressed_keys();

		if !keys.is_empty() {
			self.highlight();
		}

		if input::key_pressed(Key::Tab) {
			self.start_browser();
		}

	}

	fn draw(&self) {

		g2d::translate(vec2!(16));

		for l in self.content.lines() {
			g2d::text(&format!("{}", l));
			g2d::translate(vec2!(0, 18));
		}

	}

}

