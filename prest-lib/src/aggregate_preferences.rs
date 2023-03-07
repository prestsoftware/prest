use std::fmt;
use std::ops::{Add,Mul,AddAssign};
use std::io::{Read,Write};
use std::iter::{Sum,FromIterator};
use std::collections::HashSet;
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
pub enum Mode {
    Weighted,
    Iterated,
}

impl codec::Decode for Mode {
    fn decode<R : Read>(f : &mut R) -> codec::Result<Mode> {
        let tag : String = codec::Decode::decode(f)?;
        match tag.as_str() {
            "weighted" => Ok(Mode::Weighted),
            "iterated" => Ok(Mode::Iterated),
            _ => Err(codec::Error::BadEnumTag),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    mode : Mode,
    subjects : Vec<Packed<estimation::Response>>,
}

impl codec::Decode for Request {
    fn decode<R : Read>(f : &mut R) -> codec::Result<Request> {
        Ok(Request {
            mode: codec::Decode::decode(f)?,
            subjects: codec::Decode::decode(f)?,
        })
    }
}

pub struct Response {
    best_instances : Vec<Packed<model::Instance>>,
}

impl codec::Encode for Response {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        (&self.best_instances).encode(f)
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

            Precomputation(e) => e.fmt(f),
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
fn aggregate<T>(ps : &[(Score, Preorder)]) -> Result<T, Error>
    where T : FromIterator<Preorder>
{
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

    Ok(best_preorders.into_iter().cloned().collect())
}

fn extract_preorder(instance : model::Instance) -> Result<Preorder, Error> {
    match instance {
        model::Instance::PreorderMaximization(p)
            if p.is_total()
                => Ok(p),

        _ => Err(Error::NotUtilityMaximization),
    }
}

fn combos<'a, T>(xs : &'a Vec<Vec<T>>) -> impl Iterator<Item=Vec<&'a T>> {
    let mut state = None;

    std::iter::from_fn(move || match state {
        // first iteration
        None => {
            state = Some(vec![0; xs.len()]);
            Some(xs.iter().map(|x| &x[0]).collect())
        },

        Some(ref mut indexes) => {
            // advance the indexes
            let mut i = 0;
            loop {
                if i >= indexes.len() {
                    // we're done
                    return None;
                }

                indexes[i] += 1;
                if indexes[i] >= xs[i].len() {
                    // carry
                    indexes[i] = 0;
                    i += 1;
                    continue;
                } else {
                    // no need to carry anymore,
                    // we're done
                    break;
                }
            }

            // return the result
            Some(indexes.iter().zip(xs).map(
                |(&j, x)| &x[j]
            ).collect())
        },
    })
}

pub fn run(req : Request) -> Result<Response, Error> {
    // collect preorders from all subjects
    let result : Vec<Preorder> = match &req.mode {
        Mode::Weighted => {
            let subjects : Vec<(Score, Preorder)> = req.subjects.into_iter().flat_map(
                |Packed(subj)| {
                    let score = Score::new(1, subj.best_instances.len() as u32);
                    subj.best_instances.into_iter().map(
                        move |info| Ok((score, extract_preorder(info.instance.into_unpacked())?))
                    )
                }
            ).collect::<Result<_, Error>>()?;

            aggregate(&subjects)?
        }

        Mode::Iterated => {
            // collect best instances for each subject
            let subjects : Vec<Vec<Preorder>> = req.subjects.into_iter().map(
                |Packed(subj)|
                    subj.best_instances.into_iter().map(
                        |info| extract_preorder(
                            info.instance.into_unpacked()
                        )
                    ).collect()
            ).collect::<Result<_, Error>>()?;

            // aggregate each chain
            let mut chain_orders : HashSet<Preorder> = HashSet::new();
            combos(&subjects).try_for_each(
                |chain| Ok(chain_orders.extend(
                    aggregate::<Vec<Preorder>>(&{
                        let x : Vec<(Score, Preorder)> = chain.into_iter().map(
                            |p| (Score::from(1), p.clone())
                        ).collect::<Vec<(Score, Preorder)>>();
                        x
                    })?
                )) as Result<(), Error>
            )?;

            // finally, aggregate across all chains
            aggregate(
                &std::iter::repeat(Score::from(1))
                    .zip(chain_orders)
                    .collect::<Vec<_>>()
            )?
        }
    };

    Ok(Response{
        best_instances: result.into_iter().map(
            |p| Packed(
                model::Instance::PreorderMaximization(p)
            )
        ).collect()
    })
}
