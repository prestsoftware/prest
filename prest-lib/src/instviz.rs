use std::io::{Read,Write};
use std::fmt;
use std::result::Result;
use base64::prelude::BASE64_STANDARD;
use base64::engine::Engine;

use crate::codec;
use crate::codec::{Decode,Encode};
use crate::alt_set::AltSet;
use crate::model::Instance;
use crate::preorder::Preorder;

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

pub struct GraphRepr {
    vertices : Vec<AltSet>,  // classes of equivalence
    edges : Vec<(AltSet, AltSet)>,  // (P, Q) such that P ≥ Q
}

impl Encode for GraphRepr {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        (&self.vertices, &self.edges).encode(f)
    }
}

pub struct Response {
    graphs : Vec<GraphRepr>,
    extra_info : Vec<(String, String)>,
}

impl Encode for Response {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        (&self.graphs, &self.extra_info).encode(f)
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

pub fn graph_repr(p : &Preorder) -> GraphRepr {
    let g = p.to_poset_graph();
    GraphRepr{
        edges: g.edges.iter().map(
                |&(p, q)| (
                    g.vertices[p].clone(),
                    g.vertices[q].clone(),
                )
            ).collect(),
        vertices: g.vertices,
    }
}

pub fn graph_response(p : &Preorder) -> Response {
    Response{
        graphs: vec![graph_repr(p)],
        extra_info: vec![],
    }
}

pub fn run(req : Request) -> Result<Response, Error> {
    let bytes = BASE64_STANDARD.decode(&req.instance_code)?;
    let instance = codec::decode_from_memory(&bytes)?;

    match instance {
        Instance::PreorderMaximization(ref p) =>
            Ok(Response{
                graphs: vec![graph_repr(p)],
                extra_info: vec![],
            }),

        Instance::Unattractiveness{ref p, mask:_} =>
            Ok(Response{
                graphs: vec![graph_repr(p)],
                extra_info: vec![],
                // TODO show mask in extras
            }),

        Instance::UndominatedChoice(ref p) =>
            Ok(Response{
                graphs: vec![graph_repr(p)],
                extra_info: vec![],
            }),

        Instance::PartiallyDominantChoice{ref p, fc:_} =>
            Ok(Response{
                graphs: vec![graph_repr(p)],
                extra_info: vec![],
                // TODO show FC in extras
            }),

        Instance::Swaps(ref p) =>
            Ok(Response{
                graphs: vec![graph_repr(p)],
                extra_info: vec![],
            }),

        Instance::StatusQuoUndominatedChoice(ref p) =>
            Ok(Response{
                graphs: vec![graph_repr(p)],
                extra_info: vec![],
            }),

        Instance::Overload{ref p, limit} =>
            Ok(Response{
                graphs: vec![graph_repr(p)],
                extra_info: vec![
                    ("Threshold".to_string(), limit.to_string()),
                ],
            }),

        Instance::TopTwo(ref p) =>
            Ok(Response{
                graphs: vec![graph_repr(p)],
                extra_info: vec![],
            }),

        Instance::SequentiallyRationalizableChoice(ref p, ref q) =>
            Ok(Response{
                graphs: vec![graph_repr(p), graph_repr(q)],
                extra_info: vec![],
            }),
    }
}
