// wengwengweng

use std::fs;
use std::path::PathBuf;

use dirty::*;
use dirty::math::*;
use input::Key;
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use syntect::highlighting::Style;

use crate::Act;
use crate::Browser;

pub struct Buffer {

	mode: Mode,
	selections: Vec<(Pos, Pos)>,
	path: String,
	content: String,
	rendered: Vec<Vec<(TStyle, String)>>,

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

struct TStyle {
	fg: Color,
	bg: Color,
}

impl TStyle {

	fn from_syntect_style(sty: &Style) -> Self {

		let fg = sty.foreground;
		let bg = sty.background;

		let fg = color!(
			fg.r as f32 / 255.0,
			fg.g as f32 / 255.0,
			fg.b as f32 / 255.0,
			fg.a as f32 / 255.0
		);

		let bg = color!(
			bg.r as f32 / 255.0,
			bg.g as f32 / 255.0,
			bg.b as f32 / 255.0,
			bg.a as f32 / 255.0
		);

		return Self {
			fg: fg,
			bg: bg,
		};

	}

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

		g2d::push();

		for line in &self.rendered {

			g2d::push();

			for (style, text) in line {

				g2d::color(style.fg);
				g2d::text(&text);
				g2d::translate(vec2!(12 * text.len(), 0));

			}

			g2d::pop();
			g2d::translate(vec2!(0, 18));

		}

		g2d::pop();

	}

	fn highlight(&mut self) {

		let ps = SyntaxSet::load_defaults_newlines();
		let ts = ThemeSet::load_defaults();

		let syntax = ps.find_syntax_by_extension("rs").unwrap();
		let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

		self.rendered = self.content
			.lines()
			.map(|l| h.highlight(l, &ps))
			.map(|v| v
				 .iter()
				 .map(|(sty, text)| (TStyle::from_syntect_style(sty), String::from(*text)))
				 .collect())
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

		self.draw_text();

// 		for l in self.content.lines() {
// 			g2d::text(&format!("{}", l));
// 			g2d::translate(vec2!(0, 18));
// 		}

	}

}

