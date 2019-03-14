// wengwengweng

use pest::Parser;
use pest_vm::Vm;
use pest_meta::parser;
use pest_meta::{optimizer, validator};

use super::*;

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum Span {
	Normal,
	Comment,
	String,
	Keyword,
	Type,
	Number,
	Ident,
}

pub struct Syntax {
	vm: Option<Vm>,
}

impl Syntax {

	pub fn new(code: &str) -> Self {

		let mut vm = None;

		if let Ok(pairs) = parser::parse(parser::Rule::grammar_rules, code) {
			if let Ok(ast) = parser::consume_rules(pairs) {
				vm = Some(Vm::new(optimizer::optimize(ast.clone())));
			}
		}

		return Self {
			vm: vm,
		};

	}

	pub fn none() -> Self {
		return Self {
			vm: None,
		};
	}

	pub fn parse(&self, line: &str) -> Vec<RenderedChunk> {

		if let Some(vm) = &self.vm {

			let mut last = 0;
			let mut rendered = vec![];

			if let Ok(mut r) = vm.parse("line", line) {

				if let Some(file) = r.next() {

					for record in file.into_inner() {

						let span = record.as_span();
						let rule = record.as_rule();
						let start = span.start();
						let end = span.end();

						if start > last {

							rendered.push(RenderedChunk::Text {
								span: Span::Normal,
								text: String::from(&line[last..start]),
							});

						}

						if rule == "tab" {
							rendered.push(RenderedChunk::Shift);
						} else {
							rendered.push(RenderedChunk::Text {
								span: rule.into(),
								text: String::from(&line[start..end]),
							});
						}

						last = end;

					}

					if last < line.len() {

						rendered.push(RenderedChunk::Text {
							span: Span::Normal,
							text: String::from(&line[last..line.len()]),
						});

					}

					return rendered;

				}

			}

		}

		return RenderedChunk::from_plain(line);

	}


}

impl From<&str> for Span {
	fn from(r: &str) -> Span {
		return match r {
			"keyword" => Span::Keyword,
			"type" => Span::Type,
			"string" => Span::String,
			"number" => Span::Number,
			_ => Span::Normal,
		};
	}
}

