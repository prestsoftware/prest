use std::io::{Read,Write};
use codec;
use codec::{Decode,Encode};
use std::fmt;
use std::result::Result;

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

pub enum Error {
}

impl Encode for Error {
    fn encode<W : Write>(&self, _f : &mut W) -> codec::Result<()> {
        match *self {
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, _f : &mut fmt::Formatter) -> fmt::Result {
        match *self { }
    }
}

pub fn run(_req : Request) -> Result<Response, Error> {
    return Ok(Response{
        edges: vec![(0,1),(1,2),(1,3)],
    })
}
