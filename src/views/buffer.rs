// wengwengweng

use dirty::*;
use dirty::math::*;
use input::Key;
use input::Mouse;
use input::TextInput;

use crate::Act;
use crate::Browser;
use crate::buffer::*;

pub struct ViewConf {
	scroll_off: u32,
	scale: f32,
	line_space: i32,
	font: g2d::Font,
}

impl Default for ViewConf {
	fn default() -> Self {
		return Self {
			scroll_off: 3,
			scale: 1.5,
			line_space: 2,
			font: g2d::Font::new(
				gfx::Texture::from_bytes(crate::FONT),
				crate::FONT_COLS,
				crate::FONT_ROWS,
				crate::FONT_CHARS,
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

	pub fn start_browser(&self) {

		if let Some(parent) = self.buffer.path.parent() {

			let mut browser = Browser::new(parent.to_path_buf());

			browser.select_item(&self.buffer.path);
			crate::start(crate::views::browser::View::new(browser));

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
								'd' => self.buffer.del_line(),
								'<' => self.buffer.move_line_start_insert(),
								'>' => self.buffer.move_line_end_insert(),
								':' => self.buffer.start_command(),
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
							self.buffer.scroll_up();
						},

						TextInput::Down => {
							self.buffer.scroll_down();
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
								self.buffer.scroll_up();
							}
						} else if scroll.y < 0 {
							for _ in 0..scroll.y.abs() {
								self.buffer.scroll_down();
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
							self.buffer.scroll_up();
						},

						TextInput::Down => {
							self.buffer.scroll_down();
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
						self.buffer.scroll_up();
					} else if scroll.y < 0 {
						self.buffer.scroll_down();
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

		}

		self.buffer.render();

	}

	fn draw(&self) {

		let buf = &self.buffer;

		g2d::scale(vec2!(self.conf.scale));
		g2d::set_font(&self.conf.font);

		let (w, h) = window::size().into();
		let (w, h) = (w as f32 / self.conf.scale, h as f32 / self.conf.scale);
		let tw = g2d::font_width();
		let th = g2d::font_height() as i32 + self.conf.line_space;

		g2d::color(buf.theme.background);
		g2d::rect(vec2!(w, h));

		// viewport
		g2d::translate(vec2!(12, 0));

		// content
		g2d::push();

		for (ln, line) in buf.rendered.iter().enumerate() {

			let real_line = ln as u32 + buf.start_line;

			// cursor line
			if real_line == buf.cursor.line {

				g2d::push();
				g2d::color(buf.theme.cursor_line);
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
				if real_line == buf.cursor.line && !cursor_drawn {

					if col >= buf.cursor.col as usize {

						let diff = buf.cursor.col as i32 - col as i32;

						g2d::push();
						g2d::translate(vec2!(diff * tw as i32, 0));
						g2d::color(buf.theme.cursor);

						match buf.mode {
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

