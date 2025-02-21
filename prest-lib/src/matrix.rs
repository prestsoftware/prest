use std::mem;
use std::slice;
use std::io::{Read,Write};
use byteorder::{NativeEndian,ReadBytesExt};
use crate::codec::{self,Encode,Decode};

#[derive(Debug, Clone, PartialEq)]
pub struct Matrix<T> {
    pub nrows : usize,
    pub ncols : usize,
    values : Vec<T>,
}

impl<T> Matrix<T> {
    pub fn new(nrows : usize, ncols : usize, values : Vec<T>) -> Matrix<T> {
        assert_eq!(nrows * ncols, values.len());
        Matrix{nrows, ncols, values}
    }

    pub fn from_slice(nrows : usize, ncols : usize, values : &[T]) -> Matrix<T> where T : Clone {
        Matrix::new(nrows, ncols, values.iter().cloned().collect())
    }

    pub fn from_indices<F>(nrows : usize, ncols : usize, f : F) -> Matrix<T>
        where F : Fn(usize, usize) -> T
    {
        Matrix {
            nrows,
            ncols,
            values: {
                let mut values = Vec::new();
                for i in 0..nrows {
                    values.extend((0..ncols).map(|j| f(i, j)));
                }
                values
            }
        }
    }

    pub fn row(&self, i : usize) -> &[T] {
        &self.values[i*self.ncols..(i+1)*self.ncols]
    }

    pub fn iter_rows(&self) -> slice::Chunks<T> {
        self.values.chunks(self.ncols)
    }

    pub fn get(&self, i : usize, j : usize) -> T where T : Copy {
        self.values[i*self.ncols + j]
    }

    pub fn get_ref(&self, i : usize, j : usize) -> &T {
        &self.values[i*self.ncols + j]
    }

    pub fn get_mut_ref(&mut self, i : usize, j : usize) -> &mut T {
        &mut self.values[i*self.ncols + j]
    }
}

pub fn dot_product(xs : &[f32], ys : &[f32]) -> f32 {
    assert_eq!(xs.len(), ys.len());
    xs.into_iter().zip(ys).map(|(x,y)| x*y).sum()
}

// (Adapted from `byteorder`.)
unsafe fn mem_bytes<'a, T : Copy>(slice : &'a [T]) -> &'a [u8] { unsafe {
    slice::from_raw_parts(
        slice.as_ptr() as *const u8,
        mem::size_of::<T>() * slice.len(),
    )
}}

// decode as numpy array
impl Decode for Matrix<f32> {
    fn decode<R : Read>(f : &mut R) -> codec::Result<Matrix<f32>> {
        // protocol:
        //
        // int: number of dimensions (must be 2)
        // int: number of rows
        // int: number of columns
        // int: number of bytes in array (= 4 * cols * rows)
        // cols*rows floats: the array
        //
        let ndims : u8 = Decode::decode(f)?;
        assert_eq!(ndims, 2, "only 2-dimensional matrices are supported, got {} dimensions", ndims);

        let (nrows, ncols) = Decode::decode(f)?;
        let nbytes : usize = Decode::decode(f)?;
        assert_eq!(nbytes, 4 * nrows * ncols);

        let mut values = vec![0f32; nrows * ncols];
        f.read_f32_into::<NativeEndian>(&mut values)?;

        Ok(Matrix::new(nrows, ncols, values))
    }
}

// encode as numpy array
impl Encode for Matrix<f32> {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        // header
        (2u8, self.nrows, self.ncols, 4 * self.nrows * self.ncols).encode(f)?;

        // the array
        f.write_all(unsafe {
            mem_bytes(&self.values)
        })?;

        Ok(())
    }
}

