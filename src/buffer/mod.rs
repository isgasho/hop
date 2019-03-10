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
use regex::Regex;
use clipboard::ClipboardProvider;
use clipboard::ClipboardContext;

use crate::Act;
use crate::Browser;

mod ft;
mod theme;
mod ft_test;

use ft::*;
use theme::*;

pub struct Buffer {

	mode: Mode,
	cursor: Pos,
	path: PathBuf,
	content: Vec<String>,
	rendered: Vec<Vec<RenderedChunk>>,
	start_line: u32,
	undo_stack: Vec<State>,
	redo_stack: Vec<State>,
	clipboard: ClipboardContext,
	modified: bool,
	conf: Conf,
	log: Vec<String>,
	filetype: FileType,
	theme: Theme,

}

#[derive(Debug, Clone)]
struct State {
	content: Vec<String>,
	cursor: Pos,
	modified: bool,
}

#[derive(Clone)]
pub struct Conf {
	scroll_off: u32,
	scale: f32,
	line_space: i32,
	break_chars: HashSet<char>,
	font: g2d::Font,
}

impl Default for Conf {

	fn default() -> Self {

		let mut break_chars = HashSet::new();

		break_chars.insert(' ');
		break_chars.insert(',');
		break_chars.insert('.');
		break_chars.insert(';');
		break_chars.insert(':');
		break_chars.insert('"');
		break_chars.insert('(');
		break_chars.insert(')');
		break_chars.insert('{');
		break_chars.insert('}');
		break_chars.insert('[');
		break_chars.insert(']');
		break_chars.insert('_');
		break_chars.insert('-');
		break_chars.insert('\'');

		return Self {
			scroll_off: 3,
			scale: 1.5,
			line_space: 2,
			break_chars: break_chars,
			font: g2d::Font::new(
				gfx::Texture::from_bytes(crate::FONT),
				crate::FONT_COLS,
				crate::FONT_ROWS,
				crate::FONT_CHARS,
			),
		};

	}

}

#[derive(Clone, Copy, Debug)]
pub struct Pos {
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

#[derive(Clone, Debug)]
pub enum Mode {
	Normal,
	Insert,
	Command,
	Select(Vec<Range>),
}

#[derive(Clone, Debug)]
pub struct Range {
	pub start: Pos,
	pub end: Pos,
}

#[derive(Debug, Clone)]
enum RenderedChunk {
	Text {
		style: Style,
		text: String,
	},
	Shift(u32),
}

pub enum Error {
	IO,
}

impl Buffer {

	pub fn from_file(path: PathBuf) -> Result<Self, Error> {

		let mut buf = Self {

			mode: Mode::Normal,
			path: path,
			content: Vec::new(),
			rendered: Vec::with_capacity(1024),
			cursor: Pos::new(1, 1),
			start_line: 1,
			conf: Conf::default(),
			undo_stack: Vec::new(),
			redo_stack: Vec::new(),
			modified: false,
			clipboard: ClipboardProvider::new().unwrap(),
			log: Vec::new(),
			filetype: ft_test::rust(),
			theme: Theme::default(),

		};

		buf.read();

		return Ok(buf);

	}

	fn log(&mut self, info: &str) {
		self.log.push(info.to_owned());
	}

	fn view_range(&self) -> (u32, u32) {

		let start = self.start_line;
		let mut end = start + self.get_view_rows();

		if end > self.content.len() as u32 {
			end = self.content.len() as u32;
		}

		return (start, end);

	}

	fn render(&mut self) {

		let (start, end) = self.view_range();
		let (start, end) = (start as usize, end as usize);

		self.rendered = self.content[start - 1..end]
			.iter()
			.map(|text| {

				let mut chunks = vec![];
				let mut last = 0;

				for (i, ch) in text.char_indices() {

					if ch == '\t' {

						let prev = &text[last..i];

						if !prev.is_empty() {

							chunks.push(RenderedChunk::Text {
								style: self.theme.normal.clone(),
								text: text[last..i].to_owned(),
							});

						}

						last = i + 1;
						chunks.push(RenderedChunk::Shift(self.filetype.shift_width));

					}

				}

				chunks.push(RenderedChunk::Text {
					style: self.theme.normal.clone(),
					text: text[last..text.len()].to_owned(),
				});

				return chunks;

			})
			.collect();

	}

	fn read(&mut self) -> Result<(), Error> {

		if let Ok(content) = fs::read_to_string(&self.path) {

			self.content = content
				.lines()
				.map(|st| String::from(st))
				.collect();

			self.render();

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

	fn get_line_at(&self, ln: u32) -> Option<&String> {
		return self.content.get(ln as usize - 1);
	}

	fn get_line(&self) -> Option<&String> {
		return self.get_line_at(self.cursor.line);
	}

	fn set_line_at(&mut self, ln: u32, content: &str) {
		if let Some(line) = self.content.get_mut(ln as usize - 1) {
			*line = String::from(content);
		}
	}

	fn set_line(&mut self, content: &str) {
		self.set_line_at(self.cursor.line, content);
	}

	fn next_word_at(&self, pos: Pos) -> Option<Pos> {

		if let Some(line) = self.get_line_at(pos.line) {
			if pos.col < line.len() as u32 {
				for (i, ch) in line[pos.col as usize..].char_indices() {
					if self.conf.break_chars.contains(&ch) {
						return Some(Pos {
							col: i as u32,
							.. pos
						});
					}
				}
			}
		}

		return None;

	}

	fn next_word(&self) -> Option<Pos> {
		return self.next_word_at(self.cursor);
	}

	fn prev_word_at(&self, pos: Pos) -> Option<Pos> {

		if let Some(line) = self.get_line_at(pos.line) {
			if pos.col < line.len() as u32 {
				for (i, ch) in line[..pos.col as usize].char_indices().rev() {
					if self.conf.break_chars.contains(&ch) {
						return Some(Pos {
							col: i as u32,
							.. pos
						});
					}
				}
			}
		}

		return None;

	}

	fn prev_word(&self) -> Option<Pos> {
		return self.prev_word_at(self.cursor);
	}

	fn adjust_cursor(&mut self) {
		self.move_to(self.cursor);
	}

	fn del_line_at(&mut self, ln: u32) {

		if ln as usize <= self.content.len() {

			self.content.remove(ln as usize - 1);
			self.adjust_cursor();

		}

	}

	fn del_line(&mut self) {

		self.push();
		self.del_line_at(self.cursor.line);

	}

	fn insert_line_at(&mut self, ln: u32) {

		self.content.insert(ln as usize - 1, String::new());
		self.adjust_cursor();

	}

	fn insert_line(&mut self) {

		self.push();
		self.insert_line_at(self.cursor.line);

	}

	fn copy_line(&mut self, ln: u32) {

		if let Some(content) = self.get_line_at(ln) {
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

	fn move_to(&mut self, pos: Pos) {

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

		if let Some(line) = self.get_line_at(pos.line) {

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

		let top = self.cursor.line as i32 - self.conf.scroll_off as i32;
		let bottom = self.cursor.line as i32 - self.get_view_rows() as i32 + self.conf.scroll_off as i32 + 1;

		if self.start_line as i32 > top {
			if top > 0 {
				self.start_line = top as u32;
			} else {
				self.start_line = 1;
			}
		}

		if (self.start_line as i32) < bottom && bottom < self.content.len() as i32 {
			self.start_line = bottom as u32;
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
		if let Some(pos) = self.prev_word() {
			self.move_to(pos);
		}
	}

	fn move_next_word(&mut self) {
		if let Some(pos) = self.next_word() {
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

		if let Some(line) = self.get_line_at(pos.line) {

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

		if let Some(line) = self.get_line_at(pos.line) {
			pos.col = line.len() as u32;
		}

		self.move_to(pos);

	}

	fn move_line_end_insert(&mut self) {

		self.move_line_end();
		self.start_insert();

	}

	fn start_browser(&self) {

		if let Some(parent) = self.path.parent() {

			let mut browser = Browser::new(parent.to_path_buf());

			browser.select_item(&self.path);
			crate::start(crate::browser::view::View::new(browser));

		}

	}

	fn get_view_rows(&self) -> u32 {

		g2d::set_font(&self.conf.font);

		let (w, h) = window::size().into();
		let rows = h as f32 / ((g2d::font_height() as i32 + self.conf.line_space) as f32 * self.conf.scale);

		return rows as u32;

	}

	fn insert_str_at(&mut self, pos: Pos, text: &str) {

		if let Some(line) = self.get_line_at(pos.line) {

			let mut content = line.clone();

			content.insert_str(pos.col as usize - 1, text);
			self.push();
			self.set_line_at(pos.line, &content);

		}

	}

	fn insert_str(&mut self, text: &str) {
		self.insert_str_at(self.cursor, text);
	}

	fn insert_at(&mut self, pos: Pos, ch: char) {

		if let Some(line) = self.get_line_at(pos.line) {

			let mut content = line.clone();

			if let Some(end_char) = self.filetype.pairs.get(&ch) {
				content.insert(pos.col as usize - 1, ch);
				content.insert(pos.col as usize, *end_char);
			} else {
				content.insert(pos.col as usize - 1, ch);
			}

			if self.conf.break_chars.contains(&ch) {
				self.push();
			}

			self.set_line_at(pos.line, &content);

		}

	}

	fn insert(&mut self, ch: char) {

		self.insert_at(self.cursor, ch);
		self.move_right();

	}

	fn break_line_at(&mut self, cur: Pos) {

		if let Some(line) = self.get_line_at(cur.line).map(Clone::clone) {

			let before = String::from(&line[0..cur.col as usize - 1]);
			let mut after = String::from(&line[cur.col as usize - 1..line.len()]);
			let mut indents = 0;

			if let Some(i) = self.get_indents(cur.line) {
				indents += i;
			}

			for _ in 0..indents {
				after.insert(0, '\t');
			}

			self.push();
			self.insert_line_at(cur.line + 1);
			self.set_line_at(cur.line, &before);
			self.set_line_at(cur.line + 1, &after);
			self.move_down();
			self.move_line_start();

		}

	}

	fn break_line(&mut self) {
		self.break_line_at(self.cursor);
	}

	fn get_indents(&mut self, ln: u32) -> Option<u32> {

		if let Some(line) = self.get_line_at(ln) {

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

	fn del(&mut self) {

		let mut pos = self.cursor.clone();

		if let Some(line) = self.get_line_at(pos.line) {

			let before = &line[0..pos.col as usize - 1];

			if before.is_empty() {

				if let Some(prev_line) = self.get_line_at(pos.line - 1).map(Clone::clone) {

					let mut content = prev_line.clone();

					content.push_str(line);
					self.del_line_at(pos.line);
					self.set_line_at(pos.line - 1, &content);
					pos.line -= 1;
					pos.col = prev_line.len() as u32 + 1;

				}

			} else {

				let mut content = line.clone();

				if let Some(ch) = self.char_at(Pos::new(self.cursor.line, self.cursor.col - 1)) {

					let nch = self.char_at(self.cursor);
					let end_char = self.filetype.pairs.get(&ch).map(Clone::clone);

					if nch.is_some() && nch == end_char {
						content.remove(pos.col as usize - 1);
					}

				}

				content.remove(pos.col as usize - 2);
				self.set_line_at(pos.line, &content);
				pos.col -= 1;

			}

		}

		self.move_to(pos);

	}

	fn char_at(&self, pos: Pos) -> Option<char> {

		if let Some(content) = self.get_line_at(pos.line) {
			return content.chars().nth(pos.col as usize - 1);
		} else {
			return None;
		}

	}

	fn del_word(&mut self) {
		// ...
	}

	fn del_range(&mut self, r: Range) {
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
								'd' => self.del_line(),
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
								self.del_word();
							} else {
								self.del();
							}

						},

						TextInput::Return => {

							if input::key_down(Key::LAlt) {
								// ..
							} else {
								self.break_line();
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

		self.render();

	}

	fn draw(&self) {

		g2d::scale(vec2!(self.conf.scale));
		g2d::set_font(&self.conf.font);

		let (w, h) = window::size().into();
		let (w, h) = (w as f32 / self.conf.scale, h as f32 / self.conf.scale);
		let tw = g2d::font_width();
		let th = g2d::font_height() as i32 + self.conf.line_space;

		g2d::color(self.theme.background);
		g2d::rect(vec2!(w, h));

		// viewport
		g2d::translate(vec2!(12, 0));

		// content
		g2d::push();

		for (ln, line) in self.rendered.iter().enumerate() {

			let real_line = ln as u32 + self.start_line;

			// cursor line
			if real_line == self.cursor.line {

				g2d::push();
				g2d::color(self.theme.cursor_line);
				g2d::translate(vec2!(-12, 0));
				g2d::rect(vec2!(w, th));
				g2d::pop();

			}

			let mut col = 1;
			let mut cursor_drawn = false;

			g2d::push();

			// content
			for chunk in line {

				// cursor
				if real_line == self.cursor.line && !cursor_drawn {

					if col >= self.cursor.col as usize {

						let diff = self.cursor.col as i32 - col as i32;

						g2d::push();
						g2d::translate(vec2!(diff * tw as i32, 0));
						g2d::color(self.theme.cursor);

						match self.mode {
							Mode::Normal => g2d::rect(vec2!(tw, th)),
							Mode::Insert => g2d::rect(vec2!(tw / 4, th)),
							_ => {},
						}

						g2d::pop();
						cursor_drawn = true;

					}

				}

				match chunk {

					RenderedChunk::Text { style, text, } => {

						g2d::color(style.color);
						g2d::text(&text);
						g2d::translate(vec2!(tw * text.len() as u32, 0));
						col += text.len();

					},

					RenderedChunk::Shift(i) => {

						g2d::color(color!(0.24, 0.27, 0.33, 1));
						g2d::text("|");
						g2d::translate(vec2!(tw * i, 0));
						col += 1;

					},

				}

			}

			g2d::pop();
			g2d::translate(vec2!(0, th));

		}

		g2d::pop();

	}

}
