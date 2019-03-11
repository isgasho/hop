// wengwengweng

use std::fs;
use std::path::PathBuf;

use super::utils;

pub struct Browser {

	pub listings: Vec<Item>,
	pub path: PathBuf,
	pub conf: Conf,
	pub selection: Selection,
	pub markings: Vec<usize>,

}

pub struct Conf {
	pub ignores: FilterList,
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct Item {
	pub path: PathBuf,
	pub name: String,
	pub kind: ItemType,
}

#[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ItemType {
	Folder,
	Text,
	Image,
	Music,
}

pub enum Error {
	IO,
}

pub enum Selection {
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

pub struct FilterList {
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
		};

		browser.cd(path);

		return browser;

	}

	pub fn from_file(path: PathBuf) -> Result<Self, ()> {

		if path.is_file() {

			if let Some(parent) = path.parent() {

				let mut browser = Self::new(parent.to_path_buf());

				browser.select_item(&path);

				return Ok(browser);

			} else {
				return Err(());
			}

		} else {
			return Err(());
		}

	}

	pub fn cd(&mut self, path: PathBuf) {

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

	pub fn back(&mut self) {

		let old_path = self.path.clone();

		if let Some(parent) = self.path.parent() {
			self.cd(parent.to_path_buf());
		}

		self.select_item(&old_path);

	}

	pub fn selected(&self) -> Option<&Item> {

		if let Selection::Item(i) = self.selection {
			return self.listings.get(i);
		}

		return None;
	}

	pub fn move_up(&mut self) {

		if let Selection::Item(i) = self.selection {
			if i == 0 {
				self.selection = Selection::Back;
			} else {
				self.select_index(i - 1);
			}
		}

	}

	pub fn move_down(&mut self) {

		match self.selection {
			Selection::Back => {
				self.select_index(0);
			}
			Selection::Item(i) => {
				self.select_index(i + 1);
			}
		}

	}

	pub fn select_index(&mut self, i: usize) {

		if self.listings.get(i).is_some() {
			self.selection = Selection::Item(i);
		}

	}

	pub fn refresh(&mut self) {

		self.listings = vec![];

		let mut dirs = vec![];
		let mut files = vec![];

		if let Ok(paths) = self.path.read_dir() {

			for p in paths
				.filter_map(Result::ok)
				.map(|e| e.path()) {

				if let Some(name) = utils::get_fname(&p) {

					if !self.conf.ignores.check(name) {

						if p.is_dir() {

							dirs.push(Item {
								name: name.to_owned(),
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
								name: name.to_owned(),
								path: p,
								kind: kind,
							});

						}

					}
				}

			}

// 			dirs.sort();
// 			files.sort();

			self.listings.append(&mut dirs);
			self.listings.append(&mut files);

		}

	}

	pub fn mkdir(&mut self, name: &str) {

		if fs::create_dir(name).is_ok() {
			self.select_item(&PathBuf::from(name));
		}

	}

}

