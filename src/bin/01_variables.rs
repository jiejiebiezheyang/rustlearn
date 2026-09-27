//! 第 1 章：变量、常量、基本类型与可变性
//!
//! 本章打地基：Rust 的"变量"其实叫绑定（binding），默认**不可变**，需要改写必须显式
//! 写 `mut`。接着讲清 `const` 与 `static` 的区别、标量（scalar）与复合（compound）类型、
//! 类型推断（type inference）、整数溢出（integer overflow）的四种显式策略、`as` 强制
//! 转换，以及"一切皆表达式"（everything is an expression）。
//!
//! 运行：`cargo run --bin 01_variables`
//!
//! 关键知识点：`let`/`mut`/遮蔽（shadowing）；`const` 与 `static`；整数、浮点、`bool`、
//! `char`；tuple 与 array；类型推断与显式标注；整数溢出（debug 下 panic、`wrapping_*`、
//! `checked_*`、`saturating_*`、`overflowing_*`）；数值字面量的下划线与类型后缀；
//! `as` 转换；块表达式与"一切皆表达式"。

use std::mem::size_of;

/// 本章标题：`const` 在编译期求值后被内联（inline）到每个使用点，不占运行时内存。
const CHAPTER_TITLE: &str = "第 1 章：变量、常量、基本类型与可变性";

/// 满分常量：演示 `const` 可用在数组字面量等编译期位置。
const MAX_SCORE: u32 = 100;

/// 缓冲区长度：演示 `const` 可作为数组长度 `[T; N]` 里的 `N`。
const BUFFER_LEN: usize = 4;

/// 全局静态量：`static` 地址唯一、生命周期为 `'static`，程序运行期间一直存在。
static PROGRAM_NAME: &str = "rustlearn";

/// 演示 `let` 的默认不可变性与 `mut`：意外二次赋值是常见的 bug 源，把"可变"变成必须
/// 显式声明的东西后，编译器就能指出所有非预期写入。什么时候用 `mut`：确实要重新赋值时。
fn demo_let_and_mut() {
    let immutable = 10;
    // immutable = 11; // ❌ 编译错误：cannot assign twice to immutable variable
    println!("不可变绑定 immutable = {immutable}");

    let mut mutable = 10;
    mutable += 1; // 只有声明为 mut 的绑定才能重新赋值

    // ⚠️ 常见坑: `mut` 是"允许修改"而非"必须修改"；能不加就不加，读代码更省心。
    println!("可变绑定 mutable = {mutable}");
}

/// 演示遮蔽（shadowing）：用新的同名 `let` 绑定覆盖旧绑定。同名但类型/含义不同的中间值
/// 不必另起名字；与 `mut` 的区别是遮蔽创建**新绑定**，因此可以改变类型。
fn demo_shadowing() {
    let spaces = "   "; // 类型 &str
    let spaces = spaces.len(); // 同名新绑定，类型变成 usize
    println!("遮蔽可以改变类型：spaces = {spaces}");

    let x = 5;
    let x = x + 1; // 新绑定基于旧值计算
    {
        let x = x * 2; // 内层作用域的遮蔽，出了这个块就失效
        println!("内层 x = {x}");
    }
    println!("回到外层 x = {x}");
    // ⚠️ 常见坑: 遮蔽容易让"同一概念的两个值"重名而降低可读性，
    // 只在类型转换或逐步加工同一份数据时使用。
}

/// 演示 `const` 与 `static` 的用法与区别。用 `const`：值编译期可算、要放在数组长度等
/// 编译期位置、想零开销复用字面量。用 `static`：需要唯一固定地址（全局配置、FFI 全局量）。
fn demo_const_and_static() {
    let scores: [u32; 3] = [90, 95, MAX_SCORE]; // 常量可出现在数组字面量里
    let buffer = [0u8; BUFFER_LEN]; // 常量可作为数组长度
    println!("{PROGRAM_NAME} 满分 = {MAX_SCORE}，示例 = {scores:?}");
    println!("BUFFER_LEN = {BUFFER_LEN}，buffer.len() = {}", buffer.len());

    let name: &'static str = PROGRAM_NAME; // static 可直接借用为 'static
    println!("static 可长期借用：{name}");
    // ⚠️ 常见坑: `static mut` 需要 unsafe 且极易造成数据竞争（data race）；
    // 本系列不用 unsafe，可变全局状态请用 `OnceLock` / `Mutex`（第 11、14 章）。
    // ⚠️ 常见坑: `const` 没有固定地址，`&CONST` 可能指向不同的临时副本，
    // 不要靠地址做身份判断（identity）；需要地址唯一就用 `static`。
}

/// 演示四类标量类型：整数、浮点、`bool`、`char`。整数选择：优先 `i32`；索引/长度用
/// `usize`；与外部协议对齐用固定宽度类型（`u8`/`i16`/`u64`…）；`i128` 只留给超大整数。
/// `char` 表示单个 Unicode 标量值（Unicode scalar value），而不是一个字节。
fn demo_scalar_types() {
    let int_default = 42; // 整数字面量默认推断为 i32
    let float_default = 3.5; // 浮点字面量默认推断为 f64
    let small: u8 = 255;
    let index: usize = 0;
    let big: i128 = 170_141_183_460_469_231_731_687_303_715_884_105_727;
    let (si32, sf64) = (size_of::<i32>(), size_of::<f64>());
    let (su8, susize) = (size_of::<u8>(), size_of::<usize>());

    println!("i32 {si32} 字节，f64 {sf64} 字节");
    println!("u8 {su8} 字节，usize {susize} 字节（随平台而变）");
    println!("int = {int_default}，small = {small}，index = {index}");
    println!("float = {float_default}，big 的末位 = {}", big % 10);

    let is_ready: bool = true;
    let chinese: char = '中';
    let escaped: char = '\u{4E2D}'; // 转义写法，与 '中' 是同一个字符
    println!("bool = {is_ready}，char = {chinese} / {escaped}");
    println!("char 占 {} 字节", size_of::<char>());
    println!("'中' 的 UTF-8 编码占 {} 字节", chinese.len_utf8());
    // ⚠️ 常见坑: `char` 是 4 字节的 Unicode 标量值，不是 C 的 1 字节 `char`，
    // 也不能直接索引字符串里的"第 n 个字符"（UTF-8 边界见第 3 章）。
    // ⚠️ 常见坑: `usize` 在 32 位平台只有 4 字节，`as u32` 不是跨平台安全转换。
}

/// 演示复合类型：tuple（元组）与 array（数组）。tuple 用于临时打包异构值、或让函数返回
/// 多个值；array 要求元素同类型且长度是类型的一部分（`[T; N]`），因此整体放在栈上，
/// 没有堆分配开销。
fn demo_compound_types() {
    let person: (i32, f64, char) = (30, 1.75, 'C');
    let (age, height, unit) = person; // 解构（destructuring）绑定
    println!("解构：age = {age}, height = {height}, unit = {unit}");
    println!("下标：person.0 = {}, person.2 = {}", person.0, person.2);
    // ⚠️ 常见坑: tuple 下标必须是编译期字面量（`person.0` 合法，`person[i]` 不合法）；
    // 字段超过 3~4 个就该定义 struct（第 4 章）。

    let primes: [i32; 5] = [2, 3, 5, 7, 11];
    let zeros = [0u8; 4]; // [值; 长度] 重复初始化，要求元素是 Copy
    println!("primes = {primes:?}，zeros = {zeros:?}");
    // ⚠️ 常见坑: `[i32; 5]` 与 `[i32; 6]` 是不同类型，不能互相赋值；
    // 长度要到运行时才知道时请用 `Vec`（第 9 章）。
}

/// 演示类型推断（type inference）与显式标注。推断是局部且双向的：编译器会结合字面量、
/// 后续用法乃至函数签名反推类型；一旦推断不唯一或想固定语义，就显式标注（或 turbofish）。
fn demo_type_inference() {
    let inferred = 42; // 字面量默认推断为 i32
    let explicit: u64 = 42; // 需要无符号或更大范围时显式标注
    let from_use = 6; // 类型由 `i32::pow` 的签名反推为 i32
    let squared = i32::pow(from_use, 2);
    let parsed: u32 = "1024".parse().unwrap(); // 解析目标类型由标注确定
    println!("inferred = {inferred}，explicit = {explicit}");
    println!("squared = {squared}，parsed = {parsed}");

    let flagged: u8 = 0b1010_1010; // 二进制字面量 + 下划线分隔
    let mask = 0xFF_u16; // 后缀：类型直接写在字面量末尾
    println!("flagged = {flagged}，mask = {mask}");
    // ⚠️ 常见坑: 推断不出来会报 "type annotations needed"，例如
    // `let v = "1".parse().unwrap(); println!("{v}");` —— 字面量和用法都没给线索。
    // 两种解法：`let v: i32 = ...`（标注）或 turbofish `"1".parse::<i32>().unwrap()`。
}

/// 演示数值字面量的写法：下划线分隔、进制前缀、类型后缀、字节字面量。这些写法编译后
/// 完全等价，只为可读性服务：下划线表示数量级，进制前缀用于表达位模式（掩码、权限位）。
fn demo_numeric_literals() {
    let decimal = 1_000_000u32; // 十进制 + u32 后缀
    let hex = 0xFF_EC_D1; // 十六进制
    let octal = 0o755; // 八进制（Unix 权限位常用）
    let binary = 0b1100_0011u8; // 二进制 + 后缀，位掩码常用
    let byte = b'A'; // 字节字面量：类型是 u8，值为 65
    let scientific = 1.5e3_f64; // 科学计数法 = 1500.0

    println!("decimal = {decimal}，hex = {hex}，octal = {octal}");
    println!("binary = {binary}，byte = {byte}，scientific = {scientific}");
    // ⚠️ 常见坑: 进制前缀写漏不会报错，只是含义完全变了
    // （把 `0o755` 写成 `755` 照样编译）；读代码时要看清前缀。
    // ⚠️ 常见坑: 整数字面量默认 i32，`3_000_000_000` 会编译报错（超出 i32::MAX），
    // 需要显式后缀 `3_000_000_000u64` 或类型标注。
}

/// 演示整数溢出（integer overflow）的四种显式处理策略。Rust 在 **debug** 构建下让溢出
/// panic，在 **release** 下默认按二进制补码回绕（wrapping）——两种行为不一致。所以只要
/// 溢出可能是业务语义（哈希、计数器回绕、位运算），就应显式选策略，而不是依赖构建模式。
fn demo_overflow() {
    let max = u8::MAX; // 255
    println!("max = {max}");
    println!("wrapping_add(1)    = {}", max.wrapping_add(1)); // 0：回绕
    println!("checked_add(1)     = {:?}", max.checked_add(1)); // None：显式判空
    println!("saturating_add(1)  = {}", max.saturating_add(1)); // 255：饱和到边界
    println!("overflowing_add(1) = {:?}", max.overflowing_add(1)); // (0, true)：值 + 标志

    // checked_* 适合"溢出就是错误"，可配合 `?` / match 转成 Result 向上传播
    let quota: u16 = 60_000;
    let used: u16 = 5_000;
    match quota.checked_sub(used) {
        Some(left) => println!("剩余配额 = {left}"),
        None => println!("配额不足"),
    }
    // ⚠️ 常见坑: `cargo run` 与 `cargo run --release` 的溢出行为不同，别用
    // "release 下没 panic"证明代码正确，该判空就用 checked_*。
    // ⚠️ 常见坑: 直接写 `max + 1` 会在 debug 下 panic 并让退出码非 0，所以本章不
    // 演示它；想亲眼看到，把它替换成 `max + 1` 再 `cargo run`。
}

/// 演示 `as` 强制类型转换（cast）。`as` 只做数值/位模式层面的转换，**不做范围检查**，
/// 因此显式且有损。什么时候用：整数之间刻意截断或扩展、`char` 与整数互转、枚举转整数。
fn demo_as_cast() {
    let wide: i32 = 300;
    let truncated = wide as u8; // 只保留低 8 位：300 % 256 = 44
    let widened = truncated as i32; // 由 u8 无符号扩展
    println!("300 as u8 = {truncated}，再 as i32 = {widened}");

    let pi = 3.99_f64;
    println!("3.99_f64 as i32 = {}", pi as i32); // 3：向零截断，不是四舍五入
    println!("(-3.99_f64) as i32 = {}", (-3.99_f64) as i32); // -3

    let letter: char = 'A';
    let code = letter as u32; // char -> u32 是无损的
    println!("'A' as u32 = {code}");
    // ⚠️ 常见坑: `as` 既不 panic 也不报错，越界是静默截断，属于数值 bug 高发区；
    // 需要"越界即错误"时改用返回 Result 的 `u8::try_from(wide)`（第 6 章）。
    // ⚠️ 常见坑: 浮点转整数会截断小数，超出目标范围时**饱和**到边界值（NaN 得 0）；
    // 与 C 的未定义行为不同，但同样不该依赖。
}

/// 演示"一切皆表达式"（everything is an expression）：块、`if`、`loop` 都有值。不必先
/// `let mut x;` 再在每个分支里赋值，于是不可能出现"某个分支忘了赋值"；块还自带作用域。
fn demo_block_expression() {
    let y = {
        let x = 3; // x 只在块内可见
        x + 1 // 最后一行不加分号：它的值就是整个块的值
    };
    println!("块表达式的值 y = {y}");

    let n = 7;
    let label = if n > 5 { "big" } else { "small" }; // if 是表达式而非语句
    println!("{n} 属于 {label}");

    let mut counter = 0;
    let doubled = loop {
        counter += 1;
        if counter == 5 {
            break counter * 2; // break 可以带值，因此 loop 能当表达式用
        }
    };
    println!("loop 以 break 返回值：doubled = {doubled}");
    // ⚠️ 常见坑: 块的最后一行多写一个分号就变成语句，值成了 `()`，
    // 典型报错是 "expected integer, found `()`" / "mismatched types"。
    // ⚠️ 常见坑: `if` 当表达式时各分支必须返回同一类型，而且不能省略 `else`。
}

/// 程序入口：按小节顺序演示本章所有知识点。
fn main() {
    println!("{CHAPTER_TITLE}（示例程序：{PROGRAM_NAME}）");

    println!("\n=== 1. let / mut 与遮蔽（shadowing） ===");
    demo_let_and_mut();
    demo_shadowing();

    println!("\n=== 2. const 与 static 的区别 ===");
    demo_const_and_static();

    println!("\n=== 3. 标量、复合类型与数值字面量 ===");
    demo_scalar_types();
    demo_compound_types();
    demo_numeric_literals();

    println!("\n=== 4. 类型推断、显式标注与 as 转换 ===");
    demo_type_inference();
    demo_as_cast();

    println!("\n=== 5. 整数溢出（integer overflow） ===");
    demo_overflow();

    println!("\n=== 6. 块表达式与「一切皆表达式」 ===");
    demo_block_expression();
}
