// wengwengweng

use std::fs;
use std::path::PathBuf;
use std::collections::HashSet;

use clipboard::ClipboardProvider;
use clipboard::ClipboardContext;

use super::*;
use ft::*;

pub struct Buffer {

	pub mode: Mode,
	pub cursor: Pos,
	pub path: PathBuf,
	pub content: Vec<String>,
	pub rendered: Vec<Vec<RenderedChunk>>,
	pub undo_stack: Vec<State>,
	pub redo_stack: Vec<State>,
	pub clipboard: ClipboardContext,
	pub modified: bool,
	pub conf: Conf,
	pub log: Vec<String>,
	pub filetype: FileType,

}

#[derive(Debug, Clone)]
pub struct State {
	content: Vec<String>,
	cursor: Pos,
	modified: bool,
}

#[derive(Clone)]
pub struct Conf {
	break_chars: HashSet<char>,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum Span {
	Normal,
	Comment,
	String,
	Keyword,
	Type,
	Number,
	Ident,
}

pub enum Event {
	CursorMove {
		from: Pos,
		to: Pos
	},
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
		break_chars.insert('<');
		break_chars.insert('>');
		break_chars.insert('_');
		break_chars.insert('-');
		break_chars.insert('@');
		break_chars.insert('\'');

		return Self {
			break_chars: break_chars,
		};

	}

}

#[derive(Clone, Copy, Debug)]
pub struct Pos {
	pub line: u32,
	pub col: u32,
}

impl Pos {

	pub fn new(line: u32, col: u32) -> Self {
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
	Search,
}

#[derive(Clone, Debug)]
pub struct Range {
	pub start: Pos,
	pub end: Pos,
}

#[derive(Debug, Clone)]
pub enum RenderedChunk {
	Text {
		span: Span,
		text: String,
	},
	Shift,
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
			conf: Conf::default(),
			undo_stack: Vec::new(),
			redo_stack: Vec::new(),
			modified: false,
			clipboard: ClipboardProvider::new().unwrap(),
			log: Vec::new(),
			filetype: ft_test::rust(),

		};

		buf.read();

		return Ok(buf);

	}

	pub fn log(&mut self, info: &str) {
		self.log.push(info.to_owned());
	}

	pub fn render(&mut self, start: usize, end: usize) {

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
								span: Span::Normal,
								text: text[last..i].to_owned(),
							});

						}

						last = i + 1;
						chunks.push(RenderedChunk::Shift);

					}

				}

				chunks.push(RenderedChunk::Text {
					span: Span::Normal,
					text: text[last..text.len()].to_owned(),
				});

				return chunks;

			})
			.collect();

	}

	pub fn read(&mut self) -> Result<(), Error> {

		if let Ok(content) = fs::read_to_string(&self.path) {

			self.content = content
				.lines()
				.map(|st| String::from(st))
				.collect();

			return Ok(());

		} else {

			return Err(Error::IO);

		}

	}

	pub fn write(&self) {

		if let Ok(_) = fs::write(&self.path, &self.content.join("\n")) {
			// ...
		} else {
			// ...
		}

	}

	pub fn modified(&self) -> bool {
		return self.modified;
	}

	pub fn get_line_at(&self, ln: u32) -> Option<&String> {
		return self.content.get(ln as usize - 1);
	}

	pub fn get_line(&self) -> Option<&String> {
		return self.get_line_at(self.cursor.line);
	}

	pub fn set_line_at(&mut self, ln: u32, content: &str) {
		if let Some(line) = self.content.get_mut(ln as usize - 1) {
			*line = String::from(content);
		}
	}

	pub fn set_line(&mut self, content: &str) {
		self.set_line_at(self.cursor.line, content);
	}

	pub fn next_word_at(&self, pos: Pos) -> Option<Pos> {

		if let Some(line) = self.get_line_at(pos.line) {

			if pos.col < line.len() as u32 {

				for (i, ch) in line[pos.col as usize + 1..].char_indices() {

					if self.conf.break_chars.contains(&ch) {
						return Some(Pos {
							col: pos.col + 1 + i as u32,
							.. pos
						});
					}

				}

				return Some(Pos {
					col: line.len() as u32,
					.. pos
				});

			}

		}

		return None;

	}

	pub fn next_word(&self) -> Option<Pos> {
		return self.next_word_at(self.cursor);
	}

	pub fn prev_word_at(&self, pos: Pos) -> Option<Pos> {

		if let Some(line) = self.get_line_at(pos.line) {

			if pos.col <= line.len() as u32 {

				let end = clamp(pos.col as i32 - 2, 0, line.len() as i32);

				for (i, ch) in line[..end as usize].char_indices().rev() {

					if self.conf.break_chars.contains(&ch) {
						return Some(Pos {
							col: i as u32 + 2,
							.. pos
						});
					}

				}

				return Some(Pos {
					col: 1,
					.. pos
				});

			}

		}

		return None;

	}

	pub fn prev_word(&self) -> Option<Pos> {
		return self.prev_word_at(self.cursor);
	}

	pub fn adjust_cursor(&mut self) {
		self.move_to(self.cursor);
	}

	pub fn del_line_at(&mut self, ln: u32) {

		if ln as usize <= self.content.len() {

			self.content.remove(ln as usize - 1);

			if self.content.is_empty() {
				self.content = vec!["".to_owned()];
			}

			self.adjust_cursor();

		}

	}

	pub fn del_line(&mut self) {

		self.push();
		self.del_line_at(self.cursor.line);

	}

	pub fn insert_line_at(&mut self, ln: u32) {

		self.content.insert(ln as usize - 1, String::new());
		self.adjust_cursor();

	}

	pub fn insert_line(&mut self) {

		self.push();
		self.insert_line_at(self.cursor.line);

	}

	pub fn copy_line_at(&mut self, ln: u32) {

		if let Some(content) = self.get_line_at(ln) {
			self.clipboard.set_contents(content.clone()).unwrap();
		}

	}

	pub fn copy_line(&mut self) {
		self.copy_line_at(self.cursor.line);
	}

	pub fn push(&mut self) {

		self.undo_stack.push(State {
			content: self.content.clone(),
			cursor: self.cursor.clone(),
			modified: self.modified,
		});

	}

	pub fn undo(&mut self) {

		if let Some(state) = self.undo_stack.pop() {

			self.content = state.content;
			self.move_to(state.cursor);

		}

	}

	pub fn cursor_bound(&self, pos: Pos) -> Pos {

		if pos.col < 1 {
			return self.cursor_bound(Pos {
				col: 1,
				.. pos
			});
		}

		if pos.line < 1 {
			return self.cursor_bound(Pos {
				line: 1,
				.. pos
			});
		}

		if let Some(line) = self.get_line_at(pos.line) {

			let len = line.len() as u32 + 1;

			if pos.col > len {

				return self.cursor_bound(Pos {
					col: len,
					.. pos
				});

			}

		}

		let lines = self.content.len() as u32;

		if pos.line > lines && lines > 0 {
			return self.cursor_bound(Pos {
				line: lines,
				.. pos
			});
		}

		return pos;

	}

	pub fn move_to(&mut self, pos: Pos) {
		self.cursor = self.cursor_bound(pos);
	}

	pub fn move_left(&mut self) {

		self.move_to(Pos {
			col: self.cursor.col - 1,
			.. self.cursor
		});

	}

	pub fn move_right(&mut self) {

		self.move_to(Pos {
			col: self.cursor.col + 1,
			.. self.cursor
		});

	}

	pub fn move_up(&mut self) {

		self.move_to(Pos {
			line: self.cursor.line - 1,
			.. self.cursor
		});

	}

	pub fn move_down(&mut self) {

		self.move_to(Pos {
			line: self.cursor.line + 1,
			.. self.cursor
		});

	}

	pub fn move_prev_word(&mut self) {
		if let Some(pos) = self.prev_word() {
			self.move_to(pos);
		}
	}

	pub fn move_next_word(&mut self) {
		if let Some(pos) = self.next_word() {
			self.move_to(pos);
		}
	}

	pub fn start_normal(&mut self) {

		if let Mode::Normal = self.mode {
			return;
		}

		self.mode = Mode::Normal;
		self.move_left();

	}

	pub fn start_insert(&mut self) {

		if let Mode::Insert = self.mode {
			return;
		}

		self.mode = Mode::Insert;
		self.move_right();

	}

	pub fn start_command(&mut self) {

		if let Mode::Command = self.mode {
			return;
		}

		self.mode = Mode::Command;

	}

	pub fn start_search(&mut self) {

		if let Mode::Search = self.mode {
			return;
		}

		self.mode = Mode::Search;

	}

	pub fn line_start_at(&self, mut pos: Pos) -> Pos {

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

			return self.cursor_bound(pos);

		}

		return pos;

	}

	pub fn move_line_start(&mut self) {
		self.cursor = self.line_start_at(self.cursor);
	}

	pub fn move_line_start_insert(&mut self) {

		self.move_line_start();
		self.start_insert();
		self.move_left();

	}

	pub fn line_end_at(&self, mut pos: Pos) -> Pos {

		if let Some(line) = self.get_line_at(pos.line) {
			pos.col = line.len() as u32;
			return self.cursor_bound(pos);
		}

		return pos;

	}

	pub fn move_line_end(&mut self) {
		self.cursor = self.line_end_at(self.cursor);
	}

	pub fn move_line_end_insert(&mut self) {

		self.move_line_end();
		self.start_insert();

	}

	pub fn insert_str_at(&mut self, mut pos: Pos, text: &str) -> Pos {

		if let Some(mut line) = self.get_line_at(pos.line).map(Clone::clone) {

			line.insert_str(pos.col as usize - 1, text);
			self.push();
			self.set_line_at(pos.line, &line);
			pos.col += text.len() as u32;

			return self.cursor_bound(pos);

		}

		return pos;

	}

	pub fn insert_str(&mut self, text: &str) {
		self.cursor = self.insert_str_at(self.cursor, text);
	}

	pub fn insert_at(&mut self, mut pos: Pos, ch: char) -> Pos {

		if let Some(mut line) = self.get_line_at(pos.line).map(Clone::clone) {

			if let Some(end_char) = self.filetype.pairs.get(&ch) {
				line.insert(pos.col as usize - 1, ch);
				line.insert(pos.col as usize, *end_char);
			} else {
				line.insert(pos.col as usize - 1, ch);
			}

			if self.conf.break_chars.contains(&ch) {
				self.push();
			}

			self.set_line_at(pos.line, &line);
			pos.col += 1;

			return self.cursor_bound(pos);

		}

		return pos;

	}

	pub fn insert(&mut self, ch: char) {
		self.cursor = self.insert_at(self.cursor, ch);
	}

	pub fn break_line_at(&mut self, mut pos: Pos) -> Pos {

		if let Some(line) = self.get_line_at(pos.line).map(Clone::clone) {

			let before = String::from(&line[0..pos.col as usize - 1]);
			let mut after = String::from(&line[pos.col as usize - 1..line.len()]);
			let mut indents = 0;

			if let Some(i) = self.get_indents(pos.line) {
				indents += i;
			}

			for _ in 0..indents {
				after.insert(0, '\t');
			}

			self.push();
			self.insert_line_at(pos.line + 1);
			self.set_line_at(pos.line, &before);
			self.set_line_at(pos.line + 1, &after);
			pos.line += 1;

			return self.line_start_at(pos);

		}

		return pos;

	}

	pub fn break_line(&mut self) {
		self.cursor = self.break_line_at(self.cursor);
	}

	pub fn get_indents(&mut self, ln: u32) -> Option<u32> {

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

	pub fn del_at(&mut self, mut pos: Pos) -> Pos {

		if let Some(mut line) = self.get_line_at(pos.line).map(Clone::clone) {

			let before = &line[0..pos.col as usize - 1];

			if before.is_empty() {

				if let Some(mut prev_line) = self.get_line_at(pos.line - 1).map(Clone::clone) {

					let col = prev_line.len() as u32 + 1;

					prev_line.push_str(&line);
					self.del_line_at(pos.line);
					self.set_line_at(pos.line - 1, &prev_line);
					pos.line -= 1;
					pos.col = col;

				}

			} else {

				if let Some(ch) = self.char_at(Pos::new(self.cursor.line, self.cursor.col - 1)) {

					let nch = self.char_at(self.cursor);
					let end_char = self.filetype.pairs.get(&ch).map(Clone::clone);

					if nch.is_some() && nch == end_char {
						line.remove(pos.col as usize - 1);
					}

				}

				line.remove(pos.col as usize - 2);
				self.set_line_at(pos.line, &line);
				pos.col -= 1;

			}

			return pos;

		}

		return pos;

	}

	pub fn del(&mut self) {
		self.cursor = self.del_at(self.cursor);
	}

	pub fn char_at(&self, pos: Pos) -> Option<char> {

		if let Some(content) = self.get_line_at(pos.line) {
			return content.chars().nth(pos.col as usize - 1);
		} else {
			return None;
		}

	}

	pub fn del_word_at(&mut self, pos: Pos) -> Pos {
		if let Some(prev_pos) = self.prev_word_at(pos) {
			return self.del_range(Range {
				start: prev_pos,
				end: Pos {
					col: pos.col - 1,
					.. pos
				},
			});
		}
		return self.cursor;
	}

	pub fn del_word(&mut self) {
		let pos = self.del_word_at(self.cursor);
		self.move_to(pos);
	}

	pub fn del_range(&mut self, r: Range) -> Pos {

		let start = r.start;
		let end = r.end;

		if start.line == end.line {

			if let Some(line) = self.get_line_at(start.line) {

				let mut line = line.clone();
				let start_col = clamp(start.col as usize - 1, 0, line.len());
				let end_col = clamp(end.col as usize, 0, line.len());

				self.push();
				line.replace_range(start_col..end_col, "");
				self.set_line_at(start.line, &line);

				return start;

			}

		}

		return self.cursor;

	}

	pub fn search(&self, target: &str) -> Vec<Pos> {

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

pub fn clamp<N: PartialOrd>(x: N, min: N, max: N) -> N {

	if min > max {
		return clamp(x, max, min);
	}

	if x < min {
		return min;
	} else if x > max {
		return max;
	} else {
		return x;
	}

}

