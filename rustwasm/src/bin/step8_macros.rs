extern crate mal;

use std::env;

use mal::step8_macros_run as run;

fn main() {
    run(&env::args().collect::<Vec<_>>());
}
