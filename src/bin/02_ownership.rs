//! 第 2 章：所有权与借用
//!
//! 所有权（ownership）是 Rust 最核心也最"反直觉"的设计：它把内存释放的时机从
//! "运行时 GC 或手动 free"变成"编译期可证明的规则"，于是既没有 GC 停顿，也不会有
//! use-after-free / double free。本章讲清三件事：值住在栈（stack）还是堆（heap）、
//! 赋值与传参时发生的是移动（move）还是按位拷贝（Copy）、以及不转移所有权时如何
//! 通过借用（borrow）安全地读改数据。
//!
//! 运行：`cargo run --bin 02_ownership`
//!
//! 关键知识点：栈与堆；移动（move）语义；`Copy` vs `Clone`；所有权随函数参数转移与
//! 返回；借用（borrow）规则入门；悬垂引用（dangling reference）为何编译不过；
//! `Drop` 作用域与释放顺序；`mem::drop` 提前释放。

use std::mem::{self, size_of};

/// 本章标题，仅用于 `main()` 的开场输出。
const CHAPTER_TITLE: &str = "第 2 章：所有权与借用";

/// 演示栈（stack）与堆（heap）的分工：谁更快、谁更灵活、谁负责释放。栈的大小编译期
/// 已知、按后进先出分配，只需移动栈指针，极快；堆用于大小运行时才确定或需要动态增长
/// 的数据，要经分配器（allocator）申请，代价更高。
fn demo_stack_and_heap() {
    let on_stack: i32 = 42; // 整个值都在栈上
    let on_heap = String::from("hello"); // 句柄在栈上，字符数据在堆上
    println!("i32 占 {} 字节（全在栈上）", size_of::<i32>());
    // String 的栈上句柄由 ptr + len + cap 三个字段组成
    println!("String 句柄占 {} 字节", size_of::<String>());
    println!("堆上数据占 {} 字节（len）", on_heap.len());
    println!("on_stack = {on_stack}，on_heap = {on_heap}");
    // ⚠️ 常见坑: 别用 `size_of::<String>()` 估计字符串总内存，堆上还有 len 字节数据
    // （容量 cap 通常更大）。
    // ⚠️ 常见坑: 线程栈默认只有约 8 MiB，深递归或把大数组当局部变量会栈溢出
    // （stack overflow，进程直接 abort，无法 catch）。
}

/// 演示移动（move）语义：把堆数据的所有权交给新绑定，旧绑定当场失效。若 `let s2 = s1;`
/// 之后两者都能用，它们会在各自作用域结束时各 drop 一次、造成 double free；Rust 用
/// "移走即失效"在编译期消灭这个 bug，且不需要运行时引用计数或 GC。
fn demo_move_semantics() {
    let s1 = String::from("hello");
    let s2 = s1; // move：栈上的句柄被按位复制，s1 从此不可用

    // println!("{s1}"); // ❌ 编译错误：borrow of moved value: `s1`
    println!("所有权已 move 给 s2：{s2}");

    // 整数是 Copy 类型：赋值发生的是拷贝，两边都能继续用
    let n1 = 42;
    let n2 = n1;
    println!("Copy 类型赋值后两边都能用：n1 = {n1}，n2 = {n2}");
    // ⚠️ 常见坑: move 之后的报错是 "value borrowed here after move"；新手的"修法"
    // 是无脑加 `.clone()`，正确修法通常是改成借用（见后文）或重排数据流。
    // ⚠️ 常见坑: 循环里写 `for x in v` 会把 `v` 整体 move 掉，循环后 `v` 不可用；
    // 只想读取时写 `for x in &v`（`iter()` 与 `into_iter()` 见第 10 章）。
}

/// 演示 `Copy`（按位复制）与 `Clone`（显式深拷贝）的区别。`Copy` 是 marker trait：
/// 实现了它的类型在赋值/传参时自动按位复制，原值仍可用（整数、浮点、`bool`、`char`，
/// 以及元素全为 `Copy` 的 tuple/array）。`Clone` 必须显式调用 `.clone()`；`String`、
/// `Vec` 属于这一类，因为复制要额外分配堆内存，这个代价必须让调用者看得见。
fn demo_copy_vs_clone() {
    // Copy：隐式、廉价，由编译器自动插入拷贝
    let pair = (1u8, 'x'); // (Copy, Copy) 的元组自身也是 Copy
    let pair_copy = pair;
    let array = [1u16, 2, 3]; // 元素是 Copy，数组就是 Copy
    let array_copy = array;
    println!("Copy 后两边都可用：{pair:?} / {pair_copy:?}");
    println!("数组同理：{array:?} / {array_copy:?}");

    // Clone：必须显式调用，堆上的数据被真正复制一份
    let original = String::from("rust");
    let duplicated = original.clone();
    println!("Clone：original = {original}，duplicated = {duplicated}");
    // ⚠️ 常见坑: `Copy` 与 `Clone` 不是一回事——类型可以是 `Clone` 但不是 `Copy`
    // （`String` 正是如此）；反过来 `Copy` 类型必然实现了 `Clone`（约束 `Copy: Clone`）。
    // ⚠️ 常见坑: 给类型派生 `Copy` 后"赋值"会变成静默复制，将来一旦加入堆字段就必须
    // 去掉 `Copy`，属于破坏性变更；不确定时先只派生 `Clone`（第 4 章）。
}

/// 接收 `String` 的所有权并返回它的长度；函数返回时 `text` 在这里被 drop。
fn measure_and_consume(text: String) -> usize {
    text.len()
}

/// 在函数内部构造并返回 `String`，把所有权交给调用方。返回局部变量本身完全合法：值被
/// move 到调用方的绑定上，而不是返回指向已释放局部变量的引用（见 `demo_dangling_reference`）。
fn make_greeting(name: &str) -> String {
    format!("Hello, {name}!")
}

/// 接收所有权、就地修改后再把所有权归还——"链式传递所有权"的经典模式；若函数只是改一改
/// 内容，更好的做法是接收 `&mut String` 借用。
fn append_suffix(mut text: String) -> String {
    text.push_str(" (moved and returned)");
    text
}

/// 演示所有权如何随函数参数转移、又如何随返回值回到调用方。
fn demo_ownership_across_functions() {
    let owned = String::from("ownership");
    let len = measure_and_consume(owned); // owned 的所有权被移交出去

    // println!("{owned}"); // ❌ 编译错误：use of moved value: `owned`
    println!("传入函数并被 drop 的字符串长度 = {len}");

    let greeting = make_greeting("Rust"); // 返回值的所有权归调用方
    println!("函数返回的新字符串 = {greeting}");

    let chained = append_suffix(greeting); // greeting 被 move 进函数……
    println!("归还后的字符串 = {chained}"); // ……改完再作为返回值归还

    // ⚠️ 常见坑: 函数签名写成 `String` 就等于拿走所有权，哪怕函数体只读一次；只读场景
    // 应写 `&str` / `&mut String`。编译器报的 "value moved here" 会指出移交所有权的调用点。
}

/// 计算字符串长度，但不取得所有权。
///
/// 参数类型 `&str` 是字符串切片引用（string slice reference）：调用方只**借出**数据，
/// 函数返回后原字符串依然可用。这是"只读参数"的首选签名。
fn borrowed_len(text: &str) -> usize {
    text.len()
}

/// 演示借用（borrow）规则入门：任意多个不可变借用，或唯一一个可变借用。借用解决什么
/// 问题：函数往往只需读或临时改数据，不该把所有权整包拿走；引用让"临时访问"变成零成本的
/// 编译期检查，这套检查器叫借用检查器（borrow checker）。
fn demo_borrow_basics() {
    let owned = String::from("borrowed");
    let length = borrowed_len(&owned); // 只借出 &str，所有权仍在自己手里
    println!("借用计算长度 = {length}，之后仍可用 owned = {owned}");

    // 多个不可变借用（&T）可以共存：只读不冲突
    let r1 = &owned;
    let r2 = &owned;
    println!("多个不可变借用共存：{r1} / {r2}");

    // 可变借用（&mut T）必须独占：同一时刻不能有任何其他借用
    let mut editable = String::from("hi");
    let mr = &mut editable;
    mr.push_str(" there");
    // println!("{editable}"); // ❌ mr 仍将被使用，此时不能再借用 editable
    println!("通过可变借用修改后的值：{mr}");
    // ⚠️ 常见坑: "同一时刻"由 NLL（非词法生命周期）决定，看的是最后一次使用而非代码块
    // 的花括号（第 3 章详述），所以借用常常比看起来更早结束。
    // ⚠️ 常见坑: 可变借用没结束就读原变量会报 "cannot borrow `x` as immutable because
    // it was mutably borrowed"；把读操作挪到借用结束之后即可。
}

/// 说明悬垂引用（dangling reference）为什么编译不过：下面这段代码若取消注释会编译失败，
/// 因为 `created` 是局部变量，函数返回时它已被 drop，返回的引用便指向已释放的内存。
/// 在 C 里这是未定义行为（UB），Rust 选择在编译期拒绝。
///
/// ```ignore
/// fn dangling() -> &String {
///     let created = String::from("lost"); // created 在函数末尾被 drop
///     &created                            // ❌ 返回局部变量的引用
/// }
/// ```
///
/// 正确做法：返回拥有所有权的值，或由调用方提供数据、函数只返回其中的引用
/// （"返回的引用与输入活得一样久"要靠生命周期标注表达，第 12 章）。
fn demo_dangling_reference() {
    // 合法替代一：把所有权交给调用方，由调用方决定何时释放
    let owned = String::from("safe");
    let reference: &str = &owned; // 引用生命周期短于 owned，编译器可证明其有效
    println!("借用活不过持有者，所以永不悬垂：{reference}");
    // 合法替代二：直接返回值而不是引用
    let returned = make_greeting("borrow");
    println!("返回所有权而非引用：{returned}");
    // ⚠️ 常见坑: "borrowed value does not live long enough" 几乎总意味着引用比数据活得
    // 久；要么提升数据的作用域，要么改为返回值/返回所有权。
}

/// 一个带自定义析构逻辑的类型，用来观察 `Drop` 的触发时机。真实项目里这里可能是文件
/// 句柄、`MutexGuard` 锁守卫、连接池里的连接等**需要确定性释放**的资源。
struct Dropper {
    /// 在 drop 输出里标识是哪个实例。
    name: &'static str,
}

/// 为 `Dropper` 实现 `Drop`：值离开作用域时编译器自动插入这段清理代码，
/// 这就是 RAII（Resource Acquisition Is Initialization）在 Rust 里的落地方式。
impl Drop for Dropper {
    /// 打印一行"被释放"日志，替代真实的资源清理逻辑。
    fn drop(&mut self) {
        println!("    [drop] {} 被释放", self.name);
    }
}

/// 演示 `Drop` 的作用域（scope）与释放顺序：值在离开作用域时 drop，同一作用域内的多个
/// 绑定按**声明顺序的逆序** drop（后进先出）。
fn demo_drop_scope() {
    println!("  进入外层作用域");
    let first = Dropper { name: "first" };
    let second = Dropper { name: "second" };
    println!("  已按顺序创建：{} -> {}", first.name, second.name);
    {
        let inner = Dropper { name: "inner" };
        println!("  内层作用域创建：{}", inner.name);
    } // inner 在此处 drop
    println!("  内层已结束，回到外层");
    // 函数返回时按 second -> first 的顺序 drop
    // ⚠️ 常见坑: `let _ = value;` 会**立刻** drop（`_` 不是绑定，是通配模式），而
    // `let _guard = value;` 会持有到作用域结束；锁/守卫场景下两者语义完全不同。
}

/// 演示用 `mem::drop` 提前释放值。`std::mem::drop` 只是一个普通函数（签名
/// `fn drop<T>(_: T)`）：它接收所有权、函数体什么都不做，返回时形参被 drop，于是"提前
/// 释放"就变成"把值 move 进一个立刻结束的函数"。什么时候用：想尽早释放大内存、
/// 文件句柄或锁，而不必等到作用域结束。
fn demo_mem_drop() {
    let early = Dropper { name: "early" };
    println!("  手动释放前：{} 仍存活", early.name);
    mem::drop(early); // 所有权被 move 进 drop()，析构立即执行

    // println!("{}", early.name); // ❌ 编译错误：use of moved value
    println!("  手动释放后，函数还没结束");

    // 对比：显式块也能达到"提前"效果，但 mem::drop 的释放点更精确
    {
        let scoped = Dropper { name: "scoped" };
        println!("  块内：{} 存活", scoped.name);
    }
    // ⚠️ 常见坑: `drop(x)` 之后 `x` 已被移动，任何后续使用都是编译错误；若只是想忽略
    // 返回值，请写 `let _ = x;`，不要用 `drop` 表达无关意图。
    // ⚠️ 常见坑: `drop(&x)` 只丢掉一个引用，**不会**释放 `x`（clippy 的 drop_ref 会就此
    // 告警）；要提前释放，所有权必须先回到自己手上。
}

/// 程序入口：按小节顺序演示本章所有知识点。
fn main() {
    println!("{CHAPTER_TITLE}");

    println!("\n=== 1. 栈与堆（stack / heap） ===");
    demo_stack_and_heap();

    println!("\n=== 2. 移动（move）语义与 Copy / Clone ===");
    demo_move_semantics();
    demo_copy_vs_clone();

    println!("\n=== 3. 所有权随函数参数转移与返回 ===");
    demo_ownership_across_functions();

    println!("\n=== 4. 借用（borrow）规则入门 ===");
    demo_borrow_basics();

    println!("\n=== 5. 悬垂引用为何编译不过 ===");
    demo_dangling_reference();

    println!("\n=== 6. Drop 作用域、mem::drop 提前释放 ===");
    demo_drop_scope();
    demo_mem_drop();
}
