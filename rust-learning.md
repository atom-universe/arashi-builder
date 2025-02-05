# Rust 学习笔记

> 记录实现开发过程中的 Rust 学习心得

## 1. 基础语法特点

### 1.1 变量与所有权
```rust
// 1. 变量默认不可变
let x = 5;
// x = 6;  // 错误！

// 2. 使用 mut 声明可变变量
let mut y = 5;
y = 6;  // 正确

// 3. 所有权转移
let s1 = String::from("hello");
let s2 = s1;
// println!("{}", s1);  // 错误！s1 的所有权已转移给 s2
```

### 1.2 结构体和枚举
```rust
// 1. 结构体定义和使用
#[derive(Debug)]  // 派生 Debug trait，使其可打印
struct Config {
    port: String,
    root: String,
}

// 2. 枚举和模式匹配
enum Command {
    Start { port: String },
    Stop,
}

match command {
    Command::Start { port } => println!("Starting on {}", port),
    Command::Stop => println!("Stopping"),
}
```

## 2. 模式匹配

### 2.1 match 表达式

个人感觉这就是 Rust 严谨（麻烦）的一点，要把每种可能的情况都定义清楚。
> 如果不想讨论每种情况，可以用 if let 表达式

```rust
// 1. 基本匹配
let number = 13;
match number {
    0 => println!("零"),
    1 | 2 => println!("一或二"),
    3..=9 => println!("三到九"),
    _ => println!("其他数字"),
}

// 2. 解构枚举
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
}

match msg {
    Message::Quit => println!("退出"),
    Message::Move { x, y } => println!("移动到 ({}, {})", x, y),
    Message::Write(text) => println!("文本: {}", text),
}

// 3. 带条件的匹配
match number {
    n if n < 0 => println!("负数"),
    n if n > 0 => println!("正数"),
    _ => println!("零"),
}
```

### 2.2 if let 语法

语法就是 `if let [枚举] = 表达式 {} else {}`，也就是当某个表达式（不）满足某个枚举条件的时候，执行对应的内容。

```rust
// 1. 基本用法 - 只关心一种模式
let config = Some(3);
if let Some(num) = config {
    println!("配置值是: {}", num);
}

// 2. 替代 match 的写法
// 使用 match
match optional {
    Some(value) => println!("值是: {}", value),
    None => (),
}
// 使用 if let（更简洁）
if let Some(value) = optional {
    println!("值是: {}", value);
}

// 3. 实际项目中的例子
if let Some(pkg_path) = resolve_module_path(&self.root_dir, module_name).await {
    // 处理找到的路径
} else {
    // 处理未找到的情况
}
```

### 2.3 while let 模式
```rust
// 1. 基本用法 - 持续处理 Some 值
let mut stack = Vec::new();
stack.push(1);
stack.push(2);

while let Some(top) = stack.pop() {
    println!("弹出: {}", top);
}

// 2. 在异步代码中使用
while let Some(msg) = channel.recv().await {
    // 处理消息
}
```

### 2.4 let else 模式（Rust 1.65+）
```rust
// 1. 基本用法 - 提前返回
let Some(value) = get_optional() else {
    return None;
};

// 2. 实际应用
let Ok(file) = File::open("config.json") else {
    println!("无法打开配置文件");
    return;
};

// 3. 多个条件组合
let (Some(x), Some(y)) = (get_x(), get_y()) else {
    return Err("坐标无效");
};
```

## 3. 模块系统

### 3.1 基本概念
```rust
// 1. 声明模块 (src/main.rs)
mod cli;      // 查找 src/cli.rs 或 src/cli/mod.rs
mod utils;    // 查找 src/utils.rs 或 src/utils/mod.rs

// 2. 引入模块项
use cli::Config;  // 引入具体项
use utils::*;     // 引入所有公有项
```

### 3.2 可见性规则
```rust
mod app {
    // 1. 默认私有
    fn private_fn() {}
    
    // 2. pub 使项公有
    pub fn public_fn() {}
    
    // 3. pub(crate) 限制可见性范围
    pub(crate) fn crate_visible() {}
}
```

## 4. 错误处理

### 4.1 Result 类型

什么时候会出现 Result 类型? 
个人理解是在IO的时候会出现得比较多。

```rust
// 1. 基本使用
fn read_file(path: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}

// 2. ? 运算符
fn process_file(path: &str) -> Result<(), std::io::Error> {
    // ? 运算符相当于:
    // let content = match read_file(path) {
    //     Ok(content) => content,
    //     Err(e) => return Err(e)
    // };
    // 如果 read_file 返回 Ok，则 content 获得其中的值
    // 如果返回 Err，则直接从当前函数返回该错误
    let content = read_file(path)?;
    println!("{}", content);
    Ok(())
}
```

## 5. 特征（Trait）

差不多相当于 `interface`


### 5.1 基本使用
```rust
// 1. 定义特征
trait Handler {
    fn handle(&self, request: Request) -> Response;
}

// 2. 实现特征
impl Handler for MyHandler {
    fn handle(&self, request: Request) -> Response {
        // 处理请求
    }
}
```

## 6. 异步编程

### 6.1 基础概念

语法上和 JS 类似

```rust
// 1. async 函数
async fn fetch_data() -> Result<String, Error> {
    // 异步操作
}

// 2. .await 等待
async fn process() {
    let data = fetch_data().await?;
}
```


## 7. 并发控制

### 7.1 锁与并发

由于做前端开发更多，所以其实对多线程并不是很熟悉。。。

1. **为什么需要锁？**
```rust
// 一个共享的计数器
// Mutex是互斥锁，Arc是跨线程的智能指针
let counter = Arc::new(Mutex::new(0));
let counter_clone = counter.clone();

// 在另一个线程中修改它
std::thread::spawn(move || {
    *counter_clone.lock().unwrap() += 1;
});
```

2. **同步锁 vs 异步锁**
```rust
// 同步锁会阻塞线程
let lock = std::sync::RwLock::new(data);
// 调用 write 方法的时候，会获得一个【写锁】，有写锁的线程可以修改变量
let guard = lock.write().unwrap();  // 线程会在这里等待
// 其他线程完全停止，直到获得锁

// 异步锁不会阻塞线程
let lock = tokio::sync::RwLock::new(data);
let guard = lock.write().await;  // 线程可以去做其他事
// 当前任务暂停，线程可以执行其他任务
```

3. **await vs RwLock 的区别**

对于异步锁这个概念，说实话我感觉有些困惑，因为根据我的 JS 经验来看，await 就已经在发挥锁的能力了。
> 个人理解：锁是针对于线程的，await 是针对任务的

```rust
// await 是任务级别的暂停（任务，就 JS 的经验来看，当成函数调用看就行了）
async fn process_data() {
    let result = fetch_data().await;  // 暂停当前任务，等待 IO
    // 这里的暂停不影响其他任务使用 result
}

// RwLock 是跨任务的数据共享
async fn handle_request(shared_cache: Arc<RwLock<Cache>>) {
    let cache = shared_cache.write().await;  // 确保多个任务不会同时写入
    cache.insert(key, value);
    // 其他任务必须等待写操作完成
}
```

### 7.2 实际应用场景

两个请求并发过来，读取内存数据，这时候读的都是50，然后一个执行+1，另一个执行-1，分别得到51和49，
再写到内存中，这时候问题就来了，数据对不上了！

```rust
// 在我们的项目中：依赖构建的并发控制
pub struct DepCache {
    cache_dir: PathBuf,
    building: HashSet<String>,  // 正在构建的包集合
    metadata: HashMap<String, String>,  // 缓存元数据
}

impl DepCache {
    async fn get_or_build(&mut self, pkg_name: &str) -> Result<PathBuf> {
        // 1. 多个请求可能同时要求构建同一个包
        // 2. await 只能保证当前任务的异步等待
        // 3. RwLock 确保不会重复构建同一个包
        if self.building.contains(pkg_name) {
            // 等待其他任务完成构建
            while self.building.contains(pkg_name) {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
    }
}
```
