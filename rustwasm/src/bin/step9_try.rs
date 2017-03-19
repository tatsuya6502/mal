extern crate mal;

use std::env;

use mal::step9_try_run as run;

fn main() {
    run(&env::args().collect::<Vec<_>>());
}
