# Rec-JIT

A Basic Just-In-Time Compiler for a simple math recurrence relation based on
<https://nullprogram.com/blog/2015/03/19/>. Currently only works on x86_64 Linux
System.

## Building

To use this program, you need [Rust] to compile it.

```console
$ git clone https://github.com/matematikaadit/rec-jit.git
$ cd rec-jit
$ cargo build
```

## Explanation and Usage

From the console run

```console
$ cargo run
```

Then supply a three lines. The first line is the recurrence expression. The second
line is the initial value. And the third line is the number of iteration.

For example typing in the stdin:

```text
+2 *3
0
10
```

Or using sample `input.txt`:
```console
$ cargo run < input.txt
```

Is the same as calculating the recurrence `U[n] = (U[n-1] + 2) * 3`, with the initial
value 0, and the number of iteration 10. Thus the program will give output:

```text
Term 0: 0
Term 1: 1
Term 2: 4
Term 3: 13
Term 4: 40
Term 5: 121
Term 6: 364
Term 7: 1093
Term 8: 3280
Term 9: 9841
Term 10: 29524
```

[Playground Demo][] (might be outdated). For full explanation, see the
[reddit thread].

## How it works

Basically, the program will parse the expression in the first line and
translate each operation into corresponding x86_64 instruction.
All this instruction is saved in a memmapped buffer which will be transmuted into
a `fn(i64) -> i64` function after the execute bit is set on its page.

To find the corresponding instruction for each operator, file `peek.s` is provided.
It's used as follow:

```console
$ nasm peek.s
$ ndisasm -b64 peek
```

The output then used to implement the corresponding operator and instruction.

## Reference

- [A Basic Just-In-Time Compiler](https://nullprogram.com/blog/2015/03/19/)
- [/r/dailyprogrammer challenge thread][reddit thread]
- [mmap crate for inspiration][mmap]

## Future Ideas and Possible Improvement

- Bot for executing this program
- Make it works on Windows
- RPN calculator
- use LLVM

## License

Unlicense, see [UNLICENSE](/UNLICENSE) for more explanation.

[Rust]: https://rustup.rs/
[Playground Demo]: https://play.rust-lang.org/?gist=13c286c91e751227b2265fb499a96a66&version=stable&mode=debug&edition=2015
[reddit thread]: http://redd.it/2z68di
[mmap]: https://github.com/rbranson/rust-mmap
