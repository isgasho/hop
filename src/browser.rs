// wengwengweng

use std::path::PathBuf;
use std::ffi::OsStr;

use dirty::*;
use input::Key;

use crate::Act;

pub struct Browser {
	listings: Vec<PathBuf>,
	path: PathBuf,
	conf: Conf,
	selection: Selection,
}

pub struct Conf {
	ignores: FilterList,
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

	pub fn new(path: &str) -> Self {

		let mut browser = Browser {
			listings: Vec::new(),
			path: PathBuf::from(path),
			conf: Conf::default(),
			selection: Selection::Back,
		};

		browser.refresh();

		return browser;

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

	pub fn find(&self, path: &PathBuf) -> Option<usize> {

		for (i, p) in self.listings.iter().enumerate() {
			if p == path {
				return Some(i);
			}
		}

		return None;

	}

	pub fn back(&mut self) {

		let old_path = self.path.clone();

		if let Some(parent) = self.path.parent() {
			self.cd(parent.to_path_buf());
		}

		if let Some(i) = self.find(&old_path) {
			self.select_item(i);
		}

	}

	pub fn enter(&mut self) {

		match self.selection {

			Selection::Item(i) => {

				if let Some(path) = self.listings.get(i) {
					if path.is_dir() {
						self.cd(path.clone());
					} else if path.is_file() {
						// ...
					}
				}

			},

			Selection::Back => {
				self.back();
			}

		}

	}

	pub fn move_up(&mut self) {

		if let Selection::Item(i) = self.selection {
			if i == 0 {
				self.selection = Selection::Back;
			} else {
				self.select_item(i - 1);
			}
		}

	}

	pub fn move_down(&mut self) {

		match self.selection {
			Selection::Back => {
				self.selection = Selection::Item(0);
			}
			Selection::Item(i) => {
				self.select_item(i + 1);
			}
		}

	}

	pub fn select_item(&mut self, i: usize) {
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

				if !self.conf.ignores.check(&get_fname(&p)) {

					if p.is_dir() {
						dirs.push(p);
					} else if p.is_file() {
						files.push(p);
					}

				}

			}

			dirs.sort();
			files.sort();

			self.listings.append(&mut dirs);
			self.listings.append(&mut files);

		}

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

		if input::key_pressed(Key::K) {
			self.move_up();
		}

	}

	fn draw(&self) {

		let (w, h) = window::size();

		g2d::color(color!(0.10, 0.13, 0.17, 1));
		g2d::rect(vec2!(w, h));

		g2d::push();
		g2d::translate(vec2!(24));

		g2d::color(color!(0.98, 0.78, 0.39, 1));
		g2d::text("..");
		g2d::translate(vec2!(0, 24));

		for (i, path) in self.listings.iter().enumerate() {

			let name = get_fname(path);

			if path.is_dir() {

				g2d::color(color!(0.38, 0.7, 0.7, 1));
				g2d::text("+");
				g2d::push();
				g2d::translate(vec2!(16, 0));
				g2d::color(color!(0.4, 0.6, 0.8, 1));
				g2d::text(&format!("{}", name));
				g2d::pop();

			} else {

				g2d::push();
				g2d::translate(vec2!(16, 0));
				g2d::color(color!());
				g2d::text(&format!("{}", name));
				g2d::pop();

			}

			g2d::translate(vec2!(0, 24));

		}

		g2d::pop();

		g2d::translate(vec2!(0, 20));

		match self.selection {
			Selection::Back => {
			},
			Selection::Item(i) => {
				g2d::translate(vec2!(0, (i + 1) * 24));
			}
		}

		g2d::color(color!(1, 1, 1, 0.02));
		g2d::rect(vec2!(240, 24));

	}

}

fn osstr_to_str(osstr: &OsStr) -> String {
	return osstr
		.to_os_string()
		.into_string()
		.expect("failed to parse fname");
}

fn get_fname(path: &PathBuf) -> String {

	let osstr = path
		.file_name()
		.expect("failed to parse fname");

	return osstr_to_str(osstr);

}

