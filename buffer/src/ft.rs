// wengwengweng

use std::collections::HashMap;
use std::collections::HashSet;

use regex::Regex;

use super::*;

pub struct FileType {

	pub name: String,
	pub comment: Option<String>,
	pub shift_width: u32,
	pub auto_indent: bool,
	pub expand_tab: bool,
	pub indent_forward: Vec<Regex>,
	pub indent_backward: Vec<Regex>,
	pub syntax: Syntax,
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
			auto_indent: true,
			expand_tab: false,
			indent_forward: vec![],
			indent_backward: vec![],
			pairs: HashMap::new(),
			syntax: Syntax::none(),
		};
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

