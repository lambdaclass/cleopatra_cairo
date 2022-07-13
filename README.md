# Cleopatra Cairo VM
[![rust](https://github.com/lambdaclass/cleopatra_cairo/actions/workflows/rust.yml/badge.svg)](https://github.com/lambdaclass/cleopatra_cairo/actions/workflows/rust.yml) [![benchmark](https://github.com/lambdaclass/cleopatra_cairo/actions/workflows/bench.yml/badge.svg)](https://lambdaclass.github.io/cleopatra_cairo/) [![codecov](https://codecov.io/gh/lambdaclass/cleopatra_cairo/branch/main/graph/badge.svg?token=D5FYEQ4E94)](https://codecov.io/gh/lambdaclass/cleopatra_cairo)

![](./bonaparte.webp)

Cleaopatra is a Rust implementation of the Cairo VM. The VM is fully functional but we don't support hints yet.

The code of the original Cairo VM can be found [here](https://github.com/starkware-libs/cairo-lang).

## Dependencies
- Rust
- Cargo
- PyEnv for running the original VM and compiling cairo programs

## Running Cleopatra
Compile with `cargo build --release`, once  the binary is built, it can be found in `target/release/` under the name `cleopatra-run`.
To run a compiled json program through the VM, call the executable giving it the path and name to the file to be executed.

Full compilation and execution example:
```bash
git clone https://github.com/lambdaclass/cleopatra_cairo.git
cd cleopatra_cairo

cargo build --release
./target/release/cleopatra-run tests/support/fibonacci_compiled.json
```

## Testing
Run the test suite with cargo:
```bash
cargo test
```

## Code Coverage
Track of the project's code coverage: [Codecov](https://app.codecov.io/gh/lambdaclass/cleopatra_cairo).

## Cleopatra benchmarks:
Running a [Cairo program](./bench/criterion/fibonacci_1000.cairo) that gets the 1000th Fibonacci number we got the following benchmarks:
* Execution time with [Criterion](./docs/benchmarks/criterion_benchmark.pdf)
* [Flamegraph](./docs/benchmarks/flamegraph.svg)
* Github action [results](https://lambdaclass.github.io/cleopatra_cairo/)

Run the benchmark suite with cargo:
```bash
cargo bench
```

## Cairo

* From Cairo Documentation: [How Cairo Works](https://www.cairo-lang.org/docs/how_cairo_works/index.html#how-cairo-works)
* [Cairo – a Turing-complete STARK-friendly CPU architecturer](https://eprint.iacr.org/2021/1063)
* [A Verified Algebraic Representation of Cairo Program Execution](https://arxiv.org/pdf/2109.14534.pdf)
* [Cairo Verifier](https://github.com/patrickbiel01/Cairo_Verifier) in Rust

## Original Cairo VM Internals

We wrote a document explaining how the Cairo VM works. It can be found [here](./docs/python_vm/README.md).

## Compilers and interpreters

These is a list of recommended books to learn how to implement a compiler or an interpreter.

* [How I wrote my own "proper" programming language - Mukul Rathi](https://mukulrathi.com/create-your-own-programming-language/intro-to-compiler/)
* [Introduction to Compilers and Language Design - Douglas Thain](http://compilerbook.org)
* [Beautiful Racket - Matthew Flatt](https://beautifulracket.com)
* [Crafting interpreters - Robert Nystrom](https://craftinginterpreters.com)
* [Engineering a Compiler - Keith D. Cooper, Linda Torczon](https://www.goodreads.com/en/book/show/1997607.Engineering_a_Compiler)

## Computational integrity and Zero Knowledge Proofs

### Basics
* [Intro to zero knowledge proofs](https://www.youtube.com/watch?v=HUs1bH85X9I)
* [Security and Privacy for Crypto with Zero-Knowledge Proofs](https://www.youtube.com/watch?v=3NL0ThdvWMU)
* [A Hands-On Tutorial for Zero-Knowledge Proofs Series](http://www.shirpeled.com/2018/09/a-hands-on-tutorial-for-zero-knowledge.html)

### ZK SNARKs
* [What are zk-SNARKs?](https://z.cash/technology/zksnarks/)
* [Vitalik's introduction to how zk-SNARKs are possible](https://vitalik.ca/general/2021/01/26/snarks.html)
* [Vitalik's post on quadratic arithmetic programs](https://medium.com/@VitalikButerin/quadratic-arithmetic-programs-from-zero-to-hero-f6d558cea649)
* [Why and How zk-SNARK Works - Maksym Petkus](https://arxiv.org/abs/1906.07221)
* [Comparing General Purpose zk-SNARKs](https://medium.com/coinmonks/comparing-general-purpose-zk-snarks-51ce124c60bd)
* [Dark forest's intro + circuits PART 1](https://blog.zkga.me/intro-to-zksnarks)
* [Dark forest's intro + circuits PART 2](https://blog.zkga.me/df-init-circuit)

### ZK STARKs

Introduction:
* [Cryptography Stack Exchange Answer](https://crypto.stackexchange.com/questions/56327/what-are-zk-starks)
* [Hasu gets STARK-pilled - with Eli Ben-Sasson](https://youtu.be/-6BtBUbiUIU)
* [Cairo for Blockchain Developers](https://www.cairo-lang.org/cairo-for-blockchain-developers/)
* [Why STARKs are the key to unlocking blockchain scalability](https://twitter.com/0xalec/status/1529915544324800512?s=12&t=FX6TgXCZY1iWcWmbc7oqSw)
* STARKs whitepaper: [Scalable, transparent, and post-quantum secure computational integrity](https://eprint.iacr.org/2018/046)
* STARKs vs. SNARKs: [A Cambrian Explosion of Crypto Proofs](https://nakamoto.com/cambrian-explosion-of-crypto-proofs/)

Vitalik Buterin's blog series on zk-STARKs:
* [STARKs, part 1: Proofs with Polynomials](https://vitalik.ca/general/2017/11/09/starks_part_1.html)
* [STARKs, part 2: Thank Goodness it's FRI-day](https://vitalik.ca/general/2017/11/22/starks_part_2.html)
* [STARKs, part 3: Into the Weeds](https://vitalik.ca/general/2018/07/21/starks_part_3.html)

Alan Szepieniec's STARK tutorial:
* [Anatomy of a STARK](https://aszepieniec.github.io/stark-anatomy/)

StarkWare's STARK Math blog series:
* [STARK Math: The Journey Begins](https://medium.com/starkware/stark-math-the-journey-begins-51bd2b063c71)
* [Arithmetization I](https://medium.com/starkware/arithmetization-i-15c046390862)
* [Arithmetization II](https://medium.com/starkware/arithmetization-ii-403c3b3f4355)
* [Low Degree Testing](https://medium.com/starkware/low-degree-testing-f7614f5172db)
* [A Framework for Efficient STARKs](https://medium.com/starkware/a-framework-for-efficient-starks-19608ba06fbe)
