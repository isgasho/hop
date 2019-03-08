// wengwengweng

use std::collections::HashMap;

use regex::Regex;

pub struct FileType {

	pub name: String,
	pub comment: Option<String>,
	pub shift_width: u32,
	pub expand_tab: bool,
	pub indent_forward: Vec<Regex>,
	pub indent_backward: Vec<Regex>,
	pub syntax: Option<Syntax>,
	pub pairs: HashMap<char, char>,

}

impl FileType {

	fn from_yaml(data: &str) -> Self {
		unimplemented!();
	}

}

impl Default for FileType {
	fn default() -> Self {
		return Self {
			name: String::from("text"),
			comment: None,
			shift_width: 4,
			expand_tab: false,
			indent_forward: vec![],
			indent_backward: vec![],
			pairs: HashMap::new(),
			syntax: None,
		};
	}
}

pub struct Syntax {
	keywords: Vec<String>,
	types: Vec<String>,
}

impl Syntax {

	pub fn new() -> Self {
		return Self {
			keywords: vec![],
			types: vec![],
		};
	}

	pub fn add_keywords(&mut self, words: &[&str]) {
		for w in words {
			self.keywords.push(w.to_owned().to_string());
		}
	}

	pub fn add_types(&mut self, types: &[&str]) {
		for w in types {
			self.types.push(w.to_owned().to_string());
		}
	}

}

pub struct Registry {
	// ...
}

impl Registry {

	pub fn new() -> Self {
		unimplemented!();
	}

	pub fn add(&mut self) {
		unimplemented!();
	}

}

