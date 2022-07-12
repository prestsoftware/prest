use std::io::{Read,Write};
use codec;
use codec::{Decode,Encode};

#[derive(Debug)]
pub struct Request {
    instance_code : String,
}

impl Decode for Request {
    fn decode<R : Read>(f : &mut R) -> codec::Result<Self> where Self : Sized {
        Ok(Request {
            instance_code: Decode::decode(f)?,
        })
    }
}

pub struct Response {
    edges : Vec<(u32, u32)>,
}

impl Encode for Response {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        self.edges.encode(f)
    }
}

pub fn run(req : Request) -> Response {
    return Response{
        edges: vec![(0,1),(1,2),(1,3)],
    }
}
