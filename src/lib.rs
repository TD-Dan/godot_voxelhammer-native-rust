use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Node)]
struct HelloWorld;

#[gdnative::methods]
impl HelloWorld {
	fn new(_owner: &Node) -> Self {
		HelloWorld
	}

	#[export]
	fn _ready(&self, _owner: &Node) {
		godot_print!("hello, world.")
	}
}


fn init(handle: InitHandle) {
	godot_print!("VoxelHammer Native Rust loaded.");
	handle.add_tool_class::<HelloWorld>();
}

godot_init!(init);
