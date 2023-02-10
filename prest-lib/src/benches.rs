#![cfg(feature = "bench")]

use test::Bencher;

use preorder_u64;
use preorder_u64_size;
use preorder_bitvec;
use preorder_matrix;
use preorder_u64_array;
use preorder_u64_vec;

use model;

#[bench]
fn all_preorders_u64(b : &mut Bencher) {
    b.iter(|| preorder_u64::all(5).len());
}

#[bench]
fn all_preorders_u64_size(b : &mut Bencher) {
    b.iter(|| preorder_u64_size::all(5).len());
}

#[bench]
fn all_preorders_bitvec(b : &mut Bencher) {
    b.iter(|| preorder_bitvec::all(5).len());
}

#[bench]
fn all_preorders_u64_array(b : &mut Bencher) {
    b.iter(|| preorder_u64_array::all(5).len());
}

#[bench]
fn all_preorders_u64_vec(b : &mut Bencher) {
    b.iter(|| preorder_u64_vec::all(5).len());
}

#[bench]
fn all_preorders_matrix(b : &mut Bencher) {
    b.iter(|| preorder_matrix::all(5).len());
}

#[bench]
fn all_instances_linear(b : &mut Bencher) {
    b.iter(|| model::linear_instances(8).count());
}
