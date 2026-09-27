//! 第 8 章：trait 与动态分发（traits & dynamic dispatch）
//!
//! trait 是 Rust 表达"某个类型能做什么"的方式，作用类似其他语言的接口
//! （interface），但更强：既能作为约束参与静态分发（static dispatch），
//! 也能变成 trait 对象（trait object）参与动态分发（dynamic dispatch）。
//!
//! 学完本章，你可以在自己的类型上实现 trait、用 `dyn Trait` 写插件式代码，
//! 也能判断何时该用泛型、何时该为灵活性付出一次虚表查找的代价。
//!
//! 运行：`cargo run --bin 08_traits`
//!
//! 关键知识点：
//! - trait 定义与默认方法
//! - 为自定义类型实现 trait
//! - supertrait
//! - `dyn Trait` trait 对象
//! - 静态分发 vs 动态分发（vtable）
//! - `impl Trait` 作为返回值
//! - 对象安全（object safety）与孤儿规则（orphan rule）
//! - 在集合里存放 trait 对象

use std::fmt::{self, Display};
use std::str::FromStr;

/// "可摘要内容"的统一行为接口。
///
/// 解决什么问题：让不同数据结构共享同一套调用方式，调用方不必知道具体类型。
/// 什么时候用：当"行为"比"数据形状"更重要时（渲染器、中间件、插件系统）。
trait Summary {
    /// 必须实现：作者或来源。
    fn author(&self) -> String;

    /// 必须实现：正文内容。
    fn body(&self) -> String;

    /// 默认方法（default method）：用已有方法拼出摘要，实现者可以不写。
    fn summarize(&self) -> String {
        format!("{}：{}", self.author(), self.body())
    }

    /// 默认方法可以被覆盖（override），这里给出"按字符截断"的实现。
    fn short(&self) -> String {
        // ⚠️ 常见坑: 不要用 `&body[..12]` 按字节截断——切在中文等多字节
        //    UTF-8 序列中间会直接 panic。用 chars() 按字符取才安全。
        let clipped: String = self.body().chars().take(12).collect();
        format!("{clipped}…")
    }
}

/// 新闻文章：只实现两个必须项，`summarize` 复用默认实现。
struct NewsArticle {
    /// 标题（本章把它当作正文）。
    headline: String,
    /// 记者署名。
    reporter: String,
}

/// 为自定义类型实现 trait：必须补齐所有没有默认实现的方法。
impl Summary for NewsArticle {
    fn author(&self) -> String {
        self.reporter.clone()
    }

    fn body(&self) -> String {
        self.headline.clone()
    }
}

/// 推文：覆盖默认的 `short`，演示"默认方法可被替换"。
struct Tweet {
    /// 用户名（不含 `@`）。
    username: String,
    /// 推文内容。
    content: String,
}

/// 只实现必须项，`summarize` 继续走默认实现。
impl Summary for Tweet {
    fn author(&self) -> String {
        format!("@{}", self.username)
    }

    fn body(&self) -> String {
        self.content.clone()
    }

    /// 覆盖默认方法：推文本身很短，不需要截断。
    fn short(&self) -> String {
        self.content.clone()
    }
}

/// 孤儿规则（orphan rule）：`impl Trait for Type` 至少要有一方属于本 crate。
///
/// `Summary` 定义在本文件（本地 trait），`String` 来自标准库（外部类型），
/// "本地 trait + 外部类型"允许；反过来 `impl Display for String` 会被拒绝
/// （外部 trait + 外部类型），否则标准库和用户 crate 都能实现，冲突无法裁决。
/// 什么时候用：想给第三方类型加能力时——先定义自己的 trait，再为它实现。
impl Summary for String {
    fn author(&self) -> String {
        "std::string::String".to_string()
    }

    fn body(&self) -> String {
        self.clone()
    }
}

/// supertrait（超 trait）：想实现 `Card` 就必须先实现 `Display`。
///
/// 解决什么问题：默认方法里想用 `{}` 打印 `self`，就必须保证 `Self: Display`。
/// 什么时候用：trait 之间确实存在能力依赖时，如标准库的 `Error: Debug + Display`。
trait Card: Display {
    /// 返回卡片编号。
    fn card_id(&self) -> u32;

    /// 默认方法：渲染一张卡片，内部依赖 supertrait 提供的 `Display`。
    fn render(&self) -> String {
        format!("[{}] #{}", self, self.card_id())
    }
}

/// 会员卡：先满足 supertrait，再实现 `Card`。
struct Member {
    /// 会员名。
    name: String,
    /// 会员编号。
    id: u32,
}

/// 第一步：满足 supertrait 的条件。
impl Display for Member {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "会员 {}", self.name)
    }
}

/// 第二步：实现 `Card`，此时默认方法 `render` 里的 `{self}` 才能编译。
impl Card for Member {
    fn card_id(&self) -> u32 {
        self.id
    }
}

/// 对象安全（object safety）：能配上 `dyn` 的 trait 必须满足若干条件。
///
/// 关键限制之一是"不能有泛型方法"——vtable 无法为无穷多个类型参数准备入口。
/// 补救办法是给泛型方法加 `where Self: Sized`，把它排除出 vtable，
/// 从而恢复对象安全（该方法仍能在具体类型上静态调用）。
trait Labeled {
    /// 非泛型方法：会进入 vtable，可以通过 `dyn Labeled` 调用。
    fn label(&self) -> &str;

    /// 泛型方法：`where Self: Sized` 让它不参与动态分发。
    fn parse_field<T: FromStr>(&self, raw: &str) -> Option<T>
    where
        Self: Sized,
    {
        raw.parse::<T>().ok()
    }
}

/// 同一个类型可以实现多个 trait：`Member` 复用为 `Labeled` 的实现者。
impl Labeled for Member {
    fn label(&self) -> &str {
        &self.name
    }
}

/// 静态分发（static dispatch）：`impl Trait` / 泛型参数在编译期完成单态化。
///
/// 调用方直接跳到具体类型的代码，没有查表开销，属于零成本抽象；
/// 代价是每种传入类型都会生成一份代码，二进制体积随之增大。
fn static_summary(item: &impl Summary) -> String {
    item.summarize()
}

/// 动态分发（dynamic dispatch）：`&dyn Summary` 是"胖指针"（fat pointer），
/// 由数据指针与 vtable 指针组成，调用时经 vtable 查函数地址。
/// 什么时候用：具体类型运行期才能确定，或想避免为大量类型生成重复代码。
fn dynamic_summary(item: &dyn Summary) -> String {
    item.summarize()
}

/// 在同一集合里存放不同具体类型的 trait 对象，逐个求摘要。
///
/// `Box<dyn Summary>` 用一次堆分配换来"异构集合"：元素大小可以不同，
/// 因为 `dyn Summary` 是未定长类型（unsized），必须放在指针后面。
fn summarize_all(items: &[Box<dyn Summary>]) -> Vec<String> {
    items.iter().map(|item| item.summarize()).collect()
}

/// `impl Trait` 作为返回值：向调用方隐藏具体类型，只承诺"实现了 `Summary`"。
///
/// ⚠️ 常见坑: 返回位置只能对应唯一的具体类型，两个分支分别返回 `NewsArticle`
///    和 `Tweet` 会编译失败；要返回不同类型，只能写成 `-> Box<dyn Summary>`。
fn make_summary() -> impl Summary {
    NewsArticle {
        headline: "Rust 让抽象零成本".to_string(),
        reporter: "编辑部".to_string(),
    }
}

/// 程序入口：按小节顺序演示本章所有知识点。
fn main() {
    println!("=== 1. trait 定义与默认方法 ===");
    let article = NewsArticle {
        headline: "trait 让类型共享行为".to_string(),
        reporter: "编辑部".to_string(),
    };
    println!("summarize（默认）: {}", article.summarize());
    println!("short（默认）: {}", article.short());

    println!("\n=== 2. 为自定义类型实现 trait（含覆盖默认方法）===");
    let tweet = Tweet {
        username: "rustlang".to_string(),
        content: "hello traits".to_string(),
    };
    println!("summarize（默认）: {}", tweet.summarize());
    println!("short（被覆盖）: {}", tweet.short());
    let quoted = "由 String 实现 Summary".to_string();
    println!("String 版本: {}", quoted.summarize());

    println!("\n=== 3. supertrait ===");
    let member = Member {
        name: "小明".to_string(),
        id: 42,
    };
    println!("Display: {member}");
    println!("Card::render: {}", member.render());

    println!("\n=== 4. dyn Trait（trait 对象）===");
    let as_object: &dyn Summary = &article;
    println!("dynamic_summary(&article) = {}", dynamic_summary(as_object));

    println!("\n=== 5. 静态分发 vs 动态分发（vtable）===");
    println!("static_summary(&tweet) = {}", static_summary(&tweet));
    let dyn_width = std::mem::size_of::<&dyn Summary>();
    let concrete_width = std::mem::size_of::<&NewsArticle>();
    println!("&dyn Summary = {dyn_width} 字节，&NewsArticle = {concrete_width} 字节");

    println!("\n=== 6. impl Trait 作为返回值 ===");
    println!("make_summary: {}", make_summary().summarize());

    println!("\n=== 7. 对象安全与孤儿规则 ===");
    let parsed: Option<u32> = member.parse_field::<u32>("42");
    println!("Labeled::label = {}", member.label());
    println!("parse_field::<u32>(\"42\") = {parsed:?}");
    let object: &dyn Labeled = &member;
    println!("通过 dyn 调用 = {}", object.label());

    println!("\n=== 8. 在集合里存放 trait 对象 ===");
    let mut items: Vec<Box<dyn Summary>> =
        vec![Box::new(article), Box::new(tweet), Box::new(make_summary())];
    items.push(Box::new(quoted));
    for (index, text) in summarize_all(&items).iter().enumerate() {
        println!("[{index}] {text}");
    }
}
