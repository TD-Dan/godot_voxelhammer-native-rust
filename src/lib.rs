use gdnative::prelude::*;
use gdnative::core_types::Int32Array;
use gdnative::core_types::ByteArray;
use gdnative::api::Mesh;
use gdnative::api::ArrayMesh;
use gdnative::api::SurfaceTool;

use std::collections::HashMap;

#[derive(NativeClass)]
#[inherit(Node)]
#[user_data(user_data::MutexData<NativeWorkerRust>)]
struct NativeWorkerRust;

// Faces of a cube
const CUBE_FACE_LEFT: [[i32;3];6] = [[0,0,0],[1,0,0],[0,1,0],[1,0,0],[1,1,0],[0,1,0]];   // -x
const CUBE_FACE_RIGHT: [[i32;3];6] = [[1,0,1],[0,1,1],[1,1,1],[0,0,1],[0,1,1],[1,0,1]];  // x
const CUBE_FACE_BOTTOM: [[i32;3];6] = [[0,0,0],[0,0,1],[1,0,0],[1,0,1],[1,0,0],[0,0,1]]; // -y
const CUBE_FACE_TOP: [[i32;3];6] = [[0,1,0],[1,1,0],[0,1,1],[1,1,1],[0,1,1],[1,1,0]];    // y
const CUBE_FACE_FRONT: [[i32;3];6] = [[0,0,0],[0,1,0],[0,0,1],[0,1,0],[0,1,1],[0,0,1]];  // -z
const CUBE_FACE_BACK: [[i32;3];6] = [[1,0,0],[1,0,1],[1,1,0],[1,0,1],[1,1,1],[1,1,0]];   // z

#[gdnative::methods]
impl NativeWorkerRust {
	fn new(_owner: &Node) -> Self {
		NativeWorkerRust
	}

	#[export]
	fn _ready(&self, _owner: &Node) {
		godot_print!("VoxelHammer Native Rust worker active.")
	}

	#[export]
	fn create_mesh(&self, _owner: &Node, size: Vector3,  material: Int32Array, smooth: ByteArray, visible: ByteArray) -> Ref<ArrayMesh,Unique>{
		let mut surface_tools: HashMap<i32, Ref<SurfaceTool,Unique>> = HashMap::new();
		let mesh_buffer = ArrayMesh::new();
		let largest_size = size.x.max(size.y).max(size.z);
		let mut smooth_group_active = false;
		let sx = size.x as i32;
		let sy = size.y as i32;
		let sz = size.z as i32;

		// Loop trough all indices once
		for x in 0..sx {
			for y in 0..sy {
				for z in 0..sz {
					let ci : i32 = x + y*sx + z*sx*sy;
					let material_at_index = material.get(ci);
					if material_at_index != 0 && visible.get(ci) != 0 {
						let mut st_o = surface_tools.get(&material_at_index);
						match st_o {
							Some(_v) => (),
							None => {
								let new_st = SurfaceTool::new();
								new_st.begin(Mesh::PRIMITIVE_TRIANGLES);
								surface_tools.insert(material_at_index, new_st);
								st_o = surface_tools.get(&material_at_index);
							}
						}
						let st = st_o.unwrap();

						if smooth.get(ci) != 0 {
							if !smooth_group_active {
								st.add_smooth_group(true);
								smooth_group_active = true;
							}
						}
						else if smooth_group_active {
							st.add_smooth_group(false);
							smooth_group_active = false;
						}

						if x == 0 || material.get(ci-1) == 0 {
							for vert in CUBE_FACE_FRONT.iter() {
								st.add_uv(Vector2::new( (vert[2]+z) as f32 / largest_size, 1.0 - (vert[1]+y) as f32 / largest_size));
								st.add_vertex(Vector3::new( (vert[0]+x)as f32, (vert[1]+y) as f32, (vert[2]+z) as f32));
							}
						}
					}
				}
			}
		}

		return mesh_buffer;
	}
}


fn init(handle: InitHandle) {
	godot_print!("VoxelHammer Native Rust loaded.");
	handle.add_tool_class::<NativeWorkerRust>();
}

godot_init!(init);
