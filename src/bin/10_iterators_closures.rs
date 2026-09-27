//! 第 10 章：迭代器与闭包（iterators & closures）
//!
//! 迭代器是 Rust 的「数据流水线」：`iter()` 产生迭代器，`map`/`filter` 这类**适配器
//! （adapter）** 把它接成新的迭代器，最后 `collect`/`fold`/`for` 这类**消费者（consumer）**
//! 才真正驱动它跑起来。整条链是惰性（lazy）的，中间不产生临时集合，元素一个接一个流过
//! 整条流水线，因此既省内存，也容易被优化成零成本抽象（zero-cost abstraction）。
//!
//! 闭包（closure）是「能捕获环境变量的匿名函数」，它让 `map`/`filter` 能直接用上外层
//! 局部变量。闭包按捕获方式自动实现 `Fn`/`FnMut`/`FnOnce`，泛型函数用哪个作约束，
//! 就决定了它接受多「强」的闭包。
//!
//! 运行：`cargo run --bin 10_iterators_closures`
//!
//! 关键知识点：
//! - 闭包语法与捕获（不可变借用 / 可变借用 / `move`）
//! - `Fn` / `FnMut` / `FnOnce`
//! - 迭代器惰性（lazy）
//! - 适配器 `map` / `filter` / `fold` / `collect` / `enumerate` / `zip` /
//!   `chain` / `flat_map` / `take`
//! - 自定义 `Iterator` 实现
//! - `iter()` vs `into_iter()`

use std::collections::BTreeSet;

/// 把 `f` 应用到切片每个元素，演示 `Fn` 约束（可多次调用、只读环境）。
fn apply_all<F: Fn(i32) -> i32>(xs: &[i32], f: F) -> Vec<i32> {
    xs.iter().map(|x| f(*x)).collect()
}

/// 连续调用闭包两次，演示 `FnMut` 约束（闭包内部带可变状态）。
fn call_twice<F: FnMut() -> u32>(mut f: F) -> (u32, u32) {
    (f(), f())
}

/// 调用一次并交出结果，演示 `FnOnce` 约束（闭包会消费自己捕获的值）。
fn consume_once<F: FnOnce() -> String>(f: F) -> String {
    f()
}

/// 演示闭包的三种捕获方式：不可变借用、可变借用、`move`。
///
/// 捕获的「强度」由编译器推断：只读就借 `&`，要改就借 `&mut`，写了 `move` 就夺走所有权。
fn demo_closure_capture() {
    let greeting = String::from("hello");
    // 不可变借用捕获：闭包只读 greeting，用完之后 greeting 依然可用。
    let len_of_greeting = || greeting.len();
    let len = len_of_greeting();
    println!("不可变借用捕获 len = {len}，greeting 仍可用 = {greeting}");

    // 可变借用捕获：闭包体改了 counter，所以闭包变量本身必须声明为 mut。
    let mut counter = 0;
    let mut bump = || {
        counter += 1;
        counter
    };
    let (first, second) = (bump(), bump());
    // bump 的最后一次使用在上面，借用到此结束，现在可以读 counter。
    println!("可变借用捕获 {first} -> {second}，外部 counter = {counter}");

    // move 捕获：把变量**移动**进闭包；需要 'static 的场景（如 thread::spawn）必须用它。
    let owned = String::from("moved");
    let take_owned = move || owned.len();
    println!("move 捕获 len = {}", take_owned());
    // ⚠️ 常见坑: move 之后外部不能再使用 owned，取消下一行注释会报 E0382（use of moved value）。
    // println!("{owned}");
}

/// 演示 `Fn`/`FnMut`/`FnOnce` 的区别与递进关系。
///
/// 三者是**继承关系**：`Fn: FnMut: FnOnce`。任何 `Fn` 都能当 `FnMut`/`FnOnce` 用，
/// 反之不行。所以泛型约束写得越「弱」（写 `FnOnce`），能接受的闭包越多，但你能对它
/// 做的事也越少（`FnOnce` 只能调用一次）。
fn demo_fn_traits() {
    // 只读捕获 + 可多次调用 → 自动实现 Fn。
    let base = 10;
    let add_base = move |x: i32| x + base;
    println!(
        "Fn（可多次调用、只读环境）: {:?}",
        apply_all(&[1, 2, 3], add_base)
    );

    // 捕获可变引用 + 可多次调用 → FnMut。
    let mut calls = 0;
    let tick = || {
        calls += 1;
        calls
    };
    println!("FnMut（可多次调用、会改环境）: {:?}", call_twice(tick));
    println!("外部看到 calls = {calls}");

    // 把捕获的 String 移出闭包 → 只能调用一次 → FnOnce。
    let payload = String::from("consumed");
    let give_away = move || payload;
    println!("FnOnce（只能调用一次）: {}", consume_once(give_away));
}

/// 演示迭代器的惰性（lazy）：适配器只「接线」，不消费就不执行。
///
/// `map`/`filter` 返回的是新的迭代器结构体，它们既不遍历也不分配内存；只有
/// `collect`/`fold`/`sum`/`for` 这些消费者驱动时元素才会流动。
/// ⚠️ 常见坑: 忘了消费适配器就等于什么都没做，编译器会用 `unused_must_use` 警告提醒你。
fn demo_lazy() {
    let values = [1, 2, 3, 4, 5];
    println!("下面这行之后、collect 之前，不应该出现任何 map 输出：");
    let doubled: Vec<i32> = values
        .iter()
        .map(|x| {
            // 这条 println 只在被消费时执行，它就是「惰性」的证据。
            println!("  map 拿到 {x}");
            x * 2
        })
        .filter(|x| *x > 4)
        .collect();
    println!("collect 触发整条流水线，结果 = {doubled:?}");
}

/// 演示适配器 `map`/`filter` 与消费者 `fold`/`collect`。
///
/// `filter` 的闭包收到的是 `&Item`，要多解一层引用；`collect` 收进什么集合由目标类型决定。
fn demo_adapters() {
    let nums = [1, 2, 3, 4, 5, 6];

    let even_squares: Vec<i32> = nums
        .iter()
        .filter(|n| **n % 2 == 0)
        .map(|n| n * n)
        .collect();
    println!("偶数的平方 = {even_squares:?}");

    // fold(初值, |累加器, 元素| 新累加器)：最通用的消费者，累加器类型可与元素类型不同。
    let running: Vec<i32> = nums.iter().fold(Vec::new(), |mut acc, n| {
        let prev = acc.last().copied().unwrap_or(0);
        acc.push(prev + n);
        acc
    });
    println!("fold 前缀和 = {running:?}");

    // collect 目标类型不唯一时必须标注；BTreeSet 顺便做到去重 + 有序。
    let distinct_sorted: BTreeSet<i32> = nums.iter().copied().collect();
    println!("collect 成 BTreeSet = {distinct_sorted:?}");
}

/// 演示 `enumerate` / `zip` / `chain` / `flat_map` / `take` 这些「胶水」适配器。
///
/// 它们把多个迭代器拼起来、把嵌套结构摊平、按位置截取，让你不必手写下标循环。
fn demo_combinators() {
    let names = ["alice", "bob", "carol"];
    let scores = [85, 92, 78];

    // enumerate：附带从 0 开始的下标，替代手写 `for i in 0..v.len()`。
    let indexed: Vec<(usize, &str)> = names.iter().copied().enumerate().collect();
    println!("enumerate: {indexed:?}");

    // zip：两个迭代器按**较短**的一方配对，多出来的元素直接丢弃。
    let roster: Vec<(&str, i32)> = names.iter().copied().zip(scores).collect();
    println!("zip: {roster:?}");

    // chain：先耗尽左边再消费右边，用于拼接同 Item 类型的迭代器。
    let letters: Vec<char> = "abc".chars().chain("xyz".chars()).collect();
    println!("chain: {letters:?}");

    // flat_map = map + flatten：先把每个元素展开成一段序列，再把它们摊平。
    let lines = ["hello world", "rust lang"];
    let words: Vec<&str> = lines
        .iter()
        .flat_map(|line| line.split_whitespace())
        .collect();
    println!("flat_map: {words:?}");

    // take(n) 按位置截取，也是给无限迭代器「刹车」的手段（见最后一节）。
    let first_two: Vec<&str> = names.iter().copied().take(2).collect();
    println!("take(2): {first_two:?}");
}

/// 演示 `iter()` / `iter_mut()` / `into_iter()` 三者的区别。
///
/// - `iter()`：借用集合，`Item = &T`，集合之后仍可用；
/// - `iter_mut()`：可变借用，`Item = &mut T`，用来就地修改；
/// - `into_iter()`：**消费**集合，`Item = T`，交出所有权，之后集合不可再用。
///
/// `for x in collection` 对集合等价于 `into_iter()`；想借用要写 `for x in &collection`。
fn demo_iter_vs_into_iter() {
    // iter()：只借不拿，words 之后还可继续使用。
    let words = vec![String::from("rust"), String::from("learn")];
    let lengths: Vec<usize> = words.iter().map(|w| w.len()).collect();
    println!("iter() 得到长度 {lengths:?}，words 仍可用 = {words:?}");

    // into_iter()：拿走所有权，Item 是 String，words 从此不可再用。
    // ⚠️ 常见坑: 对 Vec 写 `for s in words` 等价于 `words.into_iter()`，循环过后
    // words 已被移动，再用会报 E0382。
    let upper: Vec<String> = words.into_iter().map(|w| w.to_uppercase()).collect();
    println!("into_iter() 得到 {upper:?}");

    // 数组同样遵守这套规则（Rust 2021 起 `for n in nums` 对数组也是按值消费）。
    let mut nums = [1, 2, 3];
    for n in &mut nums {
        *n *= 2;
    }
    println!("iter_mut() 就地翻倍 = {nums:?}");
}

/// 斐波那契（Fibonacci）数列生成器，演示如何手写 `Iterator`。
///
/// 实现 `Iterator` 只需提供 `Item` 类型和 `next()`：返回 `Some(元素)` 并推进状态。
/// 本结构是**无限**迭代器，所以消费端必须自己 `take(n)`。
struct Fibonacci {
    /// 下一次 `next()` 要返回的数。
    current: u64,
    /// 再下一个数（用于推进状态）。
    next: u64,
}

impl Default for Fibonacci {
    /// 从 `0, 1` 开始；示例调用 `Fibonacci::default().take(5)`。
    fn default() -> Self {
        Self {
            current: 0,
            next: 1,
        }
    }
}

/// 为 `Fibonacci` 实现 `Iterator`：只需给出 `Item` 类型并实现 `next()`。
impl Iterator for Fibonacci {
    /// 序列元素类型：64 位无符号整数。
    type Item = u64;

    /// 返回下一个斐波那契数并推进状态；无限序列永远返回 `Some`，限流交给调用方。
    fn next(&mut self) -> Option<u64> {
        let out = self.current;
        self.current = self.next; // 状态推进完全由这两个字段承担
        self.next += out;
        Some(out)
    }
}

/// 程序入口：按小节顺序演示本章所有知识点。
fn main() {
    println!("=== 1. 闭包的三种捕获方式 ===");
    demo_closure_capture();
    println!("\n=== 2. Fn / FnMut / FnOnce ===");
    demo_fn_traits();
    println!("\n=== 3. 迭代器的惰性 ===");
    demo_lazy();
    println!("\n=== 4. 适配器 map / filter / fold / collect ===");
    demo_adapters();
    println!("\n=== 5. 组合器 enumerate / zip / chain / flat_map / take ===");
    demo_combinators();
    println!("\n=== 6. iter() vs into_iter() ===");
    demo_iter_vs_into_iter();
    println!("\n=== 7. 自定义 Iterator 实现 ===");
    let fib: Vec<u64> = Fibonacci::default().take(10).collect();
    println!("斐波那契前 10 项 = {fib:?}");
}
