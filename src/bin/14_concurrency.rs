//! 第 14 章：并发编程
//!
//! 并发的难点不是"怎么开线程"，而是"**怎么在编译期排除数据竞争**"：Rust 用 `Send`/`Sync` 两条
//! auto trait + 所有权规则，把数据竞争（data race）变成编译错误。本文件输出**完全确定**：线程只
//! 做计算并返回值，由 `main` join 后按固定顺序打印；channel 的消息先收集排序再输出。
//!
//! 运行：`cargo run --bin 14_concurrency`
//!
//! 关键知识点：`spawn`/`join`、`move` 闭包跨线程传数据、mpsc 多生产者、`recv`（阻塞）与
//! `try_recv`（非阻塞）、`Mutex` 与锁中毒、`Arc<Mutex<T>>`、`RwLock`、原子类型（`AtomicUsize`）、
//! `Send`/`Sync`、`thread::scope` 借用栈上数据、死锁（deadlock）坑。

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

/// 演示 spawn/join 的线程数量，写成常量便于测试与解释。
const WORKERS: usize = 3;

/// 第 1 节：`thread::spawn` + `join`。把互相独立的计算分到多个 OS 线程（1:1 模型，不是绿色线程），
/// 用 `join` 收集结果并同步。
/// ⚠️ 常见坑: `spawn` 返回 `JoinHandle<T>`，**忘了 join 也不会报错**，主线程一退出进程就结束、
/// 子线程被直接杀掉；`join` 返回 `Result`，子线程 panic 时是 `Err`（panic 被隔离在该线程内）。
fn demo_spawn_join() -> Vec<u64> {
    // 这里不用 `for` 手动 push 句柄：`spawn` 顺序就是后面的 join 顺序，结果因此固定。
    (1..=WORKERS as u32)
        .map(|n| thread::spawn(move || u64::from(n) * u64::from(n)))
        .map(|h| h.join().expect("子线程 panic 了"))
        .collect()
}

/// 第 2 节：`move` 闭包跨线程传数据。新线程可能比创建它的作用域活得更久，借用栈上数据不安全；
/// `move` 把所有权（ownership）交给闭包，由该线程负责生命周期。
/// ⚠️ 常见坑: 忘写 `move` 会得到 `closure may outlive the current function`；也**不能**把
/// `Rc<T>` 传进线程（它不是 `Send`），要用 `Arc<T>`。
fn demo_move_closure() -> (usize, usize) {
    let payload = String::from("move 把这块堆内存交给子线程");
    let tags: Vec<u32> = vec![10, 20, 30];
    // 不加 move：payload/tags 是借用，编译器会拒绝；加了 move：所有权转移进闭包，闭包结束即
    // 释放，绝不悬垂。join 之后主线程再用 payload 会报 "used after move"。
    let handle = thread::spawn(move || (payload.len(), tags.iter().sum::<u32>() as usize));
    handle.join().expect("子线程 panic 了")
}

/// 第 3 节：mpsc channel —— 多生产者、单消费者。这是"消息传递（message passing）"式共享：线程
/// 之间不共享内存，只靠消息的所有权转移通信（mpsc = multi-producer, single-consumer）。返回按
/// `(生产者名, 数值)` 排序后的结果，输出与线程调度无关。
/// ⚠️ 常见坑: 生产者 `Sender` 未 drop 时 `recv()` 会一直阻塞等新消息；用 `for msg in rx`
/// 结束循环的条件是**所有** Sender 都被丢弃。
fn demo_channel_multi_producer() -> Vec<(String, u32)> {
    // `mpsc::channel()` 返回 (tx, rx)；clone 出来的 tx 就是"新生产者"。
    let (tx, rx) = mpsc::channel::<(String, u32)>();
    let mut handles = Vec::new();
    for (index, name) in ["alpha", "beta", "gamma"].iter().enumerate() {
        let tx_clone = tx.clone();
        let owned_name = (*name).to_string();
        // send 会移动消息所有权：发送后本线程不能再使用它。
        handles.push(thread::spawn(move || {
            let msg = (owned_name, (index as u32 + 2).pow(2));
            tx_clone.send(msg).expect("接收端已关闭");
        }));
    }
    for handle in handles {
        handle.join().expect("生产者线程 panic 了");
    }
    // 原始 tx 必须 drop，否则 rx 收不到"所有生产者已退出"的信号。
    drop(tx);
    // 收 3 条：到达顺序由调度决定，先收集、最后排序再返回（迭代器在通道空时结束）。
    let mut received: Vec<(String, u32)> = rx.iter().collect();
    received.sort();
    received
}

/// 第 4 节：`recv`（阻塞）与 `try_recv`（非阻塞）：`recv` 没消息就挂起当前线程，直到有数据或所有
/// 发送端关闭（返回 `Err`）；`try_recv` 立刻返回 `Ok` / `Err(TryRecvError::Empty)`（暂时没消息）
/// / `Err(TryRecvError::Disconnected)`（发送端全退出），适合"手头还有别的事，顺便看一眼"的轮询。
fn demo_try_recv() -> Vec<u32> {
    let (tx, rx) = mpsc::channel::<u32>();
    for value in [3, 1, 2] {
        tx.send(value).expect("接收端已关闭");
    }
    let mut drained = Vec::new();
    // `while let` 只处理 Ok；Empty（暂时没消息）与 Disconnected（发送端全部退出）都结束循环。
    while let Ok(value) = rx.try_recv() {
        drained.push(value);
    }
    // 同一线程内 channel 是 FIFO，所以这里保留发送顺序 3,1,2（确定）。
    drained
}

/// 第 5 节：`Mutex<T>` 与锁中毒（poisoning）：数据被**包在锁里**，不拿到锁就拿不到 `&mut T`；
/// `lock()` 返回 `Result` 正是为了表达"上一个持锁线程 panic 过"。本函数故意制造一次中毒。
/// ⚠️ 常见坑: 中毒后 `lock()`/`try_lock()` 对后来者都返回 `Err`，因为锁无法判断那块数据是否已被
/// 写坏。恢复手段：`PoisonError::into_inner()` 取回数据、`Mutex::get_mut()`（需要 `&mut Mutex`，
/// `Arc<Mutex<T>>` 拿不到）或换新的 `Mutex::new(v)`；**别盲目 `lock().unwrap()`**，那会把一次
/// panic 放大成"后来线程连环 panic"。`clear_poison()`(1.77) 超出本工程 MSRV 1.75，不能用。
fn demo_mutex_poisoning() -> (i32, bool) {
    // 子线程马上会 panic。默认 panic hook 会往 stderr 打一行
    // "thread '<unnamed>' (333) panicked at src/bin/14_concurrency.rs:...:",
    // 其中线程编号每次运行都不同（不确定输出），也容易让读者误以为程序失败了。
    // 所以这里临时换成静默 hook，演示完立刻还原——这是"故意触发 panic"的通用做法。
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));

    // Arc 提供跨线程共享；没有它，Mutex 会被 move 进 spawn 的闭包，主线程就再也拿不到了。
    let shared = Arc::new(Mutex::new(0_i32));
    let writer = Arc::clone(&shared);
    let handle = thread::spawn(move || {
        // 先正常改一次，再 panic：锁因此被标记为 poisoned。
        let mut guard = writer.lock().expect("首次加锁不应失败");
        *guard += 1;
        panic!("故意 panic：让锁中毒");
    });
    assert!(handle.join().is_err(), "子线程应当 panic");
    // 承认中毒事实并用 `into_inner()` 取回数据；真实代码要先判断这份数据是否可信（这里确定是 1）。
    let value = match shared.lock() {
        Ok(guard) => *guard,
        Err(err) => *err.into_inner(),
    };
    // 再试一次：中毒标记不会自动清除，所以这次同样失败；这就是"必须显式处理"的证据。
    let still_poisoned = shared.try_lock().is_err();

    // 还原默认 panic hook，后面的代码（其它小节）行为不受影响。
    std::panic::set_hook(previous_hook);
    // 返回 (子线程写入的值, 后续加锁是否持续失败)：1 与 true 都是确定的。
    (value, still_poisoned)
}

/// 第 6 节：`Arc<Mutex<T>>` 让多个线程共享并修改同一份数据。`Arc`（atomically reference counted）
/// 提供**跨线程**共享所有权，`Mutex` 提供互斥访问，两者组合是标准库给出的标准答案。
/// ⚠️ 常见坑: `Rc<Mutex<T>>` 编译不过，因为 `Rc` 的引用计数不是原子的（不是 `Send`/`Sync`）；
/// `Arc::clone` 只加引用计数，不深拷贝里面的数据。
fn demo_arc_mutex() -> i32 {
    let counter = Arc::new(Mutex::new(0_i32));
    let mut handles = Vec::new();
    for _ in 0..4 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                // 临界区（critical section）尽量短：这里只做一次加法。
                *counter.lock().expect("锁未中毒") += 1;
            }
        }));
    }
    for handle in handles {
        handle.join().expect("worker panic 了");
    }
    // 所有写线程都 join 完了，这里必然是 400，输出确定。
    // ⚠️ 必须绑成局部变量：块尾直接写 `*counter.lock()...` 时 MutexGuard 临时值会比 counter
    // 更晚 drop，借用检查器报 E0597，绑给 let 就能让 guard 在这一行就释放。
    let total = *counter.lock().expect("锁未中毒");
    total
}

/// 第 7 节：`RwLock` 适合读多写少：多个读者可同时持锁（`&T`），写者独占（`&mut T`）；原子类型
/// 提供无锁（lock-free）的单变量操作。
/// ⚠️ 常见坑: 原子操作依赖内存序（memory ordering）：`Relaxed` 只保证该变量自身原子，不建立与
/// 其他变量的同步关系，拿不准时先用最严格的 `SeqCst`（顺序一致）。
fn demo_rwlock_and_atomic() -> (i32, bool, usize) {
    // 读写锁：先让写线程改一次，再让两个读线程并发读。write() 会阻塞到拿到独占锁为止。
    let config = Arc::new(RwLock::new(7_i32));
    let writer = Arc::clone(&config);
    // 写线程改完就退出，主线程稍后再自己读一次来验证结果。
    thread::spawn(move || *writer.write().expect("写锁未中毒") += 5)
        .join()
        .expect("写线程 panic 了");
    let mut readers = Vec::new();
    for _ in 0..2 {
        let config = Arc::clone(&config);
        // read() 允许多个线程同时进入，因此两次读不会互相阻塞。
        readers.push(thread::spawn(move || *config.read().expect("读锁未中毒")));
    }
    let both_ok = readers
        .into_iter()
        .all(|h| h.join().expect("读线程 panic 了") == 12);
    // 写线程已 join，锁已释放，这里再读一次不会阻塞；顺带确认写操作真的生效。
    let written = *config.read().expect("读锁未中毒");
    // AtomicUsize 无锁累加；fetch_add 返回**旧值**（最容易记错的地方）。
    let hits = Arc::new(AtomicUsize::new(0));
    let mut workers = Vec::new();
    for _ in 0..4 {
        let hits = Arc::clone(&hits);
        workers.push(thread::spawn(move || {
            for _ in 0..10 {
                // SeqCst 是最严格的内存序：教学与正确性优先时选它。
                hits.fetch_add(1, Ordering::SeqCst);
            }
        }));
    }
    for worker in workers {
        worker.join().expect("worker panic 了");
    }
    (written, both_ok, hits.load(Ordering::SeqCst))
}

/// 第 8 节：`Send`（值可安全移动到别的线程）与 `Sync`（`&T` 可安全共享给别的线程）都是 auto
/// trait，由编译器自动推导；`assert_send_sync` 把这条约束放到编译期检查。
/// ⚠️ 常见坑: `Rc<T>`/`RefCell<T>` 是 `!Send`，`Cell<T>`/裸指针是 `!Sync`；跨线程使用时报错
/// 形式是 "trait bound not satisfied"，而不是运行时数据竞争 —— 这正是 Rust 的承诺。
fn demo_send_sync() {
    assert_send_sync::<Mutex<i32>>();
    assert_send_sync::<Arc<String>>();
    // 结构体里只要含 `Rc<T>` 字段，整个结构体会自动变成 !Send，无法送进线程。
    println!("Mutex<i32> 与 Arc<String> 都满足 Send + Sync；Rc/RefCell 则不是");
}

/// 编译期断言：只有同时满足 `Send + Sync` 的类型才能作为参数传入。
fn assert_send_sync<T: Send + Sync>() {}

/// 第 9 节：`thread::scope`（Rust 1.63）借用栈上数据。不必为"借一点数据"就 `Arc` + `move` 转移
/// 所有权：scope 保证所有线程在最外层闭包结束前被 join，因此编译器允许它们借用栈变量。
/// ⚠️ 常见坑: scope 结束会自动 join，但**返回值必须显式收集**，否则拿不到线程结果。
fn demo_thread_scope(data: &[u32]) -> BTreeMap<&'static str, u32> {
    let mut stats = BTreeMap::new();
    thread::scope(|scope| {
        // 直接借用外层 data，无需 Arc、无需 move。
        let sum = scope.spawn(|| data.iter().sum::<u32>());
        let max = scope.spawn(|| data.iter().copied().max().unwrap_or(0));
        stats.insert("sum", sum.join().expect("求和线程 panic 了"));
        stats.insert("max", max.join().expect("求最大值线程 panic 了"));
    });
    stats
}

/// 第 10 节：死锁（deadlock）与全局加锁顺序。经典成因是两个线程以**相反顺序**获取同一组锁
/// （ABBA）；标准库没有死锁检测，一旦发生就是永久挂起，只能靠约定与设计避免。正确做法是让所有
/// 线程按 "first -> second" 的同一全局顺序加锁，就无法形成循环等待。
/// ⚠️ 常见坑: 1) 嵌套加锁顺序不统一；2) 忘记 drop guard 就再锁同一把锁（`std::sync::Mutex`
/// 不可重入，会直接死锁）；3) 临界区里做耗时 IO。应急手段：`try_lock()` 失败就放弃重试。
fn demo_deadlock_avoidance() -> (i32, i32) {
    let first = Arc::new(Mutex::new(0_i32));
    let second = Arc::new(Mutex::new(0_i32));
    let mut handles = Vec::new();
    for _ in 0..2 {
        // 两个线程拿到同一对锁的两组句柄。
        let first = Arc::clone(&first);
        let second = Arc::clone(&second);
        handles.push(thread::spawn(move || {
            // 全局加锁顺序：永远先 first 再 second；guard 在语句结束时就 drop。
            *first.lock().expect("锁未中毒") += 1;
            *second.lock().expect("锁未中毒") += 10;
        }));
    }
    for handle in handles {
        handle.join().expect("worker panic 了");
    }
    // ⚠️ 同样先绑定再返回：MutexGuard 临时值若留在块尾，会比 first/second 更晚 drop（E0597）。
    let first_value = *first.lock().expect("锁未中毒");
    let second_value = *second.lock().expect("锁未中毒");
    (first_value, second_value)
}

/// 程序入口：按小节顺序演示本章所有知识点。
fn main() {
    println!("=== 1. thread::spawn 与 join ===");
    println!(
        "1..={WORKERS} 的平方(按 join 顺序): {:?}",
        demo_spawn_join()
    );
    println!("\n=== 2. move 闭包跨线程传数据 ===");
    let (text_len, sum) = demo_move_closure();
    println!("子线程算出的 (字符串长度, 数组和) = ({text_len}, {sum})");
    println!("\n=== 3. mpsc channel 与多生产者 ===");
    // 收到的一批消息先排序再打印，输出与线程调度无关。
    for (name, value) in demo_channel_multi_producer() {
        println!("生产者 {name} 发送 {value}");
    }
    println!("\n=== 4. recv 与 try_recv ===");
    println!("try_recv 排空的顺序: {:?}", demo_try_recv());
    println!("\n=== 5. Mutex 与锁中毒 ===");
    // 子线程会 panic（panic hook 已在演示内部临时静默），我们 join 后按 Err 处理，退出码仍为 0。
    let (value, still_poisoned) = demo_mutex_poisoning();
    println!("panic 前写入的值 = {value}，中毒后 lock()/try_lock() 持续失败 = {still_poisoned}");
    println!("\n=== 6. Arc<Mutex<T>> 共享状态 ===");
    println!("4 线程 x 100 次自增的最终值 = {}", demo_arc_mutex());
    println!("\n=== 7. RwLock 与原子类型 ===");
    let (written, both_ok, total) = demo_rwlock_and_atomic();
    println!("写锁后的值 = {written}，两个读线程都读到 12 = {both_ok}，原子累加 = {total}");
    println!("\n=== 8. Send 与 Sync 的含义 ===");
    demo_send_sync();
    println!("\n=== 9. thread::scope 借用栈上数据 ===");
    let data = [4_u32, 8, 15, 16, 23, 42];
    // BTreeMap 迭代顺序由 key 排序决定，因此输出稳定（HashMap 不行）。
    for (metric, value) in demo_thread_scope(&data) {
        println!("{metric} = {value}");
    }
    println!("\n=== 10. 死锁与全局加锁顺序 ===");
    let (first, second) = demo_deadlock_avoidance();
    println!("按 first->second 顺序加锁: first = {first}, second = {second}");
}
