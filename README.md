# C Rustls

This crate contains C bindings for the [rustls](https://docs.rs/rustls) TLS
library. It also contains a small demo C program that uses those bindings
to make an HTTPS request.

# Build

You'll need to [install the Rust toolchain](https://rustup.rs/) and a C
compiler (gcc and clang should both work). Once you've got the Rust toolchain
installed, run `cargo install cbindgen`. Then, to build in debug mode:

    make

To install:

    make install

To build and install in optimized mode:

    make PROFILE=release install

# Conventions

This library defines an enum, rustls_result, to indicate success or failure of
a function call. All fallible functions return a rustls_result. If a function
has other outputs, it provides them using output parameters (pointers to
caller-provided objects). For instance:

```rust
rustls_result rustls_client_session_read(const rustls_client_session *session,
                                         uint8_t *buf,
                                         size_t count,
                                         size_t *out_n);
```

In this example, `buf` and `out_n` are output parameters.

## Structs

For a given struct, all functions that start with the name of that struct are
either associated functions or methods of that struct. For instance,
`rustls_client_session_read` is a method of `rustls_client_session`. A function
that takes a pointer to a struct as the first parameter is considered a method
on that struct. Structs in this library are always created and destroyed by
library code, so the header file only gives a declaration of the structs, not
a definition.

As a result, structs are always handled using pointers. For each struct, there
is generally a function ending in `_new()` to create that struct. Once you've
got a pointer to a struct, it's your responsibility to (a) ensure no two
threads are concurrently mutating that struct, and (b) free that struct's
memory exactly once. Freeing a struct's memory will usually be accomplished
with a function starting with the struct's name and ending in `_free()`.

You can tell if a method will mutate a struct by looking at the first
parameter. If it's a `const*`, the method is non-mutating. Otherwise, it's 
mutating.

## Input and Output Parameters

Input parameters will always be either a const pointer or a primitive type
(int, size_t, etc). Output parameters will always be a non-const pointer.

The caller is responsible for ensuring that the memory pointed to be output
parameters is not being concurrently accessed by other threads. For primitive
types and pointers-to-pointers this is most commonly accomplished by passing
the address of a local variable on the stack that has no references elsewhere.
For buffers, stack allocation is also a simple way to accomplish this, but if
the buffer is allocated on heap and references to it are shared among threads,
the caller will need to take additional steps to prevent concurrent access
(for instance mutex locking, or single-threaded I/O).

When an output parameter is a pointer to a pointer (e.g. 
`rustls_client_session **session_out`, the function will set its argument
to point to an appropriate object on success. The caller is considered to take
ownership of that object and be responsible for the requirements above:
preventing concurrent mutation, and freeing it exactly once.

For a method, the first parameter will always be a pointer to the struct being
operated on. Next will come some number of input parameters, then some number
of output parameters. 

As a minor exception to the above: When an output parameter is a byte buffer
(*uint8_t), the next parameter will always be a size_t denoting the size of
the buffer. This is considered part of the output parameters even though it is
not directly modified.

There are no in/out parameters. When an output buffer is passed, the library
only writes to that buffer and does not read from it.

For fallible functions, values are only written to the output arguments if
the function returns success. There are no partial successes or partial
failures. Callers must check the return value before relying on the values
pointed to by output arguments.

## NULL

The library checks all pointers in arguments for NULL and will return an error
rather than dereferencing a NULL pointer. For some methods that are infallible
except for the possibility of NULL (for instance
`rustls_client_session_is_handshaking`), the library returns a convenient
type (e.g. `bool`) and uses a suitable fallback value if an input is NULL.

## Panics

In case of a bug (e.g. exceeding the bounds of an array), Rust code may
emit a panic. Panics are treated like exceptions in C++, unwinding the stack.
Unwinding past the FFI boundary is undefined behavior, so this library catches
all unwinds and turns them into RUSTLS_RESULT_PANIC (when the function is
fallible).

Functions that are theoretically infallible don't return rustls_result, so we
can't return RUSTLS_RESULT_PANIC. In those cases, if there's a panic, we'll
return a default value suitable to the return type: NULL for pointer types,
false for bool types, and 0 for integer types.
