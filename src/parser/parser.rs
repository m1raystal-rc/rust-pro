use std::error::Error;
use std::fs::File;
use std::io::Read;
use serde_json::Value;
use crate::parser::descriptor::ParserDes;

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
		
		pub fn glb(&self) -> Result<(), Box<dyn Error>> {
				let mut glb_file = File::open(self.path)?;
				let mut data = Vec::new();
				glb_file.read_to_end(&mut data)?;
				let json_length = u32::from_le_bytes([data[12], data[13], data[14], data[15]]) as usize;
				let data_json: Value = serde_json::from_slice(&data[20..20 + json_length])?;
				println!("{:?}", data_json);
				Ok(())
		}
}