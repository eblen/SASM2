# SASM2 Disassembler

The disassembler is straightforward to use and works with binaries of any size. It accepts the following flags, all of which are optional:
-h: This help message
-i: Input  file (STDIN  is default)
-o: Output file (STDOUT is default)
-a: Starting address in hex (0x0000 is default)

The -a option will add an org at the top for the given address. It also affects label names, which contain an address.

# Algorithm

Broadly, the disassembler works by finding the largest region of legal code, removing it, finding the next largest, removing it, etc. until region sizes drop below 10. Remaining bytes are considered raw data. The disassembler considers ALL possible sets of legal code. That is, each byte is considered a possible starting point. This seems to be a simple but effective algorithm, but it needs more testing. I will add more details once I've done more experimentation with real-world codes and larger codes.
