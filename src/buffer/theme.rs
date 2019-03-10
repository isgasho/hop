// wengwengweng

use dirty::*;
use dirty::math::*;

pub struct Theme {

	pub normal: Style,
	pub comment: Style,
	pub string: Style,
	pub keyword: Style,
	pub types: Style,
	pub number: Style,
	pub ident: Style,
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

		return Self {

			normal: Style::new(white, none, FontStyle::Normal),
			comment: Style::new(grey, none, FontStyle::Normal),
			string: Style::new(green, Color::all(0.0), FontStyle::Normal),
			keyword: Style::new(purple, Color::all(0.0), FontStyle::Normal),
			types: Style::new(yellow, Color::all(0.0), FontStyle::Bold),
			number: Style::new(orange, Color::all(0.0), FontStyle::Normal),
			ident: Style::new(blue, Color::all(0.0), FontStyle::Normal),
			search: Style::new(white, yellow, FontStyle::Bold),
			highlight: Style::new(white, yellow, FontStyle::Bold),
			background: black,
			cursor: Color::new(1.0, 1.0, 1.0, 0.4),
			cursor_line: Color::new(1.0, 1.0, 1.0, 0.04),

		};
	}
}

