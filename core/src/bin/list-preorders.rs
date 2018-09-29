extern crate prest;
extern crate byteorder;

use std::fs::File;
use byteorder::{WriteBytesExt,LittleEndian};
use prest::fast_preorder::{self,FastPreorder};

fn main() {
    let mut f = File::create("preorders-7.bin").unwrap();

    for &FastPreorder(bits) in &fast_preorder::all(7) {
        f.write_u64::<LittleEndian>(bits).unwrap();
    }
}
