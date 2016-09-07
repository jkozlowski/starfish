//extern crate num;
//
//use num::{Num, One};
//use std::ops::{Not, Sub, BitAnd};
//use std::convert::{From};

// seastar has it generic, I don't know how to do that...
#[inline(always)]
pub fn align_down(n: usize, align: usize) -> usize {
    //  return v & ~(align - 1);
    return n & !(align - 1);
}