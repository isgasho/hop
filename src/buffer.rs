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
	selections: Vec<(CurPos, CurPos)>,
	cursors: Vec<CurPos>,
	path: String,
	content: Vec<String>,
	rendered: Vec<Vec<(WordStyle, String)>>,

}

pub struct Conf {
	// ...
}

#[derive(Clone, Copy)]
struct CurPos {
	line: u32,
	col: u32,
}

impl CurPos {
	fn new(line: u32, col: u32) -> Self {
		return Self {
			line: line,
			col: col,
		};
	}
}

enum Mode {
	Normal,
	Insert,
}

struct WordStyle {
	fg: Color,
	bg: Color,
}

impl WordStyle {

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
			content: Vec::new(),
			rendered: Vec::new(),
			cursors: vec![CurPos::new(1, 1)],

		};

		buf.read();

		return buf;

	}

	fn get_visible_lines(&self) {
		unimplemented!();
	}

	fn get_line(&self, n: usize) -> Option<&String> {
		return self.content.get(n);
	}

	fn draw_text(&self) {

		g2d::push();

		for line in &self.rendered {

			g2d::push();

			for (style, text) in line {

				g2d::color(style.fg);
				g2d::text(&text);
				g2d::translate(vec2!(10 * text.len(), 0));

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
			.iter()
			.map(|l| h.highlight(l, &ps))
			.map(|v| v
				 .iter()
				 .map(|(sty, text)| (WordStyle::from_syntect_style(sty), String::from(*text)))
				 .collect())
			.collect();

	}

	fn read(&mut self) {

		if let Ok(content) = fs::read_to_string(&self.path) {

			self.content = content
				.lines()
				.map(|st| String::from(st))
				.collect();

			self.highlight();
		} else {
			unimplemented!("dialog error (failed to read file)");
		}

	}

	fn write(&self) {

		if let Ok(_) = fs::write(&self.path, &self.content.join("\n")) {
			// ...
		} else {
			unimplemented!("dialog error (failed to write file)");
		}

	}

	fn move_left(&mut self) {

		for cur in &mut self.cursors {

			if cur.col > 1 {

				cur.col -= 1;

			} else {

				if let Some(line) = self.content.get(cur.line as usize - 2) {

					cur.line -= 1;

					if line.is_empty() {
						cur.col = 1;
					} else {
						cur.col = line.len() as u32;
					}

				}

			}
		}

	}

	fn move_right(&mut self) {

		for cur in &mut self.cursors {

			if cur.col < self.content[(cur.line - 1) as usize].len() as u32 {

				cur.col += 1;

			} else {

				if let Some(line) = self.content.get(cur.line as usize + 1) {

					cur.line += 1;
					cur.col = 1;

				}

			}

		}

	}

	fn move_up(&mut self) {

		for cur in &mut self.cursors {

			if let Some(line) = self.content.get(cur.line as usize - 1) {

				if let Some(up_line) = self.content.get(cur.line as usize - 2) {

					cur.line -= 1;

					if cur.col as usize > up_line.len() {
						if up_line.is_empty() {
							cur.col = 1;
						} else {
							cur.col = up_line.len() as u32;
						}
					}

				}

			}

		}

	}

	fn move_down(&mut self) {

		for cur in &mut self.cursors {

			if let Some(line) = self.content.get(cur.line as usize - 1) {

				if let Some(down_line) = self.content.get(cur.line as usize) {

					cur.line += 1;

					if cur.col as usize > down_line.len() {
						if down_line.is_empty() {
							cur.col = 1;
						} else {
							cur.col = down_line.len() as u32;
						}
					}

				}

			}

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

		if input::key_pressed(Key::H) {
			self.move_left();
		}

		if input::key_pressed(Key::L) {
			self.move_right();
		}

		if input::key_pressed(Key::J) {
			self.move_down();
		}

		if input::key_pressed(Key::K) {
			self.move_up();
		}

		if let Some(scroll) = input::scroll_delta() {
			// ...
		}

	}

	fn draw(&self) {

		g2d::translate(vec2!(16));
		self.draw_text();
		g2d::translate(vec2!(-3, 0));

		for cur in &self.cursors {

			let w = 10;
			let h = 18;

			g2d::translate(vec2!((cur.col - 1) * w, (cur.line - 1) * h));
			g2d::color(color!(1, 1, 1, 0.2));
			g2d::rect(vec2!(w, h));

		}

// 		for l in self.content.lines() {
// 			g2d::text(&format!("{}", l));
// 			g2d::translate(vec2!(0, 18));
// 		}

	}

}

