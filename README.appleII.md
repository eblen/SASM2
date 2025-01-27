SASM2 supports creating and running Apple II programs using OpenEmulator:

https://openemulator.github.io/

Here are steps to run the moo example program:

1) Start OpenEmulator and select the "Apple II". This machine is easiest to use because it begins in the system monitor.
2) Switch the monitor to the "Composite color monitor" since the game makes use of multiple colors.
3) Assemble the moo example program with SASM2 and output it in the Apple II system monitor format. For example, if all files are in the same directory: "./sasm2 -i ./moo.asm -f apple" (-s is not needed, because Apple II is the default system.)
4) Cut and paste the output to OpenEmulator. You should hear lots of beeps! (You may want to make sure the volume is turned down to avoid startling anyone around you.) Be sure to press enter to enter the last line of hex values to the system monitor. Be careful to cut and paste the output exactly. Cutting and pasting from an editor, such as vim, may add extra spaces or other characters and silently fail. The system monitor is picky...
5) Type "A00G" and hit enter. The game should now start. You will only see a black screen, but you can now begin typing in 4-digit guesses.

"A00G" is a system monitor command that tells the Apple II to jump to address "A00" and execute the code found there. "A00" is the address where the moo program is loaded (specified in the program with the "org a00" command).

Congratulations! This should get you started writing programs for the Apple II in SASM2! There are many references available online for learning the ins and outs of this wonderful machine.

