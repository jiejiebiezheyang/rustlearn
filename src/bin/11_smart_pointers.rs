//! 第 11 章：智能指针（smart pointers）
//!
//! 智能指针 = "拥有数据的结构体 + `Deref`/`Drop` 实现"，它们用堆上的一块数据换取
//! 普通引用给不了的能力：递归数据结构（`Box`）、多所有者共享（`Rc`），以及在
//! `&self` 方法里修改内部状态（`RefCell`/`Cell`，即内部可变性 interior mutability）。
//!
//! 运行：`cargo run --bin 11_smart_pointers`
//!
//! 关键知识点：
//! - `Box<T>` 与递归类型
//! - `Rc<T>` 共享所有权与 `strong_count`
//! - `RefCell<T>` 内部可变性与运行时借用检查
//! - `Rc<RefCell<T>>` 组合
//! - `Cell` 与 `Weak` 简介
//! - `Deref` 与 Deref 强制转换
//! - `Drop` 与 RAII
//! - `Rc` 循环引用导致内存泄漏

use std::cell::{Cell, RefCell};
use std::ops::Deref;
use std::rc::{Rc, Weak};

/// 用 `Box` 做间接寻址的递归表达式树。
///
/// 变体里直接内嵌 `Expr` 会让类型大小无限（编译器算不出栈布局）；`Box<Expr>` 把
/// 子节点放堆上，枚举本身只存定长指针，递归类型（recursive type）才成立。
///
/// 示例调用：`Expr::num(1).add(Expr::num(2)).eval()` 得到 `3`。
#[derive(Debug)]
enum Expr {
    /// 叶子节点：一个字面量整数。
    Num(i64),
    /// 内部节点：两个被 `Box` 包起来的子表达式相加。
    Add(Box<Expr>, Box<Expr>),
}

/// `Expr` 的构造函数与求值逻辑。
impl Expr {
    /// 构造叶子节点，`value` 是要保存的整数值。
    fn num(value: i64) -> Self {
        Expr::Num(value)
    }

    /// 构造加法节点，`other` 是另一个子表达式；调用后 `self` 被移动进树里。
    fn add(self, other: Expr) -> Self {
        Expr::Add(Box::new(self), Box::new(other))
    }

    /// 递归求值：叶子直接取值，加法节点递归两边再相加。
    fn eval(&self) -> i64 {
        match self {
            Expr::Num(value) => *value,
            // l、r 的类型是 &Box<Expr>，方法调用会自动穿透 Box 解引用
            Expr::Add(l, r) => l.eval() + r.eval(),
        }
    }
}

/// `Rc<T>` 的共享所有权（shared ownership）：`Rc::clone` 只加引用计数不深拷贝数据。
///
/// ⚠️ 常见坑: `shared.clone()` 也能编译，但看起来像深拷贝，社区惯例写 `Rc::clone`。
fn demo_rc_shared_ownership() {
    let shared = Rc::new(vec![1, 2, 3]);
    let second = Rc::clone(&shared);
    let third = Rc::clone(&shared);
    println!("第三个句柄看到同一份数据: {third:?}");
    println!("三个句柄时 strong_count = {}", Rc::strong_count(&shared));
    drop(second); // 最后一个所有者消失时数据才被释放
    println!("释放后 strong_count = {}", Rc::strong_count(&shared));
}

/// `RefCell<T>` 的内部可变性与运行时借用检查。
///
/// `RefCell` 把借用规则从编译期挪到运行期：`&self` 也能拿到 `borrow_mut()`，
/// 代价是违反规则时不再编译报错，而是 panic（想优雅处理就用 `try_borrow_mut`）。
fn demo_refcell_runtime_check() {
    let cell = RefCell::new(10);
    let shared_view = cell.borrow();
    // 已有不可变借用时再申请可变借用：try_borrow_mut 安全地返回 Err
    match cell.try_borrow_mut() {
        Ok(_) => println!("不应该发生"),
        Err(error) => println!("运行时借用检查拒绝第二次借用: {error}"),
    }
    println!("不可变借用仍可读: {}", *shared_view);
    drop(shared_view); // 借用守卫（Ref/RefMut）活到语句结束，手动结束它才能再改

    // ⚠️ 常见坑: `borrow_mut()` 冲突时直接 panic，且它没有返回值，
    // 想观察这个行为只能 catch_unwind；生产代码优先用 try_borrow_mut。
    *cell.borrow_mut() = 20; // 守卫已释放，这次可变借用成功
    println!("拿到可变借用并写入: {}", cell.borrow());
}

/// `Rc<RefCell<T>>` 组合：多个所有者 + 每个所有者都能修改。
///
/// `Rc` 只给共享的只读访问，`RefCell` 补上内部可变性，两者叠加是单线程里最常见的
/// "共享可变状态"写法。⚠️ 常见坑: `Rc` 不是 `Send`，跨线程要换 `Arc<Mutex<T>>`。
fn demo_rc_refcell() {
    let left = Rc::new(RefCell::new(vec![1, 2]));
    let right = Rc::clone(&left);
    right.borrow_mut().push(3);
    println!("通过 right 追加后 left 看到: {:?}", left.borrow());
    println!("此时 strong_count = {}", Rc::strong_count(&left));
}

/// 只支持 `Copy` 值读写的内部可变性容器：比 `RefCell` 更轻，但拿不到内部引用。
struct HitCounter {
    /// 命中次数；`Cell` 让 `&self` 方法也能修改它。
    hits: Cell<u32>,
}

/// `HitCounter` 的构造与计数操作。
impl HitCounter {
    /// 创建一个计数为 0 的计数器。
    ///
    /// ⚠️ 常见坑: 命名为 `new()` 会触发 clippy::new_without_default（无参 new 需配 `Default`）。
    fn with_zero() -> Self {
        HitCounter { hits: Cell::new(0) }
    }

    /// 记录一次命中；注意参数是 `&self` 而不是 `&mut self`。
    fn hit(&self) {
        self.hits.set(self.hits.get() + 1);
    }

    /// 读取当前命中次数。
    fn total(&self) -> u32 {
        self.hits.get()
    }
}

/// `Weak<T>` 是不增加 strong count 的弱引用。
///
/// `upgrade()` 成功说明数据还活着，返回 `None` 说明所有者已全部消失。
fn demo_weak() {
    let owner = Rc::new(String::from("被观察的数据"));
    let watcher: Weak<String> = Rc::downgrade(&owner);
    println!("还在时 upgrade 成功 = {}", watcher.upgrade().is_some());
    drop(owner);
    println!("释放后 upgrade 成功 = {}", watcher.upgrade().is_some());
}

/// 极简"米"新类型（newtype），只为演示 `Deref` 而存在；可以当 `&f64` 用。
struct Meters(f64);

/// 让 `Meters` 能解引用成 `f64`，这是 Deref 强制转换（deref coercion）的入口。
impl Deref for Meters {
    type Target = f64;

    /// 返回内部 `f64` 的引用。
    fn deref(&self) -> &f64 {
        &self.0
    }
}

/// 接收 `&f64`：调用处传 `&Meters` 会自动做 Deref 强制转换。
///
/// 这正是 `&String` 能传给 `&str` 参数、`&Vec<T>` 能传给 `&[T]` 参数的原理。
fn print_temperature(celsius: &f64) {
    println!("温度 = {celsius} 摄氏度");
}

/// 用 `Drop` 演示 RAII：值离开作用域时编译器自动插入 `drop()`，资源无需手写清理；
/// 同一作用域内按声明顺序**逆序**释放。
struct ResourceGuard {
    /// 资源名，仅用于打印释放顺序。
    name: &'static str,
}

/// 释放时打印资源名，用来观察析构顺序。
impl Drop for ResourceGuard {
    /// 由编译器在离开作用域时自动调用。
    ///
    /// ⚠️ 常见坑: 不能手写 `guard.drop()`（报 E0040），提前释放只能写 `drop(guard)`。
    fn drop(&mut self) {
        println!("释放资源: {}", self.name);
    }
}

/// 演示 RAII 的释放顺序与 `drop()` 提前释放。
fn demo_drop_raii() {
    {
        let _first = ResourceGuard { name: "first" };
        let _second = ResourceGuard { name: "second" };
        println!("块内创建 first、second（离开块时逆序释放）");
    } // 先打印 "释放资源: second"，再打印 "释放资源: first"

    let early = ResourceGuard { name: "early" };
    drop(early); // 提前释放：不等作用域结束
    println!("early 已在作用域结束前释放");
}

/// 可以互相指向的节点，用来制造 `Rc` 循环引用（reference cycle）。
///
/// 元组里的 `RefCell<Option<Rc<Node>>>` 是关键：`RefCell` 允许创建之后再接线，
/// `Option` 允许节点暂时没有后继。
struct Node(RefCell<Option<Rc<Node>>>);

/// `Rc` 循环引用导致的内存泄漏（memory leak）。
///
/// 两节点互相持有时谁的 strong count 都不会降到 0；外部句柄一 drop，就再没有强引用
/// 能访问它们了。规避办法是让其中一边改用 `Weak`（父子、观察者结构常用）。
fn demo_rc_cycle_leak() {
    let first = Rc::new(Node(RefCell::new(None)));
    let second = Rc::new(Node(RefCell::new(None)));
    *first.0.borrow_mut() = Some(Rc::clone(&second));
    *second.0.borrow_mut() = Some(Rc::clone(&first));
    println!("成环后 strong_count = {}", Rc::strong_count(&first));

    let watcher = Rc::downgrade(&first); // 弱引用不参与引用计数
    drop(first);
    drop(second);
    // ⚠️ 常见坑: upgrade 仍然成功，说明两个节点都没被释放，只是再也访问不到了。
    println!("释放句柄后 upgrade 成功 = {}", watcher.upgrade().is_some());
}

/// 程序入口：按小节顺序演示本章所有知识点。
fn main() {
    println!("=== 1. Box<T> 与递归类型 ===");
    // 表达式树 1 + (2 + 4)：没有 Box 这个类型的大小无法计算
    let expression = Expr::num(1).add(Expr::num(2).add(Expr::num(4)));
    println!("表达式结构 = {expression:?}");
    println!("求值结果 = {}", expression.eval());

    println!("\n=== 2. Rc<T> 共享所有权与 strong_count ===");
    demo_rc_shared_ownership();

    println!("\n=== 3. RefCell<T> 内部可变性与运行时借用检查 ===");
    demo_refcell_runtime_check();

    println!("\n=== 4. Rc<RefCell<T>> 组合 ===");
    demo_rc_refcell();

    println!("\n=== 5. Cell 与 Weak 简介 ===");
    let counter = HitCounter::with_zero();
    counter.hit();
    counter.hit();
    println!("Cell 计数 = {}", counter.total());
    demo_weak();

    println!("\n=== 6. Deref 与 Deref 强制转换 ===");
    let room = Meters(21.5);
    print_temperature(&room); // &Meters 自动转成 &f64
    println!("显式解引用得到 = {}", *room);

    println!("\n=== 7. Drop 与 RAII ===");
    demo_drop_raii();

    println!("\n=== 8. Rc 循环引用导致内存泄漏 ===");
    demo_rc_cycle_leak();
}
