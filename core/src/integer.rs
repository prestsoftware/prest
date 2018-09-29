use num::BigUint;
use std::io::{Read,Write};
use std::fmt::{self, Debug, Display, Formatter};
use std::ops::{MulAssign, Mul, AddAssign, Add};
use std::iter::Sum;
use num::{Zero, One, zero, one};
use byteorder::ReadBytesExt;

use codec::{self,Encode,Decode};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct Integer {
    value: BigUint
}

impl From<u32> for Integer {
    fn from(x : u32) -> Integer {
        Integer{ value: BigUint::from(x) }
    }
}

impl MulAssign<u32> for Integer {
    fn mul_assign(&mut self, other : u32) {
        self.value *= other
    }
}

impl MulAssign<usize> for Integer {
    fn mul_assign(&mut self, other : usize) {
        self.value *= other
    }
}

impl Mul<Integer> for u32 {
    type Output = Integer;
    fn mul(self, other : Integer) -> Integer {
        Integer{ value: self * other.value }
    }
}

impl Mul<Integer> for usize {
    type Output = Integer;
    fn mul(self, other : Integer) -> Integer {
        Integer{ value: self * other.value }
    }
}

impl Mul<Integer> for Integer {
    type Output = Integer;
    fn mul(self, other : Integer) -> Integer {
        Integer{ value: self.value * other.value }
    }
}

impl AddAssign<Integer> for Integer {
    fn add_assign(&mut self, other : Integer) {
        self.value += other.value
    }
}

impl Add<Integer> for Integer {
    type Output = Integer;
    fn add(self, other : Integer) -> Integer {
        Integer{ value: self.value + other.value }
    }
}

impl Debug for Integer {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        Debug::fmt(&self.value, f)
    }
}

impl Display for Integer {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        Display::fmt(&self.value, f)
    }
}

impl Sum for Integer {
    fn sum<I : Iterator<Item=Integer>>(it : I) -> Integer {
        let mut result = zero();
        for x in it {
            result += x;
        }
        result
    }
}

impl Zero for Integer {
    fn zero() -> Integer {
        Integer{ value: zero() }
    }

    fn is_zero(&self) -> bool {
        self.value.is_zero()
    }
}

impl One for Integer {
    fn one() -> Integer {
        Integer{ value: one() }
    }
}

impl Encode for Integer {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        let mut bytes = self.value.to_radix_le(128);
        for i in 0..bytes.len()-1 {
            bytes[i] |= 0x80;  // mark every but the last byte
        }
        f.write(&bytes)?;
        Ok(())
    }
}

impl Decode for Integer {
    fn decode<R : Read>(f : &mut R) -> codec::Result<Integer> {
        let mut bytes = Vec::new();
        loop {
            let byte = f.read_u8()?;
            bytes.push(byte & 0x7F);

            if byte < 0x80 {
                break;
            }
        }
        
        match BigUint::from_radix_le(&bytes, 128) {
            Some(x) => Ok(Integer{value: x}),
            None => Err(codec::Error::BadInteger),
        }
    }
}

