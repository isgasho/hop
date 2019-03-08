// wengwengweng

use regex::Regex;

pub struct FileType {

	pub name: String,
	pub comment: String,
	pub shift_width: u32,
	pub expand_tab: bool,
	pub indent_forward: Vec<Regex>,
	pub indent_backward: Vec<Regex>,
// 	pub syntax: Syntax,

}

pub struct Syntax {
	keywords: Vec<String>,
	contained: Vec<Container>,
}

struct Container {
	start: String,
	end: String,
}

impl FileType {
	fn from_yaml(data: &str) -> Self {
		unimplemented!();
	}
}

