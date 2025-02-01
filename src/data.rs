use std::collections::HashMap;
use std::sync::LazyLock;

pub fn get_instr_info(mnemonic: &str) -> Result<&InstrInfo, &str> {
    match ISA_BY_MNEMONIC.get(mnemonic) {
        Some(i) => Ok(i),
        // TODO: Detailed errors about unsupported or missing flags
        None => Err("mnemonic not found"),
    }
}

pub fn get_instr_size(mnemonic: &str) -> Result<u8, &str> {
    match ISA_BY_MNEMONIC.get(mnemonic) {
        Some(i) => match i.op {
            OpType::None => Ok(1),
            OpType::U8 => Ok(2),
            OpType::U16 => Ok(3),
        },
        None => Err("mnemonic not found"),
    }
}

pub fn get_instr_info_from_opcode(opcode: u8) -> Option<&'static InstrInfo> {
    return ISA_BY_OPCODE[opcode as usize];
}

pub fn get_instr_size_from_opcode(opcode: u8) -> Option<u8> {
    match ISA_BY_OPCODE[opcode as usize] {
        Some(i) => match i.op {
            OpType::None => Some(1),
            OpType::U8 => Some(2),
            OpType::U16 => Some(3),
        },
        None => None,
    }
}
pub fn is_relative_branch_instruction(mnemonic: &str) -> bool {
    let instrs = ["bpl", "bmi", "bvc", "bvs", "bcc", "bcs", "bne", "beq"];
    return instrs.contains(&mnemonic.to_lowercase().as_str());
}

pub struct InstrInfo {
    pub mnemonic: String,
    pub opcode: u8,
    pub op: OpType,
}

pub enum OpType {
    U8,
    U16,
    None,
}

fn new_instr(mnemonic: &str, opcode: u8, op: OpType) -> (String, InstrInfo) {
    (mnemonic.to_string(), InstrInfo { mnemonic: mnemonic.to_string(), opcode, op })
}

static ISA_BY_MNEMONIC: LazyLock<HashMap<String, InstrInfo>> = LazyLock::new(|| {
    HashMap::from([
        new_instr("adci", 0x69, OpType::U8),
        new_instr("adcz", 0x65, OpType::U8),
        new_instr("adczx", 0x75, OpType::U8),
        new_instr("adca", 0x6d, OpType::U16),
        new_instr("adcax", 0x7d, OpType::U16),
        new_instr("adcay", 0x79, OpType::U16),
        new_instr("adcnx", 0x61, OpType::U8),
        new_instr("adcny", 0x71, OpType::U8),
        new_instr("andi", 0x29, OpType::U8),
        new_instr("andz", 0x25, OpType::U8),
        new_instr("andzx", 0x35, OpType::U8),
        new_instr("anda", 0x2d, OpType::U16),
        new_instr("andax", 0x3d, OpType::U16),
        new_instr("anday", 0x39, OpType::U16),
        new_instr("andnx", 0x21, OpType::U8),
        new_instr("andny", 0x31, OpType::U8),
        new_instr("asl", 0x0a, OpType::None),
        new_instr("aslz", 0x06, OpType::U8),
        new_instr("aslzx", 0x16, OpType::U8),
        new_instr("asla", 0x0e, OpType::U16),
        new_instr("aslax", 0x1e, OpType::U16),
        new_instr("bitz", 0x24, OpType::U8),
        new_instr("bita", 0x2c, OpType::U16),
        new_instr("bpl", 0x10, OpType::U8),
        new_instr("bmi", 0x30, OpType::U8),
        new_instr("bvc", 0x50, OpType::U8),
        new_instr("bvs", 0x70, OpType::U8),
        new_instr("bcc", 0x90, OpType::U8),
        new_instr("bcs", 0xb0, OpType::U8),
        new_instr("bne", 0xd0, OpType::U8),
        new_instr("beq", 0xf0, OpType::U8),
        new_instr("brk", 0x00, OpType::None),
        new_instr("cmpi", 0xc9, OpType::U8),
        new_instr("cmpz", 0xc5, OpType::U8),
        new_instr("cmpzx", 0xd5, OpType::U8),
        new_instr("cmpa", 0xcd, OpType::U16),
        new_instr("cmpax", 0xdd, OpType::U16),
        new_instr("cmpay", 0xd9, OpType::U16),
        new_instr("cmpnx", 0xc1, OpType::U8),
        new_instr("cmpny", 0xd1, OpType::U8),
        new_instr("cpxi", 0xe0, OpType::U8),
        new_instr("cpxz", 0xe4, OpType::U8),
        new_instr("cpxa", 0xec, OpType::U16),
        new_instr("cpyi", 0xc0, OpType::U8),
        new_instr("cpyz", 0xc4, OpType::U8),
        new_instr("cpya", 0xcc, OpType::U16),
        new_instr("decz", 0xc6, OpType::U8),
        new_instr("deczx", 0xd6, OpType::U8),
        new_instr("deca", 0xce, OpType::U16),
        new_instr("decax", 0xde, OpType::U16),
        new_instr("eori", 0x49, OpType::U8),
        new_instr("eorz", 0x45, OpType::U8),
        new_instr("eorzx", 0x55, OpType::U8),
        new_instr("eora", 0x4d, OpType::U16),
        new_instr("eorax", 0x5d, OpType::U16),
        new_instr("eoray", 0x59, OpType::U16),
        new_instr("eornx", 0x41, OpType::U8),
        new_instr("eorny", 0x51, OpType::U8),
        new_instr("clc", 0x18, OpType::None),
        new_instr("sec", 0x38, OpType::None),
        new_instr("cli", 0x58, OpType::None),
        new_instr("sei", 0x78, OpType::None),
        new_instr("clv", 0xb8, OpType::None),
        new_instr("cld", 0xd8, OpType::None),
        new_instr("sed", 0xf8, OpType::None),
        new_instr("incz", 0xe6, OpType::U8),
        new_instr("inczx", 0xf6, OpType::U8),
        new_instr("inca", 0xee, OpType::U16),
        new_instr("incax", 0xfe, OpType::U16),
        new_instr("jmpa", 0x4c, OpType::U16),
        new_instr("jmpn", 0x6c, OpType::U8),
        new_instr("jsra", 0x20, OpType::U16),
        new_instr("ldai", 0xa9, OpType::U8),
        new_instr("ldaz", 0xa5, OpType::U8),
        new_instr("ldazx", 0xb5, OpType::U8),
        new_instr("ldaa", 0xad, OpType::U16),
        new_instr("ldaax", 0xbd, OpType::U16),
        new_instr("ldaay", 0xb9, OpType::U16),
        new_instr("ldanx", 0xa1, OpType::U8),
        new_instr("ldany", 0xb1, OpType::U8),
        new_instr("ldxi", 0xa2, OpType::U8),
        new_instr("ldxz", 0xa6, OpType::U8),
        new_instr("ldxzy", 0xb6, OpType::U8),
        new_instr("ldxa", 0xae, OpType::U16),
        new_instr("ldxay", 0xbe, OpType::U16),
        new_instr("ldyi", 0xa0, OpType::U8),
        new_instr("ldyz", 0xa4, OpType::U8),
        new_instr("ldyzx", 0xb4, OpType::U8),
        new_instr("ldya", 0xac, OpType::U16),
        new_instr("ldyax", 0xbc, OpType::U16),
        new_instr("lsr", 0x4a, OpType::None),
        new_instr("lsrz", 0x46, OpType::U8),
        new_instr("lsrzx", 0x56, OpType::U8),
        new_instr("lsra", 0x4e, OpType::U16),
        new_instr("lsrax", 0x5e, OpType::U16),
        new_instr("nop", 0xea, OpType::None),
        new_instr("orai", 0x09, OpType::U8),
        new_instr("oraz", 0x05, OpType::U8),
        new_instr("orazx", 0x15, OpType::U8),
        new_instr("oraa", 0x0d, OpType::U16),
        new_instr("oraax", 0x1d, OpType::U16),
        new_instr("oraay", 0x19, OpType::U16),
        new_instr("oranx", 0x01, OpType::U8),
        new_instr("orany", 0x11, OpType::U8),
        new_instr("tax", 0xaa, OpType::None),
        new_instr("txa", 0x8a, OpType::None),
        new_instr("dex", 0xca, OpType::None),
        new_instr("inx", 0xe8, OpType::None),
        new_instr("tay", 0xa8, OpType::None),
        new_instr("tya", 0x98, OpType::None),
        new_instr("dey", 0x88, OpType::None),
        new_instr("iny", 0xc8, OpType::None),
        new_instr("rol", 0x2a, OpType::None),
        new_instr("rolz", 0x26, OpType::U8),
        new_instr("rolzx", 0x36, OpType::U8),
        new_instr("rola", 0x2e, OpType::U16),
        new_instr("rolax", 0x3e, OpType::U16),
        new_instr("ror", 0x6a, OpType::None),
        new_instr("rorz", 0x66, OpType::U8),
        new_instr("rorzx", 0x76, OpType::U8),
        new_instr("rora", 0x6e, OpType::U16),
        new_instr("rorax", 0x7e, OpType::U16),
        new_instr("rti", 0x40, OpType::None),
        new_instr("rts", 0x60, OpType::None),
        new_instr("sbci", 0xe9, OpType::U8),
        new_instr("sbcz", 0xe5, OpType::U8),
        new_instr("sbczx", 0xf5, OpType::U8),
        new_instr("sbca", 0xed, OpType::U16),
        new_instr("sbcax", 0xfd, OpType::U16),
        new_instr("sbcay", 0xf9, OpType::U16),
        new_instr("sbcnx", 0xe1, OpType::U8),
        new_instr("sbcny", 0xf1, OpType::U8),
        new_instr("staz", 0x85, OpType::U8),
        new_instr("stazx", 0x95, OpType::U8),
        new_instr("staa", 0x8d, OpType::U16),
        new_instr("staax", 0x9d, OpType::U16),
        new_instr("staay", 0x99, OpType::U16),
        new_instr("stanx", 0x81, OpType::U8),
        new_instr("stany", 0x91, OpType::U8),
        new_instr("txs", 0x9a, OpType::None),
        new_instr("tsx", 0xba, OpType::None),
        new_instr("pha", 0x48, OpType::None),
        new_instr("pla", 0x68, OpType::None),
        new_instr("php", 0x08, OpType::None),
        new_instr("plp", 0x28, OpType::None),
        new_instr("stxz", 0x86, OpType::U8),
        new_instr("stxzy", 0x96, OpType::U8),
        new_instr("stxa", 0x8e, OpType::U16),
        new_instr("styz", 0x84, OpType::U8),
        new_instr("styzx", 0x94, OpType::U8),
        new_instr("stya", 0x8c, OpType::U16),
    ])
});

static ISA_BY_OPCODE: LazyLock<[Option<&InstrInfo>; 256]> = LazyLock::new(|| {
    let mut a = [None; 256];
    for (_, instr) in ISA_BY_MNEMONIC.iter() {
        a[instr.opcode as usize] = Some(instr);
    }
    a
});
