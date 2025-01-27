# SASM2 Simple Assembler for 6502 Programs reimplemented in Rust

This assembler supports a simple syntax and basic functionality for creating 6502 programs. It is designed as a "lower-level" assembler that strives to stay as close to machine language (ML) as possible, removing only those aspects of ML that make it impractical to use. This includes:

1) Entering opcodes: It is impractical to memorize or lookup hex codes for all of the opcodes and their different flavors. SASM uses mnemonics that map one-to-one with 6502 opcodes. So the programmer knows exactly what ML instruction is being used but without memorizing hex values. SASM uses the common three-letter abbreviations of other assemblers and appends modifiers. Thus, the programmer can take advantage of current knowledge and use 0-2 modifier flags to indicate a specific flavor of an instruction. For example, "stazx" indicates the familiar store instruction, "sta", on a zero-page address and using the X register as an offset. Additionally, using these mnemonics allows the remaining arguments to be pure numerical values, leading to a streamlined, fixed format that avoids the extra line noise of other assemblers.

2) Computing addresses: It is also impractical to expect the programmer to compute and constantly adjust addresses while writing and editing ML programs. Thus, SASM supports labels like other assemblers. To support labels, SASM uses a period to distinguish them from mnemonics or numerical data.

3) Zero page addresses. This is perhaps unique to the 6502. It is essential that the programmer be able to specify and use zero-page addresses for heavily-used memory. It is architecture-specific, though, which zero-page addresses are available and how they should be allocated for use. Thus, SASM offers a special syntax to allocate zero-page bytes and leaves it up to the assembler to properly allocate them.

See the following READMEs for more information:
README.appleII: Instructions for running SASM programs on OpenEmulator
README.atari2600: Instructions for running SASM programs on Stella
README.moo: Instructions for playing the example game program
README.sasm: Instructions for using the SASM assembly language

# Changes from SASM

Mostly, any assembly file that works in SASM works in SASM2, but there are a few minor differences:

1) All two-byte addresses should be written in the more intuitive big-endian format. In the original SASM, unlabeled addresses in an instruction had to be little-endian, which was different for labeled addresses. Note that the "moo.asm" example program has been modified in two places for this reason.

2) All numeric literals are now parsed as hex values, including zbyte lengths, offsets, etc., which removes another inconsistency from SASM. Remember to think in hex when writing SASM2 code.

3) Labels can now be used anywhere in an instruction in place of literal values. The original SASM had limits on where labels could be used. Note that zbyte names and code point markers are also considered labels and can be used in the same way as normal labels.

The "src/syntax.rs" file provides a good overview of the assembly syntax.

# Usage

Since the program is written in Rust, compilation can be done using the Rust cargo commands.

SASM2 accepts five command-line flags, all of which are optional:
-h: This help message
-i: Input  file (STDIN  is default)
-o: Output file (STDOUT is default)
-s: System:
    apple: Apple II (default)
    atari: Atari 2600
-f: Code output format:
    hex:   String of hex digits (default)
    apple: Apple II system monitor
    bin:   Machine code

The system flag currently only affects how zero-page addresses are assigned. In a nutshell, for the Apple II they are assigned from 0xff down. For the Atari 2600 they are assigned from 0x80 up. See code comments in "zpm.rs" for more information.

The format flag sets how the final result is output. The hex format is mainly for humans to study. It can help in learning and testing the assembler. The Apple II system monitor format can be copied and pasted directly into the Apple II system monitor on an emulator. See the Apple II README for more details. Finally, the bin format is binary code that can be run directly in an emulator such as Stella.

# Notes on Rust implementation

This version of SASM vastly improves on the original in terms of code design. It leans heavily on Rust's advanced enums to implement a simpler and more modular design. This is my first project in Rust, and I routinely spend long hours with the Rust compiler, but I continue to be impressed with how clean and robust the code is once it finally compiles! I have much more confidence that this version will work correctly, even though the original was written in D, which was my favorite language at the time.

