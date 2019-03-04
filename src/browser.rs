// wengwengweng

use std::fs;
use std::path::PathBuf;
use std::ffi::OsStr;
use std::collections::HashMap;

use dirty::*;
use input::Key;

use crate::Buffer;
use crate::Act;

pub struct Browser {

	listings: Vec<Item>,
	path: PathBuf,
	conf: Conf,
	selection: Selection,
	markings: Vec<usize>,
	font: g2d::Font,
	text_tex: gfx::Texture,
	folder_tex: gfx::Texture,
	selection_tex: gfx::Texture,
	back_tex: gfx::Texture,
	image_tex: gfx::Texture,
	previewing: bool,
	previewed_images: HashMap<PathBuf, gfx::Texture>,

}

pub struct Conf {
	ignores: FilterList,
}

struct Item {
	path: PathBuf,
	kind: ItemType,
}

enum ItemType {
	Folder,
	Text,
	Image,
	Music,
}

pub enum Error {
	// ...
}

enum Selection {
	Item(usize),
	Back,
}

impl Default for Conf {
	fn default() -> Self {
		return Self {
			ignores: FilterList::new(&[".DS_Store", ".git"]),
		};
	}
}

struct FilterList {
	list: Vec<String>,
}

impl FilterList {

	fn new(list: &[&str]) -> Self {
		return Self {
			list: list.iter().map(|s| String::from(*s)).collect(),
		};
	}

	fn check(&self, name: &str) -> bool {

		for f in &self.list {
			if name == f {
				return true;
			}
		}

		return false;

	}

}

impl Browser {

	pub fn new(path: PathBuf) -> Self {

		let mut browser = Browser {
			listings: Vec::new(),
			path: PathBuf::new(),
			conf: Conf::default(),
			selection: Selection::Back,
			markings: Vec::new(),
			text_tex: gfx::Texture::from_bytes(include_bytes!("res/text.png")),
			folder_tex: gfx::Texture::from_bytes(include_bytes!("res/folder.png")),
			image_tex: gfx::Texture::from_bytes(include_bytes!("res/image.png")),
			back_tex: gfx::Texture::from_bytes(include_bytes!("res/back.png")),
			selection_tex: gfx::Texture::from_bytes(include_bytes!("res/selection.png")),
			previewing: false,
			previewed_images: HashMap::new(),
			font: g2d::Font::new(
				gfx::Texture::from_bytes(crate::FONT),
				crate::FONT_COLS,
				crate::FONT_ROWS,
				crate::FONT_CHARS,
			),
		};

		browser.cd(path);

		return browser;

	}

	pub fn from_file() -> Result<Self, Error> {
		unimplemented!("");
	}

	fn cd(&mut self, path: PathBuf) {

		self.path = path;
		self.refresh();

		if self.listings.get(0).is_some() {
			self.selection = Selection::Item(0);
		} else {
			self.selection = Selection::Back;
		}

	}

	pub fn select_item(&mut self, path: &PathBuf) {

		let mut index = None;

		for (i, item) in self.listings.iter().enumerate() {
			if &item.path == path {
				index = Some(i);
			}
		}

		if let Some(i) = index {
			self.select_index(i);
		}

	}

	fn back(&mut self) {

		let old_path = self.path.clone();

		if let Some(parent) = self.path.parent() {
			self.cd(parent.to_path_buf());
		}

		self.select_item(&old_path);

	}

	fn selected(&self) -> Option<&Item> {

		if let Selection::Item(i) = self.selection {
			return self.listings.get(i);
		}

		return None;
	}

	fn enter(&mut self) {

		match self.selection {

			Selection::Item(i) => {

				if let Some(item) = self.listings.get(i) {
					if let ItemType::Folder = item.kind {
						self.cd(item.path.clone());
					} else if let ItemType::Text = item.kind {
						if let Ok(buf) = Buffer::from_file(&pathbuf_to_str(&item.path)) {
							crate::start(buf);
						}
					}
				}

			},

			Selection::Back => {
				self.back();
			}

		}

	}

	fn move_up(&mut self) {

		if let Selection::Item(i) = self.selection {
			if i == 0 {
				self.selection = Selection::Back;
			} else {
				self.select_index(i - 1);
			}
		}

	}

	fn move_down(&mut self) {

		match self.selection {
			Selection::Back => {
				self.selection = Selection::Item(0);
			}
			Selection::Item(i) => {
				self.select_index(i + 1);
			}
		}

	}

	fn select_index(&mut self, i: usize) {

		if self.listings.get(i).is_some() {
			self.selection = Selection::Item(i);
		}

	}

	fn refresh(&mut self) {

		self.listings = vec![];

		let mut dirs = vec![];
		let mut files = vec![];

		if let Ok(paths) = self.path.read_dir() {

			for p in paths
				.filter_map(Result::ok)
				.map(|e| e.path()) {

				if !self.conf.ignores.check(&get_fname(&p)) {

					if p.is_dir() {

						dirs.push(Item {
							path: p,
							kind: ItemType::Folder,
						});

					} else if p.is_file() {

						let mut kind = ItemType::Text;

						if let Some(ext) = p.extension() {

							if ext == "png" {
								kind = ItemType::Image;
							}

						}

						files.push(Item {
							path: p,
							kind: kind,
						});

					}

				}

			}

// 			dirs.sort();
// 			files.sort();

			self.listings.append(&mut dirs);
			self.listings.append(&mut files);

		}

	}

	fn mkdir(&mut self, name: &str) {

		if fs::create_dir(name).is_ok() {
			self.select_item(&PathBuf::from(name));
		}

	}

	fn mkfile(&mut self, name: &str) {
		// ...
	}

	fn toggle_preview(&mut self) {
		self.previewing = !self.previewing;
	}

}

impl Act for Browser {

	fn update(&mut self) {

		if input::key_pressed(Key::Backspace) {
			self.back();
		}

		if input::key_pressed(Key::Return) {
			self.enter();
		}

		if input::key_pressed(Key::J) {
			self.move_down();
		}

		if input::key_pressed(Key::Space) {
			self.toggle_preview();
		}

		if input::key_pressed(Key::K) {
			self.move_up();
		}

		if self.previewing {

			if let Some(item) = self.selected() {

				if let ItemType::Image = item.kind {
					if !self.previewed_images.contains_key(&item.path) {
						self.previewed_images.insert(item.path.clone(), gfx::Texture::from_file(&pathbuf_to_str(&item.path)));
					}
				}

			}

		}

	}

	fn draw(&self) {

		let (w, h) = window::size().into();
		let cols = 4;
		let size = 108;

		// all
		g2d::scale(vec2!(2));
		g2d::set_font(&self.font);

		// background
		g2d::push();
		g2d::color(color!(0.48, 1, 1, 1));
		g2d::rect(vec2!(w, h));
		g2d::pop();

		g2d::push();
		g2d::translate(vec2!(32));

		// back
		g2d::push();
// 		g2d::translate(vec2!(size));
		g2d::draw(&self.back_tex, rect!(0, 0, 1, 1));
		g2d::pop();

		// items
		for (i, item) in self.listings.iter().enumerate() {

			let x = (i + 1) % cols;
			let y = (i + 1) / cols;

			g2d::push();
			g2d::translate(vec2!(x, y) * size as f32);

			let name = get_fname(&item.path);

			match item.kind {

				ItemType::Folder => g2d::draw(&self.folder_tex, rect!(0, 0, 1, 1)),
				ItemType::Text => g2d::draw(&self.text_tex, rect!(0, 0, 1, 1)),
				ItemType::Image => g2d::draw(&self.image_tex, rect!(0, 0, 1, 1)),
				_ => {},

			}

			g2d::color(color!(0, 0, 0, 1));
			g2d::translate(vec2!(12, 48));
			g2d::text(&name);
			g2d::pop();

		}

		if let Selection::Item(i) = self.selection {

			let x = (i + 1) % cols;
			let y = (i + 1) / cols;

			g2d::push();
			g2d::translate(vec2!(x, y) * size as f32 - vec2!(22));
			g2d::scale(vec2!((app::time() * 6.0).sin() * 0.02 + 1.0));
			g2d::draw(&self.selection_tex, rect!(0, 0, 1, 1));
			g2d::pop();

		} else if let Selection::Back = self.selection {

			g2d::push();
			g2d::translate(vec2!(-22));
			g2d::scale(vec2!((app::time() * 6.0).sin() * 0.02 + 1.0));
			g2d::draw(&self.selection_tex, rect!(0, 0, 1, 1));
			g2d::pop();

		}

		g2d::pop();

		// preview
		if self.previewing {

			if let Some(item) = self.selected() {

				g2d::push();
				g2d::color(color!(0, 0, 0, 0.7));
				g2d::rect(vec2!(w, h));
				g2d::pop();

				if let ItemType::Image = item.kind {

					if let Some(tex) = self.previewed_images.get(&item.path) {

						let tw = tex.width();
						let th = tex.height();

						g2d::push();
						g2d::translate((vec2!(w, h) / 2.0 - vec2!(tw, th)) / 2.0);
						g2d::draw(tex, rect!(0, 0, 1, 1));
						g2d::pop();

					}

				}

			}

		}

	}

}

fn osstr_to_str(osstr: &OsStr) -> String {
	return osstr
		.to_os_string()
		.into_string()
		.expect("failed to parse path");
}

fn pathbuf_to_str(path: &PathBuf) -> String {
	return osstr_to_str(path.as_os_str());
}

fn get_fname(path: &PathBuf) -> String {

	let osstr = path
		.file_name()
		.expect("failed to parse fname");

	return osstr_to_str(osstr);

}

