# The Yoke Runtime System

This is the documentation for the runtime system (RTS) of the Yoke language.

The RTS is just a small Rust library which exposes a handful of functions to the compiler: [rts/src/lib.rs](../rts/src/lib.rs)

We use a technique that I call [Inline Your Runtime](https://willmcpherson2.com/2025/05/18/inline-your-runtime.html) which essentially compiles the RTS to LLVM and injects it directly into the same module as the rest of the generated code.
This means that we can happily define as much as possible in the runtime library without missing any optimisations.

## Semantics

Yoke is a pure, functional, lazy programming language.
It is statically, strongly and structurally typed.

Programs don't crash except in the case of non-exhaustive case expressions.

## Terms

The RTS is based on an all-encompassing `Term` type:

```rust
pub struct Term {
    pub fun: extern "C" fn(*mut Term),
    pub args: *mut Term,
    pub symbol: u32,
    pub length: u16,
    pub capacity: u16,
}
```

Terms are used to express symbols like `True`, constructors like `Cons x xs` and function applications like `map not`.

### Symbols

A symbol like `True` is represented like this:

```
{
    fun: noop,
    args: [],
    symbol: 42,
    length: 0,
    capacity: 0,
}
```

In this case, `True` was interned to the value `42`.
Symbols are globally unique, so `42` uniquely identifies our symbol.
These integer values are used in switch statements.

Because it's just a symbol, we leave the `args` pointer null. The corresponding `length` and `capacity` are therefore `0`.

Finally, because a symbol is *data* and is already evaluated, we set the `fun` function pointer to `noop`.
This is a real function which will be called when the term is evaluated, but it does nothing.

### Constructors

A constructor like `Cons x xs` is represented like this:

```
{
    fun: noop,
    args: [x, xs],
    symbol: 144,
    length: 2,
    capacity: 2,
}
```

In this case, `Cons` was interned to `144`.

`args` is a buffer containing the terms `x` and `xs`, so the length and capacity are `2`.

Because Yoke is lazy, we don't evaluate the arguments. Therefore `fun` is `noop`.

### Applications

A function application like `not x` is represented like this:

```
{
    fun: not,
    args: [x],
    symbol: 0,
    length: 1,
    capacity: 1,
}
```

You can't pattern match on a function, so functions always have a symbol of `0`.

`args` is a buffer containing the term `x`.

The `fun` is set to `not`. This is a statically defined function which will be called when the term is evaluated.

### Partial applications

For partial applications, let's look at the successive applications of `map`, `map not` and `map not xs`:

```
{
    fun: noop,
    args: [_, map],
    symbol: 0,
    length: 0,
    capacity: 2,
}

{
    fun: noop,
    args: [not, map],
    symbol: 0,
    length: 1,
    capacity: 2,
}

{
    fun: map,
    args: [not, xs],
    symbol: 0,
    length: 2,
    capacity: 2,
}
```

The idea is that partial applications are data, so their `fun` must be `noop`.
Therefore the function that we actually want to apply is stored at the end of the `args` buffer.
The missing arguments are left blank.

When we apply arguments, we fill up the arguments buffer.
When we apply the last argument, we first move the function at the end of the `args` buffer into the `fun`.
Then we're left with a regular function application.

So here's what happened in the example:

1. We stored `map` at the end of `args` and set the `fun` to `noop`
2. We applied `not` by inserting it into the first argument
3. We applied `xs` by first moving `map` into `fun` and then inserting `xs` into the last argument
