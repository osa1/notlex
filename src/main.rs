extern crate notlex;

use std::io::Read;

use notlex::*;

fn main() {
    let mut stdin = String::new();
    std::io::stdin().read_to_string(&mut stdin);
    println!("{:?}", charset_parser::parse_CharSet0(&stdin));
}
