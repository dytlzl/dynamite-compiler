# dynamite-compiler

Compiler for a language with syntax similar to the C language, written in Rust

## Architecture

This program adopts pipeline architecture and processes source code as input in the flow depicted in the diagram below, resulting in assembly language output.

```mermaid
flowchart TD
    src([Source Code])
    tokenizer[Tokenizer]
    builder[AST Builder]
    generator[Assembly Generator]
    assembly([Assembly Language])

    src --> tokenizer --> builder --> generator --> assembly
```


