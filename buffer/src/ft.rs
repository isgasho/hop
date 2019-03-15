// wengwengweng

use std::collections::HashMap;

use regex::Regex;

use super::*;

pub struct FileType {

	pub name: String,
	pub match_fname: Option<Regex>,
	pub comment: Option<String>,
	pub shift_width: u32,
	pub auto_indent: bool,
	pub expand_tab: bool,
	pub indent_forward: Option<Regex>,
	pub indent_backward: Option<Regex>,
	pub syntax: Syntax,
	pub pairs: HashMap<char, char>,

}

impl Default for FileType {
	fn default() -> Self {
		return Self {
			name: String::from("text"),
			match_fname: None,
			comment: None,
			shift_width: 4,
			auto_indent: true,
			expand_tab: false,
			indent_forward: None,
			indent_backward: None,
			pairs: HashMap::new(),
			syntax: Syntax::none(),
		};
	}
}

pub struct FTRegistry {
	list: HashMap<String, FileType>,
}

impl FTRegistry {

	pub fn new() -> Self {
		return Self {
			list: HashMap::new(),
		};
	}

	pub fn add(&mut self, ft: FileType) {
		self.list.insert(ft.name.clone(), ft);
	}

	pub fn get(&self, name: &str) -> Option<&FileType> {
		return self.list.get(name);
	}

	pub fn find_for(&self, fname: &str) -> Option<&FileType> {

		for ft in self.list.values() {
			if let Some(match_fname) = &ft.match_fname {
				if match_fname.is_match(fname) {
					return Some(ft);
				}
			}
		}

		return None;

	}

}

