//! 第 05 章：枚举、模式匹配与 Option
//!
//! Rust 的枚举（enum）不是"一组常量的集合"，而是**代数数据类型**（algebraic data type）：
//! 每个变体（variant）都能携带自己的数据，再配合 `match` 的穷尽性（exhaustiveness）检查，编译器会在编译期证明"你没有漏掉任何一种情况"。
//!
//! `Option<T>` 就是标准库用同一套机制定义的枚举：把"可能没有值"编码进类型系统，从语言层面消灭 null 引用。
//!
//! 运行：`cargo run --bin 05_enums`
//!
//! 关键知识点：
//! - 带数据的枚举变体（unit / tuple / struct variant）
//! - `Option<T>`
//! - `match` 穷尽性
//! - match guard
//! - 解构绑定（destructuring）
//! - `if let` / `while let` / `let ... else`
//! - 通配符 `_` 与 `matches!`
//! - `Option` 组合子（`map`、`and_then`、`unwrap_or`）

use std::fmt;

/// 几何图形：演示"每个变体携带不同数据"的枚举。
///
/// `Point` 是 unit variant（不携带数据），`Circle` / `Rectangle` 是 struct variant。
/// 同一时刻一个值只能是其中一个变体，所以"半径"和"宽高"绝不会同时出现。
#[derive(Debug, PartialEq)]
enum Shape {
    /// 圆：只需要半径
    Circle { radius: f64 },
    /// 矩形：需要宽和高
    Rectangle { width: f64, height: f64 },
    /// 单位变体：不携带任何数据
    Point,
}

/// 任务状态机：演示用枚举替代"字符串状态 + 一堆 if"。
///
/// 解决的问题：字符串状态拼错也不报错，而枚举状态写成非法值根本编译不过。
#[derive(Debug)]
enum Task {
    /// 等待调度（unit variant）
    Pending,
    /// 运行中，携带进度百分比（struct variant）
    Running { progress: u8 },
    /// 已完成
    Done,
    /// 失败并携带原因（tuple variant）
    Failed(String),
}

/// 计算图形面积：`match` 必须覆盖所有变体（穷尽性）。
///
/// 参数用不可变借用 `&Shape`，避免把所有权移进函数。
/// 这里**故意不写** `_ =>`：将来给 `Shape` 加变体时，编译器会报错提醒你回来处理。
fn shape_area(shape: &Shape) -> f64 {
    match shape {
        // 解构绑定（match ergonomics）：`shape` 是引用，但能直接把字段绑成 `&f64`
        Shape::Circle { radius } => std::f64::consts::PI * radius * radius,
        Shape::Rectangle { width, height } => width * height,
        Shape::Point => 0.0,
    }
}

/// 为 `Task` 实现 `Display`：枚举 + `match` 是 Rust 格式化输出的标准搭配。
///
/// 解决的问题：`match` 表达式返回 `fmt::Result`，每个分支各自 `write!` 即可。
impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Task::Pending => write!(f, "等待中"),
            Task::Running { progress } => write!(f, "运行中 {progress}%"),
            Task::Done => write!(f, "已完成"),
            Task::Failed(reason) => write!(f, "失败: {reason}"),
        }
    }
}

/// match guard：模式匹配成功之后，再用 `if` 追加一个运行时布尔条件。
///
/// 解决的问题：模式本身无法表达"值必须大于 0"这类条件，guard 补上了这一层。
/// ⚠️ 常见坑: guard 按书写顺序求值，第一个成立的胜出，所以顺序就是语义。
fn classify_number(n: i32) -> &'static str {
    match n {
        0 => "零",
        n if n < 0 => "负数",
        n if n % 2 == 0 => "正偶数",
        _ => "正奇数",
    }
}

/// 通配符 `_`：表示"这个位置我不关心"，可当最后的兜底分支，也可在元组/结构体模式里忽略字段。
fn describe_pair(pair: (Option<i32>, bool)) -> String {
    match pair {
        // `_` 出现在元组位置：只关心第一个元素是否为大数值
        (Some(value), _) if value > 100 => format!("大数值: {value}"),
        (Some(value), true) => format!("数值 {value} 已确认"),
        (Some(value), false) => format!("数值 {value} 未确认"),
        (None, _) => String::from("没有数值"),
    }
}

/// `if let`：只关心一种模式，其余情况统统走 `else`。
///
/// 解决的问题：用 `match` 写单个分支时不得不补一句 `_ => {}`，很啰嗦。
/// ⚠️ 常见坑: `if let` 放弃了穷尽性检查 —— 将来给枚举加变体它不会提醒你，核心逻辑请优先用 `match`。
fn print_circle_only(shape: &Shape) {
    if let Shape::Circle { radius } = shape {
        println!("  这是圆，半径 {radius}");
    } else {
        println!("  不是圆，跳过");
    }
}

/// `while let`：只要模式匹配成功就继续循环，常用于消费式地取空一个容器。
///
/// ⚠️ 常见坑: 循环体里必须有能改变匹配结果的操作（这里 `pop` 会最终返回 `None`），否则就是死循环。
fn drain_stack(mut stack: Vec<&str>) -> usize {
    let mut popped = 0;
    while let Some(top) = stack.pop() {
        println!("  弹出: {top}");
        popped += 1;
    }
    popped
}

/// `let ... else`（Rust 1.65+）：匹配成功就继续往下走，失败则执行 `else` 分支。
///
/// 解决的问题：以前要写 `let x = match ... { Some(v) => v, None => return ... }`，
/// 或者被迫补一个 `?`。`let ... else` 把"提前退出"和"继续用绑定值"分开写，更扁。
/// ⚠️ 常见坑: `else` 块必须发散（diverge），只能是 `return` / `break` / `continue` / `panic!`。
fn first_long_word<'a>(words: &[&'a str]) -> &'a str {
    let Some(word) = words.iter().copied().find(|w| w.len() >= 4) else {
        return "（没有长单词）";
    };
    // 走到这里 `word` 已经是绑定好的 `&str`，不需要再解包
    word
}

/// `matches!`：把"这个值匹配某个模式吗"压缩成一次布尔判断。
///
/// 等价于 `match value { pattern => true, _ => false }`，但更短也更易读。
fn is_in_flight(task: &Task) -> bool {
    matches!(task, Task::Running { .. })
}

/// `matches!` 也支持 or-pattern（`|`）和范围模式：这里判断 HTTP 状态码是否属于"短响应"区间。
fn is_short_status(code: u16) -> bool {
    matches!(code, 100..=199 | 300..=399)
}

/// `Option` 组合子：链式表达"有值就变换，没值就短路"。
///
/// 解决的问题：层层 `match` 处理 `Option` 会写成"金字塔"；组合子把任意一步的失败都自动短路成 `None`。
/// `map` 负责变换 `Some` 里的值，`ok()` 则把 `Result` 降级成 `Option`（丢掉错误细节）。
fn parse_positive_doubled(raw: &str) -> Option<i32> {
    raw.trim()
        .parse::<i32>()
        .ok()
        .filter(|n| *n > 0)
        .map(|n| n * 2)
}

/// `and_then`：当下一步操作**本身也可能返回 `Option`** 时用它。
///
/// 解决的问题：用 `map` 会得到嵌套的 `Option<Option<T>>`，`and_then` 会把它压平。
/// 这里从形如 `"host=localhost;port=9000"` 的配置里取值，任何一步失败都返回 `None`。
fn lookup_port(config: Option<&str>, key: &str) -> Option<u16> {
    config
        .and_then(|text| text.split(';').find(|entry| entry.starts_with(key)))
        .and_then(|entry| entry.split('=').nth(1))
        .and_then(|value| value.trim().parse::<u16>().ok())
}

/// `unwrap_or`：为"没有值"提供一个兜底值。
///
/// ⚠️ 常见坑: 兜底值如果是昂贵计算（分配、IO），用 `unwrap_or_else(|| ...)`，
/// 否则无论有没有值都会先算一遍。
fn port_or_default(config: Option<&str>) -> u16 {
    lookup_port(config, "port").unwrap_or(8080)
}

/// 查找用户名第一次出现的下标：找不到就返回 `None`，而不是约定俗成的 `-1`。
///
/// 这就是 `Option<T>` 的价值：调用方必须显式处理"没找到"。
fn find_user(users: &[&str], target: &str) -> Option<usize> {
    users.iter().position(|user| *user == target)
}

/// 程序入口：按小节顺序演示本章所有知识点。
fn main() {
    println!("=== 1. 带数据的枚举变体与 match 穷尽性 ===");
    let shapes = [
        Shape::Circle { radius: 1.0 },
        Shape::Rectangle {
            width: 3.0,
            height: 4.0,
        },
        Shape::Point,
    ];
    for shape in &shapes {
        println!("  {shape:?} 的面积 = {:.3}", shape_area(shape));
    }

    println!("\n=== 2. match guard ===");
    for n in [-5, 0, 4, 7] {
        println!("  {n} -> {}", classify_number(n));
    }

    println!("\n=== 3. 解构绑定与通配符 `_` ===");
    println!("  {}", describe_pair((Some(200), true)));
    println!("  {}", describe_pair((Some(7), false)));
    println!("  {}", describe_pair((None, true)));

    println!("\n=== 4. if let 与 while let ===");
    print_circle_only(&shapes[0]);
    print_circle_only(&shapes[2]);
    let popped = drain_stack(vec!["alpha", "beta", "gamma"]);
    println!("  共弹出 {popped} 个元素");

    println!("\n=== 5. let ... else ===");
    println!(
        "  [hi, rust, ok] -> {}",
        first_long_word(&["hi", "rust", "ok"])
    );
    println!("  [a, bb]        -> {}", first_long_word(&["a", "bb"]));

    println!("\n=== 6. matches! 宏 ===");
    let tasks = [
        Task::Pending,
        Task::Running { progress: 40 },
        Task::Done,
        Task::Failed(String::from("超时")),
    ];
    for task in &tasks {
        println!("  {task} -> 在飞行中? {}", is_in_flight(task));
    }
    println!("  状态码 204 属于短响应? {}", is_short_status(204));
    println!("  状态码 500 属于短响应? {}", is_short_status(500));

    println!("\n=== 7. Option<T> 与组合子（map / and_then / unwrap_or） ===");
    let users = ["alice", "bob"];
    println!("  bob 的下标 = {:?}", find_user(&users, "bob"));
    println!("  zoe 的下标 = {:?}", find_user(&users, "zoe"));
    println!(
        "  \" 42 \" 解析并翻倍 = {:?}",
        parse_positive_doubled(" 42 ")
    );
    println!("  \"-3\" 解析并翻倍  = {:?}", parse_positive_doubled("-3"));
    println!("  \"abc\" 解析并翻倍 = {:?}", parse_positive_doubled("abc"));
    println!(
        "  显式端口 = {}",
        port_or_default(Some("host=localhost;port=9000"))
    );
    println!("  缺省端口 = {}", port_or_default(Some("host=localhost")));
    println!("  没有配置 = {}", port_or_default(None));
}
