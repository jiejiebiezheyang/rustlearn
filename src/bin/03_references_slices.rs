//! 第 03 章：引用与切片（references & slices）
//!
//! 第 02 章的 ownership 解决了“谁负责释放内存”，但如果每次传参都要交出所有权，
//! 代码会很难写。本章讲 Rust 的第二根支柱：**借用（borrowing）**——
//! 用 `&T` / `&mut T` 临时借用数据而不夺走所有权，再用切片（slice）“借一段”数组或字符串。
//! 借用在编译期由 borrow checker 检查，能静态排除数据竞争（data race）与迭代器失效。
//!
//! 运行：`cargo run --bin 03_references_slices`
//!
//! 关键知识点：
//! - `&T` 与 `&mut T` 的别名规则（aliasing rule）：同一时刻要么多个 `&T`，要么一个 `&mut T`
//! - NLL（非词法生命周期，Non-Lexical Lifetimes）
//! - `String` 与 `&str` 的关系与解引用强制转换（deref coercion）
//! - UTF-8 字符边界与字符串切片 panic 坑
//! - 数组切片 `&[T]` 与可变切片 `&mut [T]`
//! - `get()` 与索引 `[]` 的区别
//! - 常用字符串方法：`split`、`trim`、`starts_with`

use std::panic::{catch_unwind, set_hook, take_hook, UnwindSafe};

/// 本章演示 UTF-8 字符边界的固定字符串：8 个字符，但占 12 个字节。
const SAMPLE_UTF8: &str = "你好, Rust";

/// 只读借用 `&str`，返回字节长度。示例调用：`borrowed_len("abc")`。
///
/// 参数是借用而非 `String`，所以调用方仍然拥有原字符串。
fn borrowed_len(text: &str) -> usize {
    text.len()
}

/// 可变借用 `&mut String`，末尾追加感叹号。示例：`push_exclamation(&mut text)`。
fn push_exclamation(text: &mut String) {
    text.push('!');
}

/// 在“静音 panic”的前提下运行 `f`，返回它是否 panic 了。
///
/// `&s[0..1]`、`data[99]` 这类访问会 panic，而本章既要展示这些坑、
/// 又要让示例以退出码 0 跑完，所以临时把 panic hook 换成空实现
/// （默认 hook 会往 stderr 打印），再用 `catch_unwind` 捕获并恢复 hook。
fn panics<F: FnOnce() + UnwindSafe>(f: F) -> bool {
    let previous = take_hook();
    set_hook(Box::new(|_| {}));
    let caught = catch_unwind(f).is_err();
    set_hook(previous);
    caught
}

/// 演示不可变借用 `&T`：借用方只能读，所有权仍在原变量手里。
fn demo_shared_borrow() {
    let owned = String::from("borrow me");
    // `&String` 通过解引用强制转换（deref coercion）自动变成 `&str`
    let len = borrowed_len(&owned);
    // ⚠️ 常见坑: 忘写 `&` 会把 String move 进函数，函数返回后再用 owned 会报 E0382
    println!("{owned:?} 借用后仍然可用，字节长度 = {len}");
}

/// 演示别名规则（aliasing rule）与可变借用 `&mut T`。
///
/// 规则：同一时刻要么任意多个 `&T`，要么恰好一个 `&mut T`，两者不能并存。
fn demo_aliasing_rule() {
    let mut text = String::from("borrow");
    // 多个只读借用共存完全合法：谁都不能改，不会互相干扰
    let a = &text;
    let b = &text;
    println!("两个只读借用同时存在: {a:?} / {b:?}");
    // 下面三行编译不过（E0502: cannot borrow as mutable because it is also borrowed
    // as immutable）——读引用还活着时不允许可变借用：
    //   let r = &text;
    //   text.push('!');
    //   println!("{r}");
    push_exclamation(&mut text);
    println!("可变借用修改后 = {text:?}");
}

/// 演示 NLL（非词法生命周期）：借用的有效期由**最后一次使用**决定，
/// 而不是由所在代码块的 `}` 决定。
fn demo_nll() {
    let mut data = vec![1, 2, 3];
    let first = &data[0];
    // first 的最后一次使用；Rust 2015 的词法生命周期（lexical lifetimes）
    // 会认为借用持续到块尾，从而让下面的 push 报 E0502，NLL 则允许
    println!("借用期间读到第一个元素 = {first}");
    data.push(4);
    println!("push 之后 data = {data:?}");

    let tail = &data[1..];
    println!("tail = {tail:?}");
    // ⚠️ 常见坑: NLL 不是“引用随时自动失效”。只要引用后面还会被用到，
    //            编译器就不会放行这次可变借用（仍然是 E0502）。
    data.push(5);
    println!("data = {data:?}");
}

/// 演示 `String` 与 `&str` 的关系：`String` 拥有堆上的 UTF-8 缓冲区（可增删改），
/// `&str` 是借来的 UTF-8 视图（胖指针：地址 + 字节长度），不能修改。
fn demo_string_and_str() {
    let owned: String = String::from("hello 世界");
    // 三种把 String 当 &str 用的方式，都不发生拷贝
    let via_deref: &str = &owned;
    let via_as_str: &str = owned.as_str();
    let head: &str = &owned[0..5];
    let (bytes, chars) = (owned.len(), owned.chars().count());
    println!("owned = {owned:?}: {bytes} 字节 / {chars} 个字符");
    println!("via_deref = {via_deref:?}, via_as_str = {via_as_str:?}, head = {head:?}");
    // ⚠️ 常见坑: `&str` 不能 push；拼接要先 `format!` 或 `to_string()` 造出 String
    println!("拼接出新 String = {:?}", format!("{via_deref}!"));
}

/// 演示 UTF-8 字符边界：字符串切片按**字节**下标切，
/// 切在非字符边界（char boundary）上会 panic。
fn demo_utf8_boundary() {
    let text = SAMPLE_UTF8; // 每个汉字 3 字节，所以 0..1 落在“你”的中间
    println!(
        "{text:?} 共 {} 字节 / {} 个字符",
        text.len(),
        text.chars().count()
    );

    // 正确做法 1：按字符找字节边界，`char_indices` 给出每个字符的起始字节下标
    let end = text
        .char_indices()
        .nth(2)
        .map(|(index, _)| index)
        .unwrap_or(text.len());
    println!("安全切片 [0..{end}] = {:?}", &text[0..end]);

    // 正确做法 2：完全不管字节下标，用 chars() 收集成新 String
    let first_two: String = text.chars().take(2).collect();
    println!("chars().take(2) = {first_two:?}");

    // 错误做法：&text[0..1] → panic: byte index 1 is not a char boundary
    let did_panic = panics(|| {
        let bad = &text[0..1];
        println!("这行不会执行: {bad:?}");
    });
    println!("&text[0..1] 是否 panic: {did_panic}");
    // ⚠️ 常见坑: 用 `s[..n]` 截断用户输入（昵称、标题）极易 panic；
    //            要么先用 `is_char_boundary` 校验，要么改用 `chars()` API。
}

/// 返回切片所有元素之和。参数写成 `&[T]`（而不是 `&Vec<T>`）后，
/// 同一个函数就能同时接受数组、`Vec` 和子切片——切片是最通用的只读视图。
fn sum_all(values: &[i32]) -> i32 {
    values.iter().sum()
}

/// 把切片里的每个元素就地翻倍，演示可变切片 `&mut [T]`。
fn double_all(values: &mut [i32]) {
    // ⚠️ 常见坑: 用 `for i in 0..values.len()` 加下标会触发 clippy 的 needless_range_loop，
    //            也更容易越界；迭代器版本既安全又更快。
    for value in values.iter_mut() {
        *value *= 2;
    }
}

/// 演示数组切片 `&[T]` 与可变切片 `&mut [T]`。
fn demo_slices() {
    let array = [1, 2, 3, 4, 5];
    let mut vector = vec![10, 20, 30];
    let tail = &array[2..]; // 子切片：与 array 共享同一块栈内存，没有拷贝
    println!("sum(array) = {}", sum_all(&array)); // &[i32; 5] → &[i32]
    println!("sum(vector) = {}", sum_all(&vector)); // &Vec<i32> → &[i32]
    println!("sum(tail) = {}", sum_all(tail)); // tail 是胖指针：地址 + 长度
    double_all(&mut vector); // &mut Vec<i32> → &mut [i32]
    println!("double_all 之后 vector = {vector:?}");
}

/// 演示 `get()` 与索引 `[]` 的区别：前者返回 `Option<&T>`（安全，需处理 None），
/// 后者越界直接 panic（不可恢复，只适合“越界即 bug”的场景）。
fn demo_get_vs_index() {
    let data = [10, 20, 30];
    println!("data.get(1) = {:?}", data.get(1)); // Some(&20)
    println!("data.get(99) = {:?}", data.get(99)); // None，程序继续跑

    let index = data.len() + 1; // 故意越界；用变量是为了避免编译期常量折叠检查
    let did_panic = panics(|| {
        let out_of_range = data[index];
        println!("这行不会执行: {out_of_range}");
    });
    println!("data[{index}] 是否 panic: {did_panic}");
    // ⚠️ 常见坑: 输入来自用户/文件时一律用 `get`（或 `get_mut`），
    //            别用 `[]`，否则一个畸形输入就能让整个服务 panic。
}

/// 把 `"alice:read, bob:write"` 这样的权限串解析成 `(用户, 动作)` 列表。
///
/// 演示 `trim`（去首尾空白）、`split`（按分隔符切）、`split_once`（切成两半）；
/// 返回值借用 `raw` 的片段，不做任何字符串拷贝。
fn parse_grants(raw: &str) -> Vec<(&str, &str)> {
    raw.trim()
        .split(',') // ⚠️ 常见坑: split 会产出空串（"a,,b" → ["a", "", "b"]），必须过滤
        .filter_map(|item| {
            let item = item.trim();
            if item.is_empty() {
                return None;
            }
            let (user, action) = item.split_once(':')?; // 没有 ':' 就返回 None，被跳过
            Some((user.trim(), action.trim()))
        })
        .collect()
}

/// 演示常用字符串方法：`split`、`trim`、`starts_with`。
fn demo_string_methods() {
    let raw = "  alice:read , bob:write,  , carol:admin ";
    for (user, action) in parse_grants(raw) {
        // starts_with 判断前缀，常用来做简单的分类/路由
        let level = if action.starts_with("adm") {
            "高权限"
        } else {
            "普通"
        };
        println!("{user} → {action}（{level}）");
    }
    // trim 也会去掉 \n 与 \t，所以处理整行输入时很常用
    println!(
        "trim 前 = {:?}, trim 后 = {:?}",
        "\t  keep me  \n",
        "\t  keep me  \n".trim()
    );
}

/// 程序入口：按小节顺序演示第 3 章的全部知识点。
fn main() {
    println!("=== 1. 不可变借用 &T ===");
    demo_shared_borrow();

    println!("\n=== 2. 可变借用 &mut T 与别名规则 ===");
    demo_aliasing_rule();

    println!("\n=== 3. NLL（非词法生命周期）===");
    demo_nll();

    println!("\n=== 4. String 与 &str ===");
    demo_string_and_str();

    println!("\n=== 5. UTF-8 字符边界与切片 panic ===");
    demo_utf8_boundary();

    println!("\n=== 6. 数组切片 &[T] / &mut [T] ===");
    demo_slices();

    println!("\n=== 7. get() 与索引 [] 的区别 ===");
    demo_get_vs_index();

    println!("\n=== 8. 常用字符串方法 split / trim / starts_with ===");
    demo_string_methods();
}
