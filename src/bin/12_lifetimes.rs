//! 第 12 章：生命周期（lifetimes）
//!
//! 生命周期是借用检查器（borrow checker）用来验证"引用不会比它指向的数据活得更久"
//! 的编译期信息。绝大多数时候它是自动推导的；只有编译器无法从签名上判断
//! 输入和输出的关系时，才需要我们手写标注。生命周期不改变任何数据的实际寿命，
//! 它只是把已经存在的关系写成类型的一部分。
//!
//! 运行：`cargo run --bin 12_lifetimes`
//!
//! 关键知识点：
//! - 为什么需要生命周期标注
//! - 泛型生命周期参数 `'a`
//! - 函数签名中的三条省略规则（elision rules）
//! - 结构体持有引用
//! - `impl` 块与方法中的生命周期
//! - `'static`
//! - 生命周期约束 `T: 'a`
//! - 常见报错（返回局部变量引用）
//!
//! 三条省略规则（编译器按顺序套用，套不上就必须手写）：
//! 1. 每个省略的**输入**引用各自获得一个独立的匿名生命周期；
//! 2. 如果恰好只有一个输入生命周期，它被赋给所有省略的**输出**生命周期；
//! 3. 如果参数里有 `&self` 或 `&mut self`，`self` 的生命周期被赋给省略的输出。

use std::fmt::Display;

/// 返回两个字符串切片中更长的那一个，长度相同时返回第一个。
///
/// 为什么必须写 `'a`：函数有两个引用入参，编译器无法判断返回值借用的是 `x`
/// 还是 `y`，也就无法在调用点检查返回值会不会悬垂。
/// `<'a>` 是泛型生命周期参数，`&'a str` 读作"至少活 `'a` 这么久的字符串切片"；
/// 调用处 `'a` 被推导为两个实参生命周期的交集（即较短的那个）。
///
/// 示例调用：`longest("abcd", "ab")` 返回 `"abcd"`。
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() >= y.len() {
        x
    } else {
        y
    }
}

/// 只取第一个单词：演示省略规则 2——只有一个输入生命周期时，输出直接采用它。
///
/// 所以这里一个 `'a` 都不用写，语义与 `fn first_word<'a>(text: &'a str) -> &'a str` 相同。
fn first_word(text: &str) -> &str {
    text.split_whitespace().next().unwrap_or("")
}

/// 拼接两个来源的数据并返回拥有所有权的 `String`。
///
/// 返回值不借用任何入参，于是两个引用各拿一个独立的匿名生命周期（省略规则 1），
/// 调用时一个字符串长期存在、另一个用完就销毁也完全合法。
fn describe(left: &str, right: &str) -> String {
    format!("{left} + {right}")
}

/// 从句子中借出一段文本（slice），不复制字符串。
///
/// 结构体里存引用就必须带生命周期参数：`'a` 说明"`part` 和它来源的那个字符串
/// 活得一样久"，因此 `Excerpt` 实例不能比原字符串活得更久。
#[derive(Debug)]
struct Excerpt<'a> {
    /// 借来的文本片段。
    part: &'a str,
}

/// `Excerpt` 的构造、读取与"返回字段引用"的方法，演示方法里的生命周期怎么算。
impl<'a> Excerpt<'a> {
    /// 借入一段外部的 `&'a str` 构造摘录，不复制、不移动原字符串。
    fn new(part: &'a str) -> Self {
        Excerpt { part }
    }

    /// 读取片段内容：参数里有 `&self`，输出引用按省略规则 3 借用 `self` 的生命周期。
    ///
    /// 示例调用：`Excerpt::new("你好").part()` 返回 `"你好"`。
    fn part(&self) -> &str {
        self.part
    }

    /// 返回的引用绑定在结构体参数 `'a` 上，而不是 `&self` 借用的那个短生命周期。
    ///
    /// 这里 `'a` 不能省：一旦省略，规则 3 会把输出绑到 `&self` 的生命周期上，
    /// 语义就变紧了。参数 `hint` 只在函数体内用一次，可以省略它的生命周期。
    fn part_with_hint(&self, hint: &str) -> &'a str {
        println!("（提示：{hint}，它在本次调用后立刻失效也没关系）");
        self.part
    }
}

/// 同时借用两个不同来源的文本，所以需要两个相互独立的生命周期参数。
///
/// 如果强行写成同一个 `'a`，两段文本就必须活得一样久，结构体会变得难用。
#[derive(Debug)]
struct Pair<'a, 'b> {
    /// 来自第一个来源的片段。
    first: &'a str,
    /// 来自第二个来源的片段，其生命周期与 `'a` 无关。
    second: &'b str,
}

/// `Pair` 的方法：两个生命周期参数原样带在 `impl` 上。
impl<'a, 'b> Pair<'a, 'b> {
    /// 把两个片段拼成新的 `String` 返回，避免再引入一个输出生命周期。
    fn joined(&self) -> String {
        format!("{}|{}", self.first, self.second)
    }
}

/// 字符串字面量的类型是 `&'static str`：它被编译进二进制，整个程序运行期间都有效。
fn static_label() -> &'static str {
    "静态生命周期 'static"
}

/// `T: 'static` 约束：`T` 内部不能含有比 `'static` 更短的借用。
///
/// 拥有所有权的值（`String`、`i32`）天然满足这个约束；`&'a str` 只有 `'a = 'static`
/// 时才行。⚠️ 常见坑: 为了"让编译通过"就给参数乱加 `'static`，
/// 会让函数拒绝一切借用数据——先想清楚是不是真的需要长期保存。
fn archive<T: Display + 'static>(value: T) -> String {
    format!("已归档: {value}")
}

/// 持有 `&'a T` 的结构体，用来演示生命周期约束 `T: 'a`。
///
/// `T: 'a` 读作"`T` 活得至少和 `'a` 一样久"，它保证字段里的 `&'a T` 不会指向
/// 比 `'a` 更早失效的数据。对 `&'a T` 这种写法它其实会被自动推导出来
/// （隐式约束，implied bound），显式写出来是为了看清规则本身；
/// 真正需要手写它的场景是 `Box<dyn Trait + 'a>`、关联类型等间接引用。
#[derive(Debug)]
struct Holder<'a, T: 'a> {
    /// 借来的值，不复制、不移动。
    value: &'a T,
}

/// `Holder` 的方法：`impl` 上不必重复写 `T: 'a`，结构体定义里的约束已经生效。
impl<'a, T> Holder<'a, T> {
    /// 原样返回借来的引用。
    ///
    /// 输出的 `'a` 来自结构体参数，比 `&self` 的匿名生命周期更长，因此不能省略。
    fn get(&self) -> &'a T {
        self.value
    }
}

/// 把 `&'a T` 装进结构体返回：`T: 'a` 正是让这件事安全成立的条件。
fn hold<'a, T: 'a>(value: &'a T) -> Holder<'a, T> {
    Holder { value }
}

// ⚠️ 常见坑: 返回局部变量的引用。下面这段是**故意编译不过**的代码，留作对照：
//
//     fn dangling() -> &String {
//         let owned = String::from("局部数据");
//         &owned
//     }
//     // error[E0106]: missing lifetime specifier（返回类型缺少生命周期）
//     // 即使补成 fn dangling<'a>() -> &'a String，仍会报：
//     // error[E0515]: cannot return reference to local variable `owned`
//
// 原因：`owned` 是函数内的局部变量，函数返回时它就被 drop 了，返回的引用
// 立刻悬垂（dangling）。**生命周期标注不能延长数据的寿命**，它只能描述
// 已经存在的关系——这是初学者最常见的误解。三种修法：
// 1. 返回拥有所有权的值（下面的 `dangling_fixed`，最常用）；
// 2. 让调用者把缓冲区作为 `&mut String` 传进来；
// 3. 让返回值借用某个入参（如 `longest`），或返回 `'static` 数据。

/// 返回拥有所有权的 `String`，而不是对局部变量的引用：最常见、最省心的修法。
///
/// 示例调用：`dangling_fixed("局部数据")` 返回一个可长期保存的 `String`。
fn dangling_fixed(seed: &str) -> String {
    let mut owned = String::from(seed);
    owned.push_str("（已复制成拥有所有权的值）");
    owned
}

/// 程序入口：按小节顺序演示本章所有知识点。
fn main() {
    println!("=== 1. 为什么需要生命周期标注 ===");
    let left = String::from("长一点的字符串");
    let right = String::from("短的");
    // 两个引用都是可用的字面数据，'a 被推导成两者中较短的那个
    println!("更长的那个 = {}", longest(&left, &right));
    println!("更长的那个 = {}", longest("直接传字面量", &left));

    println!("\n=== 2. 泛型生命周期参数 'a 与三条省略规则 ===");
    println!(
        "规则 2（唯一输入决定输出）: {}",
        first_word("hello lifetime world")
    );
    let long_lived = String::from("长期存在");
    {
        let short_lived = String::from("临时数据");
        // 规则 1：两个入参各自独立推导，短命的那个不会拖累另一个
        println!(
            "规则 1（入参互相独立）: {}",
            describe(&long_lived, &short_lived)
        );
    }
    println!("短命引用已销毁，long_lived 仍然可用: {long_lived}");

    println!("\n=== 3. 结构体持有引用 ===");
    let sentence = String::from("Rust 的生命周期是编译期信息");
    let excerpt = Excerpt::new(first_word(&sentence));
    println!("摘录 = {}", excerpt.part());
    println!("结构体调试输出 = {excerpt:?}");

    let first_source = String::from("甲");
    let second_source = String::from("乙");
    let pair = Pair {
        first: &first_source,
        second: &second_source,
    };
    println!("两个独立来源拼起来 = {}", pair.joined());

    println!("\n=== 4. impl 块与方法中的生命周期 ===");
    let text = String::from("返回值绑定在 'a 上");
    let excerpt = Excerpt::new(&text);
    let escaped;
    {
        let short_hint = String::from("短命的提示");
        escaped = excerpt.part_with_hint(&short_hint);
    } // short_hint 在此失效，但 escaped 借用的是 text，所以依然可用
    println!("提示失效后仍能读取返回值: {escaped}");

    println!("\n=== 5. 'static ===");
    println!("{}", static_label());
    println!("{}", archive(String::from("拥有所有权的 String")));
    println!("{}", archive(static_label()));

    println!("\n=== 6. 生命周期约束 T: 'a ===");
    let number = 42;
    println!("持有借来的值 = {:?}", hold(&number).get());

    println!("\n=== 7. 常见报错：返回局部变量引用 ===");
    println!("{}", dangling_fixed("局部数据"));
}
