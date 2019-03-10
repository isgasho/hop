// wengwengweng

use std::ffi::OsStr;
use std::path::PathBuf;

pub fn osstr_to_str(osstr: &OsStr) -> String {
	return osstr
		.to_os_string()
		.into_string()
		.expect("failed to parse path");
}

pub fn pathbuf_to_str(path: &PathBuf) -> String {
	return osstr_to_str(path.as_os_str());
}

pub fn get_fname(path: &PathBuf) -> String {

	let osstr = path
		.file_name()
		.expect("failed to parse fname");

	return osstr_to_str(osstr);

}

