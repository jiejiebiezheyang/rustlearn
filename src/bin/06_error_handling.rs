//! 第 06 章：错误处理
//!
//! Rust 把错误分成两类：
//! - **不可恢复错误**（unrecoverable）：`panic!` 直接终止当前线程，代表"程序有 bug"。
//! - **可恢复错误**（recoverable）：`Result<T, E>` 把失败编码进类型，交给调用者决定怎么办。
//!
//! 没有异常（exception）意味着你**无法忽略错误**：忘了处理 `Result` 会得到 `#[must_use]`
//! 警告，而 `?` 把"检查并向上传播"压缩成一个字符。学完本章你能为任何模块设计出
//! 清晰的错误边界：什么就地恢复、什么向上抛。
//!
//! 运行：`cargo run --bin 06_error_handling`
//!
//! 关键知识点：
//! - `panic!` 与不可恢复错误
//! - `Result<T, E>`、`unwrap` 与 `expect`
//! - `?` 运算符、`From` 自动转换、错误向上传播
//! - 自定义错误枚举 + `Display` + `std::error::Error`
//! - `Box<dyn Error>`
//! - 何时就地恢复、何时向上抛
//! - `main` 返回 `Result`

use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

/// 自定义错误类型：一个枚举就够了。
///
/// 解决的问题：库作者不该替调用者决定"打印还是退出"，而要给出**可编程判断**的错误值，
/// 让调用者自己决定重试、兜底还是上报。
/// `#[derive(Debug)]` 是 `std::error::Error` 的硬性前提之一（另一个是 `Display`）。
#[derive(Debug)]
enum TempError {
    /// 输入为空
    Empty,
    /// 不是整数：由 `?` 通过 `From<ParseIntError>` 自动构造
    NotAnInteger(ParseIntError),
    /// 数值超出物理上合理的摄氏范围
    OutOfRange { celsius: i32 },
}

/// `Display` 面向"给人看的错误描述"，习惯上小写、结尾不加句号。
///
/// 它决定 `println!("{e}")` 和 `?` 之外的上报文案长什么样。
impl fmt::Display for TempError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TempError::Empty => write!(f, "温度输入不能为空"),
            TempError::NotAnInteger(source) => write!(f, "温度不是合法整数: {source}"),
            TempError::OutOfRange { celsius } => {
                write!(f, "温度 {celsius}°C 超出合理范围 [-100, 100]")
            }
        }
    }
}

/// 实现 `std::error::Error` 之后，这个类型就能装进 `Box<dyn Error>`，
/// 也会被 `?` 自动装箱（标准库有 `impl From<E: Error> for Box<dyn Error>`）。
impl Error for TempError {
    /// `source()` 暴露底层错误链，方便打印 `Caused by:` 之类的上下文。
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            TempError::NotAnInteger(source) => Some(source),
            _ => None,
        }
    }
}

/// `From` 就是 `?` 的燃料：`expr?` 出错时会调用 `From::from` 把错误转成函数签名里的类型。
///
/// ⚠️ 常见坑: 这里丢掉了"是哪个字段出错"这类上下文；真实项目常用 `thiserror`
/// 或手写携带 key 的变体来保留上下文。
impl From<ParseIntError> for TempError {
    fn from(source: ParseIntError) -> Self {
        TempError::NotAnInteger(source)
    }
}

/// 演示 `panic!`：不可恢复错误。
///
/// 什么时候该 panic：调用方本该保证的不变量（invariant）被打破了，
/// 继续跑只会产生更隐蔽的错误，此时快速失败（fail fast）更好。
/// ⚠️ 常见坑: `panic!` 不能被 `?` 捕获，只有 `catch_unwind` 能拦下它，
/// 而它只适合极少数边界场景，绝不能当常规错误处理。这里包一层只为让示例继续运行。
/// 演示期间会临时装一个静默 hook：默认 hook 打印的 panic 消息带线程 id，会让输出不可复现。
fn panic_demo() -> bool {
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let outcome = std::panic::catch_unwind(|| {
        let n = -1;
        if n < 0 {
            panic!("不变量被打破：n 必须是正数，实际是 {n}");
        }
        n
    });
    // 演示结束立刻还原 hook，避免影响同一进程里的其他线程或后续代码
    std::panic::set_hook(previous_hook);
    // 返回 Err 说明闭包内确实发生了 panic，而且已被拦下、程序继续运行
    outcome.is_err()
}

/// 解析温度：`Result<T, E>` + `?` + `From` 自动转换的完整示范。
///
/// 返回 `Ok(celsius)` 表示解析成功；`Err(TempError)` 用枚举精确说明失败原因。
fn parse_temperature(raw: &str) -> Result<i32, TempError> {
    if raw.trim().is_empty() {
        return Err(TempError::Empty);
    }
    // `?` 的语义：Ok 就解包取值；Err 就 `return Err(From::from(e))` 提前返回。
    // 这里 ParseIntError 会被自动转成 TempError::NotAnInteger。
    let celsius: i32 = raw.trim().parse()?;
    if !(-100..=100).contains(&celsius) {
        return Err(TempError::OutOfRange { celsius });
    }
    Ok(celsius)
}

/// `unwrap` / `expect`：失败就 panic 的快捷方式。
///
/// 只在"失败就说明代码有 bug"或原型、测试里用。
/// ⚠️ 常见坑: 生产代码里的 `unwrap` 会把可恢复错误升级成崩溃；
/// `expect` 至少能写下你的假设，panic 信息里一目了然。
fn unwrap_and_expect() -> i32 {
    let known: i32 = "42".parse().unwrap();
    // 输入由调用方保证，失败即 bug，所以用 expect 而不是 unwrap
    let other: i32 = "8".parse().expect("字面量 \"8\" 一定可以解析成 i32");
    known + other
}

/// `Box<dyn Error>`：当函数可能冒出多种错误类型、又不想为它们定义枚举时使用。
///
/// 解决的问题：`?` 要求统一的错误类型，`Box<dyn Error>` 用 trait object 擦除具体类型。
/// ⚠️ 常见坑: 装箱会丢掉类型信息（调用方难以精确匹配），还有一次堆分配；
/// 库的公开 API 更适合用具体的错误枚举。
fn sum_csv(text: &str) -> Result<i32, Box<dyn Error>> {
    let mut parts = text.split(',');
    // `&str` 也能转成 `Box<dyn Error>`（标准库为它实现了 From）
    let first = parts.next().ok_or("缺少第一个数字")?;
    let second = parts.next().ok_or("缺少第二个数字")?;
    // ParseIntError 同样会被自动装箱
    let sum = first.trim().parse::<i32>()? + second.trim().parse::<i32>()?;
    Ok(sum)
}

/// 就地恢复（recover in place）：失败可预期且可容忍时，本层直接处理掉。
///
/// 判断标准：失败是**正常业务流的一部分**（用户输错、配置可缺省），
/// 且当前层有足够信息决定兜底值。
fn temperature_or_default(raw: &str) -> i32 {
    match parse_temperature(raw) {
        Ok(celsius) => celsius,
        Err(error) => {
            // 值得记录的仍然打印，但不再往上抛
            println!("  → 忽略非法输入（{error}），就地恢复为默认温度 20");
            20
        }
    }
}

/// 向上抛（propagate）：当前层没有足够信息决定怎么办时，把错误交给调用者。
///
/// ⚠️ 常见坑: "到处 `?`、最后在 `main` 里统一打印"会让上下文一路丢失；
/// 每跨一层，最好补上只有这一层才知道的信息（哪个文件、哪条记录）。
fn average_temperature(readings: &[&str]) -> Result<f64, TempError> {
    let mut total = 0_i32;
    let mut count = 0_i32;
    for raw in readings {
        // 一条读数失败就整体失败：? 会立刻带着原始错误返回
        total += parse_temperature(raw)?;
        count += 1;
    }
    Ok(f64::from(total) / f64::from(count))
}

/// 程序入口：按小节顺序演示本章所有知识点。
///
/// `main` 自己也能返回 `Result<(), E>`：返回 `Err` 时 Rust 会打印错误的 `Debug`
/// 表示并以非 0 退出码结束；返回 `Ok(())` 则以 0 退出。
/// ⚠️ 常见坑: 打印的是 `Debug` 而不是 `Display`，所以错误类型最好同时实现两者。
fn main() -> Result<(), Box<dyn Error>> {
    println!("=== 1. panic! 与不可恢复错误 ===");
    let panicked = panic_demo();
    println!("  闭包内发生 panic = {panicked}（已被 catch_unwind 拦下）");

    println!("\n=== 2. Result<T, E>：把失败编码进类型 ===");
    for raw in ["36", "", "abc", "999"] {
        match parse_temperature(raw) {
            Ok(celsius) => println!("  {raw:?} -> Ok({celsius})"),
            Err(error) => println!("  {raw:?} -> Err({error})"),
        }
    }

    println!("\n=== 3. unwrap 与 expect ===");
    println!("  42 + 8 = {}", unwrap_and_expect());

    println!("\n=== 4. `?` 运算符、From 自动转换与 source 错误链 ===");
    match parse_temperature("abc") {
        Ok(celsius) => println!("  意外成功: {celsius}"),
        Err(error) => {
            println!("  错误: {error}");
            match error.source() {
                Some(source) => println!("  根因: {source}"),
                None => println!("  没有更底层的错误"),
            }
        }
    }
    // 空错误变体走的是另一个分支，说明枚举让调用方可以精确匹配
    println!(
        "  空输入匹配到 Empty? {}",
        matches!(parse_temperature(""), Err(TempError::Empty))
    );

    println!("\n=== 5. 自定义错误枚举 + Display + std::error::Error ===");
    let range_error = TempError::OutOfRange { celsius: 999 };
    println!("  Display: {range_error}");
    println!("  Debug:   {range_error:?}");

    println!("\n=== 6. Box<dyn Error>：擦除具体错误类型 ===");
    for text in ["1, 2", "1", "x, 2"] {
        match sum_csv(text) {
            Ok(sum) => println!("  {text:?} -> Ok({sum})"),
            Err(error) => println!("  {text:?} -> Err({error})"),
        }
    }

    println!("\n=== 7. 就地恢复 vs 向上抛 ===");
    println!("  合法 \"36\" -> {}", temperature_or_default("36"));
    println!("  非法 \"abc\" -> {}", temperature_or_default("abc"));
    match average_temperature(&["20", "30", "40"]) {
        Ok(avg) => println!("  平均温度 = {avg}"),
        Err(error) => println!("  平均温度失败: {error}"),
    }
    match average_temperature(&["20", "oops", "40"]) {
        Ok(avg) => println!("  平均温度 = {avg}"),
        Err(error) => println!("  平均温度失败: {error}"),
    }

    println!("\n=== 8. main 返回 Result ===");
    // 成功路径用 ? 取值；若这里真的返回 Err，进程会以非 0 退出码结束
    let celsius = parse_temperature("36")?;
    println!("  main 用 ? 拿到温度 {celsius}，最后返回 Ok(()) 使退出码为 0");
    Ok(())
}
