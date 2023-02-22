use std::io::{Read,Write};
use codec;

#[derive(Debug, Clone)]
pub struct Request {
}

impl codec::Decode for Request {
    fn decode<R : Read>(_f : &mut R) -> codec::Result<Request> {
        Ok(Request {
        })
    }
}

pub struct Response {
}

impl codec::Encode for Response {
    fn encode<W : Write>(&self, _f : &mut W) -> codec::Result<()> {
        Ok(())
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

pub fn run(_req : Request) -> Result<Response, String> {
    Ok(Response{})
}
