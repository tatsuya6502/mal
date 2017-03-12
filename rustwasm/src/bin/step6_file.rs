extern crate mal;

use std::env;

use mal::step6_file_run as run;

fn main() {
    run(env::args().collect::<Vec<_>>());
}
