// wengwengweng

use std::collections::HashMap;
use std::path::PathBuf;

use dirty::*;
use input::Key;

use crate::Act;
use super::utils;
use super::ItemType;
use super::Mode;
use super::Browser;
use super::Selection;

pub struct Conf {
	scale: f32,
	size: u32,
	margin: u32,
}

impl Default for Conf {
	fn default() -> Self {
		return Self {
			margin: 32,
			scale: 1.5,
			size: 112,
		};
	}
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
	font: g2d::Font,
	textures: HashMap<TexFlag, gfx::Texture>,
	previewed_images: HashMap<PathBuf, gfx::Texture>,
	conf: Conf,

}

impl View {

	pub fn new(browser: Browser) -> Self {

		let mut textures = HashMap::new();

		textures.insert(TexFlag::Text, gfx::Texture::from_bytes(include_bytes!("res/text.png")));
		textures.insert(TexFlag::Image, gfx::Texture::from_bytes(include_bytes!("res/image.png")));
		textures.insert(TexFlag::Folder, gfx::Texture::from_bytes(include_bytes!("res/folder.png")));
		textures.insert(TexFlag::Back, gfx::Texture::from_bytes(include_bytes!("res/back.png")));
		textures.insert(TexFlag::Selection, gfx::Texture::from_bytes(include_bytes!("res/selection.png")));

		let mut view = View {
			browser: browser,
			previewed_images: HashMap::new(),
			textures: textures,
			conf: Conf::default(),
			font: g2d::Font::new(
				gfx::Texture::from_bytes(crate::FONT),
				crate::FONT_COLS,
				crate::FONT_ROWS,
				crate::FONT_CHARS,
			),
		};

		return view;

	}

}

impl Act for View {

	fn update(&mut self) {

		if input::key_pressed(Key::Backspace) {
			self.browser.back();
		}

		if input::key_pressed(Key::Return) {
			self.browser.enter();
		}

		if input::key_pressed(Key::J) {
			self.browser.move_down();
		}

		if input::key_pressed(Key::Space) {
			self.browser.toggle_preview();
		}

		if input::key_pressed(Key::K) {
			self.browser.move_up();
		}

		if let Mode::Preview = self.browser.mode {

			if let Some(item) = self.browser.selected() {

				if let ItemType::Image = item.kind {
					if !self.previewed_images.contains_key(&item.path) {
						self.previewed_images.insert(item.path.clone(), gfx::Texture::from_file(&utils::pathbuf_to_str(&item.path)));
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
		g2d::set_font(&self.font);

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

			let name = utils::get_fname(&item.path);

			match item.kind {

				ItemType::Folder => g2d::draw(&self.textures[&TexFlag::Folder], rect!(0, 0, 1, 1)),
				ItemType::Text => g2d::draw(&self.textures[&TexFlag::Text], rect!(0, 0, 1, 1)),
				ItemType::Image => g2d::draw(&self.textures[&TexFlag::Image], rect!(0, 0, 1, 1)),
				_ => {},

			}

			g2d::color(color!(0, 0, 0, 1));
			g2d::translate(vec2!(12, 48));
			g2d::text(&name);
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
		if let Mode::Preview = browser.mode {

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
		let bar_height = 23;

		g2d::push();
		g2d::translate(vec2!(0, h - bar_height as f32));
		g2d::color(color!(1, 0, 0.5, 1));
		g2d::rect(vec2!(w, 23));
		g2d::color(color!(0, 0, 0, 1));
		g2d::line(vec2!(0, 0), vec2!(w, 0));
		g2d::color(color!());
		g2d::translate(vec2!(8, (bar_height - g2d::font_height()) / 2));
		g2d::text(&utils::pathbuf_to_str(&browser.path));
		g2d::pop();

	}

}

