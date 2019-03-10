// wengwengweng

pub struct Conf {
	scale: f32,
	size: u32,
	margin: u32,
}

#[derive(Hash, Clone, Copy, PartialEq, Eq)]
enum TexFlag {
	Folder,
	Selection,
	Text,
	Image,
	Back,
}

pub struct View {

	textures: HashMap<TexFlag, gfx::Texture>,
	previewed_images: HashMap<PathBuf, gfx::Texture>,

}

