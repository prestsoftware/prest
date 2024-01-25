use std::result;
use std::fmt::{self, Display};
use std::collections::{BTreeMap,HashSet};
use std::collections::btree_map::Entry;
use std::io::{Read,Write};

use alt::Alt;
use common::{ChoiceRow,Subject};
use codec::{self,Encode,Decode,Packed};

#[derive(Debug)]
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
        match *self {
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Request {
    subject : Packed<Subject>,
}

impl Decode for Request {
    fn decode<R : Read>(f : &mut R) -> codec::Result<Request> {
        Ok(Request {
            subject: Decode::decode(f)?,
        })
    }
}

pub struct Response {
    subject_name : String,

    // tuples of observations that violate the given axiom
    weak_stochastic_transitivity : u32,
    moderate_stochastic_transitivity : u32,
    strong_stochastic_transitivity : u32,
    weak_regularity : u32,
    strong_regularity : u32,
}

impl Encode for Response {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        (
            &self.subject_name,
            &self.weak_stochastic_transitivity,
            &self.moderate_stochastic_transitivity,
            &self.strong_stochastic_transitivity,
            &self.weak_regularity,
            &self.strong_regularity,
        ).encode(f)
    }
}

struct Transitivity {
    weak : u32,
    moderate : u32,
    strong : u32,
}

fn transitivity(_alt_count : u32, _subject : &[ChoiceRow]) -> Transitivity {
    let mut weak = 0;
    let mut moderate = 0;
    let mut strong = 0;

    Transitivity {
        weak,
        moderate,
        strong,
    }
}

pub fn run(request : &Request) -> Result<Response> {
    let subject = request.subject.unpack();
    let alt_count = subject.alternatives.len() as u32;

    let transitivity = transitivity(alt_count, &subject.choices);

    Ok(Response {
        subject_name: subject.name.clone(),
        weak_stochastic_transitivity: transitivity.weak,
        moderate_stochastic_transitivity: transitivity.moderate,
        strong_stochastic_transitivity: transitivity.strong,
        weak_regularity: 0,
        strong_regularity: 0,
    })
}

#[cfg(test)]
mod test {
}
