//! 第 13 章：模块与包管理
//!
//! 模块系统（module system）解决"**名字空间 + 可见性 + 代码组织**"三件事：代码一多就要把
//! 相关条目收进模块，并只对外暴露想暴露的部分。本章所有模块都**内联在本文件**（`mod xxx { ... }`），
//! 保证每个 bin 自包含；真实项目的文件拆分、workspace 写法、cargo 命令在注释与 `///` 文档中给出。
//!
//! 运行：`cargo run --bin 13_modules`
//!
//! 关键知识点：
//! - `mod` 内联模块 vs 拆成 `src/foo.rs` / `src/foo/mod.rs`
//! - `pub` / `pub(crate)` / `pub(super)` 与"默认私有"规则
//! - `use` 路径：`crate::`、`self::`、`super::`
//! - `pub use` 重导出（re-export）与 facade/prelude 模式
//! - `#[cfg(test)] mod tests` 单元测试模块
//! - package / crate / bin / lib 的区别、workspace 多包结构、常用 cargo 命令

/// 演示用的"项目根"常量，等价于真实项目里写在 `src/lib.rs` 的公开常量。
const PROJECT_NAME: &str = "rustlearn";

/// 第 1 节：内联模块 = 真实项目的目录树。
///
/// ```text
/// src/lib.rs           <= crate root（crate 根），写 `pub mod shapes;`
/// src/shapes.rs        <= `pub mod shapes { ... }` 的单文件等价写法
/// src/shapes/circle.rs <= shapes 里再写 `pub mod circle;` 后拆成子目录
/// src/shapes/mod.rs    <= 老写法：与 src/shapes.rs 二选一，不能同时存在
/// ```
/// Cargo 规则：`mod foo;` 去找同目录的 `foo.rs` 或 `foo/mod.rs`；两个都在会报
/// `file for module 'foo' found at both ...`。
///
/// package / crate / bin / lib 的区别：**package** = 一个 `Cargo.toml`，可含多个 crate；
/// **crate** = 编译单元（1 个 lib crate + 每个 bin crate 各自独立编译，本工程 = 1 package、
/// 16 个 bin、无 lib）；**bin** 有 `fn main()`，**lib** 没有、供别人 `use`（crate 名中的
/// `-` 写成 `_`：`my-app` 要 `use my_app::...`）；同 package 的 bin 用 **package 名**引用其 lib。
fn demo_hierarchy() {
    println!("package 名(同时是 lib crate 名) = {PROJECT_NAME}");
    // `::` 连接模块路径；从 crate 根写起是"绝对路径"，模块搬家也不失效。
    println!("绝对路径调用: {}", crate::shapes::circle::area(2.0));
    println!("元组结构体: {:?}", crate::shapes::Point(1, 2));
}

/// 形状模块：对应真实项目里的 `src/shapes.rs`（或 `src/shapes/mod.rs`）。
pub mod shapes {
    /// 二维点，演示元组结构体同样受模块可见性约束。
    #[derive(Debug, PartialEq, Eq)]
    pub struct Point(pub i32, pub i32);

    /// 圆相关计算，对应 `src/shapes/circle.rs`。
    pub mod circle {
        use std::f64::consts::PI;
        /// 半径缩放系数，不写 `pub` 即"仅本模块可见"。
        const SCALE: f64 = 1.0;
        /// 计算圆面积：`area(半径)`。示例：`circle::area(1.0) == PI`。
        pub fn area(radius: f64) -> f64 {
            PI * radius * radius * SCALE
        }
        /// 直径计算：`pub(crate)` = 本 crate 可用、外部 crate 不可见。
        /// ⚠️ 常见坑: `pub(crate)` 常被误当公开 API。它是"crate 内可见"，外部使用者永远看不到；
        /// 库的公开边界只能用 `pub`。
        pub(crate) fn diameter(radius: f64) -> f64 {
            radius * 2.0
        }
    }

    /// 演示 `pub(super)`：只对**父模块**（这里是 crate 根）及其子树可见。
    pub(super) fn module_label() -> &'static str {
        "shapes"
    }
}

/// 第 2 节：可见性规则的演示载体。
#[derive(Debug)]
pub struct Visibility;

/// 父模块：它内部的私有条目，子模块默认**不能**访问。
pub mod parent {
    /// 父模块私有项：只有 `parent` 及其后代模块能访问。
    fn private_secret() -> &'static str {
        "父模块私有数据"
    }
    /// 子模块：用 `pub(super)` 向父模块方向提供入口。
    pub mod child {
        /// 用 `super::` 从子模块读父模块私有函数。
        /// ⚠️ 常见坑: 可见性是"**子模块能看见祖先的私有项**"，反过来不行；父模块要访问
        /// 子模块内部条目，必须给那个条目加 `pub(super)`。
        pub fn read_parent_private() -> &'static str {
            super::private_secret()
        }
    }
}

/// 第 3 节：路径与 `use` 前缀。演示 `crate::`（crate 根）、`self::`（当前模块）、
/// `super::`（父模块）以及外部 crate（这里用 `std`）的绝对路径写法。
fn demo_use_paths() -> String {
    // `crate::` 是绝对路径，把模块整体搬家也不会失效。
    let crate_path = crate::PROJECT_NAME;
    // `self::` 强调"当前模块里的条目"，比裸名字更利于阅读。
    let self_path = self::helper_label();
    // `super::` 只有在嵌套模块里才有意义：crate 根没有父模块，用了会编译报错。
    let sibling = shapes::module_label();
    // 外部 crate 用绝对路径；`use std::...;` 之后可以写短名字。
    let external = std::mem::size_of::<u8>();
    format!("{crate_path}/{self_path}/{sibling}/{external}")
}

/// 供 `self::` 路径演示调用的本地函数。
fn helper_label() -> &'static str {
    "helper"
}

/// 第 4 节：`pub use` 重导出（re-export）+ 门面（facade）模式。
pub mod api {
    // `pub use` 不复制类型，只是给已有条目再挂一条对外路径。
    pub use crate::counter::Counter;
    // 重导出函数同理：使用者不必知道它原来住在 `counter` 模块里。
    pub use crate::counter::make_counter;
    /// 预导入模块（prelude）：库作者集中重导出常用类型，使用者
    /// `use lib::prelude::*;` 一次拉齐；`std::prelude` 就是这个套路。
    pub mod prelude {
        pub use super::Counter;
    }
}

/// 计数器模块：配合第 4 节证明"重导出后的两条路径是同一个类型"。
pub mod counter {
    /// 只增不减的计数器，字段私有，外部只能走方法（封装）。
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Counter {
        /// 当前计数值。
        value: u32,
    }
    /// 构造 `Counter(cur)`，是 `pub` API 的一部分（工厂函数）。
    pub fn make_counter(cur: u32) -> Counter {
        Counter { value: cur }
    }
    impl Counter {
        /// 自增 1。
        pub fn bump(&mut self) {
            self.value += 1;
        }
        /// 读取当前值。
        pub fn value(&self) -> u32 {
            self.value
        }
    }
    impl Counter {
        /// 重置为 0；`pub(crate)` 表示只有本 crate 能用。
        /// ⚠️ 常见坑: `impl` 块本身**没有**可见性（不能写 `pub impl`），但块内每个方法可
        /// 各自标注；trait 实现里的方法必须与 trait 声明一致。
        pub(crate) fn reset(&mut self) {
            self.value = 0;
        }
    }
}

/// 第 6 节：workspace 多包结构（只讲解，不执行子进程）。本工程是"单 package、多 bin、无 lib"：
///
/// ```toml
/// [package]                     # 本工程 Cargo.toml（不要修改）
/// name = "rustlearn"            # package 名 == 默认 bin/lib 名前缀
/// edition = "2021"
/// rust-version = "1.75"         # MSRV：可编译的最低 Rust 版本
/// publish = false               # 不发布到 crates.io
/// # 无 [[bin]]：src/bin/*.rs 被 Cargo 自动发现，每个文件一个 bin target
/// ```
///
/// 要拆成多包就用 workspace（虚拟清单，自身不含 `[package]`）：
///
/// ```toml
/// # 根 Cargo.toml —— workspace 清单
/// [workspace]
/// resolver = "2"                      # edition 2021 对应 resolver 2
/// members = ["crates/core", "crates/cli"]
/// [workspace.package]                 # 子包用 edition.workspace = true 继承
/// edition = "2021"
/// rust-version = "1.75"
/// [workspace.dependencies]            # 子包用 serde.workspace = true 继承
/// serde = { version = "1", features = ["derive"] }
///
/// # crates/cli/Cargo.toml —— 依赖同 workspace 的另一个 package
/// [package]
/// name = "cli"
/// edition.workspace = true
/// [dependencies]
/// core = { path = "../core" }
/// serde.workspace = true
/// ```
///
/// ⚠️ 常见坑: workspace 成员各有 `Cargo.toml` 与 `src/`，但 **`target/`、`Cargo.lock`
/// 全 workspace 只有一份**（在根目录）；`cargo build -p core` 只编译单个成员。
///
/// cargo 命令速查：`cargo run --bin 13_modules`（运行指定 bin，本工程必须带 `--bin`）、
/// `cargo check --bin X`（只做类型检查，改代码时最高频）、`cargo build --bin X`、
/// `cargo clippy --bin X -- -D warnings`、`cargo test --bin X`、`cargo add serde` /
/// `cargo remove serde`（增删依赖，会改 `Cargo.toml`）、`cargo tree --bin X`（依赖树，
/// 查重复版本）、`cargo doc --no-deps --open`（生成并打开 API 文档）、
/// `cargo metadata --format-version 1`（机器可读结构，IDE 靠它找 target）、
/// `cargo build --release`（优化构建，产物在 `target/release/`）。
fn demo_cargo_cheatsheet() {
    // 命令只做讲解，不在程序里执行（子进程输出会污染确定性输出）。
    println!("速查见本函数 `///` 文档，例如: cargo clippy --bin 13_modules -- -D warnings");
}

/// 程序入口：按小节顺序演示本章所有知识点。
fn main() {
    println!("=== 1. 内联模块与项目目录结构 ===");
    demo_hierarchy();
    println!("\n=== 2. 可见性：pub / pub(crate) / pub(super) ===");
    println!("crate 内可见: {}", shapes::circle::diameter(2.0));
    println!("pub(super) 条目: {}", shapes::module_label());
    println!(
        "子模块读父模块私有项: {}",
        parent::child::read_parent_private()
    );
    println!("\n=== 3. use 路径：crate:: / self:: / super:: ===");
    println!("组合路径结果: {}", demo_use_paths());
    println!("\n=== 4. pub use 重导出 ===");
    // 重导出的两条路径（api::Counter 与 counter::Counter）是同一个类型，可以互相赋值：把 api
    // 路径构造的值赋给声明为 counter 路径的变量，类型检查通过即证明二者同型。
    // ⚠️ 常见坑: 别写成 `let b = a; a = b;` 的回环赋值，clippy 的 almost_swapped 会直接报错。
    let mut via_api: api::Counter = api::make_counter(10);
    let mut via_module: counter::Counter = via_api;
    via_module.bump();
    via_api = via_module;
    via_api.reset();
    println!("重导出后 count = {}", via_api.value());
    // 一行 `use` 把 prelude 常用项拉进作用域，避免到处写长路径。
    use api::prelude::Counter;
    let typed: Counter = counter::make_counter(7);
    println!("prelude 类型: {typed:?}");
    println!("\n=== 5. #[cfg(test)] mod tests ===");
    println!("测试模块只在 `cargo test --bin 13_modules` 时编译，正式构建被剔除");
    println!("\n=== 6. package / crate / bin / lib 与 workspace ===");
    demo_cargo_cheatsheet();
}

/// 第 5 节：`#[cfg(test)] mod tests`。测试与被测代码同文件，因此能访问私有条目；
/// `#[cfg(test)]` 保证它**只在 `cargo test` 时参与编译**，不进入正式产物。
///
/// ⚠️ 常见坑:
/// 1. 若把 `#[cfg(test)]` 只挂在某个测试函数上，它仍会被正式构建编译（平白增加体积并
///    可能触发 dead_code），要挂在模块上。
/// 2. 惯例是把测试模块放在**文件末尾**：clippy 的 `items_after_test_module` 会警告
///    "测试模块之后还有正式条目"，因为那种顺序通常意味着代码被插错了位置。
#[cfg(test)]
mod tests {
    /// 单元测试能测私有函数，因为 `tests` 是当前模块的子模块。
    #[test]
    fn private_fn_is_reachable_from_tests() {
        assert_eq!(super::helper_label(), "helper");
    }

    /// 重导出路径与原路径指向同一个类型，可以互相赋值。
    #[test]
    fn reexport_is_same_type() {
        let mut a: super::api::Counter = super::counter::make_counter(1);
        a.bump();
        assert_eq!(a.value(), 2);
    }
    // 运行测试：`cargo test --bin 13_modules`（本工程规矩：永远带 `--bin`）
}
