use std::result;

use std::fmt;
use std::io::{Read,Write};
use alt_set::{AltSet};
use codec::{self,Packed,Encode,Decode};
use rpc_common::{Subject};

#[derive(Debug)]
pub enum Request {
    GeneralIntegrity{ subjects: Vec<Packed<Subject>> },
}

impl Decode for Request {
    fn decode<R : Read>(f : &mut R) -> codec::Result<Request> {
        use self::Request::*;
        Ok(match Decode::decode(f)? {
            0u8 => GeneralIntegrity{ subjects: Decode::decode(f)? },
            _ => panic!("wrong request tag"),
        })
    }
}

enum IssueDescription {
    RepeatedMenu(AltSet),
}

impl Encode for IssueDescription {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        use self::IssueDescription::*;
        match self {
            &RepeatedMenu(ref menu) => {
                (0u8, menu).encode(f)
            }
        }
    }
}

pub struct Issue {
    subject_name : String,
    description : IssueDescription,
}

impl Encode for Issue {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        (&self.subject_name, &self.description).encode(f)
    }
}

pub struct Response {
    issues : Vec<Issue>,
}

impl Encode for Response {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        self.issues.encode(f)
    }
}

pub enum IntegrityError {
}

impl Encode for IntegrityError {
    fn encode<W : Write>(&self, _f : &mut W) -> codec::Result<()> {
        match *self { }
    }
}

impl fmt::Display for IntegrityError {
    fn fmt(&self, _f : &mut fmt::Formatter) -> fmt::Result {
        match *self { }
    }
}

pub type Result<T> = result::Result<T, IntegrityError>;

pub fn run(req : Request) -> Result<Response> {
    use self::Request::*;
    match req {
        GeneralIntegrity{subjects} => {
            let mut issues = Vec::new();

            for Packed(subject) in subjects {
            }

            Ok(Response{issues})
        }
    }
}
