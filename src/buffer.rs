// wengwengweng

use std::fs;
use std::path::PathBuf;

use dirty::*;
use dirty::math::*;
use input::Key;
use input::TextInput;
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use syntect::highlighting::Style;

use crate::Act;
use crate::Browser;

pub struct Buffer {

	mode: Mode,
	selections: Vec<(CurPos, CurPos)>,
	cursor: CurPos,
	path: String,
	content: Vec<String>,
	rendered: Vec<Vec<(WordStyle, String)>>,
	redraw: bool,
	font: g2d::Font,

}

enum Command {
	WriteLine {
		line: u32,
		content: String,
	},
	DelLine {
		line: u32,
	}
}

pub struct Conf {
	// ...
}

struct InputStream {
	stream: Vec<TextInput>,
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
	Command,
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
			cursor: CurPos::new(1, 1),
			redraw: false,
			font: g2d::Font::new(
				gfx::Texture::from_bytes(include_bytes!("res/proggy.png")),
				95,
				1,
				r##" !"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\]^_`abcdefghijklmnopqrstuvwxyz{|}~"##,
			),

		};

		buf.read();

		return buf;

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

	fn read_line(&self, ln: u32) -> Option<&String> {
		return self.content.get(ln as usize - 1);
	}

	fn write_line(&mut self, ln: u32, content: &str) {

		if let Some(line) = self.content.get_mut(ln as usize - 1) {

			*line = String::from(content);
			self.redraw = true;

		}

	}

	fn del_line(&mut self, ln: u32) {

		self.content.remove(ln as usize - 1);
		self.redraw = true;

	}

	fn move_left(&mut self) {

		let mut cur = self.cursor.clone();

		if cur.col > 1 {

			cur.col -= 1;

		} else {

			if let Some(prev_line) = self.read_line(cur.line - 1) {

				cur.line -= 1;

				if prev_line.is_empty() {
					cur.col = 1;
				} else {
					cur.col = prev_line.len() as u32;
				}

			}

		}

		self.cursor = cur;

	}

	fn move_right(&mut self) {

		let mut cur = self.cursor.clone();

		if let Some(line) = self.read_line(cur.line) {

			if cur.col < line.len() as u32 {
				cur.col += 1;
			} else {
				if self.read_line(cur.line + 1).is_some() {
					cur.line += 1;
					cur.col = 1;
				}
			}

		}

		self.cursor = cur;

	}

	fn move_up(&mut self) {

		let mut cur = self.cursor.clone();

		if self.read_line(cur.line).is_some() {

			if let Some(prev_line) = self.read_line(cur.line - 1) {

				cur.line -= 1;

				if prev_line.is_empty() {
					cur.col = 1;
				} else {
					if cur.col as usize > prev_line.len() {
						cur.col = prev_line.len() as u32;
					}
				}

			}

		}

		self.cursor = cur;

	}

	fn move_down(&mut self) {

		let mut cur = self.cursor.clone();

		if self.read_line(cur.line).is_some() {

			if let Some(next_line) = self.read_line(cur.line + 1) {

				cur.line += 1;

				if next_line.is_empty() {
					cur.col = 1;
				} else {
					if cur.col as usize > next_line.len() {
						cur.col = next_line.len() as u32;
					}
				}

			}

		}

		self.cursor = cur;

	}

	fn draw_text(&self) {

		g2d::push();

		for line in &self.rendered {

			g2d::push();

			for (style, text) in line {

				g2d::color(style.fg);
				g2d::text(&text);
				g2d::translate(vec2!(g2d::text_width(text), 0));

			}

			g2d::pop();
			g2d::translate(vec2!(0, g2d::text_height()));

		}

		g2d::pop();

	}

	fn start_browser(&self) {

		let path = PathBuf::from(&self.path);

		if let Some(parent) = path.parent() {

			let mut browser = Browser::new(parent.to_path_buf());

			browser.select_item(&path);
			crate::start(browser);

		}

	}

	fn insert(&mut self, ch: char) {

		let mut cur = self.cursor.clone();

		if let Some(line) = self.read_line(cur.line) {

			let mut content = line.clone();

			content.insert(cur.col as usize - 1, ch);
			self.write_line(cur.line, &content);
			cur.col += 1;
			self.redraw = true;

		}

		self.cursor = cur;

	}

	fn backspace(&mut self) {

		let mut cur = self.cursor.clone();

		if let Some(line) = self.read_line(cur.line) {

			let before = &line[0..cur.col as usize - 1];

			if before.is_empty() {

				if let Some(prev_line) = self.read_line(cur.line - 1) {

					let mut content = prev_line.clone();

					content.push_str(line);
					self.del_line(cur.line);
					self.write_line(cur.line - 1, &content);

				}

			} else {

				let mut content = line.clone();

				content.remove(cur.col as usize - 2);
				self.write_line(cur.line, &content);
				cur.col -= 1;

			}

		}

		self.cursor = cur;

	}

}

impl Act for Buffer {

	fn update(&mut self) {

		match self.mode {

			Mode::Normal => {

				if input::key_pressed(Key::Return) {
					self.mode = Mode::Insert;
				}

				if input::key_pressed(Key::Tab) {
					self.start_browser();
				}

				if input::key_pressed(Key::Semicolon) {
					self.mode = Mode::Command;
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

				if input::key_pressed(Key::W) {
					self.write();
				}

				if let Some(scroll) = input::scroll_delta() {
					if scroll.y > 0 {
						self.move_up();
					} else if scroll.y < 0 {
						self.move_down();
					}
				}

			},

			Mode::Insert => {

				if let Some(i) = input::text_input() {

					match i {

						TextInput::Char(ch) => {
							self.insert(ch);
						},
						TextInput::Backspace => {
							self.backspace();
						},
						TextInput::Return => {
							// ...
						},

					}

				}

				if input::key_pressed(Key::Escape) {
					self.mode = Mode::Normal;
				}

				if let Some(scroll) = input::scroll_delta() {
					// ...
				}

			},

			Mode::Command => {

				if let Some(i) = input::text_input() {

					match i {

						TextInput::Char(ch) => {
							// ...
						},
						TextInput::Backspace => {
							// ...
						},
						TextInput::Return => {
							// ...
						},

					}

				}

				if input::key_pressed(Key::Escape) {
					self.mode = Mode::Normal;
				}

			}

		}

		if self.redraw {

			self.highlight();
			self.redraw = false;

		}

	}

	fn draw(&self) {

		g2d::scale(vec2!(2));
		g2d::set_font(&self.font);

		let (w, h) = window::size().into();

		g2d::color(color!(0.10, 0.13, 0.17, 1));
		g2d::rect(vec2!(w, h));

		g2d::translate(vec2!(16));
		self.draw_text();
		g2d::translate(vec2!(-3, 0));

		let w = g2d::text_width(" ");
		let h = g2d::text_height();

		g2d::push();
		g2d::translate(vec2!(2, 1));
		g2d::translate(vec2!((self.cursor.col - 1) * w, (self.cursor.line - 1) * h));
		g2d::color(color!(1, 1, 1, 0.5));

		match self.mode {
			Mode::Normal => g2d::rect(vec2!(w, h)),
			Mode::Insert => g2d::rect(vec2!(w / 4, h)),
			_ => {},
		}

		g2d::pop();

	}

}

