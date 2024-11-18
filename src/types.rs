// Enums for tokenizing source code lines

pub enum UInt {
    U8(u8),
    U16(u16),
}

pub enum Offset {
    Byte(u8),
    Label(String),
}

pub enum SourceLine<'a> {
    // Empty lines after removing comments
    Blank,

    // org, label, and zbyte keywords
    Org(u16),
    Label(String, UInt),
    Zbyte(String, u8),

    // Isolated labels
    LabelCodeLocation(String),

    // Instruction lines
    Instr(String, UInt),
    InstrWithLabel(String, String, Offset),

    // Lines starting with "data" keyword
    Rawdata(&'a [u8]),
}
