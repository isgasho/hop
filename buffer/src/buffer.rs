// wengwengweng

use std::fs;
use std::path::PathBuf;
use std::collections::HashSet;

use regex::Regex;
use clipboard::ClipboardProvider;
use clipboard::ClipboardContext;

use super::*;

pub struct Buffer {

	pub mode: Mode,
	pub cursor: Pos,
	pub child_cursors: Vec<Pos>,
	pub path: PathBuf,
	pub content: Vec<String>,
	pub rendered: Vec<Vec<SpannedText>>,
	pub undo_stack: Vec<State>,
	pub redo_stack: Vec<State>,
	pub clipboard: ClipboardContext,
	pub modified: bool,
	pub conf: Conf,
	pub log: Vec<String>,
	pub filetype: FileType,
	invalid_chars: HashSet<char>,

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
	content: Vec<String>,
	cursor: Pos,
	modified: bool,
}

#[derive(Clone)]
pub struct Conf {
	break_chars: HashSet<char>,
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
		break_chars.insert('\t');

		return Self {
			break_chars: break_chars,
		};

	}

}

pub type Line = u32;
pub type Col = u32;
pub type IndentLevel = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Pos {
	pub line: Line,
	pub col: Col,
}

impl Pos {

	pub fn new(line: Line, col: Col) -> Self {
		return Self {
			line: line,
			col: col,
		};
	}

}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
	Normal,
	Insert,
	Command,
	Select(Vec<Range>),
	Search {
		text: String,
	},
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Range {
	pub start: Pos,
	pub end: Pos,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpannedText {
	pub span: Span,
	pub text: String,
}

impl SpannedText {

	pub fn from_plain(text: &str) -> Vec<Self> {
		return vec![Self {
			span: Span::Normal,
			text: String::from(text),
		}];
	}

}

pub enum Error {
	IO,
}

impl Buffer {

	pub fn from_file(path: PathBuf) -> Result<Self, Error> {

		let mut registry = FTRegistry::new();
		let fname = format!("{}", path.display());

		let mut invalid_chars = HashSet::new();

		invalid_chars.insert('\u{7f}');
		invalid_chars.insert('\r');
		invalid_chars.insert('\n');
		invalid_chars.insert('\u{1b}');
		invalid_chars.insert('\u{8}');

		let mut buf = Self {

			mode: Mode::Normal,
			path: path,
			content: Vec::new(),
			rendered: Vec::with_capacity(1024),
			cursor: Pos::new(1, 1),
			child_cursors: Vec::new(),
			conf: Conf::default(),
			undo_stack: Vec::new(),
			redo_stack: Vec::new(),
			modified: false,
			clipboard: ClipboardProvider::new().unwrap(),
			log: Vec::new(),
			filetype: ft_test::rust(),
			invalid_chars: invalid_chars,

		};

		if buf.read().is_ok() {
			return Ok(buf);
		} else {
			return Err(Error::IO);
		}

	}

	pub fn log(&mut self, info: &str) {
		self.log.push(info.to_owned());
	}

	pub fn render(&mut self, start: usize, end: usize) {

		let start = clamp(start, 1, self.content.len());
		let end = clamp(end, 1, self.content.len());

		self.rendered = self.content[start - 1..end]
			.iter()
			.map(|text| {
				return self.filetype.syntax.parse(text);
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

	/// get current state for undo/redo
	pub fn get_state(&self) -> State {
		return State {
			content: self.content.clone(),
			cursor: self.cursor.clone(),
			modified: self.modified,
		};
	}

	/// set current state
	pub fn set_state(&mut self, state: State) {

		self.content = state.content;
		self.modified = state.modified;
		self.move_to(state.cursor);

	}

	/// push current state to undo stack
	pub fn push_undo(&mut self) {

		let state = self.get_state();

		if self.undo_stack.last() == Some(&state) {
			return;
		}

		self.undo_stack.push(state);

	}

	/// push current state to redo stack
	pub fn push_redo(&mut self) {
		self.redo_stack.push(self.get_state());
	}

	/// undo
	pub fn undo(&mut self) {

		if let Some(state) = self.undo_stack.pop() {
			self.push_redo();
			self.set_state(state);
		}

	}

	/// redo
	pub fn redo(&mut self) {

		if let Some(state) = self.redo_stack.pop() {
			self.push_undo();
			self.set_state(state);
		}

	}

	/// start normal mode
	pub fn start_normal(&mut self) {

		if let Mode::Normal = self.mode {
			return;
		}

		self.mode = Mode::Normal;
		self.move_left();

	}

	/// start insert mode
	pub fn start_insert(&mut self) {

		if let Mode::Insert = self.mode {
			return;
		}

		self.mode = Mode::Insert;
		self.move_right();

	}

	/// start command mode
	pub fn start_command(&mut self) {

		if let Mode::Command = self.mode {
			return;
		}

		self.mode = Mode::Command;

	}

	/// start search mode
	pub fn start_search(&mut self) {

		if let Mode::Search { .. } = self.mode {
			return;
		}

		self.mode = Mode::Search {
			text: String::new(),
		};

	}

	/// get if current buffer is modified
	pub fn modified(&self) -> bool {
		return self.modified;
	}

	/// get content of a line
	pub fn get_line_at(&self, ln: Line) -> Option<&String> {
		return self.content.get(ln as usize - 1);
	}

	/// get content of current line
	pub fn get_line(&self) -> Option<&String> {
		return self.get_line_at(self.cursor.line);
	}

	/// set content of a line
	pub fn set_line_at(&mut self, ln: Line, content: &str) {

		if self.content.get(ln as usize - 1).is_some() {

			if !self.modified {
				self.push_undo();
				self.redo_stack.clear();
				self.modified = true;
			}

			self.content.get_mut(ln as usize - 1).map(|s| *s = String::from(content));

		}

	}

	/// set content of current line
	pub fn set_line(&mut self, content: &str) {
		self.set_line_at(self.cursor.line, content);
	}

	/// delete secified line
	pub fn del_line_at(&mut self, ln: Line) -> Line {

		if ln as usize <= self.content.len() {

			self.push_undo();

			if !self.modified {
				self.redo_stack.clear();
				self.modified = true;
			}

			self.content.remove(ln as usize - 1);

			if self.content.is_empty() {
				self.content = vec![String::from("")];
			}

		}

		return clamp(ln, 1, self.content.len() as Line);

	}

	/// delete current line
	pub fn del_line(&mut self) {
		self.cursor.line = self.del_line_at(self.cursor.line);
	}

	/// insert a line at secified position
	pub fn insert_line_at(&mut self, ln: Line) -> Line {

		self.push_undo();

		if !self.modified {
			self.redo_stack.clear();
			self.modified = true;
		}

		self.content.insert(ln as usize - 1, String::new());

		return clamp(ln + 1, 1, self.content.len() as Line);

	}

	/// insert a line at current cursor
	pub fn insert_line(&mut self) {
		self.cursor.line = self.insert_line_at(self.cursor.line);
	}

	// todo
	/// get next word position at specified position
	pub fn next_word_at(&self, pos: Pos) -> Option<Pos> {

		let line = self.get_line_at(pos.line)?;

		if pos.col < line.len() as Col {

			for (i, ch) in line[pos.col as usize + 1..].char_indices() {

				if self.conf.break_chars.contains(&ch) {
					return Some(Pos {
						col: pos.col + 1 + i as Col,
						.. pos
					});
				}

			}

			return Some(Pos {
				col: line.len() as Col,
				.. pos
			});

		}

		return None;

	}

	/// get next word position at current cursor
	pub fn next_word(&self) -> Option<Pos> {
		return self.next_word_at(self.cursor);
	}

	/// get previous word position at specified position
	pub fn prev_word_at(&self, pos: Pos) -> Option<Pos> {

		let line = self.get_line_at(pos.line)?;

		if pos.col <= line.len() as Col + 1 {

			let end = clamp(pos.col as i32 - 2, 0, line.len() as i32);

			for (i, ch) in line[..end as usize].char_indices().rev() {

				if self.conf.break_chars.contains(&ch) {
					return Some(Pos {
						col: i as Col + 2,
						.. pos
					});
				}

			}

			return Some(Pos {
				col: 1,
				.. pos
			});

		}

		return None;

	}

	/// get previous word position at current cursor
	pub fn prev_word(&self) -> Option<Pos> {
		return self.prev_word_at(self.cursor);
	}

	/// copy the whole specified line
	pub fn copy_line_at(&mut self, ln: Line) {

		if let Some(content) = self.get_line_at(ln).map(Clone::clone) {
			if let Ok(_) = self.clipboard.set_contents(content) {
				// ...
			} else {
				// ...
			}
		}

	}

	/// copy current line
	pub fn copy_line(&mut self) {
		self.copy_line_at(self.cursor.line);
	}

	/// paste at specified pos
	pub fn paste_at(&mut self, pos: Pos) -> Pos {

		if let Ok(content) = self.clipboard.get_contents() {
			return self.insert_str_at(pos, &content);
		}

		return pos;

	}

	/// paste at current cursor
	pub fn paste(&mut self) {
		self.cursor = self.paste_at(self.cursor);
	}

	/// returns the bound checked position of a cursor position
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

			let mut len = line.len() as Col;

			if len == 0 || self.mode == Mode::Insert {
				len = len + 1;
			}

			if pos.col > len {

				return self.cursor_bound(Pos {
					col: len,
					.. pos
				});

			}

		}

		let lines = self.content.len() as Line;

		if pos.line > lines && lines > 0 {
			return self.cursor_bound(Pos {
				line: lines,
				.. pos
			});
		}

		return pos;

	}

	/// move to a position with bound checking
	pub fn move_to(&mut self, pos: Pos) {
		self.cursor = self.cursor_bound(pos);
	}

	/// adjust current cursor
	pub fn adjust_cursor(&mut self) {
		self.cursor = self.cursor_bound(self.cursor);
	}

	/// move current cursor left
	pub fn move_left(&mut self) {
		self.move_to(Pos {
			col: self.cursor.col - 1,
			.. self.cursor
		});
	}

	/// move current cursor right
	pub fn move_right(&mut self) {
		self.move_to(Pos {
			col: self.cursor.col + 1,
			.. self.cursor
		});
	}

	/// move current cursor up
	pub fn move_up(&mut self) {
		self.move_to(Pos {
			line: self.cursor.line - 1,
			.. self.cursor
		});
	}

	/// move current cursor down
	pub fn move_down(&mut self) {
		self.move_to(Pos {
			line: self.cursor.line + 1,
			.. self.cursor
		});
	}

	/// move to the previous word
	pub fn move_prev_word(&mut self) {
		if let Some(pos) = self.prev_word() {
			self.move_to(pos);
		}
	}

	/// move to the next word
	pub fn move_next_word(&mut self) {
		if let Some(pos) = self.next_word() {
			self.move_to(pos);
		}
	}

	/// get the position that a line starts, ignoring tabs and spaces
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

			pos.col = index as Col + 1;

			return self.cursor_bound(pos);

		}

		return pos;

	}

	/// line_start_at() with cursor movement
	pub fn move_line_start(&mut self) {
		self.cursor = self.line_start_at(self.cursor);
	}

	/// call move_line_start() and enter insert mode
	pub fn move_line_start_insert(&mut self) {

		self.move_line_start();
		self.start_insert();
		self.move_left();

	}

	/// get the position that a line ends
	pub fn line_end_at(&self, mut pos: Pos) -> Pos {

		if let Some(line) = self.get_line_at(pos.line) {
			pos.col = line.len() as Col;
			return self.cursor_bound(pos);
		}

		return pos;

	}

	/// line_end_at() with cursor movement
	pub fn move_line_end(&mut self) {
		self.cursor = self.line_end_at(self.cursor);
	}

	/// call move_line_end() and enter insert mode
	pub fn move_line_end_insert(&mut self) {

		self.move_line_end();
		self.start_insert();

	}

	/// insert a str at a cursor position
	pub fn insert_str_at(&mut self, mut pos: Pos, text: &str) -> Pos {

		if let Some(mut line) = self.get_line_at(pos.line).map(Clone::clone) {

			line.insert_str(pos.col as usize - 1, text);
			self.push_undo();
			self.set_line_at(pos.line, &line);
			pos.col += text.len() as Col;

			return self.cursor_bound(pos);

		}

		return pos;

	}

	/// insert_str_at() with cursor movement
	pub fn insert_str(&mut self, text: &str) {
		self.cursor = self.insert_str_at(self.cursor, text);
	}

	/// insert a char at a cursor position
	pub fn insert_at(&mut self, mut pos: Pos, ch: char) -> Pos {

		if !ch.is_ascii() {
			return pos;
		}

		if self.invalid_chars.contains(&ch) {
			return pos;
		}

		if let Some(mut line) = self.get_line_at(pos.line).map(Clone::clone) {

			if let Some(end_char) = self.filetype.pairs.get(&ch) {
				line.insert(pos.col as usize - 1, ch);
				line.insert(pos.col as usize, *end_char);
			} else {
				line.insert(pos.col as usize - 1, ch);
			}

			if self.conf.break_chars.contains(&ch) {
				self.push_undo();
			}

			self.set_line_at(pos.line, &line);
			pos.col += 1;

			return self.cursor_bound(pos);

		}

		return pos;

	}

	/// insert_at() with cursor movement
	pub fn insert(&mut self, ch: char) {
		self.cursor = self.insert_at(self.cursor, ch);
	}

	// todo
	/// break and insert new line, calculating indent
	pub fn break_line_at(&mut self, mut pos: Pos) -> Pos {

		if let Some(line) = self.get_line_at(pos.line).map(Clone::clone) {

			let before = String::from(&line[0..pos.col as usize - 1]);
			let after = String::from(&line[pos.col as usize - 1..line.len()]);
			let indents = self.get_expected_indent_at(pos.line + 1).unwrap_or(0);

			self.push_undo();
			self.insert_line_at(pos.line + 1);
			self.set_line_at(pos.line, &before);
			self.set_line_at(pos.line + 1, &after);
			self.set_indent_at(pos.line + 1, indents);
			pos.line += 1;
			pos.col = indents + 1;

			return self.cursor_bound(pos);

		}

		return pos;

	}

	/// break_line_at() with cursor movement
	pub fn break_line(&mut self) {
		self.cursor = self.break_line_at(self.cursor);
	}

	// todo: better matching
	/// check if a line is commented
	pub fn is_commented_at(&self, ln: Line) -> bool {

		if let Some(comment) = self.filetype.comment.clone() {
			if let Some(line) = self.get_line_at(ln) {
				if let Some(front) = line.get(0..comment.len() + 1) {
					return front == &format!("{} ", comment);
				}
			}
		}

		return false;

	}

	/// check if current line is commented
	pub fn is_commented(&self) -> bool {
		return self.is_commented_at(self.cursor.line);
	}

	// todo: better matching
	/// comment given line
	pub fn comment_at(&mut self, ln: Line) {
		if !self.is_commented_at(ln) {
			if let Some(comment) = self.filetype.comment.clone() {
				self.insert_str_at(Pos::new(ln, 1), &format!("{} ", comment));
			}
		}
	}

	/// comment current line
	pub fn comment(&mut self) {
		self.comment_at(self.cursor.line);
		self.cursor = self.cursor_bound(self.cursor);
	}

	// todo: better matching
	/// uncomment given line
	pub fn uncomment_at(&mut self, ln: Line) {
		if self.is_commented_at(ln) {
			if let Some(comment) = self.filetype.comment.clone() {
				if let Some(mut line) = self.get_line_at(ln).map(Clone::clone) {
					line.replace_range(0..comment.len() + 1, "");
					self.set_line_at(ln, &line);
				}
			}
		}
		self.cursor = self.cursor_bound(self.cursor);
	}

	/// uncomment current line
	pub fn uncomment(&mut self) {
		self.uncomment_at(self.cursor.line);
	}

	/// toggle comment at given line
	pub fn toggle_comment_at(&mut self, ln: Line) {
		if self.is_commented_at(ln) {
			self.uncomment_at(ln);
		} else {
			self.comment_at(ln);
		}
	}

	/// toggle comment at current line
	pub fn toggle_comment(&mut self) {
		self.toggle_comment_at(self.cursor.line);
	}

	/// set a line's indent level
	pub fn set_indent_at(&mut self, ln: Line, level: IndentLevel) {

		if let Some(mut line) = self.get_line_at(ln).map(Clone::clone) {

			if let Some(indent) = self.get_indent_at(ln) {

				line.replace_range(0..indent as usize, "");

				for _ in 0..level {
					line.insert(0, '\t');
				}

				self.set_line_at(ln, &line);

			}

		}

	}

	/// indent a line forward
	pub fn indent_forward_at(&mut self, ln: Line) {
		if let Some(mut line) = self.get_line_at(ln).map(Clone::clone) {
			line.insert(0, '\t');
			self.set_line_at(ln, &line);
		}
	}

	/// indent current line forward
	pub fn indent_forward(&mut self) {
		self.indent_forward_at(self.cursor.line);
	}

	/// indent a line backwards
	pub fn indent_backward_at(&mut self, ln: Line) {
		if let Some(mut line) = self.get_line_at(ln).map(Clone::clone) {
			if line.chars().next() == Some('\t') {
				line.replace_range(0..1, "");
				self.set_line_at(ln, &line);
			}
		}
	}

	/// indent current line backward
	pub fn indent_backward(&mut self) {
		self.indent_backward_at(self.cursor.line);
	}

	/// get previous non empty line
	pub fn get_prev_line(&self, ln: Line) -> Option<Line> {

		if ln as usize <= self.content.len() {
			// ...
		}

		return None;

	}

	// todo: ignore comments
	/// get indent level of a line
	pub fn get_indent_at(&self, ln: Line) -> Option<IndentLevel> {

		let line = self.get_line_at(ln)?;
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

	/// get expected indent level of a line
	pub fn get_expected_indent_at(&mut self, ln: Line) -> Option<IndentLevel> {

		if !self.filetype.auto_indent {
			return Some(0);
		}

		let prev_line = self.get_line_at(ln - 1)?;
		let mut indent = self.get_indent_at(ln - 1)?;

		if let Some(forward_pat) = &self.filetype.indent_forward {
			if forward_pat.is_match(prev_line) {
				indent += 1;
			}
		}

		return Some(indent);

	}

	/// apply expected indent level of a line
	pub fn apply_expected_indent_at(&mut self, ln: Line) {
		if let Some(indent) = self.get_expected_indent_at(ln) {
			self.set_indent_at(ln, indent);
		}
	}

	/// delete char at specified position
	pub fn del_at(&mut self, mut pos: Pos) -> Pos {

		if let Some(mut line) = self.get_line_at(pos.line).map(Clone::clone) {

			let before = &line[0..pos.col as usize - 1];

			if before.is_empty() {

				if let Some(mut prev_line) = self.get_line_at(pos.line - 1).map(Clone::clone) {

					let col = prev_line.len() as Col + 1;

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

	/// delete char at current cursor
	pub fn del(&mut self) {
		self.cursor = self.del_at(self.cursor);
	}

	/// get char at position
	pub fn char_at(&self, pos: Pos) -> Option<char> {
		return self.get_line_at(pos.line)?.chars().nth(pos.col as usize - 1);
	}

	/// delete the word at specified position
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
		return pos;
	}

	/// delete the word before the cursor
	pub fn del_word(&mut self) {
		let pos = self.del_word_at(self.cursor);
		self.move_to(pos);
	}

	/// delete a range of text
	pub fn del_range(&mut self, r: Range) -> Pos {

		let start = r.start;
		let end = r.end;

		if start.line == end.line {

			if let Some(line) = self.get_line_at(start.line) {

				let mut line = line.clone();
				let start_col = clamp(start.col as usize - 1, 0, line.len());
				let end_col = clamp(end.col as usize, 0, line.len());

				self.push_undo();
				line.replace_range(start_col..end_col, "");
				self.set_line_at(start.line, &line);

				return start;

			}

		}

		return self.cursor;

	}

	/// search for the previous appearence of the given text in given line
	pub fn search_prev_inline_at(&self, pos: Pos, target: &str) -> Option<Pos> {

		let line = self.get_line_at(pos.line)?;
		let slice = line.get(0..pos.col as usize)?;
		let index = slice.rfind(target)?;

		return Some(Pos {
			col: index as Col + 1,
			.. pos
		});

	}

	/// search for the next appearence of the given text in given line
	pub fn search_next_inline_at(&self, pos: Pos, target: &str) -> Option<Pos> {

		let line = self.get_line_at(pos.line)?;
		let slice = line.get(pos.col as usize..line.len())?;
		let index = slice.find(target)?;

		return Some(Pos {
			col: index as Col + 1 + pos.col,
			.. pos
		});

	}

	/// search for the next appearence of the given text in current line
	pub fn search_next_inline(&self, target: &str) -> Option<Pos> {
		return self.search_next_inline_at(self.cursor, target);
	}

	/// search for the prev appearence of the given text
	pub fn search_prev_at(&self, pos: Pos, target: &str) -> Option<Pos> {

		if let Some(app) = self.search_prev_inline_at(pos, target) {

			return Some(app);

		} else {

			for i in (0..pos.line as usize - 1).rev() {

				let result = self.search_prev_inline_at(Pos {
					line: i as Line + 1,
					col: self.content[i].len() as Col,
				}, target);

				if result.is_some() {
					return result;
				}

			}

		}

		return None;

	}

	/// search for the prev appearence of the given text in current line
	pub fn search_prev_inline(&self, target: &str) -> Option<Pos> {
		return self.search_prev_inline_at(self.cursor, target);
	}

	/// search for the next appearence of the given text
	pub fn search_next_at(&self, pos: Pos, target: &str) -> Option<Pos> {

		if let Some(app) = self.search_next_inline_at(pos, target) {

			return Some(app);

		} else {

			for i in pos.line as usize..self.content.len() {

				let result = self.search_next_inline_at(Pos {
					line: i as Line + 1,
					col: 1,
				}, target);

				if result.is_some() {
					return result;
				}

			}

		}

		return None;

	}

	/// search for the prev appearence of the given text at current cursor
	pub fn search_prev(&self, target: &str) -> Option<Pos> {
		return self.search_prev_at(self.cursor, target);
	}

	/// search for the next appearence of the given text at current cursor
	pub fn search_next(&self, target: &str) -> Option<Pos> {
		return self.search_next_at(self.cursor, target);
	}

	/// move to the prev search result
	pub fn move_to_prev_inline(&mut self, target: &str) {
		if let Some(pos) = self.search_prev_inline(target) {
			self.move_to(pos);
		}
	}

	/// move to the next search result
	pub fn move_to_next_inline(&mut self, target: &str) {
		if let Some(pos) = self.search_next_inline(target) {
			self.move_to(pos);
		}
	}

	/// move to the prev search result
	pub fn move_to_prev(&mut self, target: &str) {
		if let Some(pos) = self.search_prev(target) {
			self.move_to(pos);
		}
	}

	/// move to the next search result
	pub fn move_to_next(&mut self, target: &str) {
		if let Some(pos) = self.search_next(target) {
			self.move_to(pos);
		}
	}

	/// add a cursor
	pub fn add_cursor(&mut self, pos: Pos) {
		self.child_cursors.push(pos);
	}

	/// reset states
	pub fn reset(&mut self) {

		self.child_cursors.clear();
		self.log.clear();
		self.mode = Mode::Normal;

	}

	pub fn get_shifted_pos(&self, pos: Pos, width: u32) -> u32 {

		let mut shift = 1;

		if let Some(line) = self.get_line_at(pos.line) {

			for (i, ch) in line.char_indices() {

				if i as u32 == pos.col - 1 {
					return shift;
				}

				if ch == '\t' {
					shift += width - (shift - 1) as u32 % width;
				} else {
					shift += 1;
				}

			}

		} else {
			return 1;
		}

		return shift;

	}

	pub fn get_unshifted_col(&self, pos: u32, ln: u32, width: u32) -> Col {

		let mut shift = 1;
		let mut col = 1;

		if let Some(line) = self.get_line_at(ln) {

			for ch in line.chars() {

				if pos <= shift {
					return col;
				}

				if ch == '\t' {
					shift += width - (shift - 1) as u32 % width;
					col += 1;
				} else {
					shift += 1;
					col += 1;
				}

			}

		} else {
			return 1;
		}

		return col;

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

