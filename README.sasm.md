The SASM assembler has a simple syntax that is based on the well-known syntax used by other 6502 assemblers. See the "moo.asm" program in the "examples" directory for a specific example.

Comments are preceded with a semicolon. Unlike most other assemblers, there is no indenting of lines. Each opcode is 3-5 characters in length. The first three characters are the well-known opcodes for the 6502. One good reference is:

http://www.6502.org/tutorials/6502opcodes.html

The next two characters, if present, are modifiers. The fourth character can be one of:

i: immediate mode  
z: zero-page address  
a: absolute address  
n: indirect address

The fifth character indicates a register offset and is either 'x' or 'y' if present.

Of course, not all instructions accept all modifiers. Some instructions, such as "tax" (transfer a to x), do not accept any modifiers. The assembler will print an error and refuse to compile for illegal mnemonics.

Instructions may take an argument, which will either be a value or a label. All values in the program are hex with no additional markup. (So write "E6", not "0xE6".) Also, all values are unsigned as far as the assembler is concerned. (The 6502 may, of course, interpret them differently.) Labels are prepended with a '.' See below for more information on labels.

Any instruction that takes an argument may also take an offset as a second argument. This offset is added to the first argument to compute the actual op. Offsets are restricted to a single byte and, like other values, always in hex and unsigned (so no negative offsets).

Locations in the program can be labeled with a '.' followed by the label. For example, ".loop_begin" or ".subroutine1". These can then be used for branching instructions (or anywhere else where labels are allowed). Zero-byte addresses can also be labeled using a special command given below.

Other assembler commands:
* data: indicates that the argument is simply data inserted into the program. The argument may be any even number of hex digits or a label, which indicates that the two-byte address itself should be inserted (useful for defining interrupt vectors, for example). Note that one-byte labels are not allowed. A label is assumed to be an address in big-endian format. It will be converted to little endian during assembly (consistent with the rest of SASM2). However, explicit bytes (even if exactly 2) are inserted as is.

* label: assign a label (argument 1) to a one or two-byte value (argument 2). Note that a '.' should not be used before the label in this command but must be used when referring to the label.

* org: Usually the first command in the program, which indicates where the program should be loaded into memory. The argument is 1-4 hex digits.  By default, org is 0x0000, which is generally not wanted except for testing and learning the assembler. Multiple org commands are allowed for defining multiple code segments.

* zbyte: allocate one or more zero-page bytes. The first argument is mandatory and is a label for the memory. The optional second argument indicates the number of bytes to allocate (1 by default) and can only be a single byte. Again, this value must be in hex and unsigned. Like the "label" command, note that a '.' should not be used before the label for the zbyte command but must be used when referring to the label.

NOTES
* You are encouraged to use a fixed format for the program, so that columns 0-4 are for the mnemonic and the argument starts at column 6, but this is not mandatory. At the moment, however, spaces are not allowed inside the mnemonic or argument. For example, "jmp fc58" cannot be written as "jmp fc 58".

* Either uppercase or lowercase can be used for the mnemonics, addresses, and hex values.
