//! 第 04 章：结构体、方法与关联函数（structs, methods & associated functions）
//!
//! 结构体（struct）把相关数据打包成一个**具名类型**：字段有名字，还能通过 `impl`
//! 挂载方法（method）与关联函数，解决 `(String, u8, bool)` 这类元组“位置含义靠猜”的问题。
//!
//! 运行：`cargo run --bin 04_structs`
//!
//! 关键知识点：
//! - 具名结构体（named struct）/ 元组结构体（tuple struct）/ 单元结构体（unit struct）
//! - 字段初始化简写（field init shorthand）与结构体更新语法 `..`
//! - 方法接收者：`&self`、`&mut self`、`self`
//! - 关联函数 `new`、多个 `impl` 块、`#[derive(Debug, Clone, PartialEq)]` 与 `Default`

/// 平面上的点：具名结构体（named struct）；特意不派生 `Copy`，
/// 这样演示 `..` 更新语法时能看到真实的移动（move）语义。
#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

/// 第一个 `impl` 块：构造与修改。同一类型的实现可以拆进多个块，编译器合并看待。
impl Point {
    /// 关联函数：没有 `self` 参数，用 `Point::new(3.0, 4.0)` 调用。
    ///
    /// Rust 没有构造函数语法，`new` 只是“返回 `Self`”的函数名约定。
    fn new(x: f64, y: f64) -> Self {
        // 字段初始化简写：变量名与字段名相同时可省 `x: x`
        Self { x, y }
    }

    /// 方法 `&self`：只读借用，返回到原点的距离。
    fn distance_from_origin(&self) -> f64 {
        // hypot 比自己写 sqrt(x*x + y*y) 更不容易溢出
        self.x.hypot(self.y)
    }

    /// 方法 `&mut self`：可变借用，把点平移 `(dx, dy)`。
    fn translate(&mut self, dx: f64, dy: f64) {
        self.x += dx;
        self.y += dy;
    }

    /// 方法 `self`：按值接收并消耗（consume）结构体，返回 `(x, y)`。
    ///
    /// 示例：`let pair = point.into_pair();`（之后 point 失效）。
    fn into_pair(self) -> (f64, f64) {
        (self.x, self.y)
    }
}

/// 第二个 `impl` 块：查询类方法。方法在哪个块里定义，调用语法都一样。
impl Point {
    /// 方法 `&self`：返回象限（"I".."IV"），坐标轴上的点按正号归类。
    fn quadrant(&self) -> &'static str {
        // ⚠️ 常见坑: 用 if/else if 反复比较同两个值会被 clippy::comparison_chain 盯上
        let east = self.x >= 0.0;
        let north = self.y >= 0.0;
        match (east, north) {
            (true, true) => "I",
            (false, true) => "II",
            (false, false) => "III",
            (true, false) => "IV",
        }
    }
}

/// 手写 `Default`：默认点是 `(1.0, 1.0)`。
///
/// 不能用 `#[derive(Default)]`：派生版会把字段填成零值（原点）——只有默认值
/// 恰等于各字段零值时才能派生。
impl Default for Point {
    /// 返回 `(1.0, 1.0)`。
    fn default() -> Self {
        Self { x: 1.0, y: 1.0 }
    }
}

/// RGB 颜色：演示元组结构体（tuple struct），字段只有序号（`self.0`、`self.1`）。
///
/// 适合“语义明确、不值得给字段命名”的小包装，例如 `struct Meters(f64);`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Rgb(u8, u8, u8);

impl Rgb {
    /// 关联函数：解析 `"#ff8800"` 或 `"ff8800"`，非法时返回 `None`。
    fn from_hex(hex: &str) -> Option<Self> {
        let digits = hex.strip_prefix('#').unwrap_or(hex);
        if digits.len() != 6 {
            return None;
        }
        // ⚠️ 常见坑: 按字节切片要求输入是 ASCII；允许多字节字符时会在非字符边界 panic
        if !digits.is_ascii() {
            return None;
        }
        let r = u8::from_str_radix(&digits[0..2], 16).ok()?;
        let g = u8::from_str_radix(&digits[2..4], 16).ok()?;
        let b = u8::from_str_radix(&digits[4..6], 16).ok()?;
        Some(Self(r, g, b))
    }

    /// 方法 `self`：格式化成 `#rrggbb`（`Rgb` 是 `Copy`，按值接收比 `&self` 更自然），
    /// 示例：`Rgb::from_hex("#ff8800")?.to_hex()`。
    fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }
}

/// 单元结构体（unit struct）：没有字段、大小为零（ZST）；它既是类型也是值，
/// 常用来表示“无状态的策略”或泛型标记（marker type）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct NoDiscount;

impl NoDiscount {
    /// 关联函数：无折扣策略，原样返回 `price`。示例：`NoDiscount::apply(99.9)`。
    fn apply(price: f64) -> f64 {
        price
    }
}

/// 用户资料：演示字段初始化简写、`..` 更新语法与非 `Copy` 字段的移动语义。
#[derive(Debug, Clone, PartialEq)]
struct Profile {
    name: String,
    age: u8,
    active: bool,
}

impl Profile {
    /// 关联函数：用 `name` 创建资料，`age` 默认 0、`active` 默认 true。
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            age: 0,
            active: true,
        }
    }
}

/// 手写 `Default`：匿名且未激活的访客（派生版会得到空字符串）。
impl Default for Profile {
    /// 返回 `name = "anonymous"`、`age = 0`、`active = false`。
    fn default() -> Self {
        Self {
            name: String::from("anonymous"),
            age: 0,
            active: false,
        }
    }
}

/// 演示具名结构体、字段初始化简写与关联函数 `new`。
fn demo_named_struct() {
    let mut origin = Point { x: 0.0, y: 0.0 }; // 字段名写全
    let (x, y) = (3.0, 4.0);
    let corner = Point { x, y }; // 简写：`x` 就是 `x: x`
    let also = Point::new(3.0, 4.0); // 关联函数用类型名调用
    println!("corner = {corner:?}, also = {also:?}");
    origin.x = 1.5; // 只有 `mut` 绑定才能改字段
    println!("origin 改 x 后 = {origin:?}");
    // ⚠️ 常见坑: 字段可变性由**绑定**决定，Rust 没有 `mut` 字段这种语法
}

/// 演示三种方法接收者：`&self`（只读）、`&mut self`（可改）、`self`（消耗）。
fn demo_method_receivers() {
    let mut point = Point::new(3.0, 4.0);
    println!("到原点距离 = {}", point.distance_from_origin()); // &self：借用
    point.translate(1.0, 2.0); // &mut self：独占借用
    println!("平移后 point = {point:?}");
    let pair = point.into_pair(); // self：把 point 移动进方法
    println!("拆成元组 = {pair:?}（此处再用 point 会报 E0382）");
    // quadrant 定义在第二个 impl 块里，调用语法与第一个块完全一致
    println!("(2,1) 在第 {} 象限", Point::new(2.0, 1.0).quadrant());
}

/// 演示元组结构体：字段按下标访问，也可以用模式解构。
fn demo_tuple_struct() {
    let orange = Rgb::from_hex("#ff8800").expect("示例常量应当是合法的十六进制颜色");
    println!("orange = {orange:?}, hex = {}", orange.to_hex());
    let Rgb(r, g, b) = orange; // 模式解构是最舒服的用法
    println!("解构: r = {r}, g = {g}, b = {b}");
    println!("派生 Default = {}", Rgb::default().to_hex());
    println!("非法输入 from_hex(\"nope\") = {:?}", Rgb::from_hex("nope"));
    // ⚠️ 常见坑: `Rgb(u8, u8, u8)` 与 `Meters(f64)` 不是同一个类型，不能互相赋值
}

/// 演示单元结构体：零大小、无状态，只作为策略/标记的载体。
fn demo_unit_struct() {
    let size = std::mem::size_of::<NoDiscount>();
    println!("折后价 = {}", NoDiscount::apply(99.9));
    println!("NoDiscount 是 ZST：size_of = {size} 字节，编译期就被优化掉");
    println!("它本身也是值: {:?}", NoDiscount);
}

/// 演示结构体更新语法（struct update syntax）`..` 与部分移动（partial move）。
fn demo_struct_update() {
    let base = Profile::new("alice");
    // 只覆盖 age，其余字段由 `..base` 填：name 是 String（非 Copy），会被 move 进新值
    let aged = Profile { age: 31, ..base };
    println!("aged = {aged:?}");
    // ⚠️ 常见坑: base.name 已被移走，此刻再用 base 会报 E0382（partial move）
    let base2 = Profile::new("carol");
    let aged2 = Profile {
        age: 25,
        ..base2.clone()
    }; // 想保留原值就显式克隆剩余字段
    println!("base2 = {base2:?}, aged2 = {aged2:?}");
    let base_point = Point::new(1.0, 2.0); // Point 的字段都是 Copy
    let shifted = Point {
        x: 10.0,
        ..base_point
    };
    println!("base_point = {base_point:?}, shifted = {shifted:?}");
}

/// 演示 `#[derive(Debug, Clone, PartialEq)]` 带来的三种能力。
fn demo_derive_traits() {
    let a = Point::new(3.0, 4.0);
    let b = a.clone(); // Clone：显式复制（Point 不是 Copy，赋值默认是移动）
    assert_eq!(a, b); // PartialEq：逐字段比较，等价于手写 impl PartialEq
    assert_ne!(a, Point::new(0.0, 0.0));
    // ⚠️ 常见坑: 派生的 PartialEq 是逐字段 `==`；f64 字段受精度影响
    //            （0.1 + 0.2 != 0.3），需要近似比较时必须手写实现。
    println!("a = {a:?}, b = {b:?}, a == b → {}", a == b); // Debug：{:?} 打印
}

/// 演示 `Default`：`Type::default()` 与 `Type { ..Default::default() }`。
fn demo_default() {
    let default_point = Point::default(); // 手写版：默认点 (1.0, 1.0)
    let custom = Point {
        x: 9.0,
        ..Default::default()
    }; // 只写关心的字段
    println!("default = {default_point:?}, custom = {custom:?}");
    // Default 的最大价值：给泛型代码和测试夹具一个“零配置起点”
    println!("Profile::default() = {:?}", Profile::default()); // 匿名、未激活
}

/// 程序入口：按小节顺序演示第 4 章的全部知识点。
fn main() {
    println!("=== 1. 具名结构体、字段初始化简写与关联函数 new ===");
    demo_named_struct();

    println!("\n=== 2. 方法接收者 &self / &mut self / self（含多个 impl 块）===");
    demo_method_receivers();

    println!("\n=== 3. 元组结构体与单元结构体 ===");
    demo_tuple_struct();
    demo_unit_struct();

    println!("\n=== 4. 结构体更新语法 .. ===");
    demo_struct_update();

    println!("\n=== 5. derive(Debug, Clone, PartialEq) ===");
    demo_derive_traits();

    println!("\n=== 6. Default ===");
    demo_default();
}
