//! 第 15 章：测试与基准测试（testing & benchmarking）
//!
//! 本章把“代码是对的”“代码够快”变成可重复执行的命令：怎么写测试、怎么只跑想跑的
//! 测试、怎么测量性能。本文件同时是两个身份：
//! - 二进制：`cargo run --bin 15_testing` 打印知识摘要（退出码 0）；
//! - 测试目标：`cargo test --bin 15_testing` 执行末尾的 `#[cfg(test)] mod tests`。
//!
//! 关键知识点：
//! - `#[cfg(test)] mod tests`：测试模块只在测试构建里存在
//! - `#[test]`、`assert!` / `assert_eq!` / `assert_ne!` 与自定义失败信息
//! - `#[should_panic(expected = "...")]`、返回 `Result` 的测试
//! - 单元测试 vs 集成测试（`tests/` 目录）vs 文档测试（`///` 代码块）
//! - 测试过滤、`--nocapture`、`--test-threads`
//! - 基准测试三种方式：`#[bench]`（仅 nightly）、Criterion、`Instant` 手动计时

use std::time::{Duration, Instant};

/// 基准测试的轮数：固定负载才能让多次测量具备可比性。
const BENCH_ROUNDS: u64 = 20_000;

/// 基准测试的输入规模：每轮计算一次 `fib(BENCH_INPUT)`。
const BENCH_INPUT: u64 = 40;

/// 把两个整数相加，演示最容易测的纯函数（pure function）。
///
/// # 返回
/// `a + b`（debug 下 `i32` 溢出会 panic，见第 1 章）。
///
/// # 示例（rustdoc 会把代码块抽成 doctest；bin 条目对外不可见，用等价表达式）
/// ```
/// assert_eq!(2 + 3, 5);
/// ```
fn add(a: i32, b: i32) -> i32 {
    // 纯函数没有副作用、不读全局状态，测试不需要准备/清理代码。
    a + b
}

/// 解析一个必须为正数的 `u32`。
///
/// `Result` 风格把“失败”写进类型里，测试能同时覆盖成功与失败两条路径。
/// # 返回：`Ok(n)` 是大于 0 的整数；`Err(msg)` 表示不是数字或数值为 0。
fn parse_positive(input: &str) -> Result<u32, String> {
    // `?` 让解析失败直接短路返回 `Err`，测试里同样可用。
    let value: u32 = input
        .trim()
        .parse()
        .map_err(|_| format!("`{input}` 不是合法的非负整数"))?;
    if value == 0 {
        return Err("数值必须大于 0".to_string());
    }
    Ok(value)
}

/// 整数除法；除数为 0 时 panic，用于演示 `#[should_panic]`。
/// # Panics：`divisor == 0` 时 panic，消息包含“除数不能为 0”。
fn divide(dividend: i32, divisor: i32) -> i32 {
    // 前置条件（precondition）检查放在入口，调用方无法绕过。
    assert_ne!(divisor, 0, "除数不能为 0");
    dividend / divisor
}

/// 迭代版斐波那契，作为手动计时的被测负载。
/// # 返回：第 `n` 项（`fib(0) == 0`）；`u64` 下 `n <= 92` 不溢出。
fn fib(n: u64) -> u64 {
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 0..n {
        let next = a + b;
        a = b;
        b = next;
    }
    a
}

// ---------------------------------------------------------------------------
// 附：Criterion 基准测试完整示例（本章刻意不加该依赖，仅作注释参考）
// 为什么它比手写 Instant 可靠？
//   1. 自动预热（warm-up）并按统计方法剔除离群值，报告均值与置信区间；
//   2. 自动决定迭代次数：纳秒级的函数多跑几轮，耗时长的少跑几轮；
//   3. 保存历史结果做性能回归检测（change detection），CI 里可卡阈值；
//   4. 用 `black_box` 阻止死代码消除（dead code elimination），否则没有副作用的
//      计算结果会被优化掉，你测到的是空循环。
// Cargo.toml：
//     [dev-dependencies] criterion = "0.5"
//     [[bench]]
//     name = "my_bench"     # 对应 benches/my_bench.rs
//     harness = false       # 必须！否则内置 harness 会注入自己的 main
// benches/my_bench.rs：
//     use criterion::{black_box, criterion_group, criterion_main, Criterion};
//     fn bench_fib(c: &mut Criterion) {
//         c.bench_function("fib(20)", |b| b.iter(|| black_box(fib(black_box(20)))));
//     }
//     criterion_group!(benches, bench_fib);  criterion_main!(benches);
// 运行：`cargo bench`（或 `cargo bench --bench my_bench`）。
// ⚠️ 常见坑: 漏写 `harness = false` 时，Cargo 注入的 main 与 `criterion_main!`
//    生成的 main 冲突，报 “cannot find function `main`”。
// ---------------------------------------------------------------------------

/// 用 `std::time::Instant` 手动计时：本章实际采用的方式。
/// # 返回：`(校验和, 耗时)`。校验和让优化器无法删掉整个循环，并提供确定性输出；
/// 耗时随机器、构建模式（debug/release）与系统负载变化。
fn bench_with_instant() -> (u64, Duration) {
    // `Instant` 是单调时钟（monotonic clock），不受系统时间调整影响。
    let start = Instant::now();
    let mut checksum = 0u64;
    for i in 0..BENCH_ROUNDS {
        checksum = checksum.wrapping_add(fib(BENCH_INPUT).wrapping_add(i));
    }
    // ⚠️ 常见坑: 手动计时把“编译优化程度”“CPU 频率”“后台进程”全算进去，
    //    debug 与 release 能差几十倍；只适合粗粒度对比，不是性能结论。
    (checksum, start.elapsed())
}

/// 程序入口：按小节打印本章知识摘要，并跑一次 `Instant` 手动计时。
///
/// 这里刻意不做断言——正确性交给末尾的 `#[cfg(test)] mod tests`。
/// 返回 `()`，所以退出码恒为 0，打印耗时不会影响退出码。
fn main() {
    println!("=== 1. 三种测试：unit / integration / doc ===");
    println!("单元测试 unit test  : 与代码同文件（同一 crate），可访问私有条目。");
    println!("集成测试 integration: 放在根 `tests/` 目录，每个文件是独立 crate，只能");
    println!("                      调用 pub API；本工程无 lib target，故按 --bin 只跑单元测试。");
    println!("文档测试 doc test   : `///` 里的 ``` 代码块被 rustdoc 抽成隐藏 main 编译运行；");
    println!("                      cargo 只为 lib target 收集 doctest，可用 `rustdoc --test");
    println!("                      src/bin/15_testing.rs` 手动验证。");

    println!("=== 2. 断言、失败信息与 panic 测试 ===");
    println!("add={} fib={} div={}", add(2, 3), fib(10), divide(10, 2));
    match parse_positive(" 7 ") {
        Ok(n) => println!("parse_positive(\" 7 \") = Ok({n})"),
        Err(e) => println!("不应该失败: {e}"),
    }
    println!("parse_positive(\"0\") 返回 {:?}", parse_positive("0"));
    println!("assert!(cond, \"期望 X，实际 {{}}\") 的自定义信息失败时原样打印；");
    println!("assert_eq!/assert_ne! 失败时额外打印 left / right 两个值。");
    println!("divide(10, 0) 会 panic，所以只在 should_panic 测试里调用。");
    println!("返回 Result 的测试写 `fn t() -> Result<(), E>`，用 `?` 串联各步骤。");

    println!("=== 3. 测试过滤与 --nocapture ===");
    for cmd in [
        "cargo test --bin 15_testing                      # 跑全部测试，默认并行",
        "cargo test --bin 15_testing assert               # 只跑名字含 `assert` 的测试",
        "cargo test --bin 15_testing -- --exact tests::assert_eq_form  # 精确匹配",
        "cargo test --bin 15_testing -- --nocapture       # 显示测试里的 println",
        "cargo test --bin 15_testing -- --show-output     # 通过也打印输出",
        "cargo test --bin 15_testing -- --test-threads=1  # 串行，排查共享状态",
        "cargo test --bin 15_testing -- --ignored         # 只跑 #[ignore] 慢测试",
    ] {
        println!("  {cmd}");
    }
    println!("⚠️ 测试默认多线程并行，别依赖执行顺序或共享的可变全局状态。");

    println!("=== 4. 基准测试的三种方式 ===");
    println!("(a) #[bench] : 标准库自带，但依赖 nightly 的 `test` crate；");
    println!("               用 `cargo +nightly bench` 运行。");
    println!("(b) Criterion: 稳定版可用的统计型 harness，完整示例见上方注释；");
    println!("               关键是 `[[bench]] harness = false`，用 `cargo bench` 运行。");
    println!("(c) Instant  : 手动计时，本章采用；`cargo run --release` 更接近真实性能。");
    let (checksum, elapsed) = bench_with_instant();
    println!("Instant 计时: {BENCH_ROUNDS} 轮 fib({BENCH_INPUT})，校验和 {checksum}（确定性）");
    println!("实测耗时 = {elapsed:?} —— ⚠️ 该数值随机器、构建模式与系统负载变化，");
    println!("仅供本地参考，不可跨机器比较；它只打印、不参与断言，退出码仍为 0。");
}

// ---------------------------------------------------------------------------
// 单元测试：`cargo test --bin 15_testing` 会编译并执行下面的模块。
// `#[cfg(test)]` 表示“只在测试构建时编译”，正式二进制里没有这个模块。
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    //! 与 `main` 同文件、可访问私有条目的测试（单元测试的典型形态）。
    //!
    //! ⚠️ 常见坑: 漏写 `#[cfg(test)]` 会让 `#[test]` 函数进入正式构建，
    //! 触发 “function is never used” 警告（配合 `-D warnings` 直接失败）。

    // `use super::*;` 把父模块的全部条目引进测试模块。
    // ⚠️ 常见坑: 漏写它会报 “cannot find function `add` in this scope”。
    use super::*;

    /// `assert!`：断言布尔表达式为真，失败即 panic，框架记为 FAILED。
    #[test]
    fn assert_bool_form() {
        let n = add(2, 3);
        assert!(n > 0, "add(2, 3) 应该是正数，实际是 {n}");
    }

    /// `assert_eq!`：断言左右相等，失败时同时打印 left / right，定位最快。
    #[test]
    fn assert_eq_form() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(-2, -3), -5, "负数相加出错：{}", add(-2, -3));
    }

    /// `assert_ne!`：断言左右不等，适合确认结果没退化成一个哨兵值。
    #[test]
    fn assert_ne_form() {
        assert_ne!(add(2, 2), 5, "2 + 2 不应该是 5");
        assert_ne!(fib(10), 0, "fib(10) 不该退化成 0");
    }

    /// 自定义失败信息：把“输入 + 期望”写进消息，失败时一眼看出是哪个用例。
    /// 表格驱动（table-driven）测试就是循环 + 自定义消息覆盖大量输入。
    #[test]
    fn custom_failure_message() {
        let cases = [(1, 1, 2), (0, 0, 0), (i32::MAX, 0, i32::MAX)];
        for (a, b, expected) in cases {
            assert_eq!(add(a, b), expected, "add({a}, {b}) 期望 {expected}");
        }
    }

    /// 返回 `Result` 的测试：`Ok(())` 通过，`Err(e)` 失败并打印 `e`。
    /// 好处是能用 `?` 串联多个可失败步骤，比一长串 `unwrap()` 干净。
    ///
    /// ⚠️ 常见坑: 返回 `Result` 的测试不能同时标 `#[should_panic]`
    /// （该属性要求返回 `()`），否则编译期就报错。
    #[test]
    fn result_returning_test() -> Result<(), String> {
        let n = parse_positive(" 42 ")?;
        assert_eq!(n, 42);
        assert_eq!(parse_positive("7")?, 7);
        Ok(())
    }

    /// 失败分支同样要测：用 `expect_err` 取出 `Err`，再断言错误内容。
    #[test]
    fn parse_positive_rejects_zero() {
        let err = parse_positive("0").expect_err("0 不是正数，必须返回 Err");
        assert_eq!(err, "数值必须大于 0");
        assert!(parse_positive("abc").is_err(), "非数字输入必须返回 Err");
    }

    /// `#[should_panic(expected = "...")]`：用例“通过”的条件是内部 panic，
    /// 且 panic 消息**包含** `expected` 子串。
    ///
    /// ⚠️ 常见坑: 省略 `expected` 时任何 panic 都算通过——包括手滑写出的
    /// 数组越界或 `unwrap` 到 `None`，于是 bug 被测试“认证”为正常行为。
    #[test]
    #[should_panic(expected = "除数不能为 0")]
    fn divide_by_zero_panics() {
        let _ = divide(10, 0);
    }

    /// 基准测试的参照实现（reference implementation）也要有测试，
    /// 否则你只是在精确地测量一个错误的结果。
    #[test]
    fn fib_matches_naive_recursive() {
        // 朴素递归版：O(2^n)，只用来做小规模交叉验证。
        fn naive(n: u64) -> u64 {
            if n < 2 {
                n
            } else {
                naive(n - 1) + naive(n - 2)
            }
        }
        assert_eq!(fib(0), 0);
        assert_eq!(fib(1), 1);
        for n in 0..=20 {
            assert_eq!(fib(n), naive(n), "fib({n}) 与递归参照实现不一致");
        }
    }
}
