// wengwengweng

use std::collections::HashMap;

use super::ft::*;

pub fn rust() -> FileType {

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
		indent_forward: vec![],
		indent_backward: vec![],
		pairs: pairs,
		syntax: None,

	}
}

