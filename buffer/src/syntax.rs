// wengwengweng

use pest_vm::Vm;
use pest_meta::parser;
use pest_meta::optimizer;

use super::*;

pub struct Syntax {
	vm: Option<Vm>,
}

impl Syntax {

	pub fn new(code: &str) -> Self {

		let mut vm = None;

		match parser::parse(parser::Rule::grammar_rules, code) {
			Ok(pairs) => {
				if let Ok(ast) = parser::consume_rules(pairs) {
					vm = Some(Vm::new(optimizer::optimize(ast.clone())));
				}
			},
			Err(e) => {
				dbg!(e);
			},
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

	pub fn parse(&self, line: &str) -> Vec<SpannedText> {

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

							rendered.push(SpannedText {
								span: Span::Normal,
								text: String::from(&line[last..start]),
							});

						}

						rendered.push(SpannedText {
							span: rule.into(),
							text: String::from(&line[start..end]),
						});

						last = end;

					}

					if last < line.len() {

						rendered.push(SpannedText {
							span: Span::Normal,
							text: String::from(&line[last..line.len()]),
						});

					}

					return rendered;

				}

			}

		}

		return SpannedText::from_plain(line);

	}


}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum Span {
	Normal,
	Comment,
	String,
	Keyword,
	PreProc,
	Type,
	Value,
	Opt,
	Ident,
	Special,
}

impl From<&str> for Span {
	fn from(r: &str) -> Span {
		return match r {
			"keyword" => Span::Keyword,
			"types" => Span::Type,
			"string" => Span::String,
			"comment" => Span::Comment,
			"ident" => Span::Ident,
			"preproc" => Span::PreProc,
			"special" => Span::Special,
			"value" => Span::Value,
			"opt" => Span::Opt,
			_ => Span::Normal,
		};
	}
}

