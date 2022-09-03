use std::io::{Read,Write};
use codec;
use codec::{Decode,Encode};
use std::fmt;
use std::result::Result;

use model::Instance;

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
    edges : Vec<(usize, usize)>,
    extra_info : Vec<(String, String)>,
}

impl Encode for Response {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        (&self.edges, &self.extra_info).encode(f)
    }
}

pub enum Error {
    Base64Decode(base64::DecodeError),
    Codec(codec::Error),
    SrcUnsupported,
}

impl From<base64::DecodeError> for Error {
    fn from(e : base64::DecodeError) -> Error { Error::Base64Decode(e) }
}

impl From<codec::Error> for Error {
    fn from(e : codec::Error) -> Error { Error::Codec(e) }
}

impl Encode for Error {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        use self::Error::*;
        match self {
            Base64Decode(_) => (0u8).encode(f),
            Codec(_) => (1u8).encode(f),
            SrcUnsupported => (2u8).encode(f),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;
        match self {
            Base64Decode(e) => write!(f, "could not decode base64: {}", e),
            Codec(e) => write!(f, "could not decode instance: {:?}", e),
            SrcUnsupported =>
                write!(f, "Sequentially Rationalizable Choice not supported in visualisation at the moment"),
        }
    }
}

pub fn run(req : Request) -> Result<Response, Error> {
    let bytes = base64::decode(&req.instance_code)?;
    let instance = codec::decode_from_memory(&bytes)?;

    match instance {
        Instance::PreorderMaximization(ref p) => {
            Ok(Response{
                edges: p.to_poset_graph().edges,
                extra_info: vec![],
            })
        }

        Instance::Unattractiveness{ref p, mask: ref _mask} => {
            Ok(Response{
                edges: p.to_poset_graph().edges,
                extra_info: vec![],
            })
        }

        Instance::UndominatedChoice(ref p) => {
            Ok(Response{
                edges: p.to_poset_graph().edges,
                extra_info: vec![],
            })
        }

        Instance::PartiallyDominantChoice{ref p, fc:_} => {
            Ok(Response{
                edges: p.to_poset_graph().edges,
                extra_info: vec![],
            })
        }

        Instance::Swaps(ref p) => {
            Ok(Response{
                edges: p.to_poset_graph().edges,
                extra_info: vec![],
            })
        }

        Instance::StatusQuoUndominatedChoice(ref p) => {
            Ok(Response{
                edges: p.to_poset_graph().edges,
                extra_info: vec![],
            })
        }

        Instance::Overload{ref p, limit} => {
            Ok(Response{
                edges: p.to_poset_graph().edges,
                extra_info: vec![
                    ("Threshold".to_string(), limit.to_string()),
                ],
            })
        }

        Instance::TopTwo(ref p) => {
            Ok(Response{
                edges: p.to_poset_graph().edges,
                extra_info: vec![],
            })
        }

        Instance::SequentiallyRationalizableChoice(ref p, ref _q) => {
            Ok(Response{
                edges: p.to_poset_graph().edges,
                extra_info: vec![],
            })
        }
    }
}
