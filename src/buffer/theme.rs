// wengwengweng

struct Theme {
	normal: Style,
	comment: Style,
	string: Style,
	keyword: Style,
	number: Style,
	ident: Style,
	background: Color,
	cursor: Color,
	cursorline: Color,
}

struct Style {
	fg: Color,
	bg: Color,
	style: FontStyle,
}

enum FontStyle {
	Normal,
	Bold,
	Italic,
	BoldItalic,
}
