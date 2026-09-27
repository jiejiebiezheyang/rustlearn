//! 第 16 章：常用标准库与生态
//!
//! 前 15 章都在讲语言本身，本章回答一个更实际的问题：
//! **真实项目里每天真正会用到的标准库和 crate 是哪些，怎么用才不出错？**
//!
//! 覆盖内容：
//! - `String` / `&str` 的区别与互转（这是新手放弃率最高的一道坎）
//! - `format!` 与字符串拼接的性能陷阱
//! - `std::fs`：读写文件、递归建目录、清理
//! - `std::io`：`Write`、`BufRead::lines`、`stdin().read_line`（含不阻塞的写法建议）
//! - `std::env`：`args`、`var`、`temp_dir`、`current_dir`
//! - `serde` + `serde_json`：结构体 ←→ JSON，以及 `Value` 动态取值
//! - `chrono`：`Utc::now`、`Local`、`Duration`、格式化与 RFC 3339 解析
//! - `Box<dyn Error>`：把多种错误来源汇合到一个签名里
//!
//! 运行：`cargo run --bin 16_std_ecosystem`
//!
//! ⚠️ 本章的输出**不是完全确定性**的：打印当前时间的那几行会随运行时刻变化。
//! 这是刻意的——时间本来就是变化的，程序里要断言时间请用"范围/存在性"断言，
//! 不要断言具体值（详见文件末尾 `demo_chrono` 的注释）。

// `use` 是 Rust 的作用域导入语法（第 13 章详解）。
// 这里每个导入都来自标准库或 Cargo.toml 中声明的 crate。
use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

// chrono 的 trait 必须显式导入才能调用其方法：
// `Datelike` 提供 .year()/.month()，`Timelike` 提供 .hour()/.minute()。
// ⚠️ 常见坑: 忘记 `use chrono::Datelike;` 会报 "no method named `year`"，
// 报错信息不会告诉你缺哪个 trait，只会在候选列表里提示 trait 未在作用域内。
use chrono::{DateTime, Datelike, Duration, Local, TimeZone, Utc};
// serde 的 derive 宏：给类型自动生成序列化/反序列化代码。
use serde::{Deserialize, Serialize};

/// 一个会被序列化成 JSON 的业务结构体。
///
/// `#[derive(Serialize, Deserialize)]` 解决什么问题？
/// 手写"结构体 → JSON 文本"和"JSON 文本 → 结构体"的代码又长又容易写错。
/// derive 宏在编译期生成这些代码，零运行时反射开销（对比 Java/Go 的反射式 JSON 库）。
///
/// 什么时候用：几乎任何需要配置读写、HTTP 请求/响应、存盘的场景。
///
/// ⚠️ 常见坑:
/// 1. 字段名默认与 Rust 字段名完全一致（这里是 snake_case），
///    对接 camelCase 的接口要加 `#[serde(rename_all = "camelCase")]`。
/// 2. 反序列化时**缺失字段会直接报错**，可选字段必须写 `Option<T>`，
///    或加 `#[serde(default)]`。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Task {
    /// 任务 id（JSON 中为数字，反序列化时类型不匹配会立刻报错，这正是强类型的好处）
    id: u32,
    /// 任务标题
    title: String,
    /// 是否完成；`#[serde(default)]` 表示 JSON 里没有这个字段时用 `bool::default()`（即 false）
    #[serde(default)]
    done: bool,
    /// 标签列表；没有该字段时默认为空 vec
    #[serde(default)]
    tags: Vec<String>,
}

/// 演示 `String` 与 `&str` 的区别、互转以及所有权语义。
///
/// 一句话总结：
/// - `String` = 堆上、可增长、拥有所有权（owned）
/// - `&str`  = 借用的 UTF-8 文本切片，可能指向 `String`、字面量或其它缓冲区
///
/// 什么时候该用：
/// - **函数参数**一律用 `&str`（除非你真的需要接管所有权），
///   这样 `String`、`&String`、字面量都能传进来（靠 Deref 强制转换）。
/// - **结构体字段 / 需要修改或长期持有**时用 `String`。
fn demo_string_vs_str() {
    // 字面量 "hello" 的类型是 &'static str：它被硬编码进二进制，永不释放。
    let literal: &str = "hello";
    // 从字面量创建可变的 String（会发生一次堆分配 + 拷贝）。
    let owned: String = String::from(literal);
    // to_string() 与 String::from 等价；.into() 也能，但类型推断失败时不如前两者清晰。
    let also_owned: String = literal.to_string();

    println!("literal = {literal}, owned = {owned}, also_owned = {also_owned}");

    // &String -> &str：Deref 强制转换自动完成，所以接 &str 的函数能收 &String。
    println!("借用长度 = {}", fn_takes_str(&owned));
    // 三者都能传给同一个 &str 参数，这就是"参数用 &str"的价值。
    println!(
        "三种调用都合法: {} {} {}",
        fn_takes_str(literal),
        fn_takes_str(&owned),
        fn_takes_str(&also_owned)
    );

    // 常见转换方向（记住这四种就够日常用）：
    let from_str_slice: String = String::from("abc"); // &str -> String
    let back_to_slice: &str = &from_str_slice; // String -> &str（借用，不拷贝）
    let owned_slice: String = back_to_slice.to_owned(); // &str -> String
    println!(
        "转换链长度: {} {} {}",
        from_str_slice.len(),
        back_to_slice.len(),
        owned_slice.len()
    );

    // ⚠️ 常见坑: len() 返回的是**字节数**不是字符数！
    // UTF-8 中一个汉字占 3 字节，用 len() 做"字数"统计会得到错误结果。
    let cn = String::from("你好");
    println!(
        "「你好」: 字节数 = {}, 字符数 = {}",
        cn.len(),
        cn.chars().count()
    );
}

/// 接受 `&str` 的函数：调用者可以传字面量、`&String`、`&str` 等任何文本。
///
/// ⚠️ 常见坑: 把参数写成 `&String` 会让调用方被迫多写 `&`，
/// 也会触发 clippy 的 `ptr_arg` 警告——参数位置请一律用 `&str`。
fn fn_takes_str(s: &str) -> usize {
    s.len()
}

/// 演示 `format!` 与拼接：什么时候用哪个、性能差别在哪。
fn demo_format_and_build() {
    let user = "ada";
    let count = 3;

    // format! 返回新的 String，是最常用、最不易出错的构造方式。
    let msg = format!("{user} 有 {count} 条新消息");
    println!("{msg}");

    // ⚠️ 常见坑: `+` 运算符的第一个操作数必须是 String（会 move 走它），
    // 后面的必须是 &str，而且每次 `+` 都可能重新分配内存。
    // 循环里用 `+` 拼接是经典性能杀手（O(n²) 拷贝）。
    let owned = String::from("hello");
    let joined = owned + " " + "world"; // owned 被移动，此后不能再使用 owned
    println!("+ 拼接 = {joined}");

    // 正确做法：预先估算容量 + push_str，避免反复扩容。
    let mut buf = String::with_capacity(64);
    for i in 0..count {
        buf.push_str(&format!("[{i}]"));
    }
    println!("push_str 拼接 = {buf}（容量 = {}）", buf.capacity());

    // write! 宏可以把格式化结果写进任何实现了 fmt::Write 的缓冲区（不会分配新 String）。
    use std::fmt::Write as _; // 局部导入，避免与 io::Write 冲突
    let mut out = String::new();
    let _ = write!(out, "{user}:{count}");
    println!("write! 到 String = {out}");
}

/// 演示 `std::fs`：写文件、读文件、递归建目录、清理。
///
/// 返回临时目录路径，供后面的小节继续使用。
///
/// 什么时候用：处理配置、日志、数据导入导出等本地文件场景。
/// ⚠️ 常见坑:
/// 1. 文件操作全部返回 `io::Result`，`unwrap()` 会在 CI/容器里因为路径不存在而 panic；
///    真实项目应把错误包装后向上抛（本章用 `?` 传播）。
/// 2. 不要写死绝对路径，用 `env::temp_dir()` 或项目内的相对路径。
/// 3. `fs::write` 会**截断已有文件**，需要追加请用 `OpenOptions::append(true)`。
fn demo_fs() -> Result<PathBuf, Box<dyn Error>> {
    // 每次运行用进程 id 造一个独立目录，避免并行运行互相覆盖。
    let dir = env::temp_dir().join(format!("rustlearn_ch16_{}", std::process::id()));

    // create_dir_all 会递归创建缺失的父目录；create_dir 只创建最后一级。
    fs::create_dir_all(&dir)?;

    let file_path = dir.join("tasks.json");
    let content = "line-1\nline-2\nline-3\n";
    fs::write(&file_path, content)?; // 一次写完全部内容

    // read_to_string 要求内容是合法 UTF-8；二进制数据请用 fs::read（得到 Vec<u8>）。
    let read_back = fs::read_to_string(&file_path)?;
    println!(
        "写入并读回 {} 字节（与原始内容一致: {}）",
        read_back.len(),
        read_back == content
    );

    // 元数据：文件大小、是否为文件。注意这就是 stat 调用，会有系统开销。
    let meta = fs::metadata(&file_path)?;
    println!(
        "文件大小 = {} 字节, is_file = {}",
        meta.len(),
        meta.is_file()
    );

    // 演示递归创建嵌套目录
    let nested = dir.join("a").join("b");
    fs::create_dir_all(&nested)?;
    println!("嵌套目录创建成功: {}", nested.exists());

    Ok(dir)
}

/// 演示 `std::io`：`Write`、`BufRead::lines`、以及标准输入的正确姿势。
///
/// 为什么需要 `BufReader`？
/// 每次 `read` 都是一次系统调用；逐字节读文件会比缓冲读慢几十倍。
/// `BufReader` 解决的就是这个"小步读大量数据"的性能问题。
/// ⚠️ 常见坑: 参数写 `&PathBuf` 而不是 `&Path` 会触发 clippy 的 `ptr_arg`——
/// 和 `&String` vs `&str` 是同一类问题，永远借用"更弱"的那个类型。
fn demo_io(dir: &Path) -> Result<(), Box<dyn Error>> {
    // --- 1. 缓冲区读取：按行迭代 -------------------------------------------
    let path = dir.join("tasks.json");
    let reader = BufReader::new(File::open(&path)?);
    // lines() 迭代出的每一项是 io::Result<String>，所以循环里要 `?` 解包；
    // 它同时会去掉行尾的 \n（以及 Windows 下的 \r\n）。
    let mut line_count = 0;
    let mut last_line = String::new();
    for line in reader.lines() {
        let line = line?;
        line_count += 1;
        last_line = line;
    }
    println!("按行读取: {line_count} 行, 最后一行 = {last_line}");

    // --- 2. 显式写入并 flush ------------------------------------------------
    // stdout 默认是行缓冲（连到终端时），println! 会自动换行刷新；
    // 但用 write! 写入时不带换行，需要手动 flush 才能保证立刻可见。
    let mut stdout = std::io::stdout();
    write!(stdout, "用 write! 写入无换行的内容...")?;
    stdout.flush()?; // ⚠️ 常见坑: 忘记 flush，输出会"卡"在缓冲区里看起来丢了
    println!(" 然后 println! 补一个换行");

    // --- 3. 标准输入的正确姿势（本节只讲解，不实际读取，避免程序阻塞）------
    // 读一行用户输入的最小写法：
    //
    //     let mut input = String::new();
    //     std::io::stdin().read_line(&mut input)?;   // 包含结尾的 '\n'
    //     let input = input.trim();                  // ⚠️ 一定要 trim，否则比较字符串永远失败
    //
    // 需要按行循环处理（例如管道输入）时改用：
    //
    //     for line in std::io::stdin().lock().lines() {
    //         let line = line?;
    //         // ...
    //     }
    //
    // ⚠️ 常见坑: 直接 `read_line().unwrap()`，遇到 EOF（例如 `< /dev/null`）会返回 Ok(0)
    // 而不是错误；如果输入流被关闭也可能得到 Err，交互式程序要区分处理。
    // 另外 stdin() 每次调用都加锁，循环里应先 `let stdin = std::io::stdin();` 再 `stdin.lock()`。
    println!("（stdin 示例见源码注释：read_line + trim + lock() 的用法）");

    Ok(())
}

/// 演示 `std::env`：命令行参数、环境变量、临时目录、当前目录。
///
/// ⚠️ 常见坑:
/// 1. `args()` 的第 0 项是**程序自身的路径**，从第 1 项开始才是用户参数；
///    直接 `args[1]` 会因为越界 panic，请用 `nth(1)` 或先 `collect()`。
/// 2. `var()` 在变量不存在时返回 `Err`，不要 `unwrap()`，用 `unwrap_or` / `unwrap_or_else` 兜底。
/// 3. 不要用 `env::set_var` 修改进程环境变量：新版本 Rust 已将其标记为不安全
///    （多线程下修改环境变量是数据竞争），本章因此不演示它。
fn demo_env() -> Result<(), Box<dyn Error>> {
    // collect 成 Vec<String> 方便按索引取；第 0 项是程序路径。
    let args: Vec<String> = env::args().collect();
    println!(
        "参数个数 = {}（args[0] 是程序自身路径，用户参数从 args[1] 开始）",
        args.len()
    );
    // 取第一个用户参数；没有就返回 None，绝不 panic。
    match args.get(1) {
        Some(first) => println!("第一个用户参数 = {first}"),
        None => println!("没有传用户参数（试试: cargo run --bin 16_std_ecosystem -- hi）"),
    }

    // 环境变量：存在就用它，不存在就用默认值。
    let mode = env::var("RUSTLEARN_MODE").unwrap_or_else(|_| "<未设置>".to_string());
    println!("RUSTLEARN_MODE = {mode}");

    // 临时目录：跨平台（Linux 下通常是 /tmp，Windows 下是 %TEMP%）。
    println!("temp_dir 可用 = {}", env::temp_dir().is_dir());

    // 当前目录可能失败（例如目录被删除），所以返回 Result。
    let cwd = env::current_dir()?;
    // file_name() 返回 Option<&OsStr>（根目录时是 None），用 map_or_else 兜底。
    let cwd_name = cwd
        .file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    println!("current_dir 最后一层 = {cwd_name}");

    Ok(())
}

/// 演示 `serde` + `serde_json`：结构体 ←→ JSON 的双向转换。
///
/// ⚠️ 常见坑:
/// 1. `from_str` 会**严格校验类型**：JSON 里 `"id": "1"`（字符串）喂给 `u32` 字段会报错，
///    这是好事（在边界处失败），但错误信息很长，建议用 `map_err` 加上下文。
/// 2. `Value` 索引 `v["key"]` 在 key 不存在时**不会报错**，而是返回 `Value::Null`；
///    拼写错误的 key 会静默变成 null，一定配合 `as_str()` / `as_u64()` 判断。
/// 3. `serde_json::Map` 默认按 key 排序（BTreeMap 语义），所以打印顺序是稳定的。
fn demo_serde_json() -> Result<(), Box<dyn Error>> {
    let task = Task {
        id: 7,
        title: "学会 serde".to_string(),
        done: false,
        tags: vec!["json".to_string(), "serde".to_string()],
    };

    // 紧凑输出（适合传输）与美化输出（适合人看/存配置文件）
    let compact = serde_json::to_string(&task)?;
    let pretty = serde_json::to_string_pretty(&task)?;
    println!("紧凑 JSON = {compact}");
    println!("美化 JSON =\n{pretty}");

    // 反序列化：注意 `let parsed: Task = ...` 让类型推断知道目标类型。
    let parsed: Task = serde_json::from_str(&compact)?;
    // Task 派生了 PartialEq，可以直接比较。
    println!("往返转换后相等 = {}", parsed == task);
    println!("tags[1] = {}", parsed.tags[1]);

    // 动态 JSON（不知道结构、或只需要取其中几个字段时）用 Value。
    let raw = r#"{"name":"rust","version":1,"features":["safe","fast"],"nested":{"level":3}}"#;
    let value: serde_json::Value = serde_json::from_str(raw)?;
    // 路径索引：多层嵌套直接链式取。
    println!(
        "Value 取值: name = {}, version = {}, nested.level = {}",
        value["name"], value["version"], value["nested"]["level"]
    );
    // ⚠️ 拼错的 key 会静默变成 null，而不是报错：
    println!(
        "拼错的 key -> {} (is_null = {})",
        value["naem"],
        value["naem"].is_null()
    );
    // 安全的取值方式：as_* 返回 Option，配合 unwrap_or 给默认值。
    println!(
        "features 数量 = {}",
        value["features"].as_array().map_or(0, Vec::len)
    );

    // json! 宏可以就地构造 Value，适合拼装小对象。
    let hand_made = serde_json::json!({ "ok": true, "id": task.id });
    println!("json! 构造 = {hand_made}");

    Ok(())
}

/// 演示 `chrono`：时间点、时区、时长、格式化与解析。
///
/// ⚠️ 常见坑:
/// 1. **绝不要用本地时间做存储或比较**：`Local` 会随服务器时区/夏令时变化，
///    存库和传输统一用 `Utc`，只在展示给用户时转 `Local`。
/// 2. 时间运算要用 `chrono::Duration`（新版本里叫 `TimeDelta`，是同一类型的别名），
///    不要手写 `+ 86400`，可读性差且闰秒/时区问题会埋雷。
/// 3. 本章这几行输出会随运行时刻变化，所以**不能**像其它小节那样断言具体值；
///    测试里要断言时间请断言"范围"或"格式能被解析回来"。
fn demo_chrono() -> Result<(), Box<dyn Error>> {
    // Utc::now() 拿到带时区信息的时间点（DateTime<Utc>）。
    let now_utc: DateTime<Utc> = Utc::now();
    // 下面两行输出随运行时刻变化（本章唯一的不确定输出）。
    println!("UTC 时间 = {}", now_utc.format("%Y-%m-%dT%H:%M:%SZ"));
    println!("换算成本地时间 = {}", now_utc.with_timezone(&Local));

    // 时区无关的字段取值需要 Datelike / Timelike trait（见文件顶部 use）。
    println!(
        "字段访问: 年 = {}, 月 = {}, 日 = {}",
        now_utc.year(),
        now_utc.month(),
        now_utc.day()
    );

    // 时间运算：加 7 天、减 2 小时。
    let next_week = now_utc + Duration::days(7);
    let two_hours_ago = now_utc - Duration::hours(2);
    println!(
        "now + 7d = {}, now - 2h = {}",
        next_week.format("%Y-%m-%d %H:%M"),
        two_hours_ago.format("%Y-%m-%d %H:%M")
    );
    // 两个时间点相减得到时长，可以取总秒数/天数。
    println!("两者相差 = {} 秒", (next_week - now_utc).num_seconds());

    // 解析固定字符串：用 TimeZone::timestamp_opt 或 RFC 3339 解析。
    // ⚠️ 常见坑: timestamp_opt 越界会返回 None（本地时间可能不存在），必须处理。
    let parsed = Utc.timestamp_opt(1_767_225_600, 0).single();
    println!(
        "从时间戳解析 = {}",
        parsed.map_or_else(|| "<越界>".to_string(), |t| t.to_rfc3339())
    );

    // RFC 3339 是日志/接口里最常见的时间格式，parse_from_rfc3339 一步到位。
    let rfc = DateTime::parse_from_rfc3339("2026-01-02T15:04:05Z")?;
    println!(
        "解析 RFC 3339 = {}（这是确定性输出，可以安全断言）",
        rfc.format("%Y-%m-%d %H:%M:%S %:z")
    );

    Ok(())
}

/// 演示用 `Box<dyn Error>` 把多种错误来源汇合到一个 `Result` 里。
///
/// 解决什么问题：一个函数可能同时遇到 io 错误、解析错误、环境变量错误，
/// 如果为每种都定义枚举会很啰嗦；小工具/示例代码用 `Box<dyn Error>` 最省事。
///
/// 什么时候该用：二进制程序或脚本。
/// ⚠️ 常见坑: 库（lib）不应该用它——库需要让调用方**匹配**具体错误类型，
/// 用类型擦除的 `Box<dyn Error>` 会让调用方无法区分和处理。库请用 `thiserror` 定义错误枚举，
/// 应用层用 `anyhow` 汇总。
fn demo_error_merging() -> Result<usize, Box<dyn Error>> {
    // `?` 会自动把 io::Error / serde_json::Error 通过 From 转换成 Box<dyn Error>。
    let raw = fs::read_to_string(env::temp_dir().join("rustlearn_ch16_does_not_exist.json"))?;
    Ok(raw.len())
}

/// 程序入口：按小节顺序演示本章全部知识点。
///
/// 返回 `Result` 的 `main` 会在返回 `Err` 时打印错误并以非 0 退出码结束；
/// 返回 `Ok(())` 则退出码为 0。
fn main() -> Result<(), Box<dyn Error>> {
    println!("=== 1. String 与 &str ===");
    demo_string_vs_str();

    println!("\n=== 2. format! 与字符串拼接 ===");
    demo_format_and_build();

    println!("\n=== 3. std::fs 文件读写 ===");
    let tmp_dir = demo_fs()?;

    println!("\n=== 4. std::io 读写与缓冲 ===");
    demo_io(&tmp_dir)?;

    println!("\n=== 5. std::env ===");
    demo_env()?;

    println!("\n=== 6. serde + serde_json ===");
    demo_serde_json()?;

    println!("\n=== 7. chrono 日期时间 ===");
    demo_chrono()?;

    println!("\n=== 8. Box<dyn Error> 错误汇合 ===");
    // 这里故意读一个不存在的文件，用 match 演示"就地处理错误"而不是直接 panic。
    match demo_error_merging() {
        Ok(n) => println!("意外读到了文件（{n} 字节）"),
        Err(e) => println!("预期内的错误已被优雅处理: {e}"),
    }

    // 清理本章在临时目录创建的目录，避免留下垃圾文件。
    // ⚠️ 常见坑: 清理失败不该让程序崩溃，所以这里刻意忽略结果并只提示存在性。
    fs::remove_dir_all(&tmp_dir)?;
    println!("\n临时目录已清理 = {}", !tmp_dir.exists());

    println!("\n第 16 章演示结束。");
    Ok(())
}
