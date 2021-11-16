# mirlvm

## Overview
Mirlvm is the tiny compiler infrastructure(Developing).
<br>Mirlvm consists of several pass and converts the input intermediate language program (IR) into an x86_64 program.

## Examples
An example of an input IR is shown below, and there is currently no document that explains the entire grammar.
```
# compute 50'st prime number

function $main() {
@start:
@loop:
	%n =w phi @start 5, @tloop %n, @yes %n1
	%p =w phi @start 13, @tloop %p1, @yes %p1
	%p1 =w add %p, 2
@tloop:
	%t =w phi @loop 3, @next %t1
	%r =w rem %p, %t
	jnz %r, @next, @loop
@next:
	%t1 =w add 2, %t
	%tsq =w mul %t1, %t1
	%c0 =w csgtw %tsq, %p
	jnz %c0, @yes, @tloop
@yes:
	%n1 =w add 1, %n
	%c1 =w ceqw 50, %n1
	jnz %c1, @end, @loop
@end:
	ret %p
}
```

## Compenents
The main process for each pass of mirlvm is as follows:

1. Lexical and syntactic analysis pass
<br> - Generate AST by performing lexical and syntactic analysis on IRs. 2.
2. Control Flow Graph Analysis pass
<br> - Create a control flow graph for the AST, and calculate its dominator tree and dominator frontiers.
3. MemToRegister pass
<br> - Using the calculation results of 2, convert memory access instructions to register access as much as possible. (After this pass, the IR is converted to prund-SSA format.)
4. SSA optimization pass
<br> - Perform various optimizations using the SSA format. (Currently, there are only a few optimizations implemented, and further implementation is needed in the future.)
5. SSA inverse conversion pass
<br> - Because of the presence of Phi functions in SSA programs, it is difficult to convert them directly into lower-level programs. Therefore, the Phi function is removed and the program is converted to normal format.
6. LIR conversion pass
<br> - Convert IR to lower level Low IR (LIR). This path assumes that there is an infinite number of registers.
7. Register Allocation pass
<br> - Allocates physical registers to LIR by performing register allocation.
8. Code generation pass
<br> - Generates the program for x86_64 assembly.

## Security
mirlvm currently has the following security features.
- Signed integer overflow detection

	 The following assembly instruction sequence causes an overflow.
	 ```
	 mov r10d, 44344433
	 mov r11d, 54343434
	 mov ebx, r11d
	 mov eax, ebx
	 imul r10d
	 ```
	 When such an instruction sequence is executed, it detects the overflow, outputs an error message, and forcibly terminates.
## build
Execute the command below to generate the executable file `a.out`.
If you want to run in secure mode, add `OPTION3=-Sec`.

    $ make debug OPTION2=-O1 SSAFILE=file_name
    
    
