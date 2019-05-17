use crate::error::{CompilerError};
use crate::scope::Register;
use std::{u16};
use std::iter::FromIterator;

pub use resast::prelude::*;


pub type BytecodeResult = Result<Bytecode, CompilerError>;

pub trait ToBytes {
    fn to_bytes(&self) -> Vec<u8>;
    fn length_in_bytes(&self) -> usize {
        self.to_bytes().len()
    }
}


#[derive(Debug, PartialEq, Clone)]
pub enum Instruction
{
    LoadString,
    LoadFloatNum,
    LoadLongNum,
    LoadNum,
    LoadArray,

    PropAccess,
    CallFunc,
    Eval,
    CallBytecodeFunc,
    ReturnBytecodeFunc,
    Copy,
    Exit,

    JumpCond,
    Jump,
    JumpCondNeg,

    LogicAnd,
    LogicOr,

    CompEqual,
    CompNotEqual,
    CompStrictEqual,
    CompStrictNotEqual,
    CompLessThan,
    CompGreaterThan,
    CompLessThanEqual,
    CompGreaterThanEqual,

    Add,
    Minus,
    Mul,
    Div,
    // LeftShift
    // RightShift
    // Mod,
    // Or,
    // XOr,
    // And,
    // In,
}

impl Instruction {
    fn to_byte(&self) -> u8 {
        match self {
            Instruction::LoadString => 1,
            Instruction::LoadNum => 2,
            Instruction::LoadFloatNum => 3,
            Instruction::LoadLongNum => 4,
            Instruction::LoadArray => 5,

            Instruction::PropAccess => 10,
            Instruction::CallFunc => 11,
            Instruction::Eval => 12,
            Instruction::CallBytecodeFunc => 13,
            Instruction::ReturnBytecodeFunc => 14,
            Instruction::Copy => 15,
            Instruction::Exit => 16,
            Instruction::JumpCond => 17,
            Instruction::Jump => 18,
            Instruction::JumpCondNeg => 19,

            Instruction::LogicAnd => 30,
            Instruction::LogicOr => 31,

            Instruction::CompEqual => 50,
            Instruction::CompNotEqual => 51,
            Instruction::CompStrictEqual => 52,
            Instruction::CompStrictNotEqual => 53,
            Instruction::CompLessThan => 54,
            Instruction::CompGreaterThan => 55,
            Instruction::CompLessThanEqual => 56,
            Instruction::CompGreaterThanEqual => 57,

            Instruction::Add => 100,
            Instruction::Minus => 102,
            Instruction::Mul => 101,
            Instruction::Div => 103,
        }
    }
}

#[test]
fn test_instrution_to_byte() {
    assert_eq!(Instruction::Add.to_byte(), 100);
}

#[derive(Debug, PartialEq, Clone)]
pub struct BytecodeAddrToken {
    pub ident: String
}

impl ToBytes for BytecodeAddrToken {
    fn to_bytes(&self) -> Vec<u8> {
        vec![0; 8]
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct LabelAddrToken {
    pub label: Label
}

impl ToBytes for LabelAddrToken {
    fn to_bytes(&self) -> Vec<u8> {
        vec![0; 8]
    }
}


#[derive(Debug, PartialEq, Clone)]
pub enum Operand
{
    String(String),
    FloatNum(f64),
    LongNum(i64),
    ShortNum(u8),
    Reg(u8),
    RegistersArray(Vec<u8>),

    FunctionAddr(BytecodeAddrToken),
    BranchAddr(LabelAddrToken)
}

impl Operand {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Operand::String(string) => Operand::encode_string(string.to_string()),
            Operand::FloatNum(float_num) => Operand::encode_float_num(float_num.clone()),
            Operand::LongNum(long_num) => Operand::encode_long_num(long_num.clone() as u64),
            Operand::ShortNum(num) |
            Operand::Reg(num) => vec![*num],
            Operand::RegistersArray(regs) => Operand::encode_registers_array(&regs),
            Operand::FunctionAddr(token)  => token.to_bytes(),
            Operand::BranchAddr(token) => token.to_bytes()
        }
    }

    pub fn from_literal(lit: Literal) -> Result<Self, CompilerError> {
        match lit {
            Literal::Null => Ok(Operand::Reg(0)), //TODO: Register of predefined void 0,
            Literal::String(string) => Ok(Operand::String(string)),
            Literal::Number(num) => Ok(Operand::ShortNum(num.parse().unwrap())), //TODO
            Literal::Boolean(bool) => Ok(Operand::ShortNum(bool as u8)),
            Literal::RegEx(_) | Literal::Template(_) => Err(CompilerError::Custom("regex and template literals are not supported".into()))
        }
    }

    pub fn str(string: String) -> Self {
        Operand::String(string.to_string())
    }

    pub fn function_addr(ident: String) -> Self {
        Operand::FunctionAddr(BytecodeAddrToken{ ident })
    }

    pub fn branch_addr(label: Label) -> Self {
        Operand::BranchAddr(LabelAddrToken{ label })
    }

    fn encode_string(string: String) -> Vec<u8> {
        if string.len() > u16::max_value() as usize {
            panic!("The string '{}' is too long. Encoded string may only have 65536 charachters.");
        }

        let bytes = string.as_bytes();

        let mut encoded = vec![(bytes.len() & 0xff00) as u8, (bytes.len() & 0xff) as u8];
        encoded.extend_from_slice(bytes);
        encoded
    }

    fn encode_registers_array(regs: &[Register]) -> Vec<u8> {
        if regs.len() > u8::max_value() as usize {
            panic!("Too long registers array. Encoded byte arrays may only have 256 elements.");
        }

        let mut encoded = vec![regs.len() as u8];
        encoded.extend_from_slice(regs);
        encoded
    }

    fn encode_long_num(num: u64) -> Vec<u8> {
        vec![(((num & 0xff000000_00000000) >> 56) as u8),
             (((num & 0x00ff0000_00000000) >> 48) as u8),
             (((num & 0x0000ff00_00000000) >> 40) as u8),
             (((num & 0x000000ff_00000000) >> 32) as u8),
             (((num & 0x00000000_ff000000) >> 24) as u8),
             (((num & 0x00000000_00ff0000) >> 16) as u8),
             (((num & 0x00000000_0000ff00) >> 8) as u8),
             (((num & 0x00000000_000000ff) >> 0) as u8)]
    }

    fn encode_float_num(num: f64) -> Vec<u8> {
        Operand::encode_long_num(num.to_bits())
    }
}

#[test]
fn test_encode_string() {
    assert_eq!(Operand::String("Hello World".into()).to_bytes(),
               vec![0, 11, 72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100]);
}

#[test]
fn test_encode_registers_array() {
    assert_eq!(Operand::RegistersArray(vec![]).to_bytes(),
               vec![0]);
   assert_eq!(Operand::RegistersArray(vec![1, 2, 200]).to_bytes(),
              vec![3, 1, 2, 200]);
}

#[test]
fn test_encode_long_num() {
    assert_eq!(Operand::LongNum(1234567890123456789).to_bytes(),
                vec![0x11, 0x22, 0x10, 0xf4, 0x7d, 0xe9, 0x81, 0x15]);

    assert_eq!(Operand::LongNum(-1234567890123456789 as i64).to_bytes(),
                vec![0xEE, 0xDD, 0xEF, 0x0B, 0x82, 0x16, 0x7E, 0xEB])
}

#[test]
fn test_encode_float_num() {
    assert_eq!(Operand::FloatNum(0.12345).to_bytes(),
                vec![63, 191, 154, 107, 80, 176, 242, 124]);

    assert_eq!(Operand::FloatNum(-1.1234).to_bytes(),
                vec![191, 241, 249, 114, 71, 69, 56, 239])
}

#[derive(Debug, PartialEq, Clone)]
pub struct Command
{
    pub instruction: Instruction,
    pub operands: Vec<Operand>
}

impl Command {
    pub fn new(instruction: Instruction, operands: Vec<Operand>) -> Self {
        Command {
            instruction,
            operands
        }
    }
}

impl ToBytes for Command {
    fn to_bytes(&self) -> Vec<u8> {
        let mut line = vec![self.instruction.to_byte()];
        line.append(&mut self.operands.iter().map(|operand| operand.to_bytes()).flatten().collect::<Vec<u8>>());
        line
    }
}

#[test]
fn test_command() {
    assert_eq!(Command{
        instruction: Instruction::Add,
        operands:vec![
            Operand::Reg(150),
            Operand::Reg(151),
        ]
    }.to_bytes(),
    vec![100, 150, 151]);
}


pub type Label = u32;

#[derive(Debug, PartialEq, Clone)]
pub enum BytecodeElement
{
    Command(Command),
    Label(Label)
}

impl ToBytes for BytecodeElement {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            BytecodeElement::Command(cmd) => cmd.to_bytes(),
            BytecodeElement::Label(_) => vec![]
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Bytecode
{
    pub elements: Vec<BytecodeElement>,
}

impl std::fmt::Display for Bytecode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "TO{0}", "DO")
    }
}

impl Bytecode {

    pub fn new() -> Self {
        Bytecode {
            elements: vec![]
        }
    }

    pub fn add(mut self, command: Command) -> Self {
        self.elements.push(BytecodeElement::Command(command));
        self
    }

    pub fn add_label(mut self, label: Label) -> Self {
        self.elements.push(BytecodeElement::Label(label));
        self
    }

    pub fn combine(mut self, mut other: Bytecode) -> Self {
        self.elements.append(&mut other.elements);
        self
    }

    pub fn encode(&self) -> String {
        base64::encode(&self.to_bytes())
    }

    pub fn last_op_is_return(&self) -> bool {
        match self.elements.last() {
            Some(last_element) => match last_element {
                BytecodeElement::Command(cmd) => (cmd.instruction == Instruction::ReturnBytecodeFunc),
                _ => false
            },
            None => false
        }
    }

    pub fn commands_iter_mut(&mut self) -> impl std::iter::Iterator<Item = &mut Command> {
        self.elements.iter_mut().filter_map(|element| match element {
            BytecodeElement::Command(cmd) => Some(cmd),
            BytecodeElement::Label(_) => None
        })
    }
}

impl FromIterator<Bytecode> for Bytecode {
    fn from_iter<I: IntoIterator<Item=Bytecode>>(iter: I) -> Self {
        Bytecode {
            elements: iter.into_iter().flat_map(|bc| bc.elements).collect()
        }
    }
}

impl ToBytes for Bytecode {
    fn to_bytes(&self) -> Vec<u8> {
        self.elements.iter().map(|element| element.to_bytes()).flatten().collect()
    }
}


#[test]
fn test_bytecode_to_bytes() {
    assert_eq!(Bytecode::new().to_bytes().len(), 0);
    assert_eq!(Bytecode{ elements: vec![
        BytecodeElement::Command(Command{
            instruction: Instruction::LoadNum,
            operands: vec![
                Operand::Reg(151),
                Operand::ShortNum(2),
            ]
        }),
        BytecodeElement::Command(Command{
            instruction: Instruction::LoadNum,
            operands: vec![
                Operand::Reg(150),
                Operand::ShortNum(3),
            ]
        }),
        BytecodeElement::Command(Command{
            instruction: Instruction::Mul,
            operands: vec![
                Operand::Reg(150),
                Operand::Reg(151),
            ]
        }),
        ]
    }.to_bytes(), vec![2, 151, 2, 2, 150, 3,101, 150, 151]);
}

#[test]
fn test_last_op_is_return() {
    assert_eq!(Bytecode::new().last_op_is_return(), false);
    assert_eq!(Bytecode::new().add(Command::new(Instruction::ReturnBytecodeFunc, vec![])).last_op_is_return(), true);
    assert_eq!(Bytecode::new()
                .add(Command::new(Instruction::Copy, vec![Operand::Reg(0), Operand::Reg(1)]))
                .add(Command::new(Instruction::ReturnBytecodeFunc, vec![])).last_op_is_return(), true);
    assert_eq!(Bytecode::new().add(
            Command::new(Instruction::Copy, vec![Operand::Reg(0), Operand::Reg(1)])
        ).last_op_is_return(), false);
}
