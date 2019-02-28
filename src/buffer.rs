// wengwengweng

use std::path::PathBuf;
use crate::Act;

pub struct Buffer {
	mode: Mode,
	selections: Vec<(Pos, Pos)>,
	path: PathBuf,
}

struct Pos {
	line: u32,
	col: u32,
}

enum Mode {
	Normal,
	Insert,
}

impl Buffer {
	fn new(path: PathBuf) -> Self {
		return Self {
			mode: Mode::Normal,
			selections: Vec::new(),
			path: path,
		};
	}
}

impl Act for Buffer {

	fn update(&mut self) {
		// ...
	}

	fn draw(&self) {
		// ...
	}

}

