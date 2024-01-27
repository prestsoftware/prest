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
        let total = self.total;
        self.alt_counts.into_iter().map(
            |num| Ratio::new(num, total)
        ).collect()
    }
}

type Frequencies<'a> = HashMap<&'a AltSet, MenuStats>;

fn frequencies<'a>(alt_count : u32, choice_rows : &'a [ChoiceRow]) -> Frequencies<'a> {
    let mut freq  : HashMap<&AltSet, MenuStatsRaw> = HashMap::new();
    for cr in choice_rows {
        let stats = freq.entry(&cr.menu).or_insert_with(|| MenuStatsRaw {
            alt_counts: vec![0; alt_count as usize],
            total: 0,
        });

        for choice in cr.choice.view() {
            stats.alt_counts[choice.index() as usize] += 1;
        }
        stats.total += 1;
    }

    freq.into_iter().map(
        |(menu, stats)| (menu, stats.finalise())
    ).collect()
}

fn transitivity(alt_count : u32, freq : &Frequencies) -> Transitivity {
    let mut result = Transitivity {
        weak: 0, moderate: 0, strong: 0,
    };
    const HALF : Ratio<u32> = Ratio::new_raw(1, 2);

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
                                result.weak += (pa_ac < HALF) as u32;
                                result.moderate += (pa_ac < pa_ab && pa_ac < pb_bc) as u32;
                                result.strong += (pa_ac < pa_ab || pa_ac < pb_bc) as u32;
                            }
                        }
                    }
                }
            }
        }
    }

    result
}

struct Regularity {
    weak : u32,
    strong : u32,
}

fn regularity(freq : &Frequencies) -> Regularity {
    let mut result = Regularity {
        weak: 0,
        strong: 0,
    };

    for (menu_b, p_b) in freq.iter() {
        for (menu_a, p_a) in freq.iter() {
            if !menu_a.view().is_strict_subset_of(menu_b.view()) {
                // only consider A < B
                continue;
            }

            for a in menu_a.view() {
                let pa_a = p_a[a.index() as usize];
                let pa_b = p_b[a.index() as usize];

                result.weak += (pa_a < pa_b) as u32;
                result.strong += (pa_a <= pa_b) as u32;
            }
        }
    }

    result
}

fn analyse(alt_count : u32, choices : &[ChoiceRow]) -> (Transitivity, Regularity) {
    let frequencies = frequencies(alt_count, &choices);
    let transitivity = transitivity(alt_count, &frequencies);
    let regularity = regularity(&frequencies);

    (transitivity, regularity)
}

pub fn run(request : &Request) -> Result<Response> {
    let subject = request.subject.unpack();
    let alt_count = subject.alternatives.len() as u32;

    let (transitivity, regularity) = analyse(alt_count, &subject.choices);

    Ok(Response {
        subject_name: subject.name.clone(),
        weak_stochastic_transitivity: transitivity.weak,
        moderate_stochastic_transitivity: transitivity.moderate,
        strong_stochastic_transitivity: transitivity.strong,
        weak_regularity: regularity.weak,
        strong_regularity: regularity.strong,
    })
}

#[cfg(test)]
mod test {
    use alt::Alt;
    use alt_set::AltSet;
    use common::ChoiceRow;
    use std::iter::FromIterator;

    #[test]
    fn transitivity_1() {
        let (t, r) = super::analyse(3, &choices![
            [0,1] -> [0],
            [0,2] -> [2],
            [1,2] -> [1]
        ]);

        assert_eq!(t.weak, 3);
        assert_eq!(t.moderate, 3);
        assert_eq!(t.strong, 3);

        assert_eq!(r.weak, 0);
        assert_eq!(r.strong, 0);
    }

    #[test]
    fn regularity_weak() {
        let (t, r) = super::analyse(3, &choices![
            [0,1] -> [0],
            [0,1,2] -> [1]
        ]);

        assert_eq!(t.weak, 0);
        assert_eq!(t.moderate, 0);
        assert_eq!(t.strong, 0);

        assert_eq!(r.weak, 1);
        assert_eq!(r.strong, 1);
    }

    #[test]
    fn regularity_strong() {
        let (t, r) = super::analyse(3, &choices![
            [0,1] -> [0],
            [0,1,2] -> [0]
        ]);

        assert_eq!(t.weak, 0);
        assert_eq!(t.moderate, 0);
        assert_eq!(t.strong, 0);

        assert_eq!(r.weak, 0);
        assert_eq!(r.strong, 2);
    }

    #[test]
    fn deferrals() {
        let (t, r) = super::analyse(3, &choices![
            [0,1] -> [],
            [0,1,2] -> [0]
        ]);

        assert_eq!(t.weak, 0);
        assert_eq!(t.moderate, 0);
        assert_eq!(t.strong, 0);

        assert_eq!(r.weak, 1);
        assert_eq!(r.strong, 2);
    }
}
