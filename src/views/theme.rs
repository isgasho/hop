// wengwengweng

use dirty::*;
use dirty::math::*;

use hop::buffer::Span;
use std::collections::HashMap;

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
	Italic,
	BoldItalic,
}

impl Default for Theme {

	fn default() -> Self {

		let black = Color::new(0.11, 0.14, 0.19, 1.0);
		let white = Color::new(0.78, 0.80, 0.83, 1.0);
		let grey = Color::new(0.24, 0.27, 0.33, 1.0);
		let yellow = Color::new(0.98, 0.78, 0.39, 1.0);
		let green = Color::new(0.60, 0.78, 0.58, 1.0);
		let purple = Color::new(0.77, 0.58, 0.77, 1.0);
		let orange = Color::new(0.98, 0.57, 0.34, 1.0);
		let cyan = Color::new(0.38, 0.70, 0.70, 1.0);
		let brown = Color::new(0.67, 0.47, 0.40, 1.0);
		let red = Color::new(0.93, 0.37, 0.40, 1.0);
		let blue = Color::new(0.40, 0.60, 0.80, 1.0);
		let none = Color::new(1.0, 1.0, 1.0, 0.0);

		let mut spans = HashMap::new();

		spans.insert(Span::Normal, Style::new(white, none, FontStyle::Normal));
		spans.insert(Span::Comment, Style::new(grey, none, FontStyle::Normal));
		spans.insert(Span::String, Style::new(green, none, FontStyle::Normal));
		spans.insert(Span::Keyword, Style::new(purple, none, FontStyle::Normal));
		spans.insert(Span::Type, Style::new(yellow, none, FontStyle::Bold));
		spans.insert(Span::Number, Style::new(orange, none, FontStyle::Bold));
		spans.insert(Span::Ident, Style::new(blue, none, FontStyle::Bold));

		return Self {

			spans: spans,
			normal: Style::new(white, none, FontStyle::Normal),
			search: Style::new(white, yellow, FontStyle::Bold),
			highlight: Style::new(white, yellow, FontStyle::Bold),
			background: black,
			cursor: Color::new(1.0, 1.0, 1.0, 0.4),
			cursor_line: Color::new(1.0, 1.0, 1.0, 0.04),

		};
	}
}

