// todo: remove after finish
#![allow(dead_code)]

extern crate onig;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate erased_serde;

pub mod grammar;
pub mod inter;
pub mod matcher;
pub mod rule;
pub mod scanner;
