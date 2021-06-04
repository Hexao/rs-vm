#[macro_use]
extern crate nom;

use nom::{
    character::complete::{alpha1, char, line_ending, not_line_ending},
    combinator::{cut, map, not, recognize},
    bytes::complete::tag_no_case,
    error::{context, ParseError, VerboseError},
    multi::{many0, many1},
    branch::Alt,
    IResult,
};

use arch::registers;

use core::{fmt::Debug};
use std::any::Any;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ParameterType {
    RegisterU8,
    RegisterU16,
    Literal
}

#[derive(Debug, PartialEq, Clone)]
pub struct Registeru8 {
    reg_type: u8,
    reg_name: String,
}

impl Registeru8 {
    pub fn new(reg_type: u8, reg_name: String) -> Self {
        Self {
            reg_type,
            reg_name
        }
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct Registeru16 {
    reg_type: u8,
    reg_name: String,
}

impl Registeru16 {
    pub fn new(reg_type: u8, reg_name: String) -> Self {
        Self {
            reg_type,
            reg_name
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Literal {
    value: u16
}


pub trait Parameter {
    fn get_type(&self) -> ParameterType;
    fn as_any(&self) -> &dyn Any;
}

impl Debug for dyn Parameter {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.get_type() == ParameterType::RegisterU8 {
            return write!(f, "Parameter{{{:?}}}", self.as_any().downcast_ref::<Registeru8>().expect("downcast error"));
        } else if self.get_type() == ParameterType::RegisterU16 {
            return write!(f, "Parameter{{{:?}}}", self.as_any().downcast_ref::<Registeru16>().expect("downcast error"));
        } else if self.get_type() == ParameterType::Literal { 
            return write!(f, "Parameter{{{:?}}}", self.as_any().downcast_ref::<Literal>().expect("downcast error"));
        }
        
        write!(f, "Parameter{{{}}}", 0)
    }
}

impl PartialEq for dyn Parameter {
    fn eq(&self, other: &Self) -> bool {
        if self.get_type() != other.get_type() {
            return false;
        }

        if self.get_type() == ParameterType::RegisterU8 {
            let a: &Registeru8 = self.as_any().downcast_ref::<Registeru8>().expect("downcast error");
            let b: &Registeru8 = self.as_any().downcast_ref::<Registeru8>().expect("downcast error");
            if a != b {
                return false;
            }
        } else if self.get_type() == ParameterType::RegisterU16 {
            let a: &Registeru16 = self.as_any().downcast_ref::<Registeru16>().expect("downcast error");
            let b: &Registeru16 = self.as_any().downcast_ref::<Registeru16>().expect("downcast error");
            if a != b {
                return false;
            }
        } else if self.get_type() == ParameterType::Literal {
            let a: &Literal = self.as_any().downcast_ref::<Literal>().expect("downcast error");
            let b: &Literal = self.as_any().downcast_ref::<Literal>().expect("downcast error");
            if a != b {
                return false;
            }
        }
        true
    }
}

impl Parameter for Registeru8 {
    fn get_type(&self) -> ParameterType {
        ParameterType::RegisterU8
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}
impl Parameter for Registeru16 {
    fn get_type(&self) -> ParameterType {
        ParameterType::RegisterU16
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
impl Parameter for Literal {
    fn get_type(&self) -> ParameterType {
        ParameterType::Literal
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}


#[derive(Debug)]
struct Instruction {
    ins_type: u8,
    param1: Box<dyn Parameter>,
    param2: Box<dyn Parameter>,
}

impl Instruction {
    pub fn new(ins_type:u8, param1: Box<dyn Parameter>, param2: Box<dyn Parameter>) -> Self {
        Self {
            ins_type,
            param1,
            param2,
        }
    }
}

#[derive(Default, Debug)]
struct Program {
    instructions: Vec<Instruction>
}



named!(reg<&str, &str>, alt!(tag_no_case!("ah") | tag_no_case!("al") | tag_no_case!("ax") |
    tag_no_case!("bh") | tag_no_case!("bl") | tag_no_case!("bx") |
    tag_no_case!("ch") | tag_no_case!("cl") | tag_no_case!("cx") |
    tag_no_case!("dh") | tag_no_case!("dl") | tag_no_case!("dx") |
    tag_no_case!("ex") | tag_no_case!("fx") | tag_no_case!("gx") | tag_no_case!("hx") | tag_no_case!("acc")
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
fn test_program() {
    let input_text = "mov [$42 + !loc - ($05 * $31)] ax";
    //let mut prog = Program::default();

    let (rest, _) = upper_or_lower_str("mov", input_text).unwrap();
    let instruct_type = 0x10;
    let mut rest = rest.split_ascii_whitespace();

    let (_, matched) = reg(rest.next().unwrap()).unwrap();
    let reg_id = registers::REGISTER_NAMES.iter().position(|&name| name == matched).unwrap();
    let param1: Box<dyn Parameter> = if registers::SIZE_OF[reg_id] == 1 {
        Box::new(Registeru8::new(reg_id as u8, matched.to_string()))
    } else {
        Box::new(Registeru16::new(reg_id as u8, matched.to_string()))
    };

    let (_, matched) = reg(rest.next().unwrap()).unwrap();
    let reg_id = registers::REGISTER_NAMES.iter().position(|&name| name == matched).unwrap();
    let param2: Box<dyn Parameter> = if registers::SIZE_OF[reg_id] == 1 {
        Box::new(Registeru8::new(reg_id as u8, matched.to_string()))
    } else {
        Box::new(Registeru16::new(reg_id as u8, matched.to_string()))
    };

    let instruction = Instruction::new(instruct_type, param1, param2);
    dbg!(&instruction);
    /*let expected = Instruction { 
        ins_type: 0x10,
        param1: Box::new(Registeru8::new(2, "ah".to_string())),
        param2: Box::new(Registeru8::new(4, "ax".to_string()))
    };*/
    //assert_eq!(instruction, expected);
}
