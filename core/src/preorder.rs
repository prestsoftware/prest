use fast_preorder::FastPreorder;
use alt_set::{Block,AltSetView};
use std::mem;
use std::iter;
use std::io::{Read,Write};
use codec::{self,Encode,Decode};
use std::collections::HashSet;
use alt::Alt;

#[derive(PartialEq,Eq,PartialOrd,Ord,Clone,Hash,Debug)]
pub struct Preorder {
    blocks : Vec<Block>, // little endian
    pub size : u32,
}

impl Encode for Preorder {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        self.size.encode(f)?;

        let block_size = 8 * mem::size_of::<Block>();  // in bits
        let stride = (self.size as usize + block_size - 1) / block_size;  // round upwards
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

        let block_size = 8 * mem::size_of::<Block>();  // in bits
        let stride = (size as usize + block_size - 1) / block_size;  // round upwards
        let block_count = size as usize * stride;

        let mut blocks = Vec::new();
        for _ in 0..block_count {
            blocks.push(Decode::decode(f)?);
        }

        Ok(Preorder{size, blocks})
    }
}

impl Preorder {
    pub fn edges(&self) -> Vec<(Alt, Alt)> {
        Alt::all(self.size).flat_map(
            |i| iter::repeat(i).zip(self.upset(i))
        ).filter(|&(i,j)| i != j).collect()
    }

    pub fn simple_digraph(&self) -> Vec<(Alt, Alt)> {
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

        let mut edges : HashSet<_> = self.edges().into_iter().collect();
        let mut changing = true;

        while changing {
            changing = false;

            for (i, j) in edges.clone() {
                if is_redundant(&edges, i, j) {
                    edges.remove(&(i, j));
                    changing = true;
                }
            }
        }

        edges.into_iter().collect()
    }

    pub fn diagonal(size : u32) -> Preorder {
        let block_size = 8 * mem::size_of::<Block>() as u32;  // in bits
        let stride = (size + block_size - 1) / block_size;  // round upwards
        let nblocks = size * stride;

        let mut result = Preorder{ blocks: vec![0; nblocks as usize], size: size };
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

        Preorder { blocks: blocks, size: size }
    }

    #[inline]
    pub fn set_leq(&mut self, Alt(i) : Alt, Alt(j) : Alt, leq : bool) {
        // i = row, j = column
        // assuming that block_size is a power of two
        // and the compiler will optimise it to bit shifts
        let block_size = 8 * mem::size_of::<Block>() as u32;  // in bits
        let stride = (self.size + block_size - 1) / block_size;  // round upwards
        let offset = j / block_size;
        let bit_ofs = j % block_size;

        if leq {
            self.blocks[(i * stride + offset) as usize] |= 1 << bit_ofs;
        } else {
            self.blocks[(i * stride + offset) as usize] &= !(1 << bit_ofs);
        }
    }

    #[inline]
    pub fn leq(&self, Alt(i) : Alt, Alt(j) : Alt) -> bool {
        // i = row, j = column
        // assuming that block_size is a power of two
        // and the compiler will optimise it to bit shifts
        let block_size = 8 * mem::size_of::<Block>() as u32;  // in bits
        let stride = (self.size + block_size - 1) / block_size;  // round upwards
        let offset = j / block_size;
        let bit_ofs = j % block_size;
        (self.blocks[(i * stride + offset) as usize] >> bit_ofs) & 0x1 != 0
    }

    #[inline]
    pub fn lt(&self, i : Alt, j : Alt) -> bool {
        self.leq(i,j) && !self.leq(j, i)
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
        let block_size = 8 * mem::size_of::<Block>() as u32;  // in bits
        let stride = (self.size + block_size - 1) / block_size;  // in blocks, round upwards
        AltSetView {
            blocks: &self.blocks[(i*stride) as usize .. (i*stride + stride) as usize]
        }
    }

    pub fn as_linear_order(&self) -> Vec<Alt> {
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

        order.into_iter().map(
            |i| i.expect("not a linear order")
        ).collect()
    }

    pub fn from_values<T: PartialOrd>(values : &[T]) -> Preorder {
        let block_size = 8 * mem::size_of::<Block>();  // in bits
        let stride = (values.len() + block_size - 1) / block_size;  // round upwards
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
            blocks: blocks,
            size: values.len() as u32,
        }
    }

    pub fn stuff(&self, target_size : u32, mask : Block) -> Preorder {
        let block_size = 8 * mem::size_of::<Block>() as u32;  // in bits
        let stride_dst = (target_size + block_size - 1) / block_size;  // round upwards
        let stride_src = (self.size + block_size - 1) / block_size;  // round upwards
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
                        blocks[(ofs_dst + j_dst/block_size) as usize]
                            |= ((self.blocks[(ofs_src + j_src/block_size) as usize]
                                    >> (j_src % block_size))
                                  & 0x1)
                                << (j_dst % block_size);

                        j_src += 1;
                    }

                    col_mask >>= 1;
                }

                i_src += 1
            } else {
                // unattractive option, fill the row with ones
                let block_count = (target_size / block_size) as usize;
                for i in 0 .. block_count {
                    blocks[ofs_dst as usize + i] = Block::max_value();
                }

                if target_size % block_size > 0 {
                    blocks[ofs_dst as usize + block_count] = (1 << (target_size % block_size)) - 1;
                }
            }

            row_mask >>= 1;
        }

        Preorder{blocks, size: target_size}
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alt_set::AltSet;
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
