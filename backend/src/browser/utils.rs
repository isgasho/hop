// wengwengweng

use std::path::PathBuf;

pub fn get_ext(path: &PathBuf) -> Option<&str> {

	if let Some(osstr) = path.extension() {
		return osstr.to_str();
	} else {
		return None;
	}

}

pub fn get_fname(path: &PathBuf) -> Option<&str> {

	if let Some(osstr) = path.file_name() {
		return osstr.to_str();
	} else {
		return None;
	}

}

