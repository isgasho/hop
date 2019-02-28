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
}

pub struct Conf {
	ignores: FilterList,
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
		};

		browser.refresh();

		return browser;

	}

	pub fn cd(&mut self, path: PathBuf) {

		self.path = path;
		self.refresh();

	}

	pub fn back(&mut self) {

		if let Some(parent) = self.path.parent() {
			self.cd(parent.to_path_buf());
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

	}

	fn draw(&self) {

		g2d::push();

		for (i, path) in self.listings.iter().enumerate() {

			g2d::push();
			g2d::translate(vec2!(0, i * 24));
			g2d::text(&format!("{}", get_fname(path)));
			g2d::pop();

		}

		g2d::pop();

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

