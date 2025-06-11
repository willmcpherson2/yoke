# Low-level Intermediate Representation

Yoke is compiled to a low-level intermediate representation (LIR).
The LIR is primarily used by the compiler internally, but it is a real language you can pass to the compiler.

**The LIR is an unsafe language**.
The compiler will resolve variables and run the LLVM verification pass, but other than that, it is essentially assembly.

The LIR is uni-typed: everything is a `Term`.
See the [RTS docs](./rts.md).

## Globals

Constructors and functions must be defined statically.
The order of definitions doesn't matter, but if a name is defined multiple times, only the last definition is used.

### Constructors

```
name = arity symbol
```

For example, to define `True`:

```
True = 0 1
```

Here, `True` is a constructor with `0` arguments (so just a symbol), and the interned integer value of the symbol is `1`.

### Functions

```
name = arity block
```

For example, here's a program which returns `1`:

```
True = 0 1

main = 0 {
  load_global True
  return_symbol True
}
```

`main` must have an arity of `0`.

All functions return terms except `main` which returns a symbol.

## Self

In the [runtime system](./rts.md), all terms have a `fun` field containing a function pointer.
When a term is evaluated, it is passed by reference to its own `fun`.
The function stores its results by writing to itself.
This argument is available as a local called `self`.
`self` is not available in `main`.

Here's an example program which implements `const True False`:

```
True = 0 1

False = 0 2

main = 0 {
  load_global const
  load_global True
  load_global False
  x = new_app const { True False }
  eval x
  return_symbol x
}

const = 2 {
  x = load_arg self 0
  return x
}
```

`main` returns a symbol, whereas `const` returns a term.

`const` is able to access its first argument by indexing `self`.

## Instructions

### load\_global

```
load_global global
```

The `load_global` instruction loads a global into a local variable with the same name.
Globals must be loaded before they are accessed.

### load\_arg

```
name = load_arg local index
```

The `load_arg` instruction loads an argument from a local variable into a new local variable.

### new\_app

```
name = new_app local { local... }
```

The `new_app` instruction creates a new term with some arguments (an application).
This cannot be a partial application.

This instruction allocates.

### new\_partial

```
name = new_partial local { local... }
```

The `new_partial` instruction creates a new term with some arguments in partial application format.

This instruction allocates.

### apply\_partial

```
name = apply_partial local { local... }
```

The `apply_partial` instruction takes an existing partial application and applies additional arguments.

This instruction does not allocate.

### copy

```
name = copy local
```

The `copy` instruction performs a deep copy of a term and stores it as a local variable.

### eval

```
eval local
```

The `eval` instruction evaluates a term in-place.
Evaluation involves passing the term to its own `fun`.

### free\_args

```
free_args local
```

The `free_args` instruction deallocates the `args` buffer of the term.

### free\_term

```
free_term local
```

The `free_term` instruction recursively deallocates a term.
This includes the `args` buffer and every term in the `args` buffer.

### return\_symbol

```
return_symbol local
```

The `return_symbol` instruction returns the symbol of a term.
This is only valid in `main`.

### return

```
return local
```

The `return` instruction returns a term from a function.
This is not valid in `main`.

### switch

```
switch local { case... }
```

The `switch` instruction provides structured control flow.
It takes a term and a list of cases, each containing the name of a constructor and a block.
Control is passed to the block of the case containing a constructor with the same symbol as the term.

The globals in the cases don't need to be loaded.

There is no fallthrough and each case must return.

Here's an implementation of `not True`:

```
True = 0 1

False = 0 2

main = 0 {
  load_global True
  switch True {
    True {
      load_global False
      return_symbol False
    }
    False {
      return_symbol True
    }
  }
}
```

### todo

```
todo
```

The `todo` instruction halts the program with exit code `3`.
This is used to implement case expressions with unimplemented alternatives.
