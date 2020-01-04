# Lox

This is a rust implementation of the lox programming language. It is different from the reference implementation in that it parses to an AST first, then compiles that to bytecode.
This allows for possible optimizations later.

There is also a string seperation between compiler and VM. This makes it possible to have a seperate compiler and a 'compiled' binary format.

Right now it is very much in the development phase.