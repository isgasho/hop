// wengwengweng

use std::collections::HashMap;

use super::ft::*;

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "rust.pest"]
pub struct RustParser;

pub fn rust() -> FileType {

	if let Ok(mut parsed) = RustParser::parse(Rule::line, "let mut a: i32 = 123;") {

		if let Some(file) = parsed.next() {

			for record in file.into_inner() {
				match record.as_rule() {
					Rule::keywords => println!("Keyword: {:?}", record.as_span()),
					Rule::number => println!("Number: {:?}", record.as_span()),
					Rule::types => println!("Type: {:?}", record.as_span()),
					_ => {},
				}
			}

		} else {
			eprintln!("failed to parse2");
		}

	} else {
		eprintln!("failed to parse");
	}

	let mut pairs = HashMap::new();

	pairs.insert('(', ')');
	pairs.insert('\'', '\'');
	pairs.insert('"', '"');
	pairs.insert('{', '}');
	pairs.insert('[', ']');

	let mut syntax = Syntax::new();

	syntax.add_keyvalues(&["true", "false", "Some", "None", "Ok", "Err", "self"]);

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

