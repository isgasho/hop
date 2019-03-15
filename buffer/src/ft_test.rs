// wengwengweng

use std::collections::HashMap;

use regex::Regex;

use super::*;

pub fn rust() -> FileType {

	let syntax = Syntax::new(include_str!("res/rust.syn"));
	let mut pairs = HashMap::new();

	pairs.insert('(', ')');
	pairs.insert('\'', '\'');
	pairs.insert('"', '"');
	pairs.insert('{', '}');
	pairs.insert('[', ']');

	return FileType {

		name: String::from("rust"),
		match_fname: Some(Regex::new(".rs$").unwrap()),
		comment: Some(String::from("//")),
		shift_width: 4,
		expand_tab: false,
		auto_indent: true,
		indent_forward: Some(Regex::new(r#"\{$"#).unwrap()),
		indent_backward: Some(Regex::new(r#"^\s*\}"#).unwrap()),
		pairs: pairs,
		syntax: syntax,

	}
}

