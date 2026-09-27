//! 第 09 章：集合类型（collections）
//!
//! 本章覆盖最常用的集合：`Vec<T>`、`HashMap`/`BTreeMap`、`HashSet`、`VecDeque`。
//! 它们共享同一套心智模型：**长度（len）与容量（capacity）分离**、**插入/删除会移动元素**、
//! **迭代会借用整个集合**。理解这三点，就能预判何时发生重新分配（reallocation），
//! 以及为什么「边遍历边修改」会被 borrow checker 拦下。
//!
//! 运行：`cargo run --bin 09_collections`
//!
//! 关键知识点：
//! - `Vec` 常用操作与 `with_capacity`
//! - `HashMap` 与 `entry` API
//! - `BTreeMap` 有序遍历与 `range`
//! - `HashSet` 去重与集合运算
//! - `VecDeque`
//! - 容量与扩容机制
//! - 迭代时修改集合的借用冲突坑

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

/// 把一批整数排序后返回：`HashMap`/`HashSet` 顺序不确定，本章打印前一律排序以保证可复现。
fn sorted<I: IntoIterator<Item = i32>>(xs: I) -> Vec<i32> {
    let mut out: Vec<i32> = xs.into_iter().collect();
    out.sort_unstable();
    out
}

/// 演示 `Vec<T>` 常用操作：增删改查与排序去重。
///
/// `Vec` 是堆上的可增长数组：下标访问 O(1)、尾部 `push`/`pop` 摊还 O(1)，
/// 但中间 `insert`/`remove` 要搬移其后元素（O(n)），频繁中间插入就别选它。
fn demo_vec_ops() {
    // with_capacity 预分配：已知规模时一次把内存要够，避免反复「扩容 + 搬移」。
    // ⚠️ 常见坑: 别凭感觉写一个很大的容量；不确定时用 Vec::new() 让它在增长中调整。
    let mut v: Vec<i32> = Vec::with_capacity(8);
    v.push(30);
    v.push(10);
    v.extend([40, 10, 50]); // extend 接受任意 IntoIterator
    v.insert(0, 20); // 在下标 0 处插入，后面元素整体后移
    let first = v.remove(0); // remove 返回值且是 O(n)；pop 从尾部拿是 O(1)
    println!("remove(0) 拿走 {first}，剩下 {v:?}");

    v.sort_unstable(); // 更快且不额外分配，但不保证相等元素的原有顺序
                       // ⚠️ 常见坑: dedup 只删**相邻**重复项，不先排序就删不干净。
    v.dedup();
    println!("排序去重后: {v:?}，contains(40) = {}", v.contains(&40));
}

/// 演示容量（capacity）与扩容（reallocation）机制。
///
/// `len()` 是元素个数，`capacity()` 是不重新分配就能装下的个数。二者相等时再 `push`，
/// `Vec` 会申请更大内存、把旧元素**移动（move）** 过去并释放旧内存；单次 push 偶尔是
/// O(n)，摊还仍是 O(1)。扩容倍率是实现细节（当前标准库近似翻倍）。
fn demo_capacity_growth() {
    let mut v: Vec<i32> = Vec::with_capacity(4);
    println!("初始 len={} capacity={}", v.len(), v.capacity());

    let mut last = v.capacity();
    let mut growth = Vec::new();
    for i in 0..64 {
        v.push(i);
        if v.capacity() != last {
            growth.push((v.len(), v.capacity())); // 记录 (元素个数, 新容量)
            last = v.capacity();
        }
    }
    println!("扩容点 (len, capacity) = {growth:?}");

    // truncate 只改长度、不归还容量；确定不再增长时用 shrink_to_fit 还内存。
    v.truncate(10);
    v.shrink_to_fit();
    println!(
        "truncate(10)+shrink_to_fit 后 len={} capacity={}",
        v.len(),
        v.capacity()
    );
}

/// 演示 `HashMap` 与 `entry` API。
///
/// `HashMap` 基于哈希表，平均 O(1) 查找但迭代顺序不确定。`entry` API 的价值：
/// **一次**查找完成「不存在则插入、存在则更新」，避免 `contains_key` + `insert`
/// 的两次哈希（clippy 也会为此报 `map_entry`）。
fn demo_hashmap_entry() {
    let words = ["apple", "banana", "avocado", "blueberry", "apple"];

    let mut counts: HashMap<String, u32> = HashMap::new();
    for w in words {
        // or_insert 在键缺失时插入默认值，并返回 &mut V，可直接累加。
        *counts.entry(w.to_string()).or_insert(0) += 1;
    }
    // HashMap 迭代顺序每次运行都可能不同：要稳定输出就排序，或改用 BTreeMap。
    let mut pairs: Vec<(String, u32)> = counts.into_iter().collect();
    pairs.sort_unstable();
    println!("词频（排序后）: {pairs:?}");

    // and_modify + or_insert_with：分别描述「已存在」和「不存在」两条路径。
    let mut grouped: HashMap<char, String> = HashMap::new();
    for w in words {
        let head = w.chars().next().unwrap_or('?');
        grouped
            .entry(head)
            .and_modify(|s| s.push_str(&format!("、{w}")))
            .or_insert_with(|| w.to_string());
    }
    let mut heads: Vec<(char, String)> = grouped.into_iter().collect();
    heads.sort_unstable();
    println!("按首字母分组（排序后）: {heads:?}");
}

/// 演示 `BTreeMap` 有序遍历与 `range` 区间查询。
///
/// `BTreeMap` 用 B 树实现，键**始终有序**：适合范围查询与按序输出；代价是查找/插入
/// 为 O(log n)，而非哈希表的平均 O(1)。这里用成绩表举例。
fn demo_btreemap_range() {
    let mut scores: BTreeMap<u32, &str> = BTreeMap::new();
    scores.insert(92, "carol");
    scores.insert(85, "alice");
    scores.insert(78, "bob");
    scores.insert(96, "dave");

    // 迭代天然按键升序，不需要额外排序 —— 这是它相对 HashMap 的卖点。
    let roster: Vec<(u32, &str)> = scores.iter().map(|(k, v)| (*k, *v)).collect();
    println!("按分数升序: {roster:?}");

    // range 先 O(log n) 定位边界再顺序扫描，比全表 filter 高效。
    // `80..=95` 是闭区间，也可以写 `80..95`（右开）或 `80..`（到末尾）。
    let passing: Vec<(u32, &str)> = scores.range(80..=95).map(|(k, v)| (*k, *v)).collect();
    println!(
        "80..=95 区间: {passing:?}，最低分 = {:?}",
        scores.first_key_value()
    );
}

/// 演示 `HashSet` 去重与集合运算。
///
/// `HashSet` 就是「只有键、没有值的 `HashMap`」，用于去重与成员判断。并集/交集/差集/
/// 对称差返回的都是**惰性迭代器**，不复制数据，结果类型由你 `collect` 决定。
fn demo_hashset() {
    let a: HashSet<i32> = [1, 2, 2, 3, 4].into_iter().collect(); // 重复项自动丢弃
    let b: HashSet<i32> = [3, 4, 5, 6].into_iter().collect();
    println!("a 去重后 = {:?}", sorted(a.iter().copied()));

    // 集合运算产出 `&i32`，`.copied()` 把它变成 `i32` 再交给 sorted。
    println!("并集 = {:?}", sorted(a.union(&b).copied()));
    println!("交集 = {:?}", sorted(a.intersection(&b).copied()));
    println!("差集 a-b = {:?}", sorted(a.difference(&b).copied()));
    println!("对称差 = {:?}", sorted(a.symmetric_difference(&b).copied()));

    // insert 返回 bool：true 表示这次真的插进去了，可用来判定「第一次见到」。
    let mut seen = HashSet::new();
    println!(
        "insert(7)={} 再 insert(7)={}",
        seen.insert(7),
        seen.insert(7)
    );

    let subset: HashSet<i32> = [3, 4].into_iter().collect();
    let far: HashSet<i32> = [9, 10].into_iter().collect();
    println!(
        "contains(3)={} 子集={} 不相交={}",
        a.contains(&3),
        subset.is_subset(&a),
        a.is_disjoint(&far)
    );
}

/// 演示 `VecDeque<T>`（double-ended queue，双端队列）。
///
/// `Vec` 只在尾部高效；`VecDeque` 用环形缓冲区（ring buffer）让**两端** push/pop 都是
/// 摊还 O(1)，适合队列（FIFO）。代价是按下标访问要绕环计算、内存不保证连续。
fn demo_vecdeque() {
    let mut queue: VecDeque<&str> = VecDeque::new();
    queue.push_back("任务1");
    queue.push_back("任务2");
    queue.push_front("插队任务"); // Vec 做不到 O(1) 头部插入
    println!("队列: {queue:?}");

    // push_back + pop_front = 队列（FIFO）；push_back + pop_back = 栈（LIFO）。
    if let Some(front) = queue.pop_front() {
        println!("先处理（FIFO）: {front}");
    }
    queue.push_back("任务3");
    queue.rotate_left(1); // 环形缓冲区天生擅长旋转
    println!("rotate_left(1) 后: {queue:?}");
}

/// 演示「迭代时修改集合」的借用冲突（borrow conflict）及三种绕开方式。
///
/// 规则：同一时刻对同一个值，要么任意多个 `&`，要么至多一个 `&mut`，不能共存。
/// 迭代器在整个循环期间持有集合的借用，所以循环体里改集合必然冲突，只能换写法。
fn demo_borrow_conflict() {
    let mut nums = vec![1, 2, 3, 4, 5];

    // ❌ 下面这段看着合理却编译不过（E0502），可以取消注释体会一下：
    // for n in &nums {
    //     // for 的迭代器在整个循环期间持有 nums 的不可变借用，而 push 需要可变借用。
    //     if *n % 2 == 0 {
    //         nums.push(*n * 10);
    //     }
    // }

    // ✅ 方式一：把「要追加的元素」先收集到新集合，循环结束后一次性写回。
    let extra: Vec<i32> = nums
        .iter()
        .filter(|n| **n % 2 == 0)
        .map(|n| n * 10)
        .collect();
    nums.extend(extra);

    // ✅ 方式二：只改值、不增删时，用 iter_mut() 拿 `&mut` 元素就地修改。
    for n in nums.iter_mut() {
        if *n > 30 {
            *n = 30; // 把刚追加进来的 40 压回上限
        }
    }
    println!("iter_mut 封顶后 = {nums:?}");

    // ✅ 方式三：必须按下标删除时，用 while 手动控制下标。
    // ⚠️ 常见坑: `for i in 0..nums.len() { nums.remove(i) }` 会因元素前移而漏删一半；
    // remove 成功之后**不要**递增下标。
    let mut i = 0;
    while i < nums.len() {
        if nums[i] == 30 {
            nums.remove(i);
        } else {
            i += 1;
        }
    }

    // 最省心的「按条件删除」是 retain：一次遍历，内部就是可变借用。
    nums.retain(|n| *n != 10);
    println!("最终 nums = {nums:?}");
}

/// 程序入口：按小节顺序演示本章所有知识点。
fn main() {
    println!("=== 1. Vec 常用操作 ===");
    demo_vec_ops();
    println!("\n=== 2. 容量与扩容机制 ===");
    demo_capacity_growth();
    println!("\n=== 3. HashMap 与 entry API ===");
    demo_hashmap_entry();
    println!("\n=== 4. BTreeMap 有序遍历与 range ===");
    demo_btreemap_range();
    println!("\n=== 5. HashSet 去重与集合运算 ===");
    demo_hashset();
    println!("\n=== 6. VecDeque 双端队列 ===");
    demo_vecdeque();
    println!("\n=== 7. 迭代时修改集合的借用冲突 ===");
    demo_borrow_conflict();
}
