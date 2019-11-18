use std::fs;
use uncertWizard::*;

fn main() {
    let contents = fs::read_to_string("1,2A.ucrt")
        .expect("Could not read it.");
    let new_meas = Measurement::new(contents);
    new_meas.printout();
}
