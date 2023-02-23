use std::fmt;
use std::ops::{Add,Mul,AddAssign};
use std::io::{Read,Write};
use std::iter::Sum;
use num_rational::Ratio;
use num_traits::identities::{Zero,One};

use model;
use estimation;
use winners;
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

type Score = Ratio<u32>;

struct KemenyScore {
    lt : Score,
    eq : Score,
    gt : Score,
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
    type Output = Score;
    fn mul(self, other : &KemenyScore) -> Score {
        self.lt*other.lt
        + self.eq*other.eq
        + self.gt*other.gt
    }
}

impl Mul<Score> for &KemenyScore {
    type Output = KemenyScore;
    fn mul(self, other : Score) -> KemenyScore {
        KemenyScore {
            lt : self.lt * other,
            eq : self.eq * other,
            gt : self.gt * other,
        }
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
                    lt : if p.lt(u, v) { One::one() } else { Zero::zero() },
                    eq : if p.eq(u, v) { One::one() } else { Zero::zero() },
                    gt : if p.gt(u, v) { One::one() } else { Zero::zero() },
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
    type Output = Score;
    fn mul(self, other : &KemenyTable) -> Score {
        assert_eq!(self.scores.len(), other.scores.len());
        self.scores.iter().zip(&other.scores).map(
            |(x, y)| x * y
        ).sum()
    }
}

impl Mul<Score> for &KemenyTable {
    type Output = KemenyTable;
    fn mul(self, other : Score) -> KemenyTable {
        KemenyTable {
            scores: self.scores.iter().map(|ks| ks * other).collect()
        }
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

// using the Kemeny method
fn aggregate(ps : &[(Score, Preorder)]) -> Result<Preorder, Error> {
    assert!(!ps.is_empty(), "cannot aggregate an empty set of preferences");
    let alt_count = ps[0].1.size;

    let precomputed = Precomputed::precomputed(alt_count, None)?;
    let tbl_aggregated : KemenyTable = ps.iter().map(
        |(weight, p)|
            &KemenyTable::from_preorder(p) * *weight
    ).sum();

    // we assert non-emptiness at the beginning of the function
    let (_best_score, best_preorders) = winners::run_iter_with_score(
        &precomputed.get(alt_count)?.weak_orders,
        |p| &tbl_aggregated * &KemenyTable::from_preorder(p),
    ).unwrap();

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
    let preorders : Vec<(Score, Preorder)> = req.subjects.into_iter().flat_map(
        |Packed(subj)| {
            let score = Score::new(1, subj.best_instances.len() as u32);
            subj.best_instances.into_iter().map(
                move |info| Ok((score, extract_preorder(info.instance.into_unpacked())?))
            )
        }
    ).collect::<Result<Vec<(Score, Preorder)>, Error>>()?;

    Ok(Response{
        instance: Packed(
            model::Instance::PreorderMaximization(
                aggregate(&preorders)?
            )
        ),
    })
}
