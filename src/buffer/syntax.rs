// wengwengweng

pub struct Syntax {
	keywords: Vec<String>,
	contained: Vec<Container>,
}

struct Container {
	start: String,
	end: String,
}

