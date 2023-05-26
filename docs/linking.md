# Linking
Given a Wasm module, the compilation toolchain outputs a ELF file. These ELF
file follows a very specific format: all the relocation entries are absolute 
64-bit addressses. This is not needed for most other ELFs that will be linked
and run in a "normal" OS, such as your own laptop, since in the final compiled
image, code sections are normally placed in the same half of 32-bit address
space, and relocations can be done as 32-bit relative addresses. However, the
memory allocated to ELFs by Fix may not be in the same half as Fix's code
sections. Therefore, any reloaction between the ELFs and Fix API functions needs
to be in 64-bit absolute address.

These relocation entries are generated by `clang`, and controlled by two
options: code model and relocation model. To get absolute 64-bit address
relocation entries, we currently have the code model as "large", and the
relocation model as "static".

Given this ELF, what Fix does is essentially following the specifications of how
ELF should be processed for X86\_64. There are several things that could make
this linking process better.

1. Since all relocation entries are 64-bit absolute addresses, value of the
   relocation entries would depend on where the program is placed in the memory.
   Therefore, the same ELF program needs to be relinked for different executions
   of Fix.
   
   However, if we can have a ELF where any function call to other functions belong 
   to the ELF have relocation entries as relative 32-bit addresses and any function 
   call to Fix API functions as something else, then any in-ELF linking can be
   done, and we would only need to relink function calls to Fix API function
   calls.

   When `wasm2c` and `initcomposer` outputs are compiled to ELF under different
   code models and relocation models, the ELF would have relative 32-bit
   addresses for function calls, and go through "GLOBAL OFFSET TABLE". Figuring
   that out, or how relocation looks like for global arrays would probably make
   linking needed on the critical path faster.

2. If someone runs Fix in `gdb`, we are not able to break on code within the
   input ELF, and if the ELF throws an exception, the backtrace does not contain
   much helpful information. This is because `gdb` does not register the debug
   info of the input ELF as the it comes in after `gdb` starts running.

   `gdb` exposes [API function calls](https://sourceware.org/gdb/onlinedocs/gdb/JIT-Interface.html) 
   for registering JIT code debug infos. Ideally, if we register the ELF in a 
   similar way, `gdb` should be able to handle debug info for input ELFs correctly.