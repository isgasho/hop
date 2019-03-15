// wengwengweng

use dirty::*;
use dirty::math::*;
use input::Key;
use input::Mouse;
use input::TextInput;

use crate::Act;
use suite::browser::Browser;
use suite::buffer::*;

use super::theme::*;

include!("../res/font.rs");

pub struct ViewConf {
	scroll_off: u32,
	scale: f32,
	line_space: i32,
	font: g2d::Font,
	theme: Theme,
	line_num: bool,
	shift_width: u32,
	show_indent: bool,
	margin_left: i32,
	wrap: bool,
	wrap_prefix: String,
}

impl Default for ViewConf {
	fn default() -> Self {
		return Self {
			scroll_off: 3,
			scale: 1.5,
			line_space: 1,
			margin_left: 12,
			theme: Theme::default(),
			line_num: false,
			shift_width: 4,
			show_indent: true,
			wrap: true,
			wrap_prefix: "..".to_owned(),
			font: g2d::Font::new(
				gfx::Texture::from_bytes(FONT),
				FONT_COLS,
				FONT_ROWS,
				FONT_CHARS,
			),
		};
	}
}

pub struct View {
	start_line: u32,
	conf: ViewConf,
	buffer: Buffer,
}

impl View {

	pub fn new(buf: Buffer) -> Self {
		return Self {
			start_line: 1,
			buffer: buf,
			conf: ViewConf::default(),
		};
	}

	pub fn view_range(&self) -> (u32, u32) {

		let start = self.start_line;
		let mut end = start + self.get_view_rows();

		if end > self.buffer.content.len() as u32 {
			end = self.buffer.content.len() as u32;
		}

		return (start, end);

	}

	pub fn get_view_rows(&self) -> u32 {

		g2d::set_font(&self.conf.font);

		let (w, h) = window::size().into();
		let rows = h as f32 / ((g2d::font_height() as i32 + self.conf.line_space) as f32 * self.conf.scale);

		return rows as u32;

	}

	pub fn scroll_down(&mut self) {

		if self.start_line < self.buffer.content.len() as u32 {
			if self.buffer.cursor.line - self.start_line >= self.conf.scroll_off {
				self.start_line += 1;
			}
		}

	}

	pub fn scroll_up(&mut self) {

		if self.start_line > 1 {
			if self.buffer.cursor.line < self.start_line + self.get_view_rows() - self.conf.scroll_off {
				self.start_line -= 1;
			}
		}

	}

	pub fn start_browser(&self) {

		if let Ok(browser) = Browser::from_file(self.buffer.path.clone()) {
			crate::start(crate::browser::View::new(browser));
		}

	}

}

impl Act for View {

	fn update(&mut self) {

		match self.buffer.mode {

			Mode::Normal => {

				if let Some(i) = input::text_input() {

					match i {

						TextInput::Char(ch) => {

							match ch {
								'y' => self.buffer.copy_line(),
								'h' => self.buffer.move_left(),
								'l' => self.buffer.move_right(),
								'j' => self.buffer.move_down(),
								'k' => self.buffer.move_up(),
								'u' => self.buffer.undo(),
								'o' => self.buffer.redo(),
								'd' => self.buffer.del_line(),
								'<' => self.buffer.move_line_start_insert(),
								'>' => self.buffer.move_line_end_insert(),
								':' => self.buffer.start_command(),
								'?' => self.buffer.start_search(),
								'H' => self.buffer.move_prev_word(),
								'L' => self.buffer.move_next_word(),
								'/' => self.buffer.toggle_comment(),
								'q' => self.buffer.indent_backward(),
								'e' => self.buffer.indent_forward(),
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
					self.buffer.start_insert();
				}

				if input::key_pressed(Key::Tab) {
					self.start_browser();
				}

				if input::key_pressed(Key::W) {
					self.buffer.write();
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
								self.buffer.move_up();
							}
						} else if scroll.y < 0 {
							for _ in 0..scroll.y.abs() {
								self.buffer.move_down();
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
								self.buffer.insert(ch);
							}

						},

						TextInput::Backspace => {

							if input::key_down(Key::LAlt) {
								self.buffer.del_word();
							} else {
								self.buffer.del();
							}

						},

						TextInput::Return => {

							if input::key_down(Key::LAlt) {
								// ..
							} else {
								self.buffer.break_line();
							}

						},

						TextInput::Tab => {
							self.buffer.insert('\t');
						},

						TextInput::Up => {
							self.scroll_up();
						},

						TextInput::Down => {
							self.scroll_down();
						},

						TextInput::Left => {
							self.buffer.move_left();
						},

						TextInput::Right => {
							self.buffer.move_right();
						},

					}

				}

				if input::key_pressed(Key::Escape) {
					self.buffer.start_normal();
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
					self.buffer.start_normal();
				}

			},

			Mode::Select(_) => {
				// ...
			},

			Mode::Search { .. } => {

				if input::key_pressed(Key::Escape) {
					self.buffer.start_normal();
				}

			},

		}

		// scroll by cursor
		let top = self.buffer.cursor.line as i32 - self.conf.scroll_off as i32;
		let bottom = self.buffer.cursor.line as i32 - self.get_view_rows() as i32 + self.conf.scroll_off as i32 + 1;

		if self.start_line as i32 > top {
			if top > 0 {
				self.start_line = top as u32;
			} else {
				self.start_line = 1;
			}
		}

		if (self.start_line as i32) < bottom && bottom < self.buffer.content.len() as i32 {
			self.start_line = bottom as u32;
		}

		let (start, end) = self.view_range();

		self.buffer.render(start as usize, end as usize);

	}

	fn draw(&self) {

		let buf = &self.buffer;

		g2d::scale(vec2!(self.conf.scale));
		g2d::set_font(&self.conf.font);

		let (w, h) = window::size().into();
		let (w, h) = (w as f32 / self.conf.scale, h as f32 / self.conf.scale);
		let tw = g2d::font_width();
		let th = g2d::font_height() as i32 + self.conf.line_space;

		g2d::color(self.conf.theme.background);
		g2d::rect(vec2!(w, h));

		// viewport
		g2d::translate(vec2!(self.conf.margin_left, 0));

		// content
		g2d::push();

		for (ln, line) in buf.rendered.iter().enumerate() {

			let real_line = ln as u32 + self.start_line;

			// cursor line
			if real_line == buf.cursor.line {

				g2d::push();
				g2d::color(self.conf.theme.cursor_line);
				g2d::translate(vec2!(-self.conf.margin_left, 0));
				g2d::rect(vec2!(w, th));
				g2d::pop();

			}

			let mut col = 1;
			let mut shift_col = 0;
			let mut cursor_drawn = false;

			g2d::push();

			// content
			for chunk in line {

				// cursor
				if real_line == buf.cursor.line && !cursor_drawn {

					if col >= buf.cursor.col as usize {

						let diff = buf.cursor.col as i32 - col as i32;

						g2d::push();
						g2d::translate(vec2!(diff * tw as i32, 0));
						g2d::color(self.conf.theme.cursor);

						match buf.mode {
							Mode::Normal => g2d::rect(vec2!(tw, th)),
							Mode::Insert => g2d::rect(vec2!(tw / 4, th)),
							_ => {},
						}

						g2d::pop();
						cursor_drawn = true;

					}

				}

				let splitted = chunk.text.split('\t');
				let count = chunk.text.split('\t').count();

				for (i, text) in splitted.enumerate() {

					if let Some(style) = self.conf.theme.spans.get(&chunk.span) {
						g2d::color(style.color);
					} else {
						g2d::color(self.conf.theme.normal.color);
					}

					g2d::text(text);
					g2d::translate(vec2!(text.len() * g2d::font_width() as usize, 0));

					if i < count - 1 {

						if self.conf.show_indent {
							g2d::color(color!(0.24, 0.27, 0.33, 1));
							g2d::text("|");
						}

						g2d::translate(vec2!(g2d::font_width() * self.conf.shift_width, 0));

					}

				}

			}

			// cursor
			if real_line == buf.cursor.line && !cursor_drawn {

				let diff = buf.cursor.col as i32 - col as i32;

				g2d::push();
				g2d::translate(vec2!(diff * tw as i32, 0));
				g2d::color(self.conf.theme.cursor);

				match buf.mode {
					Mode::Normal => g2d::rect(vec2!(tw, th)),
					Mode::Insert => g2d::rect(vec2!(tw / 4, th)),
					_ => {},
				}

				g2d::pop();

			}

			g2d::pop();
			g2d::translate(vec2!(0, th));

		}


		g2d::pop();

	}

}

