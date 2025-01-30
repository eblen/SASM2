SASM2 supports creating and running Atari 2600 programs using the Stella emulator:

https://stella-emu.github.io/

Stella expects as input binary files of a certain size that represent the game ROM. The "-s atari -f bin" command-line arguments tell SASM2 to output code for the Atari 2600 in the binary format. The -s flag changes how zero-page memory is allocated. See the "zpm.d" source file for details. See the TIP section for how to ensure the correct size.

In the "example" directory, there is a simple Atari 2600 kernel that displays scrolling color bars. It can serve as a skeleton for your own Atari 2600 project. Here are steps to run it using Stella:

1) Install Stella so that you can run it from the command line, say as "stella".
2) Assemble the example kernel with SASM2. For example, if all files are in the same directory: "./sasm -i ./atari2600_sample_kernel.asm -o ./atari.bin -s atari -f bin".
3) Run it with stella: "stella atari.bin".

Congratulations! You now have a simple tool chain for creating and running your very own Atari 2600 game. Consult the many resources on the Internet to learn more.

TIP
The binary file sizes must be exact for Stella. For binary output, SASM2 will fill in areas between code sections (separated by "org" commands) with "filler bytes" (currently 0xff) but does not add additional bytes otherwise. Thus, use "org" to force exact binary sizes. For example, to force SASM2 to fill to the end of memory (0xffff), place the following at the end of the program:

org ffff  
data ff
