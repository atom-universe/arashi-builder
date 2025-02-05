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

### 2.5 模式匹配的最佳实践
1. **选择合适的匹配方式**
   - 只关心一种情况：使用 `if let`
   - 需要处理所有情况：使用 `match`
   - 需要提前返回：考虑 `let else`

2. **代码可读性**
   - 模式不要过于复杂
   - 使用适当的缩进和格式
   - 添加必要的注释说明

3. **性能考虑**
   - `match` 是穷尽的，编译器可以优化
   - `if let` 可能更高效（只关心一种情况时）

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
```rust
// 1. 基本使用
fn read_file(path: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}

// 2. ? 运算符
fn process_file(path: &str) -> Result<(), std::io::Error> {
    let content = read_file(path)?;  // 错误自动返回
    println!("{}", content);
    Ok(())
}
```

## 5. 特征（Trait）

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

### 6.2 实际应用（进阶）
```rust
// 在我们的项目中的异步处理
async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
    // 处理请求
    let response = next.run(req).await;
    // 处理响应
    Ok(response)
}
```

## 7. 并发控制（进阶）

### 7.1 锁的使用
```rust
// 1. 同步锁
let lock = std::sync::RwLock::new(data);
let guard = lock.write().unwrap();

// 2. 异步锁（更复杂）
let lock = tokio::sync::RwLock::new(data);
let guard = lock.write().await;
```

## 学习建议

1. **循序渐进**
   - 先掌握基础语法和所有权概念
   - 理解模块系统和错误处理
   - 逐步深入特征和异步编程

2. **实践方式**
   - 从小型功能开始
   - 多看编译器错误提示
   - 参考标准库文档

3. **常见陷阱**
   - 所有权和借用规则
   - 生命周期标注
   - 异步代码的类型约束
