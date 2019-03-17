// wengwengweng

use std::collections::HashMap;

use dirty::color;
use dirty::math::*;

use suite::buffer::Span;

pub struct Theme {

	pub spans: HashMap<Span, Style>,
	pub normal: Style,
	pub search: Style,
	pub highlight: Style,
	pub background: Color,
	pub cursor: Color,
	pub cursor_line: Color,

}

#[derive(Debug, Clone)]
pub struct Style {
	pub color: Color,
	pub background: Color,
	pub style: FontStyle,
}

impl Style {
	pub fn new(c: Color, b: Color, s: FontStyle) -> Self {
		return Self {
			color: c,
			background: b,
			style: s,
		};
	}
}

#[derive(Debug, Clone)]
pub enum FontStyle {
	Normal,
	Bold,
}

impl Default for Theme {

	fn default() -> Self {

		let black = color!(0.10, 0.13, 0.18, 1.0);
		let white = color!(0.78, 0.80, 0.83, 1.0);
		let grey = color!(0.24, 0.27, 0.33, 1.0);
		let yellow = color!(1, 0.8, 0.42, 1.0);
		let green = color!(0.60, 0.78, 0.58, 1.0);
		let purple = color!(0.77, 0.58, 0.77, 1.0);
		let orange = color!(0.98, 0.57, 0.34, 1.0);
		let cyan = color!(0.38, 0.70, 0.70, 1.0);
		let brown = color!(0.67, 0.47, 0.40, 1.0);
		let red = color!(0.93, 0.37, 0.40, 1.0);
		let blue = color!(0.40, 0.60, 0.80, 1.0);
		let none = color!(1.0, 1.0, 1.0, 0.0);

		let mut spans = HashMap::new();

		spans.insert(Span::Normal, Style::new(white, none, FontStyle::Normal));
		spans.insert(Span::Comment, Style::new(grey, none, FontStyle::Normal));
		spans.insert(Span::PreProc, Style::new(yellow, none, FontStyle::Normal));
		spans.insert(Span::String, Style::new(green, none, FontStyle::Normal));
		spans.insert(Span::Keyword, Style::new(purple, none, FontStyle::Normal));
		spans.insert(Span::Type, Style::new(yellow, none, FontStyle::Bold));
		spans.insert(Span::Opt, Style::new(yellow, none, FontStyle::Normal));
		spans.insert(Span::Value, Style::new(orange, none, FontStyle::Bold));
		spans.insert(Span::Ident, Style::new(blue, none, FontStyle::Bold));
		spans.insert(Span::Special, Style::new(red, none, FontStyle::Normal));

		return Self {

			spans: spans,
			normal: Style::new(white, none, FontStyle::Normal),
			search: Style::new(white, yellow, FontStyle::Bold),
			highlight: Style::new(white, yellow, FontStyle::Bold),
			background: black,
			cursor: color!(1.0, 1.0, 1.0, 0.5),
			cursor_line: color!(1.0, 1.0, 1.0, 0.03),

		};
	}
}

