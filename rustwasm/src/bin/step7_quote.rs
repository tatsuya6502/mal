extern crate mal;

use std::env;

use mal::step7_quote_run as run;

fn main() {
    run(env::args().collect::<Vec<_>>());
}
