use std;
use std::mem;
use std::io::{Read,Write,Cursor,Seek,SeekFrom};
use byteorder::{ReadBytesExt,WriteBytesExt,NativeEndian};
use std::hash::Hash;
use std::collections::{HashSet,HashMap,BTreeMap,BTreeSet};
use num_bigint::BigUint;
use num_rational::Ratio;
use num_integer::Integer;

#[derive(Debug)]
pub enum Error {
    Overflow,
    BadInteger,
    BadEnumTag,
    IO(std::io::Error),
    Unicode(std::string::FromUtf8Error),
    Other(String),
}

impl From<std::io::Error> for Error {
    fn from(e : std::io::Error) -> Error {
        Error::IO(e)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e : std::string::FromUtf8Error) -> Error {
        Error::Unicode(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Encode {
    fn encode<W : Write>(&self, f : &mut W) -> Result<()>;
}

pub trait Decode {
    fn decode<R : Read>(f : &mut R) -> Result<Self> where Self : Sized;
}

impl Encode for u8 {
    fn encode<W : Write>(&self, f : &mut W) -> Result<()> {
        Ok(f.write_u8(*self)?)
    }
}

impl Decode for u8 {
    fn decode<R : Read>(f : &mut R) -> Result<u8> {
        Ok(f.read_u8()?)
    }
}

impl Encode for f32 {
    fn encode<W : Write>(&self, f : &mut W) -> Result<()> {
        Ok(f.write_f32::<NativeEndian>(*self)?)
    }
}

impl Decode for f32 {
    fn decode<R : Read>(f : &mut R) -> Result<f32> {
        Ok(f.read_f32::<NativeEndian>()?)
    }
}

#[allow(dead_code)]
fn encode_bytes<W : Write>(f : &mut W, bytes : &[u8]) -> Result<()> {
    bytes.len().encode(f)?;
    f.write_all(bytes)?;
    Ok(())
}

#[allow(dead_code)]
fn decode_bytes<R : Read>(f : &mut R) -> Result<Vec<u8>> {
    let len : usize = Decode::decode(f)?;
    let mut result = vec![0; len];
    f.read_exact(&mut result[..])?;
    Ok(result)
}

/* cannot specialise instances (which would have a better performance)
 * use decode_bytes and encode_bytes instead
 *
impl Encode for [u8] {
    fn encode<W : Write>(&self, f : &mut W) -> Result<()> {
        self.len().encode(f)?;
        f.write_all(self)?;
        Ok(())
    }
}

impl Decode for Vec<u8> {
    fn decode<R : Read>(f : &mut R) -> Result<Vec<u8>> {
        let len : usize = Decode::decode(f)?;
        let mut result = vec![0; len];
        f.read_exact(&mut result[..])?;
        Ok(result)
    }
}
*/

impl Encode for bool {
    fn encode<W : Write>(&self, f : &mut W) -> Result<()> {
        (*self as u8).encode(f)
    }
}

impl Decode for bool {
    fn decode<R : Read>(f : &mut R) -> Result<bool> {
        match Decode::decode(f)? {
            0u8 => Ok(false),
            1u8 => Ok(true),
            _ => Err(Error::BadEnumTag),
        }
    }
}

impl<T : Encode> Encode for [T] {
    fn encode<W : Write>(&self, f : &mut W) -> Result<()> {
        self.len().encode(f)?;
        for elm in self {
            elm.encode(f)?;
        }
        Ok(())
    }
}

impl<T : Decode> Decode for Vec<T> {
    fn decode<R : Read>(f : &mut R) -> Result<Vec<T>> {
        let len : usize = Decode::decode(f)?;
        let mut result = Vec::with_capacity(len);
        for _ in 0..len {
            result.push(Decode::decode(f)?);
        }
        Ok(result)
    }
}

impl<T : Encode> Encode for Vec<T> {
    fn encode<W : Write>(&self, f : &mut W) -> Result<()> {
        self[..].encode(f)
    }
}

macro_rules! impl_prim {
    ($T:ty) => {
        impl Decode for $T {
            fn decode<R : Read>(f : &mut R) -> Result<$T> {
                let bit_width = 8 * mem::size_of::<$T>() as u32;
                let mut result = 0;
                let mut ofs = 0;

                loop {
                    let byte = f.read_u8()?;
                    let byte_raw = (byte & 0x7F) as u8;

                    if ofs + 8 - byte_raw.leading_zeros() > bit_width {
                        return Err(Error::Overflow);
                    }

                    result |= (byte_raw as $T) << ofs;
                    ofs += 7;

                    if byte < 0x80 {
                        break;
                    }
                }

                Ok(result)
            }
        }

        impl Encode for $T {
            fn encode<W : Write>(&self, f : &mut W) -> Result<()> {
                let mut x = *self;

                loop {
                    if x >= 0x80 {
                        f.write_u8(0x80 | (x & 0x7F) as u8)?;
                        x >>= 7;
                    } else {
                        f.write_u8(x as u8)?;  // x < 0x80, this is okay
                        break;
                    }
                }

                Ok(())
            }
        }
    }
}

impl_prim!(u16);
impl_prim!(u32);
impl_prim!(u64);
impl_prim!(usize);

impl<T : Encode> Encode for Option<T> {
    fn encode<W : Write>(&self, f : &mut W) -> Result<()> {
        match self {
            &None => 0u8.encode(f),
            &Some(ref x) => {
                1u8.encode(f)?;                
                x.encode(f)
            }
        }
    }
}

impl<T : Decode> Decode for Option<T> {
    fn decode<R : Read>(f : &mut R) -> Result<Option<T>> {
        match Decode::decode(f)? {
            0u8 => Ok(None),
            1u8 => Ok(Some(Decode::decode(f)?)),
            _ => Err(Error::BadEnumTag),
        }
    }
}

macro_rules! impl_tuple {
    ($($t:ident : $T:ident),*) => {
        impl<$($T : Encode),*> Encode for ($($T),*) {
            fn encode<W : Write>(&self, f : &mut W) -> Result<()> {
                let &($(ref $t),*) = self;
                $($t.encode(f)?;)*
                Ok(())
            }
        }

        impl<$($T : Decode),*> Decode for ($($T),*) {
            fn decode<R : Read>(f : &mut R) -> Result<($($T),*)> {
                $(let $t : $T = Decode::decode(f)?;)*
                Ok(($($t),*))
            }
        }
    }
}

impl_tuple!(t1 : T1, t2 : T2);
impl_tuple!(t1 : T1, t2 : T2, t3 : T3);
impl_tuple!(t1 : T1, t2 : T2, t3 : T3, t4 : T4);
impl_tuple!(t1 : T1, t2 : T2, t3 : T3, t4 : T4, t5 : T5);
impl_tuple!(t1 : T1, t2 : T2, t3 : T3, t4 : T4, t5 : T5, t6 : T6);
impl_tuple!(t1 : T1, t2 : T2, t3 : T3, t4 : T4, t5 : T5, t6 : T6, t7 : T7);
impl_tuple!(t1 : T1, t2 : T2, t3 : T3, t4 : T4, t5 : T5, t6 : T6, t7 : T7, t8 : T8);

impl<'a, T : Encode> Encode for &'a T {
    fn encode<W : Write>(&self, f : &mut W) -> Result<()> {
        (*self).encode(f)
    }
}

impl Encode for str {
    fn encode<W : Write>(&self, f : &mut W) -> Result<()> {
        self.as_bytes().encode(f)
    }
}

impl Decode for String {
    fn decode<R : Read>(f : &mut R) -> Result<String> {
        Ok(String::from_utf8(Decode::decode(f)?)?)
    }
}

impl Encode for String {
    fn encode<W : Write>(&self, f : &mut W) -> Result<()> {
        self.as_str().encode(f)
    }
}

impl Encode for () {
    fn encode<W : Write>(&self, _f : &mut W) -> Result<()> {
        Ok(())
    }
}

impl Decode for () {
    fn decode<R : Read>(_f : &mut R) -> Result<()> {
        Ok(())
    }
}

impl<T : Encode> Encode for Box<T> {
    fn encode<W : Write>(&self, f : &mut W) -> Result<()> {
        (**self).encode(f)
    }
}

impl<T : Decode> Decode for Box<T> {
    fn decode<R : Read>(f : &mut R) -> Result<Box<T>> {
        Ok(Box::new(Decode::decode(f)?))
    }
}

pub fn encode_iterator<T, I, W>(iter : I, f : &mut W) -> Result<()>
    where I : Iterator<Item=T>, T : Encode, W : Write
{
    for item in iter {
        Some(item).encode(f)?;
    }

    None::<T>.encode(f)
}

pub fn decode_iterator<T : Decode, R : Read>(f : &mut R) -> Result<Vec<T>> {
    let mut result = Vec::new();
    while let Some(item) = Decode::decode(f)? {
        result.push(item);
    }
    Ok(result)
}

pub fn encode_to_memory<T : Encode>(x : &T) -> Result<Vec<u8>> {
    let mut result = Cursor::new(Vec::new());
    x.encode(&mut result)?;
    Ok(result.into_inner())
}

pub fn decode_from_memory<T : Decode>(xs : &[u8]) -> Result<T> {
    Decode::decode(&mut Cursor::new(xs))
}

impl<T : Encode + Eq + Hash> Encode for HashSet<T> {
    fn encode<W : Write>(&self, f : &mut W) -> Result<()> {
        self.len().encode(f)?;
        for x in self {
            x.encode(f)?;
        }
        Ok(())
    }
}

impl<T : Decode + Eq + Hash> Decode for HashSet<T> {
    fn decode<R : Read>(f : &mut R) -> Result<HashSet<T>> {
        let xs : Vec<_> = Decode::decode(f)?;
        Ok(xs.into_iter().collect())
    }
}

impl<K : Encode + Eq + Hash, V : Encode> Encode for HashMap<K, V> {
    fn encode<W : Write>(&self, f : &mut W) -> Result<()> {
        self.len().encode(f)?;
        for kv in self {
            kv.encode(f)?;            
        }
        Ok(())
    }
}

impl<K : Decode + Eq + Hash, V : Decode> Decode for HashMap<K, V> {
    fn decode<R : Read>(f : &mut R) -> Result<HashMap<K, V>> {
        let xs : Vec<_> = Decode::decode(f)?;
        Ok(xs.into_iter().collect())
    }
}

impl<K : Encode, V : Encode> Encode for BTreeMap<K, V> {
    fn encode<W : Write>(&self, f : &mut W) -> Result<()> {
        self.len().encode(f)?;
        for kv in self {
            kv.encode(f)?;            
        }
        Ok(())
    }
}

impl<K : Decode + Eq + Ord, V : Decode> Decode for BTreeMap<K, V> {
    fn decode<R : Read>(f : &mut R) -> Result<BTreeMap<K, V>> {
        let xs : Vec<_> = Decode::decode(f)?;
        Ok(xs.into_iter().collect())
    }
}

impl<T : Encode + Eq + Ord> Encode for BTreeSet<T> {
    fn encode<W : Write>(&self, f : &mut W) -> Result<()> {
        self.len().encode(f)?;
        for x in self {
            x.encode(f)?;
        }
        Ok(())
    }
}

impl<T : Decode + Eq + Ord> Decode for BTreeSet<T> {
    fn decode<R : Read>(f : &mut R) -> Result<BTreeSet<T>> {
        let xs : Vec<_> = Decode::decode(f)?;
        Ok(xs.into_iter().collect())
    }
}

impl Encode for BigUint {
    fn encode<W : Write>(&self, f : &mut W) -> Result<()> {
        let mut bytes = self.to_radix_le(128);
        for i in 0..bytes.len()-1 {
            bytes[i] |= 0x80;  // mark every but the last byte
        }
        f.write_all(&bytes)?;
        Ok(())
    }
}

impl Decode for BigUint {
    fn decode<R : Read>(f : &mut R) -> Result<BigUint> {
        let mut bytes = Vec::new();
        loop {
            let byte = f.read_u8()?;
            bytes.push(byte & 0x7F);

            if byte < 0x80 {
                break;
            }
        }

        match BigUint::from_radix_le(&bytes, 128) {
            Some(x) => Ok(x),
            None => Err(Error::BadInteger),
        }
    }
}

impl<T : Encode> Encode for Ratio<T> {
    fn encode<W : Write>(&self, f : &mut W) -> Result<()> {
        (self.numer(), self.denom()).encode(f)
    }
}

impl<T : Decode+Clone+Integer> Decode for Ratio<T> {
    fn decode<R : Read>(f : &mut R) -> Result<Ratio<T>> {
        let (n, d) = Decode::decode(f)?;
        Ok(Ratio::new(n, d))
    }
}

#[derive(Debug, Clone)]
pub struct Packed<T>(pub T);

impl<T> Packed<T> {
    pub fn unpack(&self) -> &T {
        &self.0
    }

    pub fn into_unpacked(self) -> T {
        self.0
    }
}

impl<T : Encode> Encode for Packed<T> {
    fn encode<W : Write>(&self, f : &mut W) -> Result<()> {
        encode_to_memory(&self.0)?.encode(f)
    }
}

impl<T : Decode> Decode for Packed<T> {
    fn decode<R : Read>(f : &mut R) -> Result<Packed<T>> {
        // we exploit the fact that a byte array is encoded as
        // 1. length (# of bytes)
        // 2. the bytes themselves
        //
        // so we just skip the length
        // and decode the bytes directly from the input stream
        let _length : usize = Decode::decode(f)?;
        Ok(Packed(Decode::decode(f)?))
    }
}

#[allow(dead_code)]
fn encode_packed_list<T : Encode, W : Write>(f : &mut W, xs : &[T]) -> Result<()> {
    let mut buffer = Cursor::new(Vec::new());

    xs.len().encode(f)?;
    for x in xs {
        x.encode(&mut buffer)?;
        buffer.get_ref().encode(f)?;

        // go back to zero so that we can reuse the buffer
        // without reallocation
        buffer.seek(SeekFrom::Start(0))?;
        buffer.get_mut().truncate(0);
    }

    Ok(())
}

#[allow(dead_code)]
fn decode_packed_list<T : Decode, R : Read>(f : &mut R) -> Result<Vec<T>> {
    let length : usize = Decode::decode(f)?;
    let mut result = Vec::with_capacity(length);
    for _ in 0..length {
        let bytes : Vec<u8> = Decode::decode(f)?;
        result.push(decode_from_memory(&bytes)?);
    }
    Ok(result)
}

#[cfg(test)]
mod test {
    use std::fmt::Debug;
    use super::{Encode,Decode};
    use super::{encode_to_memory,decode_from_memory};

    #[test]
    fn codec() {
        fn case<T : Encode + Decode + PartialEq + Debug>(x : &T) {
            assert_eq!(
                x,
                &decode_from_memory::<T>(
                    &encode_to_memory::<T>(x).unwrap()
                ).unwrap()
            );
        }

        case(&0u8);
        case(&0u16);
        case(&127u8); case(&127u16);
        case(&128u8); case(&128u16);
        case(&129u8); case(&129u16);
        case(&255u8); case(&255u16);
        case(&u8::max_value());
        case(&u16::max_value());
        case(&u32::max_value());
        case(&u64::max_value());
        case(&usize::max_value());
    }
}
