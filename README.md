<div align="center">

# Rust by Example — 16 Runnable Chapters

A numbered, self-contained tour of core Rust: **one topic, one runnable binary.**

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Examples](https://img.shields.io/badge/examples-16%2F16%20runnable-brightgreen.svg)](#file-index)
[![Tests](https://img.shields.io/badge/tests-passing-success.svg)](#verification)
[![Code style](https://img.shields.io/badge/code%20style-cargo%20fmt%20%7C%20clippy-blueviolet.svg)](#verification)

**English** | [简体中文](README.zh-CN.md)

</div>

---

## Why this repo

Most Rust learning material is either a 600-page book or a 20-line snippet. This repo sits in
between: **16 numbered files, each one a complete program you can run, read from top to bottom,
and finish in about 20 minutes.**

- **Every chapter is self-contained.** Its own `fn main()`, its own types, no hidden shared helpers.
  Read one file and you have read the whole chapter.
- **It answers "why", not just "what".** Each syntax point is documented with the problem it
  solves, when to reach for it, and the pitfall that bites people (`// ⚠️ 常见坑:` markers).
- **It is verified, not wishful thinking.** All 16 binaries compile, pass
  `cargo clippy -- -D warnings`, are `rustfmt`-clean, and actually run with exit code 0 —
  see [Verification](#verification).
- **It covers the ugly parts too**: move semantics, borrow-checker errors, lifetime elision,
  interior mutability, `Send`/`Sync`, and the standard-library/ecosystem crates you will reach for
  on day one of a real project.

> **Note on the source comments**
> Comments inside the `.rs` files are written in **Chinese with English technical terminology**
> (`ownership`, `borrow checker`, `monomorphization`, …). The code itself, the output, and both
> READMEs are fully usable without reading Chinese — every file is small and runs standalone.

## Requirements

| Requirement | Notes |
| --- | --- |
| **Rust 1.75+** (edition 2021) | Install via [rustup](https://rustup.rs/). The toolchain's `rust-version = "1.75"` is pinned in `Cargo.toml`; every dependency's MSRV is ≤ 1.71. |
| `cargo` | Ships with Rust. |
| `rustfmt` + `clippy` | `rustup component add rustfmt clippy` (usually already installed). |
| Network access, once | The first build downloads `serde`, `serde_json`, and `chrono`. After that you can work offline. |
| **No nightly required** | Everything, including the tests, runs on stable. |

Any OS works (Linux, macOS, Windows — including WSL). Nothing in the repo depends on a
particular shell or directory layout.

## Quick start

```bash
# 1. Get the code
git clone <this-repo-url> rustlearn
cd rustlearn

# 2. Build all 16 chapters at once
cargo build

# 3. Run any chapter by its file name (without the .rs extension)
cargo run --bin 01_variables
cargo run --bin 09_collections
cargo run --bin 16_std_ecosystem      # the only chapter using external crates

# 4. Chapter 15 contains the test suite
cargo test --bin 15_testing

# 5. Run everything in one go (stops at the first failure)
for b in $(ls src/bin | sed 's/\.rs$//'); do
  printf '%-28s' "$b"
  cargo run --quiet --bin "$b" >/dev/null && echo OK || echo FAILED
done
```

There is deliberately **no `src/main.rs`**, so a bare `cargo run` will ask you to pick a chapter
with `--bin`. That is the point: the [file index](#file-index) is the entry point.

## File index

| # | File | Topic | Key concepts |
| --- | --- | --- | --- |
| 01 | [`src/bin/01_variables.rs`](src/bin/01_variables.rs) | Variables, constants, primitives, mutability | `let` / `mut` / shadowing · scalar types (`i32`, `u32`, `f64`, `bool`, `char`) · compound types (tuple, array) · `const` vs `static` · type inference vs annotation · integer overflow, `wrapping_*` / `checked_*` · `as` casts · everything is an expression |
| 02 | [`src/bin/02_ownership.rs`](src/bin/02_ownership.rs) | Ownership and borrowing | Stack vs heap · move semantics · `Copy` vs `Clone` · ownership moving into and out of functions · borrow rules · why dangling references cannot compile · `Drop` scope and drop order · `mem::drop` |
| 03 | [`src/bin/03_references_slices.rs`](src/bin/03_references_slices.rs) | References and slices | `&T` vs `&mut T` · aliasing rule (many `&` **or** one `&mut`) · non-lexical lifetimes (NLL) · `String` vs `&str` · UTF-8 char boundaries and slice panics · array slices `&[T]` · `get()` vs `[]` · `split` / `trim` / `starts_with` |
| 04 | [`src/bin/04_structs.rs`](src/bin/04_structs.rs) | Structs, methods, associated functions | Named / tuple / unit structs · field init shorthand · struct update syntax `..` · `&self` vs `&mut self` vs `self` · associated function `new` · multiple `impl` blocks · `#[derive(Debug, Clone, PartialEq)]` · `Default` |
| 05 | [`src/bin/05_enums.rs`](src/bin/05_enums.rs) | Enums, pattern matching, `Option` | Variants with data · `Option<T>` · exhaustive `match` · match guards · destructuring bindings · `if let` / `while let` / `let ... else` · `_` wildcard · `matches!` · `Option` combinators (`map`, `and_then`, `unwrap_or`) |
| 06 | [`src/bin/06_error_handling.rs`](src/bin/06_error_handling.rs) | Error handling | `panic!` vs recoverable errors · `Result<T, E>` · `unwrap` / `expect` · the `?` operator · automatic `From` conversion and error propagation · `main` returning `Result` · custom error enum + `Display` + `std::error::Error` · `Box<dyn Error>` · recover in place vs propagate |
| 07 | [`src/bin/07_generics.rs`](src/bin/07_generics.rs) | Generics and trait bounds | Generic functions · generic structs and enums · inline bounds vs `where` clauses · multiple bounds · monomorphization and zero-cost abstraction · const generics · default type parameters · generic `impl` blocks |
| 08 | [`src/bin/08_traits.rs`](src/bin/08_traits.rs) | Traits and dynamic dispatch | Trait definitions and default methods · implementing traits for your own types · supertraits · `dyn Trait` trait objects · static vs dynamic dispatch (vtables) · `impl Trait` in return position · object safety and the orphan rule · collections of trait objects |
| 09 | [`src/bin/09_collections.rs`](src/bin/09_collections.rs) | Collection types | `Vec` operations and `with_capacity` · `HashMap` and the `entry` API · `BTreeMap` ordered iteration and `range` · `HashSet` dedup and set operations · `VecDeque` · capacity and reallocation · the borrow conflict when mutating during iteration |
| 10 | [`src/bin/10_iterators_closures.rs`](src/bin/10_iterators_closures.rs) | Iterators and closures | Closure syntax and capture (by ref / by mutable ref / `move`) · `Fn` / `FnMut` / `FnOnce` · iterator laziness · `map` / `filter` / `fold` / `collect` / `enumerate` / `zip` / `chain` / `flat_map` / `take` · implementing `Iterator` yourself · `iter()` vs `into_iter()` |
| 11 | [`src/bin/11_smart_pointers.rs`](src/bin/11_smart_pointers.rs) | Smart pointers | `Box<T>` and recursive types · `Rc<T>` shared ownership and `strong_count` · `RefCell<T>` interior mutability and runtime borrow checks · `Rc<RefCell<T>>` · `Cell` and `Weak` in brief · `Deref` and deref coercion · `Drop` and RAII · reference cycles leaking memory |
| 12 | [`src/bin/12_lifetimes.rs`](src/bin/12_lifetimes.rs) | Lifetimes | Why lifetime annotations exist · generic lifetime parameter `'a` · the three elision rules · structs holding references · lifetimes in `impl` blocks and methods · `'static` · lifetime bounds `T: 'a` · the classic "returning a reference to a local" error |
| 13 | [`src/bin/13_modules.rs`](src/bin/13_modules.rs) | Modules and package management | Inline `mod` and file-based modules · `pub` / `pub(crate)` / `pub(super)` · paths (`crate::`, `self::`, `super::`) · `pub use` re-exports · private-by-default · `#[cfg(test)] mod tests` · package vs crate vs bin vs lib · workspace layout · cargo command cheat sheet |
| 14 | [`src/bin/14_concurrency.rs`](src/bin/14_concurrency.rs) | Concurrency | `thread::spawn` and `join` · `move` closures across threads · mpsc channels with multiple producers · `recv` and `try_recv` · `Mutex<T>` and lock poisoning · `Arc<Mutex<T>>` shared state · `RwLock` and atomics in brief · what `Send` and `Sync` mean · `thread::scope` borrowing stack data · deadlocks |
| 15 | [`src/bin/15_testing.rs`](src/bin/15_testing.rs) | Testing and benchmarking | `#[cfg(test)] mod tests` · `#[test]` · `assert!` / `assert_eq!` / `assert_ne!` with custom messages · `#[should_panic(expected = "...")]` · tests returning `Result` · integration tests in `tests/` and doc tests · filtering tests and `--nocapture` · benchmarking: `#[bench]` (nightly only), Criterion (annotated example), manual `Instant` timing |
| 16 | [`src/bin/16_std_ecosystem.rs`](src/bin/16_std_ecosystem.rs) | Standard library and ecosystem | `String` vs `&str` conversions · `format!` vs `push_str` and the `+` pitfall · `std::fs` read/write and `create_dir_all` · `std::io` (`Write`, `BufRead::lines`, `stdin().read_line`) · `std::env` (`args`, `var`, `temp_dir`, `current_dir`) · `serde` + `serde_json` (`to_string`, `to_string_pretty`, `from_str`, `Value`) · `chrono` (`Utc::now`, `Local`, `Duration`, formatting, RFC 3339) · `Box<dyn Error>` |

## Layout note: why `src/bin/` and not a single `src/main.rs`

Cargo treats **every file in `src/bin/` as its own crate root, and therefore its own binary**.
That single rule is what makes this repo's structure work.

| | `src/bin/NN_topic.rs` (used here) | one `src/main.rs` |
| --- | --- | --- |
| Entry points | 16 independent `fn main()`s | one `main()`; extra topics become subcommands or commented-out blocks |
| Isolation | A broken chapter fails only its own binary | One syntax error blocks everything |
| Namespacing | Each chapter has its own imports and types, freely reusing names like `Point`, `Shape`, `Task` | Name collisions force prefixes or module plumbing |
| Running | `cargo run --bin 01_variables` — the file name *is* the command | `cargo run` plus a CLI switch, or editing code to choose a topic |
| Testing | `cargo test --bin 15_testing` compiles that chapter only | `cargo test` rebuilds one big binary |
| Boilerplate | Zero — `src/bin/` is auto-discovered, no `[[bin]]` sections in `Cargo.toml` | Zero |

The trade-off: separate binaries cannot share code by default. If a chapter ever needs a common
helper, the idiomatic fix is to add `src/lib.rs` (a library crate) and have the binaries use it —
Cargo links every `src/bin/*.rs` target against `src/lib.rs` automatically. This repo does not need
one, precisely because each chapter must stay readable on its own.

There is intentionally **no `src/main.rs`**: without a default binary, `cargo run` requires
`--bin`, which forces you to name the chapter you want instead of silently running a "default" one.

## Verification

Every claim below was produced by running the commands in this repository root; nothing is copied
from docs or assumed. The table is version-neutral — re-run it on your machine any time:

```bash
cargo build
cargo clippy -- -D warnings                 # required gate
cargo clippy --all-targets -- -D warnings   # optional: also lints #[cfg(test)] code
cargo fmt --check
for b in $(ls src/bin | sed 's/\.rs$//'); do cargo run --quiet --bin "$b"; done
cargo test --bin 15_testing
```

Last verified with **Rust 1.98.1 / cargo 1.98.1** (stable toolchain, Linux x86_64).

### Build, lint and format gates

| # | Command | Scope | Result |
| --- | --- | --- | --- |
| 1 | `cargo build` | all 16 binaries | ✅ exit 0 — 16/16 targets built |
| 2 | `cargo clippy -- -D warnings` | all 16 binaries | ✅ exit 0 — 0 warnings |
| 3 | `cargo clippy --all-targets -- -D warnings` | binaries **and** `#[cfg(test)]` code | ✅ exit 0 — 0 warnings |
| 4 | `cargo fmt --check` | every `.rs` file | ✅ exit 0 — no diff |

### Every chapter actually runs

| Binary | Command | Exit | stdout | stderr |
| --- | --- | --- | --- | --- |
| `01_variables` | `cargo run --quiet --bin 01_variables` | 0 | 49 lines | 0 bytes |
| `02_ownership` | `cargo run --quiet --bin 02_ownership` | 0 | 42 lines | 0 bytes |
| `03_references_slices` | `cargo run --quiet --bin 03_references_slices` | 0 | 40 lines | 0 bytes |
| `04_structs` | `cargo run --quiet --bin 04_structs` | 0 | 30 lines | 0 bytes |
| `05_enums` | `cargo run --quiet --bin 05_enums` | 0 | 45 lines | 0 bytes |
| `06_error_handling` | `cargo run --quiet --bin 06_error_handling` | 0 | 35 lines | 0 bytes |
| `07_generics` | `cargo run --quiet --bin 07_generics` | 0 | 34 lines | 0 bytes |
| `08_traits` | `cargo run --quiet --bin 08_traits` | 0 | 33 lines | 0 bytes |
| `09_collections` | `cargo run --quiet --bin 09_collections` | 0 | 34 lines | 0 bytes |
| `10_iterators_closures` | `cargo run --quiet --bin 10_iterators_closures` | 0 | 39 lines | 0 bytes |
| `11_smart_pointers` | `cargo run --quiet --bin 11_smart_pointers` | 0 | 37 lines | 0 bytes |
| `12_lifetimes` | `cargo run --quiet --bin 12_lifetimes` | 0 | 28 lines | 0 bytes |
| `13_modules` | `cargo run --quiet --bin 13_modules` | 0 | 22 lines | 0 bytes |
| `14_concurrency` | `cargo run --quiet --bin 14_concurrency` | 0 | 32 lines | 0 bytes |
| `15_testing` | `cargo run --quiet --bin 15_testing` | 0 | 33 lines | 0 bytes |
| `16_std_ecosystem` | `cargo run --quiet --bin 16_std_ecosystem` | 0 | 64 lines | 0 bytes |

**16/16 binaries exit 0 with clean stderr.** Console output is deterministic by design — no
addresses, thread ids, timings or unsorted `HashMap` iteration — except for the current-time lines
that chapter 16 prints on purpose and documents in place. Chapter 14 demonstrates a poisoned
`Mutex` by panicking a worker thread; the panic hook is silenced for that demo so the run stays
clean and reproducible.

### Tests

| Command | Result |
| --- | --- |
| `cargo test --bin 15_testing` | ✅ `test result: ok. 8 passed; 0 failed; 0 ignored` |
| `cargo test --bin 13_modules` | ✅ `test result: ok. 2 passed; 0 failed; 0 ignored` |

The other chapters ship no test harness on purpose: each one is a program whose observable
behaviour *is* its console output, which the run table above already checks.

## Learning path

The numbering is a suggested order, but the chapters are grouped:

1. **Foundations (01–04)** — values, ownership, references/slices, structs.
   Do not skip 02–03: the borrow checker is where most people quit.
2. **Modelling data and failure (05–06)** — enums + `match` + `Option`, then `Result` and `?`.
   After these two, you can write real programs.
3. **Abstraction (07–08)** — generics, traits, and when to choose `dyn` over static dispatch.
4. **Working with data (09–10)** — collections, then the iterator/closure style that makes Rust code
   feel like Rust instead of C.
5. **Memory in depth (11–12)** — `Box`/`Rc`/`RefCell`, then lifetimes, which finally explain the
   error messages you have been working around.
6. **Building real things (13–16)** — modules and Cargo, threads, tests, and the std/ecosystem APIs
   (`serde_json`, `chrono`, `fs`, `env`) you will use on day one of a real project.

**Study pattern that works:** read a chapter, predict the output, run it, then delete a line or add
a `mut` and see which error you get. The comments tell you what the compiler will say and why.

## License

[MIT](LICENSE) © 2026 jiejiebiezheyang
