// wengwengweng

use regex::Regex;
use super::syntax::*;

pub struct FileType {
	name: String,
	comment: String,
	shift_width: u32,
	expand_tab: bool,
	forward_pats: Vec<Regex>,
	backward_pats: Vec<Regex>,
	syntax: Syntax,
}

impl FileType {
	fn from_yaml(data: &str) -> Self {
		unimplemented!();
	}
}

