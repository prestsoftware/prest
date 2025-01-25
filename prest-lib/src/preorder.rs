use crate::fast_preorder::FastPreorder;
use crate::alt_set::{Block,AltSetView,AltSet};
use crate::graph::Graph;
use crate::codec::{self,Encode,Decode};
use crate::alt::Alt;

use std::mem;
use std::iter;
use std::fmt;
use std::io::{Read,Write};
use std::collections::{HashMap,HashSet};
use base64::prelude::BASE64_STANDARD;
use base64::engine::Engine;

#[derive(PartialEq,Eq,PartialOrd,Ord,Clone,Hash,Debug)]
pub struct Preorder {
    blocks : Vec<Block>, // little endian
    pub size : u32,
}

impl Encode for Preorder {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        self.size.encode(f)?;

        const BLOCK_SIZE : usize = 8 * mem::size_of::<Block>();  // in bits
        let stride = (self.size as usize + BLOCK_SIZE - 1) / BLOCK_SIZE;  // round upwards
        assert_eq!(self.blocks.len(), self.size as usize * stride);

        for &block in &self.blocks {
            block.encode(f)?;
        }

        Ok(())
    }
}

impl Decode for Preorder {
    fn decode<R : Read>(f : &mut R) -> codec::Result<Preorder> {
        let size : u32 = Decode::decode(f)?;

        const BLOCK_SIZE : usize = 8 * mem::size_of::<Block>();  // in bits
        let stride = (size as usize + BLOCK_SIZE - 1) / BLOCK_SIZE;  // round upwards
        let block_count = size as usize * stride;

        let mut blocks = Vec::new();
        for _ in 0..block_count {
            blocks.push(Decode::decode(f)?);
        }

        Ok(Preorder{size, blocks})
    }
}

pub fn simplify_edges(edges : &[(Alt, Alt)]) -> Vec<(Alt, Alt)> {
    fn is_redundant(edges : &HashSet<(Alt, Alt)>, from : Alt, to : Alt) -> bool {
        let mut reach : HashSet<Alt> = HashSet::new();
        reach.insert(from);

        loop {
            let new_alts : HashSet<Alt> = edges.iter().filter_map(
                |&(u,v)| if reach.contains(&u) && !reach.contains(&v) && (u != from || v != to) {
                    Some(v)
                } else {
                    None
                }
            ).collect();

            if new_alts.is_empty() {
                return false;
            }

            if new_alts.contains(&to) {
                return true;
            }

            reach.extend(new_alts);
        }
    }

    let mut edges : HashSet<(Alt, Alt)> = edges.iter().cloned().collect();
    let mut is_changing = true;

    while is_changing {
        is_changing = false;

        for (i, j) in edges.clone() {
            if is_redundant(&edges, i, j) {
                edges.remove(&(i, j));
                is_changing = true;
            }
        }
    }

    edges.into_iter().collect()
}

impl Preorder {
    pub fn to_poset_graph(&self) -> Graph<AltSet> {
        // representative for every cluster
        let mut clusters : HashMap<Alt, AltSet> = HashMap::new();
        for alt in Alt::all(self.size) {
            let matches : Vec<&Alt> = clusters.keys().filter(
                |&&representative| self.eq(alt, representative)
            ).collect();

            match matches[..] {
                [] => {
                    clusters.insert(alt, AltSet::singleton(alt));
                }

                [&representative] => {
                    *clusters.get_mut(&representative).unwrap() |= &AltSet::singleton(alt);
                }

                _ => {
                    panic!("impossible: too many representatives");
                }
            }
        }

        let clusters : Vec<(Alt, AltSet)> = clusters.into_iter().collect();
        let representatives : AltSet = clusters.iter().map(|(k,_)| *k).collect();
        let relevant_edges : Vec<(Alt, Alt)>= self.edges().into_iter().filter(
            |&(i,j)| representatives.view().contains(i) && representatives.view().contains(j)
        ).collect();
        let indices : HashMap<Alt, usize> = clusters.iter().enumerate().map(|(i, (k,_))| (*k, i)).collect();
        Graph{
            vertices: clusters.iter().map(|(_, alts)| alts.clone()).collect(),
            edges: simplify_edges(&relevant_edges).into_iter().map(
                |(p,q)| (*indices.get(&p).unwrap(), *indices.get(&q).unwrap())
            ).collect()
        }
    }

    pub fn restrict(&mut self, alts : AltSetView) {
        const BLOCK_SIZE : u32 = 8 * mem::size_of::<Block>() as u32;  // in bits
        let stride = (self.size + BLOCK_SIZE - 1) / BLOCK_SIZE;  // round upwards
        for i in 0..self.size {
            let i_alt = Alt(i as u32);
            if alts.contains(i_alt) {
                for j in 0..stride {
                    self.blocks[(i*stride + j) as usize] &= alts.blocks[j as usize];
                }
            } else {
                for j in 0..stride {
                    self.blocks[(i*stride + j) as usize] = 0;
                    self.set_leq(i_alt, i_alt, true);
                }
            }
        }
    }

    pub fn edges(&self) -> Vec<(Alt, Alt)> {
        Alt::all(self.size).flat_map(
            |i| iter::repeat(i).zip(self.upset(i))
        ).filter(|&(i,j)| i != j).collect()
    }

    pub fn simple_digraph(&self) -> Vec<(Alt, Alt)> {
        simplify_edges(&self.edges())
    }

    pub fn diagonal(size : u32) -> Preorder {
        const BLOCK_SIZE : u32 = 8 * mem::size_of::<Block>() as u32;  // in bits
        let stride = (size + BLOCK_SIZE - 1) / BLOCK_SIZE;  // round upwards
        let nblocks = size * stride;

        let mut result = Preorder{ blocks: vec![0; nblocks as usize], size };
        for i in Alt::all(size) {
            result.set_leq(i, i, true);
        }

        result
    }

    pub fn from_fast_preorder(size : u32, p : FastPreorder) -> Preorder {
        let FastPreorder(mut matrix) = p;

        let mut blocks = Vec::with_capacity(size as usize);

        // FastPreorder is 8 bits per row
        for _row in 0..size {
            blocks.push((matrix & 0xFF) as Block);
            matrix >>= 8;
        }

        Preorder{blocks, size}
    }

    #[inline]
    pub fn set_leq(&mut self, Alt(i) : Alt, Alt(j) : Alt, leq : bool) {
        // i = row, j = column
        // assuming that BLOCK_SIZE is a power of two
        // and the compiler will optimise it to bit shifts
        const BLOCK_SIZE : u32 = 8 * mem::size_of::<Block>() as u32;  // in bits
        let stride = (self.size + BLOCK_SIZE - 1) / BLOCK_SIZE;  // round upwards
        let offset = j / BLOCK_SIZE;
        let bit_ofs = j % BLOCK_SIZE;

        if leq {
            self.blocks[(i * stride + offset) as usize] |= 1 << bit_ofs;
        } else {
            self.blocks[(i * stride + offset) as usize] &= !(1 << bit_ofs);
        }
    }

    #[inline]
    pub fn leq(&self, Alt(i) : Alt, Alt(j) : Alt) -> bool {
        // i = row, j = column
        // assuming that BLOCK_SIZE is a power of two
        // and the compiler will optimise it to bit shifts
        const BLOCK_SIZE : u32 = 8 * mem::size_of::<Block>() as u32;  // in bits
        let stride = (self.size + BLOCK_SIZE - 1) / BLOCK_SIZE;  // round upwards
        let offset = j / BLOCK_SIZE;
        let bit_ofs = j % BLOCK_SIZE;
        (self.blocks[(i * stride + offset) as usize] >> bit_ofs) & 0x1 != 0
    }

    #[inline]
    pub fn lt(&self, i : Alt, j : Alt) -> bool {
        self.leq(i,j) && !self.leq(j, i)
    }

    #[inline]
    pub fn eq(&self, i : Alt, j : Alt) -> bool {
        self.leq(i,j) && self.leq(j,i)
    }

    pub fn is_strict(&self) -> bool {
        // no equivalences exist
        !Alt::distinct_pairs(self.size).any(
            |(i,j)| self.leq(i, j) && self.leq(j, i)
        )
    }

    pub fn is_total(&self) -> bool {
        // all comparable
        Alt::distinct_pairs(self.size).all(
            |(i,j)| self.leq(i, j) || self.leq(j, i)
        )
    }

    pub fn is_transitive(&self) -> bool {
        Alt::all(self.size).all(
            |i| Alt::all(self.size).all(
                |j| Alt::all(self.size).all(
                    |k| !self.leq(i, j) || !self.leq(j, k) || self.leq(i, k)
                ) 
            )
        )
    }

    pub fn is_reflexive(&self) -> bool {
        Alt::all(self.size).all(|i| self.leq(i, i))
    }

    pub fn upset(&self, Alt(i) : Alt) -> AltSetView {
        const BLOCK_SIZE : u32 = 8 * mem::size_of::<Block>() as u32;  // in bits
        let stride = (self.size + BLOCK_SIZE - 1) / BLOCK_SIZE;  // in blocks, round upwards
        AltSetView {
            blocks: &self.blocks[(i*stride) as usize .. (i*stride + stride) as usize]
        }
    }

    // returns a descending order
    pub fn as_linear_order(&self) -> Option<Vec<Alt>> {
        // order[k] = alternative with k dominators
        let mut order = vec![None; self.size as usize];

        for i in Alt::all(self.size) {
            // exclude self from the dominators
            order[self.upset(i).size() as usize - 1] = Some(i);
        }

        // if the order is not reflexive, the code
        // above would attempt to write to the index -1
        // because the maximal element would have an empty upset
        // (so it won't produce a wrong answer silently)

        order.into_iter().collect()
    }

    // returns a descending order of equivalence classes
    pub fn as_weak_order(&self) -> Option<Vec<Vec<Alt>>> {
        if !self.is_total() {
            return None;
        }

        // alternatives in (non-strictly) descending order
        let alts = {
            let mut alts = Alt::all(self.size).map(
                |x| (x, self.upset(x).size())
            ).collect::<Vec<_>>();
            alts.sort_unstable_by_key(|&(_, upset_size)| upset_size);
            alts
        };

        // here we use the fact that once we know that the order is total
        // we can order vertices according to the upset size ascending
        // and then group them by the upset size to obtain the classes of equivalence

        let mut result = Vec::new();
        let mut eq_class = Vec::new();
        let mut prev_upset_size = None;
        for (x, upset_size) in alts.into_iter() {
            // (u, v) means v ≥ u
            match prev_upset_size {
                // first edge
                None => {
                    prev_upset_size = Some(upset_size);
                    eq_class = vec![x];
                }

                Some(ref mut prev_degree) => {
                    if *prev_degree == upset_size {
                        // adding to the existing equivalence class
                        eq_class.push(x);
                    } else {
                        // starting a new equivalence class
                        *prev_degree = upset_size;
                        result.push(eq_class);
                        eq_class = vec![x];
                    }
                }
            }
        }

        // push the last equivalence class
        if !eq_class.is_empty() {
            result.push(eq_class);
        }

        Some(result)
    }

    pub fn from_values<T: PartialOrd>(values : &[T]) -> Preorder {
        const BLOCK_SIZE : usize = 8 * mem::size_of::<Block>();  // in bits
        let stride = (values.len() + BLOCK_SIZE - 1) / BLOCK_SIZE;  // round upwards
        let mut blocks = vec![0; values.len() * stride];

        let mut ofs = 0;
        let mut bit = 0x1;

        for i in 0 .. values.len() {
            if bit != 0x1 {
                // the divide does not happen to fall exactly between blocks
                // close the current block and move on
                bit = 0x1;
                ofs += 1;
            }

            for j in 0 .. values.len() {
                if values[i] <= values[j] {
                    blocks[ofs] |= bit;
                }

                bit <<= 1;
                if bit == 0 {
                    // rolled all the way to the left
                    bit = 0x1;
                    ofs += 1;
                }
            }
        }

        Preorder {
            blocks,
            size: values.len() as u32,
        }
    }

    pub fn stuff(&self, target_size : u32, mask : Block) -> Preorder {
        const BLOCK_SIZE : u32 = 8 * mem::size_of::<Block>() as u32;  // in bits
        let stride_dst = (target_size + BLOCK_SIZE - 1) / BLOCK_SIZE;  // round upwards
        let stride_src = (self.size + BLOCK_SIZE - 1) / BLOCK_SIZE;  // round upwards
        let mut blocks = vec![0; (target_size * stride_dst) as usize];

        let mut row_mask = mask;
        let mut i_src = 0;
        for i_dst in 0..target_size {
            let ofs_dst = i_dst * stride_dst;

            if row_mask & 0x1 != 0 {
                let ofs_src = i_src * stride_src;

                let mut col_mask = mask;
                let mut j_src = 0;
                for j_dst in 0..target_size {
                    if col_mask & 0x1 != 0 {
                        blocks[(ofs_dst + j_dst/BLOCK_SIZE) as usize]
                            |= ((self.blocks[(ofs_src + j_src/BLOCK_SIZE) as usize]
                                    >> (j_src % BLOCK_SIZE))
                                  & 0x1)
                                << (j_dst % BLOCK_SIZE);

                        j_src += 1;
                    }

                    col_mask >>= 1;
                }

                i_src += 1
            } else {
                // unattractive option, fill the row with ones
                let block_count = (target_size / BLOCK_SIZE) as usize;
                for i in 0 .. block_count {
                    blocks[ofs_dst as usize + i] = Block::max_value();
                }

                if target_size % BLOCK_SIZE > 0 {
                    blocks[ofs_dst as usize + block_count] = (1 << (target_size % BLOCK_SIZE)) - 1;
                }
            }

            row_mask >>= 1;
        }

        Preorder{blocks, size: target_size}
    }

    pub fn to_base64(&self) -> String {
        BASE64_STANDARD.encode(
            codec::encode_to_memory(self).unwrap()
        )
    }

    pub fn from_base64(s : &str) -> codec::Result<Self> {
        codec::decode_from_memory(
            &BASE64_STANDARD.decode(s).map_err(
                |err| codec::Error::Other(format!("{:?}", err))
            )?
        )
    }

    pub fn pretty_fmt<W : fmt::Write>(&self, w : &mut W, alternatives : &[&str]) -> fmt::Result {
        if let Some(eq_classes) = self.as_weak_order() {
            let mut is_first_eq_class = true;
            for eq_class in eq_classes.into_iter() {
                if !is_first_eq_class {
                    write!(w, " > ")?;
                } else {
                    is_first_eq_class = false;
                }

                let mut is_first_alt = true;
                for Alt(i) in eq_class.into_iter() {
                    if !is_first_alt {
                        write!(w, " ~ ")?;
                    } else {
                        is_first_alt = false;
                    }

                    write!(w, "{}", alternatives[i as usize])?;
                }
            }
            Ok(())
        } else {
            let mut is_first_edge = true;
            for (u, v) in self.simple_digraph().into_iter() {
                if !is_first_edge {
                    write!(w, "; ")?;
                } else {
                    is_first_edge = false;
                }

                write!(
                    w, "{} ≥ {}",
                    alternatives[v.index() as usize],
                    alternatives[u.index() as usize],
                )?;
            }
            Ok(())
        }
    }

    pub fn pretty(&self, alternatives : &[&str]) -> String {
        let mut result = String::new();
        self.pretty_fmt(&mut result, alternatives).unwrap();
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::alt_set::AltSet;
    use std::iter::FromIterator;

    #[test]
    fn stuffing() {
        assert_eq!(
            Preorder{blocks:vec![3,2],size:2}.stuff(3, 0x5),
            Preorder{blocks:vec![5,7,4],size:3}
        );

        assert_eq!(
            Preorder{blocks:vec![1,3],size:2}.stuff(3, 0x5),
            Preorder{blocks:vec![1,7,5],size:3}
        );

        assert_eq!(
            Preorder{blocks:vec![1,2],size:2}.stuff(3, 0x5),
            Preorder{blocks:vec![1,7,4],size:3}
        );

        assert!(Preorder{blocks:vec![1,3],size:2}.stuff(3, 0x5).is_transitive());
        assert!(Preorder{blocks:vec![1,3],size:2}.stuff(3, 0x5).is_reflexive());
    }

    #[test]
    fn basic_sanity() {
        assert_eq!(
            Preorder::from_values(&[0,1,2,3]).upset(Alt(1)).iter().collect::<AltSet>(),
            alts![1, 2, 3]
        );

        assert_eq!(
            Preorder::from_values(&[0,1,2,3,4,5,6,7,8,9,10,11,12,13]).upset(Alt(7)).iter().collect::<AltSet>(),
            alts![7,8,9,10,11,12,13]
        );

        fn big_case(m : u32, n : u32) {
            let values : Vec<u32> = (0..n).collect();
            let expected : Vec<Alt> = (m..n).map(Alt).collect();

            assert_eq!(
                &Preorder::from_values(&values).upset(Alt(m)).iter().collect::<Vec<_>>(),
                &expected
            );
        }

        big_case(15, 33);
        big_case(35, 64);
        big_case(337, 1234);
    }
}
