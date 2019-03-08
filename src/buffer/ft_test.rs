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

	let mut syntax = Syntax::new();

	syntax.add_keywords(&["use", "pub", "fn", "let"]);
	syntax.add_types(&["i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128"]);

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

