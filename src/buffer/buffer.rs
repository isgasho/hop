// wengwengweng

use dirty::*;
use dirty::math::*;
use window::Key;
use window::Mouse;

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

		let size: Vec2 = window::size().into();
		let size = size / self.conf.scale;
		let rows = size.y as f32 / self.line_height();

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

	pub fn line_height(&self) -> f32 {
		return (self.conf.font.height() as i32 + self.conf.line_space) as f32;
	}

	pub fn screen_to_cursor(&self, pos: Vec2) -> Pos {

		let pos = pos / self.conf.scale;
		let v_line = (pos.y / self.line_height()) as u32;
		let line = v_line + self.start_line;
		let v_col = ((pos.x - self.conf.margin_left as f32) / self.conf.font.width() as f32) as u32 + 1;
		let col = self.buffer.get_unshifted_col(v_col, line, self.conf.shift_width);

		return Pos::new(line, col);

	}

	pub fn cursor_to_screen(&self, pos: Pos) -> Vec2 {

		let v_line = pos.line - self.start_line;
		let y = v_line as f32 * self.line_height();
		let v_col = self.buffer.get_shifted_pos(pos, self.conf.shift_width) as i32;
		let x = (v_col - 1) * self.conf.font.width() as i32 + self.conf.margin_left;

		return vec2!(x, y);

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

				if let Some(ch) = window::char_input() {

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

				}

				if window::key_pressed(Key::Return) {
					self.buffer.start_insert();
				}

				if window::key_pressed(Key::Tab) {
					self.start_browser();
				}

				if window::key_pressed(Key::W) {
					self.buffer.write();
				}

				if window::key_pressed(Key::Escape) {
					self.buffer.reset();
				}

				if window::mouse_pressed(Mouse::Left) {

					let mpos = window::mouse_pos();

					if window::key_down(Key::LAlt) {
						self.buffer.add_cursor(self.screen_to_cursor(mpos.into()));
					} else {
						self.buffer.move_to(self.screen_to_cursor(mpos.into()));
					}

				}

				if let Some(scroll) = window::scroll_delta() {

					if window::key_down(Key::LAlt) {

						if scroll.y > 0 {
							for _ in 0..scroll.y.abs() / 2 {
								self.scroll_up();
							}
						} else if scroll.y < 0 {
							for _ in 0..scroll.y.abs() / 2 {
								self.scroll_down();
							}
						}

					} else {

						if scroll.y > 0 {
							for _ in 0..scroll.y.abs() / 2 {
								self.buffer.move_up();
							}
						} else if scroll.y < 0 {
							for _ in 0..scroll.y.abs() / 2 {
								self.buffer.move_down();
							}
						}

					}

				}

			},

			Mode::Insert => {

				if window::key_pressed_repeat(Key::Back) {
					if window::key_down(Key::LAlt) {
						self.buffer.del_word();
					} else {
						self.buffer.del();
					}
				}

				if window::key_pressed_repeat(Key::Return) {

					if window::key_down(Key::LAlt) {
					} else {
						self.buffer.break_line();
					}

				}

				if window::key_pressed_repeat(Key::Up) {
					self.scroll_up();
				}

				if window::key_pressed_repeat(Key::Down) {
					self.scroll_down();
				}

				if window::key_pressed_repeat(Key::Left) {
					self.buffer.move_left();
				}

				if window::key_pressed_repeat(Key::Right) {
					self.buffer.move_right();
				}

				if let Some(ch) = window::char_input() {
					if !window::key_down(Key::LAlt) {
						self.buffer.insert(ch);
					}
				}

				if window::key_pressed(Key::Escape) {
					self.buffer.start_normal();
				}

				if let Some(scroll) = window::scroll_delta() {

					if scroll.y > 0 {
						self.scroll_up();
					} else if scroll.y < 0 {
						self.scroll_down();
					}

				}

			},

			Mode::Command => {

				if window::key_pressed(Key::Escape) {
					self.buffer.start_normal();
				}

			},

			Mode::Select(_) => {
				// ...
			},

			Mode::Search { .. } => {

				if window::key_pressed(Key::Escape) {
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
		self.buffer.adjust_cursor();

	}

	fn draw(&self) {

		let buf = &self.buffer;

		g2d::scale(vec2!(self.conf.scale));
		g2d::set_font(&self.conf.font);

		let (w, h) = window::size().into();
		let (w, h) = (w as f32 / self.conf.scale, h as f32 / self.conf.scale);
		let tw = self.conf.font.width();
		let th = self.line_height();

		// background
		g2d::color(self.conf.theme.background);
		g2d::rect(vec2!(w, h));

		let draw_cursor = |cpos: Pos| {

			// cursor
			let pos = self.cursor_to_screen(cpos);

			// cursor line
			g2d::push();
			g2d::color(self.conf.theme.cursor_line);
			g2d::translate(vec2!(0, pos.y));
			g2d::rect(vec2!(w, th));
			g2d::pop();

			// cursor
			g2d::push();
			g2d::translate(pos);
			g2d::color(self.conf.theme.cursor);

			match buf.mode {
				Mode::Normal => g2d::rect(vec2!(tw, th)),
				Mode::Insert => g2d::rect(vec2!(tw / 4, th)),
				_ => {},
			}

			g2d::pop();

		};

		draw_cursor(buf.cursor);

		for c in &buf.child_cursors {
			draw_cursor(*c);
		}

		// content
		g2d::push();
		g2d::translate(vec2!(self.conf.margin_left, 0));

		for line in &buf.rendered {

			let mut shift_col = 0;

			g2d::push();

			// content
			for chunk in line {

				let splitted = chunk.text.split('\t');
				let count = splitted.clone().count();

				for (i, text) in splitted.enumerate() {

					// text
					if let Some(style) = self.conf.theme.spans.get(&chunk.span) {
						g2d::color(style.color);
					} else {
						g2d::color(self.conf.theme.normal.color);
					}

					g2d::text(text);
					g2d::translate(vec2!(text.len() * tw as usize, 0));
					shift_col += text.len();

					// tab shift
					if i < count - 1 {

						if self.conf.show_indent {
							g2d::color(color!(0.24, 0.27, 0.33, 1));
							g2d::text("|");
						}

						let sw = self.conf.shift_width;
						let offset = sw - shift_col as u32 % sw;

						g2d::translate(vec2!(tw * offset as u32, 0));
						shift_col += offset as usize;

					}

				}

			}

			g2d::pop();
			g2d::translate(vec2!(0, th));

		}

		g2d::pop();

	}

}

