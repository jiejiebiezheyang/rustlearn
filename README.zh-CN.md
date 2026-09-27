<div align="center">

# Rust 示例教程 —— 16 个可运行章节

编号化、自包含的 Rust 核心主题之旅：**一个主题，一个可独立运行的程序。**

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Examples](https://img.shields.io/badge/examples-16%2F16%20runnable-brightgreen.svg)](#文件索引)
[![Tests](https://img.shields.io/badge/tests-passing-success.svg)](#验证结果)
[![Code style](https://img.shields.io/badge/code%20style-cargo%20fmt%20%7C%20clippy-blueviolet.svg)](#验证结果)

[English](README.md) | **简体中文**

</div>

---

## 为什么做这个仓库

市面上的 Rust 学习材料要么是 600 页的大部头，要么是 20 行的碎片代码。这个仓库想填中间的空白：
**16 个编号文件，每个都是一个完整可运行的程序，可以从头读到尾，大约 20 分钟读完一章。**

- **每一章都自包含。** 有自己的 `fn main()`、自己的类型，没有隐藏的公共辅助代码。
  读完一个文件就等于读完了这一章。
- **不只讲"是什么"，更讲"为什么"。** 每个语法点都写清楚了它解决什么问题、什么时候该用，
  以及最容易踩的坑（用 `// ⚠️ 常见坑:` 标记，方便检索）。
- **结果经过真实验证，而不是"应该能跑"。** 16 个程序全部编译通过、通过
  `cargo clippy -- -D warnings`、通过 `rustfmt` 检查、并实际运行且退出码为 0 —— 详见
  [验证结果](#验证结果)。
- **难点一个不落**：move 语义、借用检查器报错、生命周期省略、内部可变性、`Send`/`Sync`，
  以及真正开始写项目时第一天就会用到的标准库与生态 crate。

## 环境要求

| 要求 | 说明 |
| --- | --- |
| **Rust 1.75+**（edition 2021） | 用 [rustup](https://rustup.rs/) 安装。`Cargo.toml` 中声明了 `rust-version = "1.75"`，所有依赖的 MSRV 都 ≤ 1.71。 |
| `cargo` | 随 Rust 一起安装。 |
| `rustfmt` + `clippy` | `rustup component add rustfmt clippy`（一般已默认安装）。 |
| 首次构建需要联网 | 第一次构建会下载 `serde`、`serde_json`、`chrono`，之后即可离线使用。 |
| **不需要 nightly** | 包括测试在内，全部使用 stable 工具链。 |

支持任意操作系统（Linux、macOS、Windows，含 WSL）。仓库中没有任何内容依赖特定的
shell 或目录结构。

## 快速开始

```bash
# 1. 获取代码
git clone <本仓库地址> rustlearn
cd rustlearn

# 2. 一次性编译全部 16 章
cargo build

# 3. 用文件名（去掉 .rs）运行任意一章
cargo run --bin 01_variables
cargo run --bin 09_collections
cargo run --bin 16_std_ecosystem      # 唯一使用外部 crate 的一章

# 4. 第 15 章包含测试套件
cargo test --bin 15_testing

# 5. 一次性运行全部章节（遇到失败即中断）
for b in $(ls src/bin | sed 's/\.rs$//'); do
  printf '%-28s' "$b"
  cargo run --quiet --bin "$b" >/dev/null && echo OK || echo FAILED
done
```

仓库**刻意没有 `src/main.rs`**，因此直接执行 `cargo run` 会提示你用 `--bin` 指定章节。
这正是设计意图：[文件索引](#文件索引)才是入口。

## 文件索引

| 编号 | 文件 | 主题 | 关键知识点 |
| --- | --- | --- | --- |
| 01 | [`src/bin/01_variables.rs`](src/bin/01_variables.rs) | 变量、常量、基本类型与可变性 | `let` / `mut` / shadowing（遮蔽） · 标量类型（`i32`、`u32`、`f64`、`bool`、`char`） · 复合类型（tuple、array） · `const` 与 `static` 的区别 · 类型推断与显式标注 · 整数溢出、`wrapping_*` / `checked_*` · `as` 转换 · 一切皆表达式 |
| 02 | [`src/bin/02_ownership.rs`](src/bin/02_ownership.rs) | 所有权与借用 | 栈与堆 · 移动（move）语义 · `Copy` 与 `Clone` · 所有权随函数参数转移与返回 · 借用规则 · 悬垂引用为何编译不过 · `Drop` 作用域与释放顺序 · `mem::drop` |
| 03 | [`src/bin/03_references_slices.rs`](src/bin/03_references_slices.rs) | 引用与切片 | `&T` 与 `&mut T` · 别名规则（多个 `&` **或**一个 `&mut`） · 非词法生命周期（NLL） · `String` 与 `&str` · UTF-8 字符边界与切片 panic · 数组切片 `&[T]` · `get()` 与 `[]` 的区别 · `split` / `trim` / `starts_with` |
| 04 | [`src/bin/04_structs.rs`](src/bin/04_structs.rs) | 结构体、方法与关联函数 | 具名 / 元组 / 单元结构体 · 字段初始化简写 · 结构体更新语法 `..` · `&self`、`&mut self`、`self` · 关联函数 `new` · 多个 `impl` 块 · `#[derive(Debug, Clone, PartialEq)]` · `Default` |
| 05 | [`src/bin/05_enums.rs`](src/bin/05_enums.rs) | 枚举、模式匹配与 `Option` | 带数据的枚举变体 · `Option<T>` · `match` 穷尽性 · match guard · 解构绑定 · `if let` / `while let` / `let ... else` · `_` 通配符 · `matches!` · `Option` 组合子（`map`、`and_then`、`unwrap_or`） |
| 06 | [`src/bin/06_error_handling.rs`](src/bin/06_error_handling.rs) | 错误处理 | `panic!` 与可恢复错误 · `Result<T, E>` · `unwrap` / `expect` · `?` 运算符 · `From` 自动转换与错误传播 · `main` 返回 `Result` · 自定义错误枚举 + `Display` + `std::error::Error` · `Box<dyn Error>` · 何时就地恢复、何时向上传播 |
| 07 | [`src/bin/07_generics.rs`](src/bin/07_generics.rs) | 泛型与 trait 约束 | 泛型函数 · 泛型结构体与泛型枚举 · 内联约束与 `where` 子句 · 多重约束 · 单态化（monomorphization）与零成本抽象 · const 泛型 · 默认类型参数 · 泛型 `impl` 块 |
| 08 | [`src/bin/08_traits.rs`](src/bin/08_traits.rs) | trait 与动态分发 | trait 定义与默认方法 · 为自定义类型实现 trait · supertrait · `dyn Trait` trait 对象 · 静态分发与动态分发（vtable） · `impl Trait` 作为返回值 · 对象安全与孤儿规则 · 集合中存放 trait 对象 |
| 09 | [`src/bin/09_collections.rs`](src/bin/09_collections.rs) | 集合类型 | `Vec` 常用操作与 `with_capacity` · `HashMap` 与 `entry` API · `BTreeMap` 有序遍历与 `range` · `HashSet` 去重与集合运算 · `VecDeque` · 容量与扩容 · 迭代时修改集合的借用冲突 |
| 10 | [`src/bin/10_iterators_closures.rs`](src/bin/10_iterators_closures.rs) | 迭代器与闭包 | 闭包语法与捕获（不可变借用 / 可变借用 / `move`） · `Fn` / `FnMut` / `FnOnce` · 迭代器惰性 · `map` / `filter` / `fold` / `collect` / `enumerate` / `zip` / `chain` / `flat_map` / `take` · 自己实现 `Iterator` · `iter()` 与 `into_iter()` |
| 11 | [`src/bin/11_smart_pointers.rs`](src/bin/11_smart_pointers.rs) | 智能指针 | `Box<T>` 与递归类型 · `Rc<T>` 共享所有权与 `strong_count` · `RefCell<T>` 内部可变性与运行时借用检查 · `Rc<RefCell<T>>` 组合 · `Cell` 与 `Weak` 简介 · `Deref` 与 Deref 强制转换 · `Drop` 与 RAII · `Rc` 循环引用导致内存泄漏 |
| 12 | [`src/bin/12_lifetimes.rs`](src/bin/12_lifetimes.rs) | 生命周期 | 为什么需要生命周期标注 · 泛型生命周期参数 `'a` · 三条省略规则 · 结构体持有引用 · `impl` 块与方法中的生命周期 · `'static` · 生命周期约束 `T: 'a` · 经典报错"返回局部变量的引用" |
| 13 | [`src/bin/13_modules.rs`](src/bin/13_modules.rs) | 模块与包管理 | 内联 `mod` 与文件拆分 · `pub` / `pub(crate)` / `pub(super)` · 路径（`crate::`、`self::`、`super::`） · `pub use` 重导出 · 默认私有性 · `#[cfg(test)] mod tests` · package / crate / bin / lib 的区别 · workspace 结构 · cargo 命令速查 |
| 14 | [`src/bin/14_concurrency.rs`](src/bin/14_concurrency.rs) | 并发编程 | `thread::spawn` 与 `join` · `move` 闭包跨线程 · mpsc channel 与多生产者 · `recv` 与 `try_recv` · `Mutex<T>` 与锁中毒 · `Arc<Mutex<T>>` 共享状态 · `RwLock` 与原子类型简介 · `Send` 与 `Sync` 的含义 · `thread::scope` 借用栈上数据 · 死锁 |
| 15 | [`src/bin/15_testing.rs`](src/bin/15_testing.rs) | 测试与基准测试 | `#[cfg(test)] mod tests` · `#[test]` · `assert!` / `assert_eq!` / `assert_ne!` 与自定义失败信息 · `#[should_panic(expected = "...")]` · 返回 `Result` 的测试 · `tests/` 集成测试与文档测试 · 测试过滤与 `--nocapture` · 基准测试：`#[bench]`（仅 nightly）、Criterion（附注释示例）、`Instant` 手动计时 |
| 16 | [`src/bin/16_std_ecosystem.rs`](src/bin/16_std_ecosystem.rs) | 常用标准库与生态 | `String` 与 `&str` 的转换 · `format!` 与 `push_str`、`+` 拼接的坑 · `std::fs` 读写与 `create_dir_all` · `std::io`（`Write`、`BufRead::lines`、`stdin().read_line`） · `std::env`（`args`、`var`、`temp_dir`、`current_dir`） · `serde` + `serde_json`（`to_string`、`to_string_pretty`、`from_str`、`Value`） · `chrono`（`Utc::now`、`Local`、`Duration`、格式化、RFC 3339） · `Box<dyn Error>` |

## 布局说明：为什么用 `src/bin/` 而不是单个 `src/main.rs`

Cargo 会把 **`src/bin/` 下的每个文件都当作独立的 crate root，也就是一个独立的可执行程序**。
正是这一条规则撑起了本仓库的组织方式。

| | `src/bin/NN_topic.rs`（本项目采用） | 单个 `src/main.rs` |
| --- | --- | --- |
| 入口 | 16 个互不干扰的 `fn main()` | 只有一个 `main()`，其它主题要靠子命令或反复注释代码来切换 |
| 隔离性 | 某一章写错了只影响它自己的二进制 | 一处语法错误会卡住全部内容 |
| 命名空间 | 每章有自己的 `use` 和类型，可以放心重复使用 `Point`、`Shape`、`Task` 这类名字 | 命名冲突要靠前缀或模块化改造来绕开 |
| 运行方式 | `cargo run --bin 01_variables`，文件名就是命令 | `cargo run` 再加一个 CLI 开关，或改代码 |
| 测试 | `cargo test --bin 15_testing` 只编译这一章 | `cargo test` 每次都要重新构建那个大二进制 |
| 额外配置 | 零 —— `src/bin/` 会被自动发现，`Cargo.toml` 里不需要 `[[bin]]` | 零 |

代价是：多个二进制默认无法共享代码。如果哪一章确实需要公共辅助函数，惯用做法是新增
`src/lib.rs`（库 crate），让这些二进制去使用它 —— Cargo 会自动把每个 `src/bin/*.rs`
目标链接到 `src/lib.rs`。本项目不需要这么做，因为每章都必须能独立读懂。

这里**刻意不放 `src/main.rs`**：没有默认二进制时，`cargo run` 必须带 `--bin`，
这就强迫你明确说出想运行哪一章，而不是稀里糊涂跑一个"默认"程序。

## 验证结果

下表每一项都是在仓库根目录实际执行命令得到的，不是从文档抄来的结论。表格与版本无关，
你随时可以在自己的机器上重跑：

```bash
cargo build
cargo clippy -- -D warnings                 # 必需的检查
cargo clippy --all-targets -- -D warnings   # 可选：连 #[cfg(test)] 测试代码一起检查
cargo fmt --check
for b in $(ls src/bin | sed 's/\.rs$//'); do cargo run --quiet --bin "$b"; done
cargo test --bin 15_testing
```

验证环境：**Rust 1.98.1 / cargo 1.98.1**（stable 工具链，Linux x86_64）。

### 编译、Lint 与格式检查

| # | 命令 | 范围 | 结果 |
| --- | --- | --- | --- |
| 1 | `cargo build` | 全部 16 个 bin | ✅ exit 0 —— 16/16 个目标全部编译成功 |
| 2 | `cargo clippy -- -D warnings` | 全部 16 个 bin | ✅ exit 0 —— 0 warning |
| 3 | `cargo clippy --all-targets -- -D warnings` | bin **以及** `#[cfg(test)]` 测试代码 | ✅ exit 0 —— 0 warning |
| 4 | `cargo fmt --check` | 所有 `.rs` 文件 | ✅ exit 0 —— 无格式差异 |

### 每一章都实际运行过

| Bin | 命令 | 退出码 | stdout | stderr |
| --- | --- | --- | --- | --- |
| `01_variables` | `cargo run --quiet --bin 01_variables` | 0 | 49 行 | 0 字节 |
| `02_ownership` | `cargo run --quiet --bin 02_ownership` | 0 | 42 行 | 0 字节 |
| `03_references_slices` | `cargo run --quiet --bin 03_references_slices` | 0 | 40 行 | 0 字节 |
| `04_structs` | `cargo run --quiet --bin 04_structs` | 0 | 30 行 | 0 字节 |
| `05_enums` | `cargo run --quiet --bin 05_enums` | 0 | 45 行 | 0 字节 |
| `06_error_handling` | `cargo run --quiet --bin 06_error_handling` | 0 | 35 行 | 0 字节 |
| `07_generics` | `cargo run --quiet --bin 07_generics` | 0 | 34 行 | 0 字节 |
| `08_traits` | `cargo run --quiet --bin 08_traits` | 0 | 33 行 | 0 字节 |
| `09_collections` | `cargo run --quiet --bin 09_collections` | 0 | 34 行 | 0 字节 |
| `10_iterators_closures` | `cargo run --quiet --bin 10_iterators_closures` | 0 | 39 行 | 0 字节 |
| `11_smart_pointers` | `cargo run --quiet --bin 11_smart_pointers` | 0 | 37 行 | 0 字节 |
| `12_lifetimes` | `cargo run --quiet --bin 12_lifetimes` | 0 | 28 行 | 0 字节 |
| `13_modules` | `cargo run --quiet --bin 13_modules` | 0 | 22 行 | 0 字节 |
| `14_concurrency` | `cargo run --quiet --bin 14_concurrency` | 0 | 32 行 | 0 字节 |
| `15_testing` | `cargo run --quiet --bin 15_testing` | 0 | 33 行 | 0 字节 |
| `16_std_ecosystem` | `cargo run --quiet --bin 16_std_ecosystem` | 0 | 64 行 | 0 字节 |

**16/16 个 bin 退出码为 0，且 stderr 干净。** 输出被设计为确定可复现：不打印地址、线程 id、
耗时，也不直接打印未排序的 `HashMap` 迭代顺序；唯一例外是第 16 章刻意打印的"当前时间"，
并在代码里就地说明了原因。第 14 章为了演示 `Mutex` 锁中毒会让工作线程 panic，
该演示内部临时静默了 panic hook，因此运行输出依然干净、可复现。

### 测试

| 命令 | 结果 |
| --- | --- |
| `cargo test --bin 15_testing` | ✅ `test result: ok. 8 passed; 0 failed; 0 ignored` |
| `cargo test --bin 13_modules` | ✅ `test result: ok. 2 passed; 0 failed; 0 ignored` |

其余章节刻意不带测试：它们本身就是"可观察行为即控制台输出"的程序，
上面的运行表已经逐章验证过输出与退出码。

## 学习路径

编号本身就是建议的学习顺序，不过可以按下面的分组来推进：

1. **基础（01–04）** —— 值、所有权、引用与切片、结构体。
   不要跳过 02–03：借用检查器是劝退率最高的地方。
2. **数据建模与失败处理（05–06）** —— 枚举 + `match` + `Option`，然后是 `Result` 和 `?`。
   学完这两章，你就能写出真正有用的程序了。
3. **抽象能力（07–08）** —— 泛型、trait，以及什么时候该选 `dyn` 而不是静态分发。
4. **处理数据（09–10）** —— 集合类型，然后是让 Rust 代码"像 Rust"而不是像 C 的迭代器与闭包风格。
5. **深入内存（11–12）** —— `Box` / `Rc` / `RefCell`，然后是生命周期，它会真正解释你一直在绕过的那些报错。
6. **构建真实项目（13–16）** —— 模块与 Cargo、线程、测试，以及真正做项目第一天就会用到的
   标准库与生态 API（`serde_json`、`chrono`、`fs`、`env`）。

**有效的学习方式：** 先读一章，自己预测输出，再运行；然后删掉一行、或加一个 `mut`，
看看会得到什么错误。注释里已经写明编译器会怎么报错、以及为什么。

## 许可证

[MIT](LICENSE) © 2026 jiejiebiezheyang
