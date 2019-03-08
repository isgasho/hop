// wengwengweng

use super::ft::*;

pub fn rust() -> FileType {

	return FileType {

		name: String::from("rust"),
		comment: Some(String::from("//")),
		shift_width: 4,
		expand_tab: false,
		indent_forward: vec![],
		indent_backward: vec![],
		syntax: None,

	}
}

