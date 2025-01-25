pub mod consistency;

use std::io::Read;
use crate::codec::{self,Decode};
use crate::matrix::Matrix;

/* For each matrix:
 * - columns ~ goods
 * - rows    ~ observations
 */
#[derive(Debug, Clone, PartialEq)]
pub struct Subject {
    name    : String,
    prices  : Matrix<f32>,
    amounts : Matrix<f32>,
}

impl Decode for Subject {
    fn decode<R : Read>(f : &mut R) -> codec::Result<Subject> {
        Ok(Subject{
            name:    Decode::decode(f)?,
            prices:  Decode::decode(f)?,
            amounts: Decode::decode(f)?,
        })
    }
}
