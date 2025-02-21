use std::ops::{BitAndAssign,BitOrAssign,SubAssign};
use std::mem;
use std::io::{Read,Write};
use std::iter::FromIterator;
use crate::codec::{self,Encode,Decode};
use crate::alt::Alt;
use std::fmt;
use rand::Rng;
use std::string::String;
use itertools::Itertools;

pub type Block = u32;

// WARNING: AltSet::blocks must not contain trailing zeroes
// as that will affect the equality comparison. Use AltSet::normalise()
// after every mutable change.
//
// Allowing clippy::derive_hash_xor_eq is fine because the behaviour of PartialEq
// is exactly the same as if it were derived; it just adds assertions.
#[allow(clippy::derive_hash_xor_eq)]
#[derive(PartialOrd,Ord,Clone,Hash,Debug)]
pub struct AltSet {
    blocks : Vec<Block>,  // little endian
}

impl PartialEq for AltSet {
    fn eq(&self, other : &AltSet) -> bool {
        assert!(self.is_normalised());
        assert!(other.is_normalised());

        self.blocks == other.blocks
    }
}

impl Eq for AltSet { }

impl Encode for AltSet {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        self.view().encode(f)
    }
}

impl Decode for AltSet {
    fn decode<R : Read>(f : &mut R) -> codec::Result<AltSet> {
        Ok(AltSet::from_iter::<Vec<Alt>>(Decode::decode(f)?))
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct AltSetView<'a> {
    pub blocks : &'a [Block],
}

pub struct Iter<'a> {
    head : u32,
    tail : &'a [Block],
    offset: u32,
}

impl<'a> Encode for AltSetView<'a> {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        self.into_iter().collect::<Vec<_>>().encode(f)
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = Alt;

    fn next(&mut self) -> Option<Alt> {
        // note that this handles empty sets correctly
        loop {
            if self.head == 0 {
                if self.tail.is_empty() {
                    // no more blocks
                    return None;
                }

                // bump offset to the next multiple of block size
                let block_size = 8 * mem::size_of::<Block>() as u32;  // in bits
                if self.offset % block_size != 0 {
                    self.offset /= block_size;
                    self.offset += 1;
                    self.offset *= block_size;
                } else {
                    // either the last bit was set (in which case offset is already correct)
                    // or no bits were set (and the offset went through the special case below)
                }

                self.head = self.tail[0];
                self.tail = &self.tail[1..];

                if self.head == 0 {
                    // the bits won't advance so we have to do that manually
                    self.offset += block_size;
                }
            } else {
                let trail = self.head.trailing_zeros();
                let result = trail + self.offset;
                self.head >>= trail+1;
                self.offset += trail+1;
                return Some(Alt(result));
            }
        }
    }
}

impl<'a> AltSetView<'a> {
    pub fn to_blocks(&self) -> &[Block] {
        &self.blocks[..]
    }

    pub fn contains(&self, Alt(i) : Alt) -> bool {
        let block_size = 8 * mem::size_of::<Block>();  // in bits
        let offset = i as usize / block_size;

        if offset >= self.blocks.len() {
            false
        } else {
            (self.blocks[offset] >> (i as usize % block_size)) & 0x1 != 0
        }
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.iter().all(|&b| b == 0)
    }

    pub fn is_nonempty(&self) -> bool {
        self.blocks.iter().any(|&b| b != 0)
    }

    pub fn is_subseteq_of(&self, other : AltSetView) -> bool {
        self.blocks.iter().zip(other.blocks).all(
            |(&mine, &their)| (mine & !their) == 0
        )
    }

    pub fn is_strict_subset_of(&self, other : AltSetView) -> bool {
        self.is_subseteq_of(other) && !other.is_subseteq_of(*self)
    }

    pub fn is_strict_superset_of(&self, other : AltSetView) -> bool {
        other.is_strict_subset_of(*self)
    }

    pub fn size(&self) -> u32 {
        self.blocks.iter().map(|b| b.count_ones()).sum()
    }

    pub fn combinations(&self, k : u32) -> impl Iterator<Item=AltSet> + 'a + use<'a> {
        self.iter().combinations(k as usize).map(
            FromIterator::from_iter
        )
    }

    pub fn is_singleton(&self) -> bool {
        self.size() == 1
    }

    pub fn as_singleton(&self) -> Option<Alt> {
        if self.size() == 1 {
            self.iter().last()
        } else {
            None
        }
    }

    pub fn iter(&self) -> Iter<'a> {
        Iter {
            head: 0,
            tail: self.blocks,
            offset: 0,
        }
    }

    pub fn to_string(&self, alternatives : &[&str]) -> String {
        let mut s = String::new();

        for (i, alt) in self.iter().enumerate() {
            if i > 0 {
                fmt::Write::write_char(&mut s, ',').unwrap();
            }

            fmt::Write::write_str(&mut s, alternatives[alt.index() as usize]).unwrap();
        }

        s
    }
}

impl<'a> fmt::Display for AltSetView<'a> {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{")?;
        let mut first = true;
        for i in self.iter() {
            if first {
                write!(f, "{}", i)?;
                first = false;
            } else {
                write!(f, ",{}", i)?;
            }
        }
        write!(f, "}}")
    }
}

impl<'a> IntoIterator for AltSetView<'a> {
    type Item = Alt;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Iter<'a> {
        Iter {
            head: 0,
            tail: self.blocks,
            offset: 0,
        }
    }
}

impl<'a> From<AltSetView<'a>> for AltSet {
    fn from(v : AltSetView) -> AltSet {
        AltSet {
            blocks: Vec::from(v.blocks),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Powerset{
    next_val: Option<u32>,
    last_val: u32,
}

impl Iterator for Powerset {
    type Item = AltSet;

    fn next(&mut self) -> Option<AltSet> {
        match self.next_val {
            None => None,
            Some(next) => {
                // advance
                if next == self.last_val {
                    self.next_val = None;
                } else {
                    self.next_val = Some(next + 1);
                }

                // return result
                Some(AltSet{ blocks: vec![next] })
            }
        }
    }
}

impl AltSet {
    pub fn empty() -> AltSet {
        AltSet {
            blocks: Vec::new(),
        }
    }

    pub fn full(alt_count : u32) -> AltSet {
        let block_size : u32 = Block::BITS;
        let nblocks = (alt_count + block_size - 1) / block_size;  // round up

        // generate random blocks
        let mut blocks : Vec<_> = (0..nblocks).map(|_| Block::MAX).collect();

        // zero out the appropriate number of bits in the last block
        let last_block_bits = alt_count % block_size;
        blocks[nblocks as usize - 1] &= (1 << last_block_bits) - 1;

        AltSet{ blocks }
    }

    /// will never generate an empty set
    pub fn rand_nonempty<R : Rng>(rng : &mut R, alt_count : u32) -> AltSet {
        loop {
            let result = AltSet::rand_possibly_empty(rng, alt_count);
            if result.view().is_nonempty() {
                return result;
            }

            // otherwise try again
        }
    }

    /// might generate the empty set
    pub fn rand_possibly_empty<R : Rng>(rng : &mut R, alt_count : u32) -> AltSet {
        let block_size = 8 * mem::size_of::<Block>() as u32;  // in bits
        let nblocks = (alt_count + block_size - 1) / block_size;  // round up

        // generate random blocks
        let mut blocks : Vec<_> = (0..nblocks).map(|_| rng.random()).collect();

        // zero out the appropriate number of bits in the last block
        let last_block_bits = alt_count % block_size;
        blocks[nblocks as usize - 1] &= (1 << last_block_bits) - 1;

        AltSet{ blocks }
    }

    /// Does not include the empty set.
    pub fn powerset(n : u32) -> Powerset {
        assert!(n < 32);
        Powerset{ next_val: Some(1), last_val: (1 << n) - 1 }
    }

    pub fn combinations(n : u32, k : u32) -> impl Iterator<Item=AltSet> {
        Alt::all(n).combinations(k as usize).map(
            FromIterator::from_iter
        )
    }

    pub fn singleton(x : Alt) -> AltSet {
        AltSet::from_iter(&[x])
    }

    pub fn from_block(block : Block) -> AltSet {
        AltSet {
            blocks: vec![block]
        }
    }

    pub fn from_blocks(blocks : Vec<Block>) -> AltSet {
        AltSet{ blocks }
    }

    pub fn view(&self) -> AltSetView {
        AltSetView{
            blocks: &self.blocks
        }
    }

    pub fn size(&self) -> u32 {
        self.view().size()
    }

    fn is_normalised(&self) -> bool {
        if let Some(&0) = self.blocks.last() {
            false
        } else {
            true
        }
    }

    // remove trailing zeroes that can affect equality comparison
    // must be called after all shrinking operations
    fn normalise(&mut self) {
        while !self.is_normalised() {
            self.blocks.pop();
        }
    }
}

impl fmt::Display for AltSet {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        self.view().fmt(f)
    }
}

impl<'a> PartialEq<AltSetView<'a>> for AltSet {
    fn eq(&self, other : &AltSetView<'a>) -> bool {
        *self == AltSet::from_iter(*other)
    }
}

impl<'a> PartialEq<AltSet> for AltSetView<'a> {
    fn eq(&self, other : &AltSet) -> bool {
        *other == *self
    }
}

impl<'a> BitOrAssign<AltSetView<'a>> for AltSet {
    fn bitor_assign(&mut self, rhs : AltSetView) {
        for (block_lhs, block_rhs) in self.blocks.iter_mut().zip(rhs.blocks.iter()) {
            *block_lhs |= *block_rhs;
        }

        // if rhs is longer, we need to add those blocks
        if rhs.blocks.len() > self.blocks.len() {
            let ext = &rhs.blocks[self.blocks.len()..];
            self.blocks.extend(ext);
        }

        self.normalise();
    }
}

impl<'a> BitOrAssign<&'a AltSet> for AltSet {
    fn bitor_assign(&mut self, rhs : &AltSet) {
        *self |= rhs.view()
    }
}

impl<'a> BitAndAssign<AltSetView<'a>> for AltSet {
    fn bitand_assign(&mut self, rhs : AltSetView) {
        self.blocks.truncate(rhs.blocks.len());  // no effect if rhs is longer
        for (block_lhs, block_rhs) in self.blocks.iter_mut().zip(rhs.blocks.iter()) {
            *block_lhs &= *block_rhs;
        }
        self.normalise();
    }
}

impl<'a> BitAndAssign<&'a AltSet> for AltSet {
    fn bitand_assign(&mut self, rhs : &AltSet) {
        *self &= rhs.view()
    }
}

impl<'a> SubAssign<AltSetView<'a>> for AltSet {
    fn sub_assign(&mut self, rhs : AltSetView) {
        for (block_lhs, block_rhs) in self.blocks.iter_mut().zip(rhs.blocks.iter()) {
            *block_lhs &= !*block_rhs;
        }
        self.normalise();
    }
}

impl<'a> SubAssign<&'a AltSet> for AltSet {
    fn sub_assign(&mut self, rhs : &AltSet) {
        *self -= rhs.view()
    }
}

impl<'a> FromIterator<&'a Alt> for AltSet {
    fn from_iter<T : IntoIterator<Item=&'a Alt>>(iter : T) -> AltSet {
        AltSet::from_iter(iter.into_iter().cloned())
    }
}

impl FromIterator<Alt> for AltSet {
    fn from_iter<T : IntoIterator<Item=Alt>>(iter : T) -> AltSet {
        let block_size = 8 * mem::size_of::<Block>();  // in bits

        let mut blocks = Vec::new();
        let mut max_i = None;

        for Alt(i) in iter {
            match max_i {
                Some(j) if (j >= i) => (),

                _ => {
                    max_i = Some(i);
                    // we need max_i+1 bits
                    // and then we round that number up to whole blocks
                    let blocks_needed = ((i as usize + 1) + (block_size - 1)) / block_size;
                    blocks.resize(blocks_needed, 0);
                }
            }

            blocks[i as usize / block_size] |= 1 << (i as usize % block_size);
        }

        AltSet{ blocks }
    }
}

#[macro_export]
macro_rules! alts {
    ($($arg:expr_2021),*) => {
        AltSet::from_iter(&vec![$(Alt($arg)),*])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_sanity() {
        let aas = alts![2];
        println!("{:?}", aas);
        assert_eq!(aas, alts![2]);
    }
}
