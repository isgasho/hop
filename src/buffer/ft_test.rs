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

	syntax.add_keywords(&["use", "pub", "fn", "let", "return", "for", "in", "mod", "const", "match", "if", "else", "loop", "as", "enum", "struct", "impl", "trait", "type"]);
	syntax.add_keyvalues(&["true", "false", "Some", "None", "Ok", "Err", "self"]);
	syntax.add_types(&["i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32", "f64", "bool", "char", "String", "Vec", "HashMap", "HashSet", "Result", "Option", "Self", "Clone", "Default", "Debug", "Hash", "Copy", "Eq", "PartialEq"]);

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

