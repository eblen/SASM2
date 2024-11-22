// Enums for tokenizing source code lines

pub enum UInt {
    U8(u8),
    U16(u16),
}

pub enum Offset {
    Byte(u8),
    Label(String),
}

pub enum SourceLine {
    // Empty lines after removing comments
    Blank,

    // Keywords
    Org(u16),
    Label(String, UInt),
    Zbyte(String, u8),
    Data(Vec<u8>),

    // Isolated labels
    LabelCodeLocation(String),

    // Instruction lines
    Instr(String, UInt),
    InstrWithLabel(String, String, Offset),
}
