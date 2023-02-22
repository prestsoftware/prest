use std::io::{Read,Write};
use codec::{self,Packed};
use model;
use estimation;
use preorder::Preorder;

#[derive(Debug, Clone)]
pub struct Request {
    subjects : Vec<Packed<estimation::Response>>,
}

impl codec::Decode for Request {
    fn decode<R : Read>(f : &mut R) -> codec::Result<Request> {
        Ok(Request {
            subjects: codec::Decode::decode(f)?,
        })
    }
}

pub struct Response {
    instance : Packed<model::Instance>,
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

fn aggregate(ps : &[Preorder]) -> Result<Preorder, Error> {
    unimplemented!()
}

fn extract_preorder(instance : model::Instance) -> Result<Preorder, Error> {
    unimplemented!()
}

pub fn run(req : Request) -> Result<Response, Error> {
    let p = aggregate(
        &req.subjects.into_iter().map(
            |resp| aggregate(
                &resp.into_unpacked().best_instances.into_iter().map(
                    |info| extract_preorder(
                        info.instance.into_unpacked()
                    )
                ).collect::<Result<Vec<_>, Error>>()?[..]
            )
        ).collect::<Result<Vec<_>, Error>>()?[..]
    )?;

    Ok(Response{
        instance: Packed(
            model::Instance::PreorderMaximization(p)
        ),
    })
}
