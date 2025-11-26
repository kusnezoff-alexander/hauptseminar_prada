use crate::prada::architecture::PRADAArchitecture;

use super::{BitwiseOperand, BitwiseRow, Row, Rows};
use eggmock::{Id, Mig, NetworkWithBackwardEdges, Signal};
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Address {
    In(u64),
    Out(u64),
    Spill(u32),
    Const(bool),
    Bitwise(BitwiseAddress),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BitwiseAddress {
    Single(BitwiseOperand),
    Multiple(usize),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Instruction {
    /// Row Copy to a single destination
    AAPRowCopy(Address, Address),
    /// TRA on given Addresses
    AAPTRA(Address, Address, Address),
    /// Negate operand in row
    N(Address),
}

#[derive(Debug, Clone)]
pub struct Program<'a> {
    pub architecture: &'a PRADAArchitecture,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub struct ProgramState<'a> {
    program: Program<'a>,
    rows: Rows<'a>,
}

impl<'a> Program<'a> {
    pub fn new(architecture: &'a PRADAArchitecture, instructions: Vec<Instruction>) -> Self {
        Self {
            architecture,
            instructions,
        }
    }
}

impl Instruction {
    pub fn used_addresses<'a>(
        &self,
    ) -> impl Iterator<Item = Address> + 'a {
        match self {
            Instruction::AAPRowCopy(from, _) => vec!(*from).into_iter(),
            Instruction::AAPTRA(a, b, c ) => vec!(*a,*b,*c).into_iter(),
            Instruction::N(a) => vec!(*a).into_iter(),
        }
    }

    pub fn input_operands<'a>(
        &self,
    ) -> impl Iterator<Item = Address> + 'a {
        match self {
            Instruction::AAPRowCopy(from, _) => vec!(*from).into_iter(),
            Instruction::AAPTRA(a, b, c ) => vec!(*a,*b,*c).into_iter(),
            Instruction::N(a) => vec!(*a).into_iter(),
        }
    }

    /// Return which rows are to be overridden
    pub fn overridden_rows<'a>(
        &self,
        architecture: &'a PRADAArchitecture,
    ) -> impl Iterator<Item = Row> + 'a {
        return vec!().into_iter();
        todo!();
        // match self {
        //     Instruction::AAPRowCopy(from, _) => vec!(*from).into_iter(),
        //     Instruction::AAPTRA(a, b, c ) => vec!(*a,*b,*c).into_iter(),
        //     Instruction::N(a) => vec!(*a).into_iter(),
        // }
    }
}

impl<'a> ProgramState<'a> {
    pub fn new(
        architecture: &'a PRADAArchitecture,
        network: &impl NetworkWithBackwardEdges<Node = Mig>,
    ) -> Self {
        Self {
            program: Program::new(architecture, Vec::new()),
            rows: Rows::new(network, architecture),
        }
    }

    /// TODO: implement maj3,maj5,maj6,maj10 support (see SRA in PRADA paper)
    pub fn maj(&mut self, op: usize, out_signal: Signal, out_address: Option<Address>) {
        // Just a placeholder for now... .
        let instruction = Instruction::AAPTRA(Address::Out(0), Address::In(0), Address::In(0));
        self.instructions.push(instruction)
    }

    pub fn signal_copy(&mut self, signal: Signal, target: Address) {
        {
            // if any row contains the signal, then this is easy, simply copy the row into the
            // target operand
            let signal_row = self.rows.get_rows(signal).next();
            if let Some(signal_row) = signal_row {
                self.set_signal(target, signal);
                self.instructions
                    .push(Instruction::AAPRowCopy(signal_row.into(), target.into()));
                return;
            }
        }
        // otherwise we need to search the inverted signal and take a DCC row if possible, otherwise
        // use the intermediate DCC to invert
        // let mut inverted_signal_row = None;
        let mut inv_instructions = vec!();
        for row in self.rows.get_rows(signal.invert()) {
            inv_instructions.push(Instruction::N(Address::In(0)));
            // TODO: invert operand with `N` DRAM Cmd
        }

        self.instructions.append(&mut inv_instructions);


        return;
        // let inverted_signal_row =
        //     inverted_signal_row.expect("inverted signal row should be present");

    }

    /// Sets the value of the operand in `self.rows` to the given signal. If that removes the last
    /// reference to the node of the previous signal of the operator, insert spill code for the
    /// previous signal
    /// **ALWAYS** call this before inserting the actual instruction, otherwise the spill code will
    /// spill the wrong value
    fn set_signal(&mut self, address: Address, signal: Signal) {
        if let Some(previous_signal) = self.rows.set_signal(address, signal) {
            if !self.rows.contains_id(previous_signal.node_id()) {
                let spill_id = self.rows.add_spill(previous_signal);
                self.instructions
                    .push(Instruction::AAPRowCopy(address.into(), Address::Spill(spill_id)));
            }
        }
    }

    pub fn free_id_rows(&mut self, id: Id) {
        self.rows.free_id_rows(id);
    }

    pub fn rows(&self) -> &Rows {
        &self.rows
    }
}

impl<'a> Deref for ProgramState<'a> {
    type Target = Program<'a>;

    fn deref(&self) -> &Self::Target {
        &self.program
    }
}

impl DerefMut for ProgramState<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.program
    }
}

impl<'a> From<ProgramState<'a>> for Program<'a> {
    fn from(value: ProgramState<'a>) -> Self {
        value.program
    }
}

impl From<BitwiseAddress> for Address {
    fn from(value: BitwiseAddress) -> Self {
        Self::Bitwise(value)
    }
}

impl From<BitwiseOperand> for BitwiseAddress {
    fn from(value: BitwiseOperand) -> Self {
        Self::Single(value)
    }
}

impl From<BitwiseOperand> for Address {
    fn from(value: BitwiseOperand) -> Self {
        Self::Bitwise(value.into())
    }
}

impl From<BitwiseRow> for BitwiseOperand {
    fn from(value: BitwiseRow) -> Self {
        match value {
            BitwiseRow::T(t) => BitwiseOperand::T(t),
        }
    }
}

impl From<Row> for Address {
    fn from(value: Row) -> Self {
        Self::from(value)
    }
}

impl Display for Program<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let write_operand = |f: &mut Formatter<'_>, o: &BitwiseOperand| -> std::fmt::Result {
            match o {
                BitwiseOperand::T(t) => write!(f, "T{t}"),
            }
        };
        let write_address = |f: &mut Formatter<'_>, a: &Address| -> std::fmt::Result {
            match a {
                Address::In(i) => write!(f, "I{i}"),
                Address::Out(i) => write!(f, "O{i}"),
                Address::Spill(i) => write!(f, "S{i}"),
                Address::Const(c) => write!(f, "C{}", if *c { "1" } else { "0" }),
                Address::Bitwise(b) => todo!(),
            }
        };

        for instruction in &self.instructions {
            match instruction {
                Instruction::AAPRowCopy(a, b) => {
                    write!(f, "AAPRowCopy ")?;
                    write_address(f, a)?;
                    write!(f, " ")?;
                    write_address(f, b)?;
                    writeln!(f)?;
                }
                Instruction::AAPTRA(a, b, c) => {
                    write!(f, "AAPTRA ")?;
                    write_address(f, a)?;
                    write!(f, " ")?;
                    write_address(f, b)?;
                    writeln!(f)?;
                    write_address(f, c)?;
                    writeln!(f)?;
                },
                _ => todo!(),
            }
        }
        Ok(())
    }
}
