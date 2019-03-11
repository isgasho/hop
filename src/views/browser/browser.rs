// wengwengweng

use std::collections::HashMap;
use std::path::PathBuf;

use dirty::*;
use input::Key;

use crate::Act;
use hop::buffer::Buffer;
use hop::browser::*;

include!("res/font.rs");

pub struct ViewConf {
	scale: f32,
	size: u32,
	margin: u32,
	bar_height: u32,
	font: g2d::Font,
}

impl Default for ViewConf {
	fn default() -> Self {
		return Self {
			margin: 32,
			scale: 1.5,
			size: 112,
			bar_height: 23,
			font: g2d::Font::new(
				gfx::Texture::from_bytes(FONT),
				FONT_COLS,
				FONT_ROWS,
				FONT_CHARS,
			),
		};
	}
}

pub enum Mode {
	Normal,
	Preview,
}

#[derive(Hash, Clone, Copy, PartialEq, Eq)]
enum TexFlag {
	Folder,
	Selection,
	Text,
	Image,
	Back,
}

pub struct View {

	browser: Browser,
	textures: HashMap<TexFlag, gfx::Texture>,
	previewed_images: HashMap<PathBuf, gfx::Texture>,
	conf: ViewConf,
	mode: Mode,

}

impl View {

	pub fn new(browser: Browser) -> Self {

		let mut textures = HashMap::new();

		textures.insert(TexFlag::Text, gfx::Texture::from_bytes(include_bytes!("res/text.png")));
		textures.insert(TexFlag::Image, gfx::Texture::from_bytes(include_bytes!("res/image.png")));
		textures.insert(TexFlag::Folder, gfx::Texture::from_bytes(include_bytes!("res/folder.png")));
		textures.insert(TexFlag::Back, gfx::Texture::from_bytes(include_bytes!("res/back.png")));
		textures.insert(TexFlag::Selection, gfx::Texture::from_bytes(include_bytes!("res/selection.png")));

		return Self {
			browser: browser,
			previewed_images: HashMap::new(),
			textures: textures,
			conf: ViewConf::default(),
			mode: Mode::Normal,
		};

	}

	pub fn enter(&mut self) {

		let browser = &mut self.browser;

		match browser.selection {

			Selection::Item(i) => {

				if let Some(item) = browser.listings.get(i) {
					if let ItemType::Folder = item.kind {
						browser.cd(item.path.clone());
					} else if let ItemType::Text = item.kind {
						if let Ok(buf) = Buffer::from_file(item.path.clone()) {
							crate::start(crate::views::buffer::View::new(buf));
						}
					}
				}

			},

			Selection::Back => {
				browser.back();
			}

		}

	}

	pub fn toggle_preview(&mut self) {

		if let Mode::Normal = self.mode {
			self.mode = Mode::Preview;
		} else if let Mode::Preview = self.mode {
			self.mode = Mode::Normal;
		}

	}

}

impl Act for View {

	fn update(&mut self) {

		if input::key_pressed(Key::Backspace) {
			self.browser.back();
		}

		if input::key_pressed(Key::Return) {
			self.enter();
		}

		if input::key_pressed(Key::J) {
			self.browser.move_down();
		}

		if input::key_pressed(Key::Space) {
			self.toggle_preview();
		}

		if input::key_pressed(Key::K) {
			self.browser.move_up();
		}

		if let Mode::Preview = self.mode {

			if let Some(item) = self.browser.selected() {

				if let ItemType::Image = item.kind {
					if !self.previewed_images.contains_key(&item.path) {
						self.previewed_images.insert(item.path.clone(), gfx::Texture::from_file(&format!("{}", item.path.display())));
					}
				}

			}

		}

	}

	fn draw(&self) {

		let browser = &self.browser;
		let (w, h) = window::size().into();
		let (w, h) = (w as f32 / self.conf.scale, h as f32 / self.conf.scale);
		let margin = self.conf.margin;
		let size = self.conf.size;
		let cols = (w as u32 - margin * 2) / size;
		let rmargin = (w as u32 - cols * size) / 2;

		// all
		g2d::scale(vec2!(self.conf.scale));
		g2d::set_font(&self.conf.font);

		// background
		g2d::push();
		g2d::color(color!(0.48, 1, 1, 1));
		g2d::rect(vec2!(w, h));
		g2d::pop();

		g2d::push();
		g2d::translate(vec2!(rmargin, 32));

		// back
		g2d::push();
		g2d::draw(&self.textures[&TexFlag::Back], rect!(0, 0, 1, 1));
		g2d::pop();

		// items
		for (i, item) in browser.listings.iter().enumerate() {

			let x = (i + 1) % cols as usize;
			let y = (i + 1) / cols as usize;

			g2d::push();
			g2d::translate(vec2!(x, y) * size as f32);

			match item.kind {

				ItemType::Folder => g2d::draw(&self.textures[&TexFlag::Folder], rect!(0, 0, 1, 1)),
				ItemType::Text => g2d::draw(&self.textures[&TexFlag::Text], rect!(0, 0, 1, 1)),
				ItemType::Image => g2d::draw(&self.textures[&TexFlag::Image], rect!(0, 0, 1, 1)),
				_ => {},

			}

			g2d::color(color!(0, 0, 0, 1));
			g2d::translate(vec2!(12, 48));
			g2d::text(&item.name);
			g2d::pop();

		}

		if let Selection::Item(i) = browser.selection {

			let x = (i + 1) % cols as usize;
			let y = (i + 1) / cols as usize;

			g2d::push();
			g2d::translate(vec2!(x, y) * self.conf.size as f32 - vec2!(22));
			g2d::scale(vec2!((app::time() * 6.0).sin() * 0.02 + 1.0));
			g2d::draw(&self.textures[&TexFlag::Selection], rect!(0, 0, 1, 1));
			g2d::pop();

		} else if let Selection::Back = browser.selection {

			g2d::push();
			g2d::translate(vec2!(-22));
			g2d::scale(vec2!((app::time() * 6.0).sin() * 0.02 + 1.0));
			g2d::draw(&self.textures[&TexFlag::Selection], rect!(0, 0, 1, 1));
			g2d::pop();

		}

		g2d::pop();

		// preview
		if let Mode::Preview = self.mode {

			if let Some(item) = browser.selected() {

				g2d::push();
				g2d::color(color!(0, 0, 0, 0.8));
				g2d::rect(vec2!(w, h));
				g2d::pop();

				if let ItemType::Image = item.kind {

					if let Some(tex) = self.previewed_images.get(&item.path) {

						let tw = tex.width();
						let th = tex.height();

						g2d::push();
						g2d::translate((vec2!(w, h) - vec2!(tw, th)) / 2.0);
						g2d::draw(tex, rect!(0, 0, 1, 1));
						g2d::pop();

					}

				}

			}

		}

		// bar
		let bar_height = self.conf.bar_height;

		g2d::push();
		g2d::translate(vec2!(0, h - bar_height as f32));
		g2d::color(color!(1, 0, 0.5, 1));
		g2d::rect(vec2!(w, 23));
		g2d::color(color!(0, 0, 0, 1));
		g2d::line(vec2!(0, 0), vec2!(w, 0));
		g2d::color(color!());
		g2d::translate(vec2!(8, (bar_height - g2d::font_height()) / 2));
		g2d::text(&format!("{}", browser.path.display()));
		g2d::pop();

	}

}

