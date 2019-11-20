use std::io::*;
use std::fs::File;
use uncertWizard::*;

fn main() {
    let mut filename = String::new();
    print!("Let's get the uncertainty of the following file: ");
    let _=stdout().flush();
    stdin().read_line(&mut filename).expect("Did not enter nice string.");
    if let Some('\n')=filename.chars().next_back() {
        filename.pop();
    }if let Some('\r')=filename.chars().next_back() {
        filename.pop();
    }
    let contents = get_file(filename).unwrap();
    let new_meas = Measurement::new(contents);
    new_meas.printout();
}
