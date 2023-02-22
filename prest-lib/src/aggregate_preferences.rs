use std::io::{Read,Write};
use codec;
use model;
use estimation;

#[derive(Debug, Clone)]
pub struct Request {
    subjects : Vec<estimation::Response>,
}

impl codec::Decode for Request {
    fn decode<R : Read>(f : &mut R) -> codec::Result<Request> {
        Ok(Request {
            subjects: codec::Decode::decode(f)?,
        })
    }
}

pub struct Response {
    instance : model::Instance,
}

impl codec::Encode for Response {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        (&self.instance).encode(f)
    }
}

pub enum Error {
}

impl codec::Encode for Error {
    fn encode<W : Write>(&self, _f : &mut W) -> codec::Result<()> {
        match *self {
        }
    }
}

pub fn run(req : Request) -> Result<Response, String> {
    Ok(Response{
        instance: codec::decode_from_memory(&req.subjects[0].best_instances[0].instance).unwrap(),
    })
}
