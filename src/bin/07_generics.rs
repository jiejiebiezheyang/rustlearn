//! 第 7 章：泛型与 trait 约束（generics & trait bounds）
//!
//! 泛型（generics）让我们只写一次代码就能作用于多种具体类型；trait 约束
//! （trait bound）则告诉编译器"这些类型必须具备哪些能力"。两者配合，
//! 既不用为每种类型复制粘贴，又能在编译期做完整的类型检查。
//!
//! 学完本章，你能看懂 `Vec<T>`、`HashMap<K, V, S>` 这类标准库签名，
//! 也能自己写出带约束的泛型函数、泛型结构体、泛型枚举与 const 泛型。
//!
//! 运行：`cargo run --bin 07_generics`
//!
//! 关键知识点：
//! - 泛型函数
//! - 泛型结构体与泛型枚举
//! - trait 约束：内联 `T: Trait` 与 `where` 子句
//! - 多重约束
//! - 单态化（monomorphization）与零成本抽象
//! - const 泛型
//! - 默认类型参数
//! - 泛型 `impl` 块

use std::fmt::Display;
use std::ops::{Add, AddAssign};

/// 返回切片中的最大值；空切片返回 `None`。
///
/// 这是最典型的泛型函数：`T` 是类型参数（type parameter），
/// `PartialOrd + Copy` 是 trait 约束——只有能比较大小、能按位复制的类型才允许调用。
/// 调用示例：`largest(&[3, 1, 4])` → `Some(4)`。
fn largest<T: PartialOrd + Copy>(values: &[T]) -> Option<T> {
    // ⚠️ 常见坑: 若只写 `fn largest<T>(...)` 而不加约束，函数体里的 `>`
    //    根本编译不过——编译器不知道 `T` 是否支持比较。约束就是"能力契约"。
    values
        .iter()
        .copied()
        .reduce(|acc, item| if item > acc { item } else { acc })
}

/// 泛型结构体：坐标点，一个 `T` 同时决定 `x`、`y` 的类型。
///
/// 调用示例：`Point::new(1, 2)` 得到 `Point<i32>`。
#[derive(Debug, Clone, Copy, PartialEq)]
struct Point<T> {
    /// 横坐标。
    x: T,
    /// 纵坐标。
    y: T,
}

/// 泛型 `impl` 块：`impl<T>` 中的方法对任意 `T` 都可用。
impl<T> Point<T> {
    /// 用两个同类型坐标构造一个点。
    fn new(x: T, y: T) -> Self {
        Point { x, y }
    }

    /// 交换两个分量后返回新点（消费 `self`，不修改原值）。
    fn transposed(self) -> Self {
        Point {
            x: self.y,
            y: self.x,
        }
    }
}

/// 针对具体类型的 `impl` 块：只有 `Point<f64>` 才有这个方法。
///
/// 什么时候用：某个操作只对特定类型有意义时（"到原点距离"只对浮点数有用），
/// 就不要把它塞进泛型 `impl` 里，否则其他 `Point<T>` 会多出无意义的方法。
impl Point<f64> {
    /// 返回该点到原点的欧氏距离。
    fn distance_to_origin(&self) -> f64 {
        self.x.hypot(self.y)
    }
}

/// `where` 子句写法：把约束挪到签名下方，长约束链读起来更清楚。
///
/// 语义与内联 `T: Trait` 完全相同，只是排版不同；约束一多就推荐用 `where`。
/// 调用示例：`sum_all(&[1, 2, 3])` → `6`。
fn sum_all<T>(values: &[T]) -> T
where
    T: Add<Output = T> + Copy + Default,
{
    // `Default` 提供"零"作为初值，`Add` 提供累加能力，二者都由约束保证存在。
    values.iter().copied().fold(T::default(), |acc, x| acc + x)
}

/// 多重约束（multiple bounds）：一个类型参数同时满足多个 trait。
///
/// `T: Display + PartialOrd + Copy` 用 `+` 叠加约束，表示"既要能打印、
/// 又要能比较、还要能复制"，缺一不可。
/// 调用示例：`describe_max(&[10, 42])` → `"共 2 个元素，最大值 42"`。
fn describe_max<T: Display + PartialOrd + Copy>(values: &[T]) -> String {
    match largest(values) {
        Some(max) => format!("共 {} 个元素，最大值 {max}", values.len()),
        None => "空切片，没有最大值".to_string(),
    }
}

/// 泛型枚举：`Either` 表示"两种类型中恰好有一个"的值。
///
/// `Option<T>`、`Result<T, E>` 都是泛型枚举；带多个类型参数的枚举同样常见。
#[derive(Debug, Clone, Copy, PartialEq)]
enum Either<L, R> {
    /// 左侧的值。
    Left(L),
    /// 右侧的值。
    Right(R),
}

/// 泛型 `impl` 块 + `where` 子句组合：把两侧收敛成同一个类型 `T`。
impl<L, R> Either<L, R> {
    /// 分别用 `on_left` / `on_right` 处理两条分支，返回统一的类型 `T`。
    ///
    /// 调用示例：`Either::Left(1).fold(|n| n + 1, |s: &str| s.len() as i32)` → `2`。
    fn fold<T, FL, FR>(self, on_left: FL, on_right: FR) -> T
    where
        FL: FnOnce(L) -> T,
        FR: FnOnce(R) -> T,
    {
        match self {
            Either::Left(left) => on_left(left),
            Either::Right(right) => on_right(right),
        }
    }
}

/// 单态化（monomorphization）：编译期为每个具体类型各生成一份专用机器码。
///
/// `type_name::<T>()` 打印的是编译器眼中 `T` 的真实类型，说明泛型参数
/// 在运行期并不存在——这正是"零成本抽象"（zero-cost abstraction）：
/// 抽象不带来运行期开销，代价是编译时间与二进制体积的增长。
fn type_label<T>() -> &'static str {
    std::any::type_name::<T>()
}

/// 用 const 泛型对"长度任意但类型安全"的数组求和。
///
/// `const N: usize` 是编译期常量参数：`[i32; 3]` 与 `[i32; 5]` 是不同类型，
/// 却共用这一份函数体，长度不匹配的调用根本无法通过编译。
/// 调用示例：`sum_array([1, 2, 3])` → `6`。
fn sum_array<const N: usize>(values: [i32; N]) -> i32 {
    values.iter().sum()
}

/// 把容量放进类型里的定长缓冲区：`N` 是类型的一部分。
#[derive(Debug)]
struct Buffer<const N: usize> {
    /// 定长底层数组，长度由 `N` 决定。
    data: [i32; N],
}

/// const 泛型的 `impl` 块：`impl` 上声明的 `N` 与结构体上的 `N` 是同一个参数。
impl<const N: usize> Buffer<N> {
    /// 创建所有元素都等于 `fill` 的缓冲区。
    fn filled(fill: i32) -> Self {
        Self { data: [fill; N] }
    }

    /// 返回缓冲区容量（编译期常量，运行期被常量折叠）。
    fn capacity(&self) -> usize {
        N
    }

    /// 返回所有元素之和。
    fn total(&self) -> i32 {
        self.data.iter().sum()
    }
}

/// 默认类型参数（default type parameter）：不写类型实参时使用默认值。
///
/// 标准库的 `HashMap<K, V, S = RandomState>` 用的就是这个特性；
/// 注意只有类型（struct/enum/trait）支持默认参数，函数不支持。
/// 调用示例：`Counter::new()` 得到 `Counter<u32>`，也可写 `Counter::<i64>::new()`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Counter<T = u32> {
    /// 当前计数值。
    value: T,
}

/// 泛型 `impl` 块：`T` 必须支持自增赋值、复制，并能提供默认零值。
impl<T> Counter<T>
where
    T: AddAssign + Copy + Default,
{
    /// 从零开始创建一个计数器。
    fn new() -> Self {
        Self {
            value: T::default(),
        }
    }

    /// 增加 `step` 步，返回增加后的当前值。
    fn add(&mut self, step: T) -> T {
        self.value += step;
        self.value
    }
}

/// 程序入口：按小节顺序演示本章所有知识点。
fn main() {
    println!("=== 1. 泛型函数 ===");
    let numbers = [3, 1, 4, 1, 5];
    let letters = ['a', 'z', 'm'];
    println!("largest(&numbers) = {:?}", largest(&numbers));
    println!("largest(&letters) = {:?}", largest(&letters));
    println!("largest::<i32>(&[]) = {:?}", largest::<i32>(&[]));

    println!("\n=== 2. 泛型结构体与泛型 impl 块 ===");
    let point = Point::new(3.0_f64, 4.0_f64);
    println!("{point:?} 到原点距离 = {}", point.distance_to_origin());
    let transposed = Point::new(1_i32, 2_i32).transposed();
    let same = transposed == Point::new(2, 1);
    println!("transposed = {transposed:?}，等于 Point::new(2, 1)：{same}");

    println!("\n=== 3. trait 约束：内联 vs where 子句 ===");
    println!("sum_all(&[1, 2, 3, 4]) = {}", sum_all(&[1, 2, 3, 4]));
    println!("sum_all(&[0.5, 0.25]) = {}", sum_all(&[0.5_f64, 0.25_f64]));

    println!("\n=== 4. 多重约束 ===");
    println!("{}", describe_max(&[10, 42, 7]));
    println!("{}", describe_max(&["pear", "apple", "plum"]));

    println!("\n=== 5. 泛型枚举 ===");
    let left: Either<i32, &str> = Either::Left(41);
    let right: Either<i32, &str> = Either::Right("hello");
    let from_left = left.fold(|n| n + 1, |s: &str| s.len() as i32);
    let from_right = right.fold(|n| n + 1, |s: &str| s.len() as i32);
    println!("fold(Left) = {from_left}，fold(Right) = {from_right}");

    println!("\n=== 6. 单态化与零成本抽象 ===");
    println!("i32  -> {}", type_label::<i32>());
    println!("f64  -> {}", type_label::<f64>());
    println!("bool -> {}", type_label::<bool>());

    println!("\n=== 7. const 泛型 ===");
    let short = [1, 2, 3];
    let long = [1, 2, 3, 4, 5];
    println!("sum_array(short) = {}", sum_array(short));
    println!("sum_array(long) = {}", sum_array(long));
    let buffer = Buffer::<4>::filled(3);
    println!("buffer = {buffer:?}，容量 = {}", buffer.capacity());
    println!("元素和 = {}", buffer.total());

    println!("\n=== 8. 默认类型参数 ===");
    let mut default_counter = Counter::new();
    println!("默认 T = u32：{}", default_counter.add(5));
    let mut wide_counter = Counter::<i64>::new();
    println!("显式 T = i64：{}", wide_counter.add(5));
}
