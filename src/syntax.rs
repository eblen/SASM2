// Enums for tokenizing source code lines

#[derive(Copy, Clone)]
pub enum UInt {
    U8(u8),
    U16(u16),
}

pub enum Op {
    UInt(UInt),
    Label(String),
    None,
}

pub enum Offset {
    U8(u8),
    Label(String),
}

pub enum Rawdata {
    Bytes(Vec<u8>),
    Label(String),
}

pub enum SourceLine {
    // Empty lines after removing comments
    Blank,

    // Keywords
    Org(u16),
    Label(String, UInt),
    ZByte(String, u8),
    Data(Rawdata),

    // Isolated labels
    CodeMarker(String),

    // Instruction lines
    Instr(String, Op, Offset),
}
