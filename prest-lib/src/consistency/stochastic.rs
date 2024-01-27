use std::result;
use std::fmt;
use std::collections::HashMap;
use std::io::{Read,Write};
use num_rational::Ratio;
use std::iter::FromIterator;

use alt::Alt;
use alt_set::AltSet;
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

struct MenuStatsRaw {
    alt_counts : Vec<u32>,
    total : u32,
}

type MenuStats = Vec<Ratio<u32>>;

impl MenuStatsRaw {
    fn finalise(self) -> MenuStats {
        self.alt_counts.into_iter().map(
            |num| Ratio::new(num, self.total)
        ).collect()
    }
}

fn transitivity(alt_count : u32, choice_rows : &[ChoiceRow]) -> Transitivity {
    let mut weak = 0;
    let mut moderate = 0;
    let mut strong = 0;
    const HALF : Ratio<u32> = Ratio::new_raw(1, 2);

    let freq : HashMap<&AltSet, MenuStats> = {
        let mut freq  : HashMap<&AltSet, MenuStatsRaw> = HashMap::new();
        for cr in choice_rows {
            if let Some(choice) = cr.choice.view().as_singleton() {
                let stats = freq.entry(&cr.menu).or_insert_with(|| MenuStatsRaw {
                    alt_counts: vec![0; alt_count as usize],
                    total: 0,
                });

                stats.alt_counts[choice.index() as usize] += 1;
                stats.total += 1;
            }
        }

        freq.into_iter().map(
            |(menu, stats)| (menu, stats.finalise())
        ).collect()
    };

    for a in Alt::all(alt_count) {
        for b in Alt::all(alt_count) {
            if let Some(pab) = freq.get(&AltSet::from_iter([a, b])) {
                let pa_ab = pab[a.index() as usize];
                for c in Alt::all(alt_count) {
                    if let Some(pac) = freq.get(&AltSet::from_iter([a, c])) {
                        let pa_ac = pac[a.index() as usize];
                        if let Some(pbc) = freq.get(&AltSet::from_iter([b, c])) {
                            let pb_bc = pbc[b.index() as usize];
                            if pa_ab >= HALF && pb_bc >= HALF {
                                weak += (pa_ac < HALF) as u32;
                                moderate += (pa_ac < pa_ab && pa_ac < pb_bc) as u32;
                                strong += (pa_ac < pa_ab || pa_ac < pb_bc) as u32;
                            }
                        }
                    }
                }
            }
        }
    }

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
