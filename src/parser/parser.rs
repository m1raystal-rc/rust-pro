use std::error::Error;
use std::fs::File;
use std::io::Read;
use serde_json::Value;
use crate::parser::descriptor::ParserDes;
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
		pub position: [f32; 3],
}
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Indicate {
		pub indicates: [u16; 3],
}
pub struct Package {
		pub vertices: Vec<Vertex>,
		pub indicates: Vec<Indicate>,
}
pub struct Parser<'a> {
		path: &'a str,
}
impl<'a> Parser<'a> {
		pub fn new(parser_des: ParserDes<'a>) -> Result<Self, Box<dyn Error>> {
				match parser_des.path.ends_with(".glb") {
						true => Ok(Parser {
								path: parser_des.path,
						}),
						false => Err("the file must be a .glb".into()),
				}
		}
		
		pub fn glb(&self) -> Result<Package, Box<dyn Error>> {
				let mut glb_file = File::open(self.path)?;
				let mut data = Vec::new();
				glb_file.read_to_end(&mut data)?;
				
				let json_len = u32::from_le_bytes([data[12], data[13], data[14], data[15]]) as usize;
				let json: Value = serde_json::from_slice(&data[20..20 + json_len])?; //1068
				//println!("{:?}", json);
				
				let mut bin_index = json_len + 20; //1088
				if (bin_index % 4 != 0) { bin_index += 4 - (bin_index % 4); } // align
				let bin_len = //840
						u32::from_le_bytes([data[bin_index], data[bin_index + 1], data[bin_index + 2], data[bin_index + 3]]) as usize;
				
				let data_bin = bin_index + 8;
				let data_end = data_bin + bin_len;
				let vertices: Vec<Vertex> = read_vertices(&json, &data[data_bin..data_end])?;
				let indicates: Vec<Indicate> = read_indicates(&json, &data[data_bin..data_end])?;
				
				Ok(Package { vertices, indicates })
		}
}
fn read_indicates(json: &Value, data: &[u8]) -> Result<Vec<Indicate>, Box<dyn Error>> {
		let mut indicates_r: Vec<Indicate> = Vec::new();
		
		let count = json["accessors"][3]["count"].as_u64().ok_or("indicate accessors err")? as usize;
		let pos_offset = json["bufferViews"][3]["byteOffset"].as_u64().ok_or("indicate bufferViews err")? as usize;
		
		for i in (0..count).step_by(3) {
				let offset = pos_offset + i * 2;
				let f = u16::from_le_bytes([data[offset], data[offset + 1]]);
				let s = u16::from_le_bytes([data[offset + 2], data[offset + 3]]);
				let t = u16::from_le_bytes([data[offset + 4], data[offset + 5]]);
				indicates_r.push(Indicate {
						indicates: [f, s, t],
				});
				//println!("{},{}|{}|{}", i, f, s, t)
		}
		Ok(indicates_r)
}
fn read_vertices(json: &Value, data: &[u8]) -> Result<Vec<Vertex>, Box<dyn Error>> {
		let mut vertices: Vec<Vertex> = Vec::new();
		
		let count = json["accessors"][0]["count"].as_u64().ok_or("vertex accessors err")? as usize;
		let pos_offset = json["bufferViews"][0]["byteOffset"].as_u64().ok_or("vertex bufferViews err")? as usize;
		
		for i in 0..count {
				let offset = pos_offset + i * 12;
				let x = f32::from_le_bytes([
						data[offset], data[offset + 1],
						data[offset + 2], data[offset + 3]
				]);
				let y = f32::from_le_bytes([
						data[offset + 4], data[offset + 5],
						data[offset + 6], data[offset + 7]
				]);
				let z = f32::from_le_bytes([
						data[offset + 8], data[offset + 9],
						data[offset + 10], data[offset + 11]
				]);
				vertices.push(Vertex {
						position: [x, y, z],
				});
				//println!("{},{}|{}|{}", i, x, y, z)
		}
		Ok(vertices)
}