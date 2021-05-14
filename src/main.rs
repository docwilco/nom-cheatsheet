include! {concat!(env!("OUT_DIR"), "/uses.rs")}

use nom::number::Endianness;
use nom::{
    character::complete::{alpha1, digit1},
    error::ErrorKind,
    IResult,
};

fn number(input: &str) -> IResult<&str, usize> {
    let debug: IResult<_, _> = into::<&str, _, Vec<u8>, (_, _), _, _>(alpha1)("abcd");

    map(digit1, |s: &str| s.parse().unwrap())(input)
}

fn main() {
    include!(concat!(env!("OUT_DIR"), "/main.rs"));
}
