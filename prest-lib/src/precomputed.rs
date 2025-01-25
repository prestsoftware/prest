use crate::fast_preorder::{self,FastPreorder};
use crate::preorder::Preorder;
use crate::codec::{self,Encode};

use std::fmt;
use std::result;
use std::fs::File;
use std::io::Write;
use byteorder::{ReadBytesExt,LittleEndian};

type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    TooManyAlternatives(u32),
    NeedPrecomputedPreorders,
}

impl fmt::Display for Error {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::TooManyAlternatives(alt_count) =>
                write!(f, "too many alternatives: {}", alt_count),

            Error::NeedPrecomputedPreorders =>
                write!(f, "file with precomputed preorders is required"),
        }
    }
}

impl Encode for Error {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        match *self {
            Error::TooManyAlternatives(alt_count) => (0u8, alt_count).encode(f),
            Error::NeedPrecomputedPreorders => 1u8.encode(f),
        }
    }
}

pub struct Preorders {
    pub preorders : Vec<Preorder>,
    pub partial_orders : Vec<Preorder>,
    pub weak_orders : Vec<Preorder>,
}

pub struct Precomputed {
    // indexed by the number of alternatives
    preorders : Vec<Preorders>,
    fname_precomputed_preorders : Option<String>,
}

impl Precomputed {
    pub fn new(fname_precomputed_preorders : Option<&str>) -> Self {
        Precomputed {
            preorders: Vec::new(),
            fname_precomputed_preorders:
                fname_precomputed_preorders.map(String::from)
        }
    }

    pub fn precompute(&mut self, max_size : u32) -> Result<()> {
        for size in self.preorders.len() as u32 .. max_size+1 {
            let preorders = if size < 7 {
                fast_preorder::all(size).into_iter().map(
                    |p| Preorder::from_fast_preorder(size, p)
                ).collect()
            } else if size == 7 {
                if let Some(ref fname) = self.fname_precomputed_preorders {
                    let mut file = File::open(fname).unwrap();
                    let mut result = Vec::new();
                    while let Ok(bits) = file.read_u64::<LittleEndian>() {
                        result.push(Preorder::from_fast_preorder(7, FastPreorder(bits)));
                    }
                    result
                } else {
                    return Err(Error::NeedPrecomputedPreorders)
                }
            } else {
                return Err(Error::TooManyAlternatives(max_size))
            };

            let partial_orders = preorders.iter().cloned().filter(
                Preorder::is_strict
            ).collect::<Vec<_>>();

            let weak_orders = preorders.iter().cloned().filter(
                Preorder::is_total
            ).collect::<Vec<_>>();

            self.preorders.push(Preorders{
                preorders, partial_orders, weak_orders,
            });
        }

        Ok(())
    }

    pub fn get(&self, size : u32) -> Result<&Preorders> {
        if size >= self.preorders.len() as u32 {
            return Err(Error::TooManyAlternatives(size))
        }

        Ok(&self.preorders[size as usize])
    }
}
