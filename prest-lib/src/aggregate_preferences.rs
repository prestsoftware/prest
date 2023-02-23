use std::fmt;
use std::ops::{Add,Mul,AddAssign};
use std::io::{Read,Write};
use std::iter::Sum;

use model;
use estimation;
use alt::Alt;
use preorder::Preorder;
use codec::{self,Packed};
use precomputed::{self,Precomputed};

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
    NotUtilityMaximization,
    Ambiguous,
    Precomputation(precomputed::Error),
}

impl codec::Encode for Error {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        use self::Error::*;
        match self {
            NotUtilityMaximization => (0e8).encode(f),

            // we'll probably have to deal with this at some point
            Ambiguous => (1e8).encode(f),

            Precomputation(e) => (2e8, e).encode(f),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;
        match self {
            NotUtilityMaximization =>
                write!(f, "only Utility Maximization is supported"),

            Ambiguous =>
                write!(f, "Kemeny algorithm produces multiple results"),

            Precomputation(e) =>
                e.fmt(f),
        }
    }
}

impl From<precomputed::Error> for Error {
    fn from(e : precomputed::Error) -> Error {
        Error::Precomputation(e)
    }
}

struct KemenyScore {
    lt : u32,
    eq : u32,
    gt : u32,
}

impl Add<&KemenyScore> for &KemenyScore {
    type Output = KemenyScore;
    fn add(self, other : &KemenyScore) -> KemenyScore {
        KemenyScore {
            lt: self.lt + other.lt,
            eq: self.eq + other.eq,
            gt: self.gt + other.gt,
        }
    }
}

impl AddAssign<&KemenyScore> for KemenyScore {
    fn add_assign(&mut self, other : &KemenyScore) {
        self.lt += other.lt;
        self.eq += other.eq;
        self.gt += other.gt;
    }
}

impl Mul<&KemenyScore> for &KemenyScore {
    type Output = u32;
    fn mul(self, other : &KemenyScore) -> u32 {
        self.lt*other.lt
        + self.eq*other.eq
        + self.gt*other.gt
    }
}

struct KemenyTable {
    scores : Vec<KemenyScore>
}

impl KemenyTable {
    fn from_preorder(p : &Preorder) -> KemenyTable {
        KemenyTable {
            scores: Alt::distinct_pairs(p.size).map(
                |(u, v)| KemenyScore {
                    lt : p.lt(u, v) as u32,
                    eq : p.eq(u, v) as u32,
                    gt : p.gt(u, v) as u32,
                }
            ).collect()
        }
    }
}

impl Add<&KemenyTable> for &KemenyTable {
    type Output = KemenyTable;
    fn add(self, other : &KemenyTable) -> KemenyTable {
        KemenyTable {
            scores: self.scores.iter().zip(&other.scores).map(
                |(x, y)| x+y
            ).collect()
        }
    }
}

impl AddAssign<&KemenyTable> for KemenyTable {
    fn add_assign(&mut self, other : &KemenyTable) {
        assert_eq!(self.scores.len(), other.scores.len());
        self.scores.iter_mut().zip(&other.scores).for_each(
            |(x, y)| *x += y
        )
    }
}

impl Mul<&KemenyTable> for &KemenyTable {
    type Output = u32;
    fn mul(self, other : &KemenyTable) -> u32 {
        assert_eq!(self.scores.len(), other.scores.len());
        self.scores.iter().zip(&other.scores).map(
            |(x, y)| x * y
        ).sum()
    }
}

impl Sum for KemenyTable {
    fn sum<I : Iterator<Item=KemenyTable>>(it : I) -> KemenyTable {
        let mut result = None;
        for kt in it {
            match result {
                None => {
                    result = Some(kt);
                },

                Some(ref mut result) => {
                    *result += &kt;
                },
            }
        }

        result.expect("KemenyTable: empty sum")
    }
}

struct Winners<S, T> {
    best_score : Option<S>,
    winners : Vec<T>,
}

impl<S : Ord, T> Winners<S, T> {
    pub fn new() -> Winners<S, T> {
        Winners {
            best_score : None,
            winners : Vec::new(),
        }
    }

    pub fn add(&mut self, score : S, candidate : T) {
        match self.best_score {
            None => {
                self.best_score = Some(score);
                self.winners = vec![candidate];
            },

            Some(ref mut best_score) => {
                if score > *best_score {
                    *best_score = score;
                    self.winners = vec![candidate];
                } else if score == *best_score {
                    self.winners.push(candidate);
                }
            },
        }
    }

    pub fn into_result(self) -> Option<(S, Vec<T>)> {
        Some((self.best_score?, self.winners))
    }
}

// using the Kemeny method
fn aggregate(ps : &[Preorder]) -> Result<Preorder, Error> {
    assert!(!ps.is_empty(), "cannot aggregate an empty set of preferences");
    let alt_count = ps[0].size;

    let precomputed = Precomputed::precomputed(alt_count, None)?;
    let tbl_aggregated : KemenyTable = ps.iter().map(KemenyTable::from_preorder).sum();

    let mut winners = Winners::new();
    for p in &precomputed.get(alt_count)?.weak_orders {
        winners.add(
            &tbl_aggregated * &KemenyTable::from_preorder(p),
            p,
        )
    }

    // we assert non-emptiness at the beginning of the function
    let (_best_score, best_preorders) = winners.into_result().unwrap();

    if best_preorders.len() > 1 {
        Err(Error::Ambiguous)
    } else {
        // we check non-emptiness at the beginning of the function
        assert_eq!(best_preorders.len(), 1);
        Ok(best_preorders[0].clone())
    }
}

fn extract_preorder(instance : model::Instance) -> Result<Preorder, Error> {
    match instance {
        model::Instance::PreorderMaximization(p)
            if p.is_total()
                => Ok(p),

        _ => Err(Error::NotUtilityMaximization),
    }
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
