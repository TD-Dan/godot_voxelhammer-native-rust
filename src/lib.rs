use gdnative::prelude::*;
use gdnative::core_types::Int32Array;
use gdnative::core_types::ByteArray;
use gdnative::api::Mesh;
use gdnative::api::ArrayMesh;
use gdnative::api::SurfaceTool;

use std::collections::HashMap;

use itertools::Itertools; 

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
	fn create_fill(&self, _owner: &Node, size: Vector3, start: Vector3, end: Vector3, new_material: i32, new_smooth: u8, mut material: Int32Array, mut smooth: ByteArray) -> VariantArray<Unique> {
		let sx = size.x as i32;
		let sy = size.y as i32;
		let sz = size.z as i32;
		let total = sx*sy*sz;

		for z in 0..sz {
			for y in 0..sy{
				for x in 0..sx{
					if x >= start.x as i32 && x < end.x as i32 && y >= start.y as i32 && y < end.y as i32 && z >= start.z as i32 && z < end.z as i32 {
						material.set(x + y*sx + z*sx*sy, new_material);
						if new_smooth != 0 {
							smooth.set(x + y*sx + z*sx*sy, new_smooth);
						}
					}
				}
			}
		}
					
		let retarray= VariantArray::new();
		retarray.push(material);
		retarray.push(smooth);
		return retarray;
	}

	#[export]
	fn create_vis(&self, _owner: &Node, size: Vector3,  material: Int32Array, mut visible: ByteArray) -> ByteArray {
		let sx = size.x as i32;
		let sy = size.y as i32;
		let sz = size.z as i32;
		let total = sx*sy*sz;
		
		//if any dimension is 2 or less then every voxel is going to be visible
		if sx < 3 || sy < 3 || sz < 3 {
			for i in 0..total{
				visible.set(i,1);
			}
			return visible;
		}

		for z in 0..sz {
			for y in 0..sy {
				for x in 0..sx {
					let ci = x + y*sx + z*sx*sy;
					if x==0 || y == 0 || z == 0 || x == sx-1 || y == sy-1 || z == sz-1 {
						visible.set(ci,1);
					}
					else if material.get(ci + 1) != 0 && material.get(ci - 1) != 0 && material.get(ci + sx) != 0 && material.get(ci - sx) != 0 && material.get(ci + sx*sy) != 0 && material.get(ci - sx*sy) != 0 {
							visible.set(ci,0);
					}
					else {
						visible.set(ci,1);
					}
				}
			}
		}
		
		return visible;
	}

	#[export]
	fn create_mesh(&self, _owner: &Node, size: Vector3,  material: Int32Array, smooth: ByteArray, visible: ByteArray) -> VariantArray<Unique> {
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
						if x == sx-1 || material.get(ci+1) == 0 {
							for vert in CUBE_FACE_BACK.iter() {
								st.add_uv(Vector2::new( 1.0 - (vert[2]+z) as f32 / largest_size, 1.0 - (vert[1]+y) as f32 / largest_size));
								st.add_vertex(Vector3::new( (vert[0]+x)as f32, (vert[1]+y) as f32, (vert[2]+z) as f32));
							}
						}
						if y == 0 || material.get(ci-sx) == 0 {
							for vert in CUBE_FACE_BOTTOM.iter() {
								st.add_uv(Vector2::new( 1.0 - (vert[0]+x) as f32 / largest_size, (vert[2]+z) as f32 / largest_size));
								st.add_vertex(Vector3::new( (vert[0]+x)as f32, (vert[1]+y) as f32, (vert[2]+z) as f32));
							}
						}
						if y == sy-1 || material.get(ci+sx) == 0 {
							for vert in CUBE_FACE_TOP.iter() {
								st.add_uv(Vector2::new( 1.0 - (vert[0]+x) as f32 / largest_size, 1.0 - (vert[2]+z) as f32 / largest_size));
								st.add_vertex(Vector3::new( (vert[0]+x)as f32, (vert[1]+y) as f32, (vert[2]+z) as f32));
							}
						}
						if z == 0 || material.get(ci-sx*sy) == 0 {
							for vert in CUBE_FACE_LEFT.iter() {
								st.add_uv(Vector2::new( 1.0 - (vert[0]+x) as f32 / largest_size, 1.0 - (vert[1]+y) as f32 / largest_size));
								st.add_vertex(Vector3::new( (vert[0]+x)as f32, (vert[1]+y) as f32, (vert[2]+z) as f32));
							}
						}
						if z == sz-1 || material.get(ci+sx*sy) == 0 {
							for vert in CUBE_FACE_RIGHT.iter() {
								st.add_uv(Vector2::new( (vert[0]+x) as f32 / largest_size, 1.0 - (vert[1]+y) as f32 / largest_size));
								st.add_vertex(Vector3::new( (vert[0]+x)as f32, (vert[1]+y) as f32, (vert[2]+z) as f32));
							}
						}
					}
				}
			}
		}

		//let mut i = 0;
		let mut material_table: Vec<i32> = Vec::new();
		for (key, val) in surface_tools.iter().sorted() {	
			val.generate_normals(false);
			val.generate_tangents();
			
			mesh_buffer.add_surface_from_arrays(Mesh::PRIMITIVE_TRIANGLES, val.commit_to_arrays(), VariantArray::new().into_shared(), 97280);

			// godot_print!("surf {} = mat {}", i, key);
			material_table.push(*key);
			//i += 1;
		}

		// let mut i = 0;
		// for val in material_table.iter() {
		// 	godot_print!("mat_table {} = {}", i, val.to_string());
		// 	i += 1;
		// }

		let retarray= VariantArray::new();
		retarray.push(mesh_buffer as Ref<ArrayMesh,Unique>);
		retarray.push(material_table);
		return retarray;
	}
}


fn init(handle: InitHandle) {
	godot_print!("VoxelHammer Native Rust loaded.");
	handle.add_tool_class::<NativeWorkerRust>();
}

godot_init!(init);
