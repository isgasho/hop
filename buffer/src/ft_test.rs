// wengwengweng

use std::collections::HashMap;

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
		comment: Some(String::from("//")),
		shift_width: 4,
		expand_tab: false,
		auto_indent: true,
		indent_forward: vec![],
		indent_backward: vec![],
		pairs: pairs,
		syntax: syntax,

	}
}

