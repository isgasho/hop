// wengwengweng

use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;
use std::collections::HashSet;

use dirty::*;
use dirty::math::*;
use input::Key;
use input::Mouse;
use input::TextInput;
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::parsing::SyntaxReference;
use syntect::highlighting::ThemeSet;
use syntect::highlighting::Style;
use clipboard::ClipboardProvider;
use clipboard::ClipboardContext;

use crate::Act;
use crate::Browser;

pub struct Buffer {

	mode: Mode,
	cursor: Pos,
	path: String,
	content: Vec<String>,
	rendered: Vec<Vec<RenderedChunk>>,
	start_line: u32,
	syntax_set: SyntaxSet,
	syntax: Option<SyntaxReference>,
	theme_set: ThemeSet,
	undo_stack: Vec<State>,
	redo_stack: Vec<State>,
	clipboard: ClipboardContext,
	font: g2d::Font,
	modified: bool,
	conf: Conf,

}

#[derive(Debug, Clone)]
struct State {
	content: Vec<String>,
	cursor: Pos,
	modified: bool,
}

#[derive(Debug, Clone)]
enum RenderedChunk {
	Text {
		fg: Color,
		bg: Color,
		text: String,
	},
	Tab,
}

#[derive(Debug, Clone)]
pub struct Conf {
	scroll_off: u32,
	scale: f32,
	wrapped_chars: HashMap<char, char>,
	expand_tab: bool,
	shift_width: u8,
	line_space: i32,
	break_chars: HashSet<char>,
}

impl Default for Conf {

	fn default() -> Self {

		let mut wrapped_chars = HashMap::new();

		wrapped_chars.insert('(', ')');
		wrapped_chars.insert('\'', '\'');
		wrapped_chars.insert('"', '"');
		wrapped_chars.insert('{', '}');
		wrapped_chars.insert('[', ']');

		let mut break_chars = HashSet::new();

		break_chars.insert(' ');
		break_chars.insert(',');
		break_chars.insert('.');
		break_chars.insert(';');
		break_chars.insert(':');
		break_chars.insert('"');
		break_chars.insert('(');
		break_chars.insert('{');
		break_chars.insert('[');
		break_chars.insert('\'');

		return Self {
			scroll_off: 3,
			scale: 1.5,
			wrapped_chars: wrapped_chars,
			expand_tab: false,
			shift_width: 4,
			line_space: 2,
			break_chars: break_chars,
		};

	}

}

#[derive(Clone, Copy, Debug)]
struct Pos {
	line: u32,
	col: u32,
}

impl Pos {

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
	Select(Vec<Range>),
}

struct Range {
	start: Pos,
	end: Pos,
}

impl RenderedChunk {

	fn from_plain(text: &str) -> Self {

		return RenderedChunk::Text {
			fg: color!(),
			bg: color!(),
			text: String::from(text),
		};

	}

	fn from_syntect(def: &(Style, &str)) -> Self {

		let sty = def.0;
		let text = def.1;
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

		return RenderedChunk::Text {

			fg: fg,
			bg: bg,
			text: String::from(text),

		};

	}
}

pub enum Error {
	IO,
}

impl Buffer {

	pub fn from_file(path: &str) -> Result<Self, Error> {

		let syntax_set = SyntaxSet::load_defaults_newlines();
		let syntax = syntax_set.find_syntax_by_extension("rs").map(Clone::clone);

		let mut buf = Self {

			mode: Mode::Normal,
			path: path.to_owned(),
			content: Vec::new(),
			rendered: Vec::with_capacity(1024),
			cursor: Pos::new(1, 1),
			start_line: 1,
			syntax_set: syntax_set,
			syntax: syntax,
			theme_set: ThemeSet::load_defaults(),
			conf: Conf::default(),
			undo_stack: Vec::new(),
			redo_stack: Vec::new(),
			modified: false,
			clipboard: ClipboardProvider::new().unwrap(),
			font: g2d::Font::new(
				gfx::Texture::from_bytes(crate::FONT),
				crate::FONT_COLS,
				crate::FONT_ROWS,
				crate::FONT_CHARS,
			),

		};

		buf.read();

		return Ok(buf);

	}

	fn highlight_line(&mut self, ln: u32) {

		if let Some(content) = self.get_line(ln).map(Clone::clone) {

			if let Some(s) = self.rendered.get_mut(ln as usize - 1) {

				if let Some(syntax) = &self.syntax {

					let mut h = HighlightLines::new(&syntax, &self.theme_set.themes["base16-ocean.dark"]);

					*s = h.highlight(&content, &self.syntax_set)
						.iter()
						.map(RenderedChunk::from_syntect)
						.collect();

				} else {

					*s = vec![RenderedChunk::from_plain(&content)];

				}

			}

		}

	}

	fn highlight_all(&mut self) {

		if let Some(syntax) = &self.syntax {

			let mut h = HighlightLines::new(&syntax, &self.theme_set.themes["base16-ocean.dark"]);

			self.rendered = self.content
				.iter()
				.map(|l| h.highlight(l, &self.syntax_set))
				.map(|v| v.iter()
					 .map(RenderedChunk::from_syntect)
					 .collect())
				.collect();

		} else {

			self.rendered = self.content
				.iter()
				.map(|l| vec![RenderedChunk::from_plain(l)])
				.collect();

		}

	}

	fn read(&mut self) -> Result<(), Error> {

		if let Ok(content) = fs::read_to_string(&self.path) {

			self.content = content
				.lines()
				.map(|st| String::from(st))
				.collect();

			self.highlight_all();

			return Ok(());

		} else {

			return Err(Error::IO);

		}

	}

	fn write(&self) {

		if let Ok(_) = fs::write(&self.path, &self.content.join("\n")) {
			// ...
		} else {
			// ...
		}

	}

	fn modified(&self) -> bool {
		return self.modified;
	}

	fn get_line(&self, ln: u32) -> Option<&String> {
		return self.content.get(ln as usize - 1);
	}

	fn set_line(&mut self, ln: u32, content: &str) {

		if let Some(line) = self.content.get_mut(ln as usize - 1) {

			*line = String::from(content);
			self.highlight_line(ln);
			self.modified = true;

		}

	}

	fn push_line(&mut self, ln: u32, content: &str) {

		if let Some(line) = self.content.get_mut(ln as usize - 1) {

			line.push_str(content);
			self.highlight_line(ln);
			self.modified = true;

		}

	}

	fn next_word(&self, pos: Pos) -> Option<Pos> {

		if let Some(line) = self.get_line(pos.line) {
			for (i, ch) in line.char_indices().skip(pos.col as usize) {
				if ch == ' ' || ch == ',' || ch == '.' {
					return Some(Pos {
						col: i as u32,
						.. pos
					});
				}
			}
		}

		return None;

	}

	fn prev_word(&self, pos: Pos) -> Option<Pos> {

// 		if let Some(line) = self.get_line(pos.line) {
// 			for (i, ch) in line.char_indices().skip(pos.col as usize) {
// 				if ch == ' ' || ch == ',' || ch == '.' {
// 					return Some(Pos {
// 						col: i as u32,
// 						.. pos
// 					});
// 				}
// 			}
// 		}

		return None;

	}

	fn delete_line(&mut self, ln: u32) {

		self.push();
		self.content.remove(ln as usize - 1);
		self.rendered.remove(ln as usize - 1);
		self.move_up();
		self.modified = true;

	}

	fn insert_line(&mut self, ln: u32) {

		self.content.insert(ln as usize - 1, String::new());
		self.rendered.insert(ln as usize - 1, Vec::new());
		self.modified = true;
		self.move_down();

	}

	fn copy_line(&mut self, ln: u32) {

		if let Some(content) = self.get_line(ln) {
			self.clipboard.set_contents(content.clone()).unwrap();
		}

	}

	fn push(&mut self) {

		self.undo_stack.push(State {
			content: self.content.clone(),
			cursor: self.cursor.clone(),
			modified: self.modified,
		});

	}

	fn undo(&mut self) {

		if let Some(state) = self.undo_stack.pop() {

			self.content = state.content;
			self.move_to(state.cursor);
			self.highlight_all();

		}

	}

	fn scroll_down(&mut self) {

		if self.start_line < self.content.len() as u32 {
			if self.cursor.line - self.start_line >= self.conf.scroll_off {
				self.start_line += 1;
			}
		}

	}

	fn scroll_up(&mut self) {

		if self.start_line > 1 {
			if self.cursor.line < self.start_line + self.get_view_rows() - self.conf.scroll_off {
				self.start_line -= 1;
			}
		}

	}

	fn move_to(&mut self, mut pos: Pos) {

		if pos.col < 1 {
			return self.move_to(Pos {
				col: 1,
				.. pos
			});
		}

		if pos.line < 1 {
			return self.move_to(Pos {
				line: 1,
				.. pos
			});
		}

		if let Some(line) = self.get_line(pos.line) {

			let len = line.len() as u32 + 1;

			if pos.col > len {

				return self.move_to(Pos {
					col: len,
					.. pos
				});

			}

		}

		let lines = self.content.len() as u32;

		if pos.line > lines && lines > 0 {
			return self.move_to(Pos {
				line: lines,
				.. pos
			});
		}

		self.cursor = pos;

		let top = self.cursor.line - self.conf.scroll_off;
		let bottom = self.cursor.line - self.get_view_rows() + self.conf.scroll_off;

		if self.start_line > top {
			if top > 0 {
				self.start_line = top;
			} else {
				self.start_line = 1;
			}
		}

		if self.start_line < bottom && bottom < self.content.len() as u32 {
			self.start_line = bottom;
		}

	}

	fn move_left(&mut self) {

		self.move_to(Pos {
			col: self.cursor.col - 1,
			.. self.cursor
		});

	}

	fn move_right(&mut self) {

		self.move_to(Pos {
			col: self.cursor.col + 1,
			.. self.cursor
		});

	}

	fn move_up(&mut self) {

		self.move_to(Pos {
			line: self.cursor.line - 1,
			.. self.cursor
		});

	}

	fn move_down(&mut self) {

		self.move_to(Pos {
			line: self.cursor.line + 1,
			.. self.cursor
		});

	}

	fn move_prev_word(&mut self) {
		if let Some(pos) = self.prev_word(self.cursor) {
			self.move_to(pos);
		}
	}

	fn move_next_word(&mut self) {
		if let Some(pos) = self.next_word(self.cursor) {
			self.move_to(pos);
		}
	}

	fn start_normal(&mut self) {

		if let Mode::Normal = self.mode {
			return;
		}

		self.mode = Mode::Normal;
		self.move_left();

	}

	fn start_insert(&mut self) {

		if let Mode::Insert = self.mode {
			return;
		}

		self.mode = Mode::Insert;
		self.move_right();

	}

	fn start_command(&mut self) {

		if let Mode::Command = self.mode {
			return;
		}

		self.mode = Mode::Command;

	}

	fn move_line_start(&mut self) {

		let mut pos = self.cursor.clone();

		if let Some(line) = self.get_line(pos.line) {

			let mut index = 0;

			for (i, ch) in line.chars().enumerate() {
				if ch != '\t' && ch != ' ' {
					index = i;
					break;
				} else if i == line.len() - 1 {
					index = i + 1;
				}
			}

			pos.col = index as u32 + 1;

		}

		self.move_to(pos);

	}

	fn move_line_start_insert(&mut self) {

		self.move_line_start();
		self.start_insert();
		self.move_left();

	}

	fn move_line_end(&mut self) {

		let mut pos = self.cursor.clone();

		if let Some(line) = self.get_line(pos.line) {
			pos.col = line.len() as u32;
		}

		self.move_to(pos);

	}

	fn move_line_end_insert(&mut self) {

		self.move_line_end();
		self.start_insert();

	}

	fn start_browser(&self) {

		let path = PathBuf::from(&self.path);

		if let Some(parent) = path.parent() {

			let mut browser = Browser::new(parent.to_path_buf());

			browser.select_item(&path);
			crate::start(browser);

		}

	}

	fn get_view_rows(&self) -> u32 {

		g2d::set_font(&self.font);

		let (w, h) = window::size().into();
		let rows = h as f32 / ((g2d::font_height() as i32 + self.conf.line_space) as f32 * self.conf.scale);

		return rows as u32;

	}

	fn insert(&mut self, ch: char) {

		let mut cur = self.cursor.clone();

		if let Some(line) = self.get_line(cur.line) {

			let mut content = line.clone();

			if let Some(end_char) = self.conf.wrapped_chars.get(&ch) {
				content.insert(cur.col as usize - 1, ch);
				content.insert(cur.col as usize, *end_char);
			} else {
				content.insert(cur.col as usize - 1, ch);
			}

			if self.conf.break_chars.contains(&ch) {
				self.push();
			}

			cur.col += 1;
			self.set_line(cur.line, &content);

		}

		self.move_to(cur);

	}

	fn break_line(&mut self, cur: Pos) {

		self.push();
		self.insert_line(cur.line + 1);

		if let Some(line) = self.get_line(cur.line).map(Clone::clone) {

			let before = String::from(&line[0..cur.col as usize - 1]);
			let mut after = String::from(&line[cur.col as usize - 1..line.len()]);

			if let Some(indents) = self.get_indents(cur.line) {

				for _ in 0..indents {
					after.insert(0, '\t');
				}

			}

			self.set_line(cur.line, &before);
			self.set_line(cur.line + 1, &after);
			self.move_line_start();

		}

	}

	fn get_indents(&mut self, ln: u32) -> Option<u32> {

		if let Some(line) = self.get_line(ln) {

			let mut indents = 0;

			for ch in line.chars() {
				if ch == '\t' {
					indents += 1;
				} else {
					break;
				}
			}

			return Some(indents);

		}

		return None;

	}

	fn delete(&mut self) {

		let mut pos = self.cursor.clone();

		if let Some(line) = self.get_line(pos.line) {

			let before = &line[0..pos.col as usize - 1];

			if before.is_empty() {

				if let Some(prev_line) = self.get_line(pos.line - 1).map(Clone::clone) {

					let mut content = prev_line.clone();

					content.push_str(line);
					self.delete_line(pos.line);
					self.set_line(pos.line - 1, &content);
					pos.line -= 1;
					pos.col = prev_line.len() as u32 + 1;

				}

			} else {

				let mut content = line.clone();

				if let Some(ch) = self.char_at(Pos::new(self.cursor.line, self.cursor.col - 1)) {

					let nch = self.char_at(self.cursor);
					let end_char = self.conf.wrapped_chars.get(&ch).map(Clone::clone);

					if nch.is_some() && nch == end_char {
						content.remove(pos.col as usize - 1);
					}

				}

				content.remove(pos.col as usize - 2);
				self.set_line(pos.line, &content);
				pos.col -= 1;

			}

		}

		self.move_to(pos);

	}

	fn char_at(&self, pos: Pos) -> Option<char> {

		if let Some(content) = self.get_line(pos.line) {
			return content.chars().nth(pos.col as usize - 1);
		} else {
			return None;
		}

	}

	fn delete_word(&mut self) {
		// ...
	}

	fn delete_range(&mut self, r: Range) {
		// ...
	}

	fn search(&self, target: &str) -> Vec<Pos> {

		let mut results = vec![];
		let target_bytes = target.as_bytes();
		let target_len = target.len();

		for (i, line) in self.content.iter().enumerate() {

			for (offset, _) in line.char_indices() {

				let slice = &line[offset..];
				let slice_len = slice.len();
				let slice_bytes = slice.as_bytes();

				if slice_len >= target_len && target_bytes == &slice_bytes[..target_len] {
					results.push(Pos {
						line: i as u32 + 1,
						col: offset as u32 + 1,
					});
				}

			}

		}

		return results;

	}

}

impl Act for Buffer {

	fn update(&mut self) {

		match self.mode {

			Mode::Normal => {

				if let Some(i) = input::text_input() {

					match i {

						TextInput::Char(ch) => {

							match ch {
								'y' => self.copy_line(self.cursor.line),
								'h' => self.move_left(),
								'l' => self.move_right(),
								'j' => self.move_down(),
								'k' => self.move_up(),
								'u' => self.undo(),
								'd' => self.delete_line(self.cursor.line),
								'<' => self.move_line_start_insert(),
								'>' => self.move_line_end_insert(),
								':' => self.start_command(),
								_ => {},
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
							self.scroll_up();
						},

						TextInput::Down => {
							self.scroll_down();
						},

						TextInput::Left => {
							// ...
						},

						TextInput::Right => {
							// ...
						},

					}

				}

				if input::mouse_pressed(Mouse::Left) {

					let mpos: Vec2 = input::mouse_pos().into();

				}

				if input::key_pressed(Key::Return) {
					self.start_insert();
				}

				if input::key_pressed(Key::Tab) {
					self.start_browser();
				}

				if input::key_pressed(Key::W) {
					self.write();
				}

				if let Some(scroll) = input::scroll_delta() {

					if input::key_down(Key::LAlt) {

						if scroll.y > 0 {
							for _ in 0..scroll.y.abs() {
								self.scroll_up();
							}
						} else if scroll.y < 0 {
							for _ in 0..scroll.y.abs() {
								self.scroll_down();
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
								// ...
							} else {
								self.insert(ch);
							}

						},

						TextInput::Backspace => {

							if input::key_down(Key::LAlt) {
								self.delete_word();
							} else {
								self.delete();
							}

						},

						TextInput::Return => {

							if input::key_down(Key::LAlt) {
								// ..
							} else {
								self.break_line(self.cursor);
							}

						},

						TextInput::Tab => {
							self.insert('\t');
						},

						TextInput::Up => {
							self.scroll_up();
						},

						TextInput::Down => {
							self.scroll_down();
						},

						TextInput::Left => {
							self.move_left();
						},

						TextInput::Right => {
							self.move_right();
						},

					}

				}

				if input::key_pressed(Key::Escape) {
					self.start_normal();
				}

				if let Some(scroll) = input::scroll_delta() {

					if scroll.y > 0 {
						self.scroll_up();
					} else if scroll.y < 0 {
						self.scroll_down();
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
					self.start_normal();
				}

			},

			Mode::Select(_) => {
				// ...
			},

		}

	}

	fn draw(&self) {

		g2d::scale(vec2!(self.conf.scale));
		g2d::set_font(&self.font);

		let (w, h) = window::size().into();
		let tw = g2d::font_width();
		let th = g2d::font_height() as i32 + self.conf.line_space;

		g2d::color(color!(0.10, 0.13, 0.17, 1));
		g2d::rect(vec2!(w, h));

		// viewport
		g2d::translate(vec2!(8, (self.start_line as i32 - 1) * -1 * th as i32));

		// cursor
		g2d::push();
		g2d::color(color!(0.15, 0.18, 0.22, 1));
		g2d::translate(vec2!(0, (self.cursor.line - 1) as i32 * th));
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

			for chunk in line {

				match chunk {

					RenderedChunk::Text { fg, bg, text, } => {

						g2d::color(*fg);
						g2d::text(&text);
						g2d::translate(vec2!(g2d::font_width() * text.len() as u32, 0));

					},

					RenderedChunk::Tab => {},

				}


			}

			g2d::pop();
			g2d::translate(vec2!(0, th));

		}

		g2d::pop();

	}

}
