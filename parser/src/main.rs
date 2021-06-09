#[macro_use]
extern crate nom;

use nom::{
    character::complete::{space1, space0, digit1, hex_digit1, oct_digit1},
    bytes::complete::tag_no_case,
    IResult,
};

use arch::registers;
use core::{fmt::Debug};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ParameterType {
    RegisterU16,
    RegisterU8,
    Literal,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Parameter {
    RegU16(u8),
    RegU8(u8),
    Lit(u16),
}

#[derive(Debug, PartialEq, Eq)]
struct Instruction {
    name: String,
    param1: Parameter,
    param2: Parameter,
}

impl Instruction {
    pub fn new(name: &str, param1: Parameter, param2: Parameter) -> Self {
        Self {
            name: name.to_owned(),
            param1,
            param2,
        }
    }
}

#[derive(Default, Debug)]
struct Program {
    instructions: Vec<Instruction>
}

named!(ins<&str, &str>, alt!(
    tag_no_case!("mov") | tag_no_case!("add") |
    tag_no_case!("sub") | tag_no_case!("mul")
));

named!(reg_8<&str, &str>, alt!(
    tag_no_case!("ah") | tag_no_case!("al") |
    tag_no_case!("bh") | tag_no_case!("bl") |
    tag_no_case!("ch") | tag_no_case!("cl") |
    tag_no_case!("dh") | tag_no_case!("dl")
));

named!(reg_16<&str, &str>, alt!(
    tag_no_case!("ax") | tag_no_case!("bx") |
    tag_no_case!("cx") | tag_no_case!("dx") |
    tag_no_case!("ex") | tag_no_case!("fx") |
    tag_no_case!("gx") | tag_no_case!("hx") |
    tag_no_case!("acc")
));

named!(lit<&str, u16>, alt!(
    do_parse!(_prefix: tag_no_case!("0x") >> value: hex_digit1 >> (u16::from_str_radix(value, 16).unwrap())) | // unwraps are safe due to parser
    do_parse!(_prefix: tag_no_case!("0b") >> value: is_a!("01") >> (u16::from_str_radix(value, 2).unwrap())) |
    do_parse!(_prefix: tag_no_case!("0o") >> value: oct_digit1 >> (u16::from_str_radix(value, 8).unwrap())) |
    do_parse!(value: digit1 >> (value.parse::<u16>().unwrap()))
));

named!(get_param<&str, Parameter>, alt!(
    do_parse!(name: reg_16 >> (Parameter::RegU16(registers::REGISTER_NAMES.iter().position(|&n| n == name).unwrap() as u8))) |
    do_parse!(name: reg_8 >> (Parameter::RegU8(registers::REGISTER_NAMES.iter().position(|&n| n == name).unwrap() as u8))) |
    do_parse!(lit: lit >> (Parameter::Lit(lit)))
));

named!(get_ins<&str, Instruction>, do_parse!(
    name: terminated!(ins, space1) >>
    r1: terminated!(get_param, space1) >>
    r2: terminated!(get_param, space0) >>
    (Instruction::new(name, r1, r2))
));

pub fn upper_or_lower_str<'a>(to_match: &'a str, input: &'a str) -> IResult<&'a str, &'a str>  {
    tag_no_case(to_match)(input)
}

fn main() {
    println!("Hello, world!");
}

#[test]
fn test_upper_string_ok() {
    let input_text = "mov x y";
    let input_text2 = "MOV X Y";

    let output = upper_or_lower_str("mov", input_text);
    let output2 = upper_or_lower_str("mov", input_text2);

    dbg!(&output);
    dbg!(&output2);
    let expected = Ok((" x y", "mov"));
    let expected2 = Ok((" X Y", "MOV"));
    assert_eq!(output, expected);
    assert_eq!(output2, expected2);
}

#[test]
fn test_mov_instruction() {
    let input = "mov 0x4f bh";

    let (_, ins) = get_ins(input).unwrap();
    dbg!(&ins);
    assert_eq!(ins, Instruction{
        name: "mov".to_owned(),
        param1: Parameter::Lit(79),
        param2: Parameter::RegU8(5),
    });

    let input = "add 0b1010011010 0o1232";

    let (_, ins) = get_ins(input).unwrap();
    dbg!(&ins);
    assert_eq!(ins, Instruction{
        name: "add".to_owned(),
        param1: Parameter::Lit(666),
        param2: Parameter::Lit(666),
    });
}
