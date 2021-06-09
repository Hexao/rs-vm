#[macro_use]
extern crate nom;

use nom::{
    IResult, bytes::complete::{tag, tag_no_case, take_till, take}, 
    character::{
        complete::{space1, space0, digit1, hex_digit1, oct_digit1, alpha1},
    }
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
pub enum Operator {
    Plus,
    Minus,
    Multiply
}

#[derive(Debug, PartialEq, Eq)]
pub enum Parameter {
    RegU16(u8),
    RegU8(u8),
    Lit(u16),
    Expr(Vec<Parameter>),
    Var(String),
    Operator(Operator)
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
    tag_no_case!("sub") | tag_no_case!("mul") |
    tag_no_case!("lsf") | tag_no_case!("rsf") |
    tag_no_case!("and") | tag_no_case!("or") |
    tag_no_case!("xor") | tag_no_case!("cmp") |
    tag_no_case!("jmp") | tag_no_case!("jne")
));

named!(reg_8<&str, Parameter>, do_parse!(reg_8: alt!(
    tag_no_case!("ah") | tag_no_case!("al") |
    tag_no_case!("bh") | tag_no_case!("bl") |
    tag_no_case!("ch") | tag_no_case!("cl") |
    tag_no_case!("dh") | tag_no_case!("dl")
) >> (
    Parameter::RegU8(registers::REGISTER_NAMES.iter().position(|&n| n == reg_8).unwrap() as u8)
)));

named!(reg_16<&str, Parameter>, do_parse!(reg_16: alt!(
    tag_no_case!("ax") | tag_no_case!("bx") |
    tag_no_case!("cx") | tag_no_case!("dx") |
    tag_no_case!("ex") | tag_no_case!("fx") |
    tag_no_case!("gx") | tag_no_case!("hx") |
    tag_no_case!("acc")
) >> (
    Parameter::RegU16(registers::REGISTER_NAMES.iter().position(|&n| n == reg_16).unwrap() as u8)
)));

named!(lit<&str, Parameter>, do_parse!(lit: alt!(
    do_parse!(_prefix: tag_no_case!("0x") >> value: hex_digit1 >> (u16::from_str_radix(value, 16).unwrap())) | // unwraps are safe due to parser
    do_parse!(_prefix: tag_no_case!("0b") >> value: is_a!("01") >> (u16::from_str_radix(value, 2).unwrap())) |
    do_parse!(_prefix: tag_no_case!("0o") >> value: oct_digit1 >> (u16::from_str_radix(value, 8).unwrap())) |
    do_parse!(value: digit1 >> (value.parse::<u16>().unwrap()))
) >> (
    Parameter::Lit(lit)
)));

named!(get_param<&str, Parameter>, alt!(reg_8 | reg_16 | lit));

named!(get_ins<&str, Instruction>, do_parse!(
    name: terminated!(ins, space1) >>
    r1: terminated!(get_param, space1) >>
    r2: terminated!(get_param, space0) >>
    (Instruction::new(name, r1, r2))
));

named!(var<&str, Parameter>, do_parse!(_prefix: tag!(":") >> value: alpha1 >> (Parameter::Var(value.to_string()))));

named!(get_elem<&str, Parameter>, alt!(lit | var));

named!(end_bracket<&str, &str>, peek!(tag!("]")));


named!(operator<&str,Parameter>, alt!(
    do_parse!(_prefix: tag!("+") >> (Parameter::Operator(Operator::Plus))) |
    do_parse!(_prefix: tag!("-") >> (Parameter::Operator(Operator::Minus))) |
    do_parse!(_prefix: tag!("*") >> (Parameter::Operator(Operator::Multiply)))
));

pub fn upper_or_lower_str<'a>(to_match: &'a str, input: &'a str) -> IResult<&'a str, &'a str>  {
    tag_no_case(to_match)(input)
}

pub enum State {
    ExpectElement,
    ExpectOperator
}

pub fn get_expression<'a>(input: &'a str) -> IResult<&'a str, Vec<Parameter>> {
    let (mut rest, _) = tag("[")(input)?;

    let mut expr = vec![];
    let mut state = State::ExpectElement;
    let mut scope = 1;

    loop {
        dbg!(&expr);
        match state {
            State::ExpectElement => {
                let (new_rest, param) = get_elem(rest)?;
                expr.push(param);
                state = State::ExpectOperator;
                let (new_rest, _) = take_till(|c: char| !c.is_whitespace() )(new_rest)?;
                rest = new_rest;
            },
            State::ExpectOperator => {
                match end_bracket(rest) {
                    Err(_) => {},
                    Ok(_) => {
                        if scope - 1 == 0 {
                            let (new_rest, _) = take(1 as usize)(rest)?;
                            rest = new_rest;
                            break;
                        }
                    },
                }
                let (new_rest, param) = operator(rest)?;
                expr.push(param);
                state = State::ExpectElement;
                let (new_rest, _) = take_till(|c: char| !c.is_whitespace() )(new_rest)?;
                rest = new_rest;

            }
        }
    }
    Ok((rest, expr))
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
fn test_expr() {
    
    let input = "[0x42 + :var - 0o5 * 0b0010]";

    dbg!(get_expression(input));
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
