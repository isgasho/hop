// wengwengweng

use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;

use crate::Buffer;
use crate::Act;

pub mod view;
mod utils;

pub struct Browser {

	listings: Vec<Item>,
	path: PathBuf,
	conf: Conf,
	selection: Selection,
	markings: Vec<usize>,
	mode: Mode,

}

pub struct Conf {
	ignores: FilterList,
}

struct Item {
	path: PathBuf,
	kind: ItemType,
}

enum Mode {
	Normal,
	Preview,
}

#[derive(Hash, Clone, Copy, PartialEq, Eq)]
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
			mode: Mode::Normal,
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
						if let Ok(buf) = Buffer::from_file(item.path.clone()) {
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

				if !self.conf.ignores.check(&utils::get_fname(&p)) {

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

		if let Mode::Normal = self.mode {
			self.mode = Mode::Preview;
		} else if let Mode::Preview = self.mode {
			self.mode = Mode::Normal;
		}

	}

}

