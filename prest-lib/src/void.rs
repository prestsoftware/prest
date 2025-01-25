use std::fmt;
use std::io::Write;
use crate::codec::{Encode,Result};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Void {}

impl Void {
    fn elim<T>(&self) -> T {
        match *self {}
    }
}

impl Encode for Void {
    fn encode<W : Write>(&self, _f : &mut W) -> Result<()> {
        self.elim()
    }
}

impl fmt::Display for Void {
    fn fmt(&self, _f : &mut fmt::Formatter) -> fmt::Result {
        self.elim()
    }
}
