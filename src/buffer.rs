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
use clipboard::ClipboardProvider;
use clipboard::ClipboardContext;

use crate::Act;
use crate::Browser;

pub struct Buffer {

	mode: Mode,
	selections: Vec<(CurPos, CurPos)>,
	cursor: CurPos,
	path: String,
	content: Vec<String>,
	rendered: Vec<Vec<StyledWord>>,
	start_line: u32,
	syntax_set: SyntaxSet,
	theme_set: ThemeSet,
	undo_stack: Vec<State>,
	clipboard: ClipboardContext,
	font: g2d::Font,
	conf: Conf,

}

struct State {
	content: Vec<String>,
	cursor: CurPos,
}

pub struct Conf {
	scroll_off: u32,
	scale: f32,
}

impl Default for Conf {
	fn default() -> Self {
		return Self {
			scroll_off: 3,
			scale: 1.5,
		};
	}
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

#[derive(Debug, Clone)]
struct StyledWord {
	fg: Color,
	bg: Color,
	text: String,
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
			start_line: 1,
			syntax_set: SyntaxSet::load_defaults_newlines(),
			theme_set: ThemeSet::load_defaults(),
			conf: Conf::default(),
			undo_stack: Vec::new(),
			clipboard: ClipboardProvider::new().unwrap(),
			font: g2d::Font::new(
				gfx::Texture::from_bytes(crate::FONT),
				crate::FONT_COLS,
				crate::FONT_ROWS,
				crate::FONT_CHARS,
			),

		};

		buf.read();

		return buf;

	}

	fn highlight(&mut self) {

		let syntax = self.syntax_set.find_syntax_by_extension("rs").unwrap();
		let mut h = HighlightLines::new(syntax, &self.theme_set.themes["base16-ocean.dark"]);

		self.rendered = self.content
			.iter()
			.map(|l| h.highlight(l, &self.syntax_set))
			.map(|v| v
				.iter()
				.map(|(sty, text)| {

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

					return StyledWord {
						fg: fg,
						bg: bg,
						text: String::from(*text),
					}

				})
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
		}

	}

	fn del_line(&mut self, ln: u32) {

		self.push();
		self.content.remove(ln as usize - 1);

	}

	fn push(&mut self) {

		self.undo_stack.push(State {
			content: self.content.clone(),
			cursor: self.cursor.clone(),
		});

	}

	fn pop(&mut self) {

		if let Some(state) = self.undo_stack.pop() {
			self.content = state.content;
			self.cursor = state.cursor;
		}

	}

	fn view_move_down(&mut self) {

		if self.start_line < self.content.len() as u32 {
			if self.cursor.line - self.start_line >= self.conf.scroll_off {
				self.start_line += 1;
			}
		}

	}

	fn view_move_up(&mut self) {

		if self.start_line > 1 {
			if self.cursor.line < self.start_line + self.get_rows() - self.conf.scroll_off {
				self.start_line -= 1;
			}
		}

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

		if self.cursor.line < self.start_line + self.conf.scroll_off {
			self.view_move_up();
		}

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

		if self.cursor.line >= self.start_line + self.get_rows() - self.conf.scroll_off {
			self.view_move_down();
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

	fn get_rows(&self) -> u32 {

		g2d::set_font(&self.font);

		let (w, h) = window::size().into();
		let rows = h as f32 / (g2d::text_height() as f32 * self.conf.scale);

		return rows as u32;

	}

	fn insert(&mut self, ch: char) {

		let mut cur = self.cursor.clone();

		if let Some(line) = self.read_line(cur.line) {

			let mut content = line.clone();

			content.insert(cur.col as usize - 1, ch);
			self.write_line(cur.line, &content);
			cur.col += 1;

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

				if let Some(i) = input::text_input() {

					match i {

						TextInput::Char(ch) => {

							if ch == 'y' {
								if let Some(content) = self.read_line(self.cursor.line) {
									self.clipboard.set_contents(content.clone()).unwrap();
								}
							}

							if ch == 'h' {
								self.move_left();
							}

							if ch == 'l' {
								self.move_right();
							}

							if ch == 'j' {
								self.move_down();
							}

							if ch == 'k' {
								self.move_up();
							}

							if ch == 'u' {
								self.pop();
							}

							if ch == 'd' {
								self.del_line(self.cursor.line);
							}

						},

						TextInput::Backspace => {
							// ...
						},

						TextInput::Return => {
							// ...
						},

						TextInput::Tab => {
							// ...
						},

						TextInput::Up => {
							self.view_move_up();
						},

						TextInput::Down => {
							self.view_move_down();
						},

						TextInput::Left => {
							// ...
						},

						TextInput::Right => {
							// ...
						},

					}

				}

				if input::key_pressed(Key::Return) {
					self.mode = Mode::Insert;
				}

				if input::key_pressed(Key::Tab) {
					self.start_browser();
				}

				if input::key_pressed(Key::Semicolon) {
					self.mode = Mode::Command;
				}

				if input::key_pressed(Key::W) {
					self.write();
				}

				if let Some(scroll) = input::scroll_delta() {

					if input::key_down(Key::LAlt) {

						if scroll.y > 0 {
							for _ in 0..scroll.y.abs() {
								self.view_move_up();
							}
						} else if scroll.y < 0 {
							for _ in 0..scroll.y.abs() {
								self.view_move_down();
							}
						}

					} else {

						if scroll.y > 0 {
							for _ in 0..scroll.y.abs() {
								self.move_up();
							}
						} else if scroll.y < 0 {
							for _ in 0..scroll.y.abs() {
								self.move_down();
							}
						}

					}

				}

			},

			Mode::Insert => {

				if let Some(i) = input::text_input() {

					match i {

						TextInput::Char(ch) => {

							if input::key_down(Key::LAlt) {
								// ..
							} else {
								self.insert(ch);
							}

						},

						TextInput::Backspace => {

							if input::key_down(Key::LAlt) {
								// ..
							} else {
								self.backspace();
							}

						},

						TextInput::Return => {

							if input::key_down(Key::LAlt) {
								// ..
							} else {
								// ..
							}

						},

						TextInput::Tab => {
							self.insert('\t');
						},

						TextInput::Up => {
							// ...
						},

						TextInput::Down => {
							// ...
						},

						TextInput::Left => {
							// ...
						},

						TextInput::Right => {
							// ...
						},

					}

				}

				if input::key_pressed(Key::Escape) {
					self.mode = Mode::Normal;
				}

				if let Some(scroll) = input::scroll_delta() {

					if scroll.y > 0 {
						self.view_move_up();
					} else if scroll.y < 0 {
						self.view_move_down();
					}

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
						TextInput::Tab => {
							// ...
						},
						TextInput::Up => {
							// ...
						},
						TextInput::Down => {
							// ...
						},
						TextInput::Left => {
							// ...
						},
						TextInput::Right => {
							// ...
						},

					}

				}

				if input::key_pressed(Key::Escape) {
					self.mode = Mode::Normal;
				}

			}

		}

		self.highlight();

	}

	fn draw(&self) {

		g2d::scale(vec2!(self.conf.scale));
		g2d::set_font(&self.font);

		let (w, h) = window::size().into();
		let tw = g2d::text_width(" ");
		let th = g2d::text_height();

		g2d::color(color!(0.10, 0.13, 0.17, 1));
		g2d::rect(vec2!(w, h));

		// viewport
		g2d::translate(vec2!(8, (self.start_line - 1) as i32 * -1 * g2d::text_height() as i32));

		// cursor
		g2d::push();
		g2d::color(color!(0.15, 0.18, 0.22, 1));
		g2d::translate(vec2!(0, (self.cursor.line - 1) * th));
		g2d::rect(vec2!(w, th));
		g2d::translate(vec2!((self.cursor.col - 1) * tw, 0));
		g2d::color(color!(0.84));

		match self.mode {
			Mode::Normal => g2d::rect(vec2!(tw, th)),
			Mode::Insert => g2d::rect(vec2!(tw / 4, th)),
			_ => {},
		}

		g2d::pop();

		// content
		g2d::push();

		for line in &self.rendered {

			g2d::push();

			for text in line {

				g2d::color(text.fg);
				g2d::text(&text.text);
				g2d::translate(vec2!(g2d::text_width(&text.text), 0));

			}

			g2d::pop();
			g2d::translate(vec2!(0, g2d::text_height()));

		}

		g2d::pop();

	}

}

