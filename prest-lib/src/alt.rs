use std::fmt;
use std::io::{Read,Write};
use std::iter::{Map,repeat};
use std::ops::Range;
use std::vec::IntoIter;
use crate::codec::{Encode,Decode,Result};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Alt(pub u32);

impl Alt {
    #[inline]
    pub fn index(self) -> u32 {
        self.0
    }

    pub fn all(alt_count : u32) -> Map<Range<u32>, fn(u32) -> Alt> {
        (0..alt_count).map(Alt)
    }

    pub fn all_above(Alt(i) : Alt, alt_count : u32) -> Map<Range<u32>, fn(u32) -> Alt> {
        (i+1 .. alt_count).map(Alt)
    }

    pub fn all_pairs(alt_count : u32) -> IntoIter<(Alt, Alt)> {
        Alt::all(alt_count).flat_map(
            |i| repeat(i).zip(Alt::all(alt_count))
        ).collect::<Vec<_>>().into_iter()
    }

    pub fn distinct_pairs(alt_count : u32) -> IntoIter<(Alt, Alt)> {
        Alt::all(alt_count).flat_map(
            |i| repeat(i).zip(Alt::all_above(i, alt_count))
        ).collect::<Vec<_>>().into_iter()
    }
}

impl fmt::Display for Alt {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        self.index().fmt(f)
    }
}

impl Encode for Alt {
    fn encode<W : Write>(&self, f : &mut W) -> Result<()> {
        self.index().encode(f)
    }
}

impl Decode for Alt {
    fn decode<R : Read>(f : &mut R) -> Result<Alt> {
        Decode::decode(f).map(Alt)
    }
}
