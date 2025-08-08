# Cog-Os
Commands:

To run commands, open commands.txt and place each command you want to run on a new line.
(To see availiable commands go to src\comammands and read names, mod matches commands to file)

Install:

You first want to navigate to the install and run:
cargo bootimage

After it compiles you want to run (using qemu):
qemu-system-x86_64 -drive format=raw,file=target/x86_64-simple_kernel/debug/bootimage-simple-kernel.bin

***Make Sure qemu is installed***

This will launch Cog OS
