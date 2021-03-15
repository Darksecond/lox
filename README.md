# Lox

This is a rust implementation of the lox programming language. It is different from the reference implementation in that it parses to an AST first, then compiles that to bytecode.
This allows for possible optimizations later.

There is also a string seperation between compiler and VM. This makes it possible to have a seperate compiler and a 'compiled' binary format.

Right now it is very much in the development phase. All chapters (that so far have been published) in Part 3 of the book Crafting Interpreters have been implemented.
The GC is of my own design, it is currently a simpler black-white mark and sweep GC. I will replace it with a better algorithm in due time.

Also a lot of the internal VM structures are (slightly) different from the reference implementation. Either because I have a different GC design, or because it's more 'rusty'.

Better error reporting is very much a TODO still, especially in the parser where it doesn't provide any position information at all currently.
Also constants are not yet deduplicated in the compiler.

# Test suite

Tests are copied from https://github.com/munificent/craftinginterpreters/tree/master/test.