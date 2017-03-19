#![allow(non_snake_case)]

extern crate mal;

use std::env;

use mal::stepA_mal_run as run;

fn main() {
    run(&env::args().collect::<Vec<_>>());
}
