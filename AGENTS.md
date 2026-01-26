# Agent Development Guide

This document provides coding guidelines and conventions for AI agents and developers working on the Sparkle browser automation library.

## Project Overview

**Sparkle** is a Rust reimplementation of **Playwright's official Python API**, powered by thirtyfour (WebDriver) and Tokio for async operations.

### Goals
- **API Compatibility**: Maintain API compatibility with Playwright Python to ease migration and leverage existing knowledge
- **Async Runtime**: Built on Tokio for high-performance async I/O operations
- **Type Safety**: Leverage Rust's type system for compile-time safety while keeping the API ergonomic
- **WebDriver Backend**: Uses thirtyfour to communicate with WebDriver (ChromeDriver, GeckoDriver, etc.)

### Architecture
```
┌─────────────────────────────────────┐
│   Playwright Python API (Target)   │
└──────────────┬──────────────────────┘
               │ API-compatible
┌──────────────▼──────────────────────┐
│   Sparkle (Rust async_api)          │ ← You are here
│   - Browser, Page, Locator          │
│   - Built with Tokio async/await    │
└──────────────┬──────────────────────┘
               │ Adapter layer
┌──────────────▼──────────────────────┐
│   WebDriver Adapter (driver/)       │
│   - Wraps thirtyfour                │
└──────────────┬──────────────────────┘
               │ WebDriver protocol
┌──────────────▼──────────────────────┐
│   ChromeDriver / WebDriver          │
└─────────────────────────────────────┘
```

### Design Principles
1. **Match Playwright Python's API** - Method names, parameters, and behavior should mirror the Python implementation
2. **Async by default** - All I/O operations use Tokio async/await (not blocking)
3. **Builder pattern for options** - Complex options use derive_builder for ergonomic configuration
4. **Explicit error handling** - Use Result<T> and custom Error types, never panic in library code
5. **Auto-waiting and retry** - Locators automatically wait and retry like Playwright (not fully implemented yet)

## Build, Lint, and Test Commands

### Building
```bash
# Build library and binary
cargo build

# Build release version
cargo build --release

# Build examples
cargo build --examples

# Build specific example
cargo build --example basic_navigation
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests in specific module
cargo test module_name::

# Run async tests (using tokio-test)
cargo test --features tokio-test

# Run examples (integration testing)
cargo run --example basic_navigation
cargo run --example wikipedia_search
cargo run --example locator_demo
```

### Linting
```bash
# Run Clippy (default lints)
cargo clippy

# Clippy with all warnings as errors
cargo clippy -- -D warnings

# Format code (uses default rustfmt)
cargo fmt

# Check formatting without modifying
cargo fmt -- --check
```

### CLI Testing
```bash
# Install Chrome with ChromeDriver
./target/release/sparkle.exe install chrome

# List installed browsers
./target/release/sparkle.exe list

# Uninstall browser
./target/release/sparkle.exe uninstall chrome
```

## Code Style Guidelines

### Import Organization

Always organize imports in three tiers with blank lines between:

```rust
// 1. Standard library imports
use std::sync::Arc;
use std::time::Duration;

// 2. External crate imports
use thirtyfour::prelude::*;
use tokio::sync::RwLock;
use derive_builder::Builder;

// 3. Internal module imports (using crate::)
use crate::core::{Error, Result};
use crate::driver::WebDriverAdapter;
```

### Documentation

**Module-level documentation** (at file start):
```rust
//! Module name and purpose
//!
//! Detailed description of what this module provides.
```

**Item-level documentation** (before functions, structs, etc.):
```rust
/// Brief one-line summary
///
/// Detailed description when needed.
///
/// # Arguments
/// * `param` - Description of parameter
///
/// # Returns
/// Description of return value
///
/// # Example
/// ```no_run
/// # use sparkle::async_api::Page;
/// # async fn example(page: &Page) -> sparkle::core::Result<()> {
/// page.goto("https://example.com", Default::default()).await?;
/// # Ok(())
/// # }
/// ```
pub async fn goto(&self, url: &str) -> Result<()> { ... }
```

**Documentation conventions:**
- Use `# ` prefix for hidden example setup lines
- Use `no_run` attribute for examples that require external resources
- Include examples for all public API methods
- Document error conditions when relevant

### Error Handling

**Always use the custom Result type:**
```rust
use crate::core::{Error, Result};

pub async fn method(&self) -> Result<ReturnType> {
    // Use ? operator for error propagation
    let value = self.some_operation().await?;
    Ok(value)
}
```

**Create domain-specific errors:**
```rust
// Use error helper methods
return Err(Error::element_not_found(selector));
return Err(Error::timeout_duration("operation", timeout));
return Err(Error::not_implemented("feature"));
```

**Convert external errors:**
```rust
// Automatic conversion via #[from] attribute
external_call().await?;

// Manual conversion
external_call().await.map_err(|e| {
    Error::ActionFailed(format!("Failed to click: {}", e))
})?;
```

### Naming Conventions

- **Structs/Enums/Traits**: `PascalCase` (e.g., `Browser`, `LaunchOptions`)
- **Functions/methods**: `snake_case` (e.g., `new_page()`, `goto()`)
- **Variables**: `snake_case` (e.g., `browser_version`, `search_input`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `DEFAULT_TIMEOUT`)
- **Type aliases**: `PascalCase` (e.g., `type Result<T> = ...`)
- **Rust keywords as identifiers**: Use `r#` prefix (e.g., `r#type()`)

### Async Patterns

**All public API methods are async:**
```rust
pub async fn method_name(&self, param: Type) -> Result<ReturnType> {
    let result = async_operation().await?;
    Ok(result)
}
```

**Shared mutable state pattern:**
```rust
pub struct Browser {
    adapter: Arc<WebDriverAdapter>,
    closed: Arc<RwLock<bool>>,
}

pub async fn close(&self) -> Result<()> {
    let mut closed = self.closed.write().await;
    if !*closed {
        *closed = true;
        // cleanup
    }
    Ok(())
}
```

**Timeout and retry pattern:**
```rust
let start = std::time::Instant::now();
loop {
    match self.try_operation().await {
        Ok(result) => return Ok(result),
        Err(_) if start.elapsed() >= timeout => break,
        Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
    }
}
Err(Error::timeout_duration("operation", timeout))
```

### Builder Pattern

**Use derive_builder for options:**
```rust
#[derive(Debug, Clone, Builder, Default)]
#[builder(default, setter(into, strip_option))]
pub struct LaunchOptions {
    pub headless: Option<bool>,
    pub timeout: Option<Duration>,
}

// Usage:
let options = LaunchOptionsBuilder::default()
    .headless(true)
    .timeout(Duration::from_secs(30))
    .build()
    .unwrap();
```

**Manual fluent builders for complex types:**
```rust
impl ChromiumCapabilities {
    pub fn new() -> Self { ... }
    
    pub fn headless(mut self, value: bool) -> Self {
        self.headless = value;
        self
    }
    
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }
}
```

### Type Safety

- **Avoid `.unwrap()`** in library code; use `?` operator instead
- **Use `impl Into<String>`** for string parameters to accept `&str` or `String`
- **Prefer `&str`** for temporary string usage
- **Use `Option<T>`** for optional fields in option structs
- **Use `Arc<T>`** for shared ownership in async contexts

### Chrome DevTools Protocol (CDP)

**CDP instance is stored and reused:**
```rust
// In WebDriverAdapter:
pub struct WebDriverAdapter {
    driver: Arc<RwLock<Option<WebDriver>>>,
    slow_mo: Option<Duration>,
    cdp: Arc<RwLock<Option<ChromeDevTools>>>,  // Stored CDP instance
}

// CDP is created once when WebDriver is initialized
pub fn new(driver: WebDriver) -> Self {
    let cdp = ChromeDevTools::new(driver.handle.clone());
    Self {
        driver: Arc::new(RwLock::new(Some(driver))),
        slow_mo: None,
        cdp: Arc::new(RwLock::new(Some(cdp))),
    }
}

// Access CDP without recreating it
pub async fn execute_cdp(&self, command: &str) -> Result<serde_json::Value> {
    let cdp_guard = self.cdp().await?;
    let dev_tools = cdp_guard.as_ref().ok_or(Error::BrowserClosed)?;
    dev_tools.execute_cdp(command).await
}
```

**Using CDP from Browser API:**
```rust
let browser = playwright.chromium().launch(options).await?;

// Get browser version using CDP (internally uses stored CDP instance)
let version_info = browser.execute_cdp("Browser.getVersion").await?;

// Execute CDP with parameters
let params = json!({"expression": "1 + 1"});
let result = browser.execute_cdp_with_params("Runtime.evaluate", params).await?;
```

**Benefits:**
- No overhead of creating CDP instance for each call
- Consistent handle across the browser's lifetime
- Automatic cleanup when browser closes
- Thread-safe access via RwLock

## Testing Guidelines

**Write inline unit tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_function() {
        let result = sync_function();
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_async_function() {
        let result = async_function().await.unwrap();
        assert!(result.is_valid());
    }
}
```

**Testing location:**
- Inline tests at bottom of source files using `#[cfg(test)]` modules
- Integration tests can go in `tests/` directory (currently unused)
- Examples in `examples/` serve as integration tests

## Module Organization

- **`src/async_api/`** - High-level Playwright-like API (Browser, Page, Locator)
- **`src/driver/`** - WebDriver adapter layer (wraps thirtyfour)
- **`src/core/`** - Core types (Error, Result, Options)
- **`src/cli/`** - CLI commands (install, list, uninstall)
- **`src/bin/`** - Binary entry points

**Module exports:**
```rust
// In mod.rs:
pub mod submodule;
pub use submodule::PublicType;

// In lib.rs prelude:
pub mod prelude {
    pub use crate::async_api::*;
    pub use crate::core::*;
}
```

## Common Patterns

### Visibility
- `pub` - Public API
- `pub(crate)` - Internal to library
- No modifier - Private to module

### Internal constructors
```rust
pub struct Browser {
    adapter: Arc<WebDriverAdapter>,
}

impl Browser {
    // Internal constructor
    pub(crate) fn new(adapter: Arc<WebDriverAdapter>) -> Self {
        Self { adapter }
    }
}
```

### Chrome DevTools Protocol (CDP)
```rust
use thirtyfour::extensions::cdp::ChromeDevTools;

let dev_tools = ChromeDevTools::new(driver.handle.clone());
let result = dev_tools.execute_cdp("Browser.getVersion").await?;
```

## Special Notes

- **No emojis** in code or documentation unless explicitly requested
- **Clean output** - Use `println!` sparingly, prefer tracing for debug info
- **Error messages** should be descriptive and actionable
- **CLI uses `anyhow::Result`**, library uses custom `Result<T>`
- **ChromeDriver auto-launches** - no manual process management needed
- **Playwright compatibility** - API design matches Python Playwright where possible
- **Tokio-based async** - All async operations use Tokio runtime, not other async runtimes

## API Design Philosophy

When implementing new features:

1. **Check Playwright Python documentation first** - The API should match Playwright Python's behavior
2. **Method signatures** - Match Python parameter names and types (translated to Rust idioms)
3. **Return types** - Use `Result<T>` for operations that can fail, matching Playwright's error handling
4. **Options pattern** - Use builder pattern for optional parameters (e.g., `LaunchOptions`, `NavigationOptions`)
5. **Auto-waiting** - Locators should wait for elements to be ready before acting (like Playwright)
6. **Async all the way** - All I/O operations must be async using Tokio's `async/await`

### Example API Mapping

**Playwright Python:**
```python
page.goto("https://example.com", wait_until="networkidle")
page.locator("button").click()
page.fill("#username", "test")
```

**Sparkle Rust:**
```rust
page.goto("https://example.com", NavigationOptionsBuilder::default()
    .wait_until(WaitUntilState::NetworkIdle)
    .build()
    .unwrap()
).await?;
page.locator("button").click(Default::default()).await?;
page.locator("#username").fill("test").await?;
```

## Running the Project

```bash
# Build and run CLI
cargo run --release -- install chrome
cargo run --release -- list

# Run examples
cargo run --example basic_navigation --release
cargo run --example wikipedia_search --release

# Test library functionality
cargo test
```

<skills_system priority="1">

## Available Skills

<!-- SKILLS_TABLE_START -->
<usage>
When users ask you to perform tasks, check if any of the available skills below can help complete the task more effectively. Skills provide specialized capabilities and domain knowledge.

How to use skills:
- Invoke: `bunx --bun openskills read <skill-name>` (run in your shell)
  - For multiple: `bunx --bun openskills read skill-one,skill-two`
- The skill content will load with detailed instructions on how to complete the task
- Base directory provided in output for resolving bundled resources (references/, scripts/, assets/)

Usage notes:
- Only use skills listed in <available_skills> below
- Do not invoke a skill that is already loaded in your context
- Each skill invocation is stateless
</usage>

<available_skills>

<skill>
<name>coding-guidelines</name>
<description>"Use when asking about Rust code style or best practices. Keywords: naming, formatting, comment, clippy, rustfmt, lint, code style, best practice, P.NAM, G.FMT, code review, naming convention, variable naming, function naming, type naming, 命名规范, 代码风格, 格式化, 最佳实践, 代码审查, 怎么命名"</description>
<location>project</location>
</skill>

<skill>
<name>core-actionbook</name>
<description></description>
<location>project</location>
</skill>

<skill>
<name>core-agent-browser</name>
<description></description>
<location>project</location>
</skill>

<skill>
<name>core-dynamic-skills</name>
<description></description>
<location>project</location>
</skill>

<skill>
<name>core-fix-skill-docs</name>
<description></description>
<location>project</location>
</skill>

<skill>
<name>domain-cli</name>
<description>"Use when building CLI tools. Keywords: CLI, command line, terminal, clap, structopt, argument parsing, subcommand, interactive, TUI, ratatui, crossterm, indicatif, progress bar, colored output, shell completion, config file, environment variable, 命令行, 终端应用, 参数解析"</description>
<location>project</location>
</skill>

<skill>
<name>domain-cloud-native</name>
<description>"Use when building cloud-native apps. Keywords: kubernetes, k8s, docker, container, grpc, tonic, microservice, service mesh, observability, tracing, metrics, health check, cloud, deployment, 云原生, 微服务, 容器"</description>
<location>project</location>
</skill>

<skill>
<name>domain-embedded</name>
<description>"Use when developing embedded/no_std Rust. Keywords: embedded, no_std, microcontroller, MCU, ARM, RISC-V, bare metal, firmware, HAL, PAC, RTIC, embassy, interrupt, DMA, peripheral, GPIO, SPI, I2C, UART, embedded-hal, cortex-m, esp32, stm32, nrf, 嵌入式, 单片机, 固件, 裸机"</description>
<location>project</location>
</skill>

<skill>
<name>domain-fintech</name>
<description>"Use when building fintech apps. Keywords: fintech, trading, decimal, currency, financial, money, transaction, ledger, payment, exchange rate, precision, rounding, accounting, 金融, 交易系统, 货币, 支付"</description>
<location>project</location>
</skill>

<skill>
<name>domain-iot</name>
<description>"Use when building IoT apps. Keywords: IoT, Internet of Things, sensor, MQTT, device, edge computing, telemetry, actuator, smart home, gateway, protocol, 物联网, 传感器, 边缘计算, 智能家居"</description>
<location>project</location>
</skill>

<skill>
<name>domain-ml</name>
<description>"Use when building ML/AI apps in Rust. Keywords: machine learning, ML, AI, tensor, model, inference, neural network, deep learning, training, prediction, ndarray, tch-rs, burn, candle, 机器学习, 人工智能, 模型推理"</description>
<location>project</location>
</skill>

<skill>
<name>domain-web</name>
<description>"Use when building web services. Keywords: web server, HTTP, REST API, GraphQL, WebSocket, axum, actix, warp, rocket, tower, hyper, reqwest, middleware, router, handler, extractor, state management, authentication, authorization, JWT, session, cookie, CORS, rate limiting, web 开发, HTTP 服务, API 设计, 中间件, 路由"</description>
<location>project</location>
</skill>

<skill>
<name>m01-ownership</name>
<description>"CRITICAL: Use for ownership/borrow/lifetime issues. Triggers: E0382, E0597, E0506, E0507, E0515, E0716, E0106, value moved, borrowed value does not live long enough, cannot move out of, use of moved value, ownership, borrow, lifetime, 'a, 'static, move, clone, Copy, 所有权, 借用, 生命周期"</description>
<location>project</location>
</skill>

<skill>
<name>m02-resource</name>
<description>"CRITICAL: Use for smart pointers and resource management. Triggers: Box, Rc, Arc, Weak, RefCell, Cell, smart pointer, heap allocation, reference counting, RAII, Drop, should I use Box or Rc, when to use Arc vs Rc, 智能指针, 引用计数, 堆分配"</description>
<location>project</location>
</skill>

<skill>
<name>m03-mutability</name>
<description>"CRITICAL: Use for mutability issues. Triggers: E0596, E0499, E0502, cannot borrow as mutable, already borrowed as immutable, mut, &mut, interior mutability, Cell, RefCell, Mutex, RwLock, 可变性, 内部可变性, 借用冲突"</description>
<location>project</location>
</skill>

<skill>
<name>m04-zero-cost</name>
<description>"CRITICAL: Use for generics, traits, zero-cost abstraction. Triggers: E0277, E0308, E0599, generic, trait, impl, dyn, where, monomorphization, static dispatch, dynamic dispatch, impl Trait, trait bound not satisfied, 泛型, 特征, 零成本抽象, 单态化"</description>
<location>project</location>
</skill>

<skill>
<name>m05-type-driven</name>
<description>"CRITICAL: Use for type-driven design. Triggers: type state, PhantomData, newtype, marker trait, builder pattern, make invalid states unrepresentable, compile-time validation, sealed trait, ZST, 类型状态, 新类型模式, 类型驱动设计"</description>
<location>project</location>
</skill>

<skill>
<name>m06-error-handling</name>
<description>"CRITICAL: Use for error handling. Triggers: Result, Option, Error, ?, unwrap, expect, panic, anyhow, thiserror, when to panic vs return Result, custom error, error propagation, 错误处理, Result 用法, 什么时候用 panic"</description>
<location>project</location>
</skill>

<skill>
<name>m07-concurrency</name>
<description>"CRITICAL: Use for concurrency/async. Triggers: E0277 Send Sync, cannot be sent between threads, thread, spawn, channel, mpsc, Mutex, RwLock, Atomic, async, await, Future, tokio, deadlock, race condition, 并发, 线程, 异步, 死锁"</description>
<location>project</location>
</skill>

<skill>
<name>m09-domain</name>
<description>"CRITICAL: Use for domain modeling. Triggers: domain model, DDD, domain-driven design, entity, value object, aggregate, repository pattern, business rules, validation, invariant, 领域模型, 领域驱动设计, 业务规则"</description>
<location>project</location>
</skill>

<skill>
<name>m10-performance</name>
<description>"CRITICAL: Use for performance optimization. Triggers: performance, optimization, benchmark, profiling, flamegraph, criterion, slow, fast, allocation, cache, SIMD, make it faster, 性能优化, 基准测试"</description>
<location>project</location>
</skill>

<skill>
<name>m11-ecosystem</name>
<description>"Use when integrating crates or ecosystem questions. Keywords: E0425, E0433, E0603, crate, cargo, dependency, feature flag, workspace, which crate to use, using external C libraries, creating Python extensions, PyO3, wasm, WebAssembly, bindgen, cbindgen, napi-rs, cannot find, private, crate recommendation, best crate for, Cargo.toml, features, crate 推荐, 依赖管理, 特性标志, 工作空间, Python 绑定"</description>
<location>project</location>
</skill>

<skill>
<name>m12-lifecycle</name>
<description>"Use when designing resource lifecycles. Keywords: RAII, Drop, resource lifecycle, connection pool, lazy initialization, connection pool design, resource cleanup patterns, cleanup, scope, OnceCell, Lazy, once_cell, OnceLock, transaction, session management, when is Drop called, cleanup on error, guard pattern, scope guard, 资源生命周期, 连接池, 惰性初始化, 资源清理, RAII 模式"</description>
<location>project</location>
</skill>

<skill>
<name>m13-domain-error</name>
<description>"Use when designing domain error handling. Keywords: domain error, error categorization, recovery strategy, retry, fallback, domain error hierarchy, user-facing vs internal errors, error code design, circuit breaker, graceful degradation, resilience, error context, backoff, retry with backoff, error recovery, transient vs permanent error, 领域错误, 错误分类, 恢复策略, 重试, 熔断器, 优雅降级"</description>
<location>project</location>
</skill>

<skill>
<name>m14-mental-model</name>
<description>"Use when learning Rust concepts. Keywords: mental model, how to think about ownership, understanding borrow checker, visualizing memory layout, analogy, misconception, explaining ownership, why does Rust, help me understand, confused about, learning Rust, explain like I'm, ELI5, intuition for, coming from Java, coming from Python, 心智模型, 如何理解所有权, 学习 Rust, Rust 入门, 为什么 Rust"</description>
<location>project</location>
</skill>

<skill>
<name>m15-anti-pattern</name>
<description>"Use when reviewing code for anti-patterns. Keywords: anti-pattern, common mistake, pitfall, code smell, bad practice, code review, is this an anti-pattern, better way to do this, common mistake to avoid, why is this bad, idiomatic way, beginner mistake, fighting borrow checker, clone everywhere, unwrap in production, should I refactor, 反模式, 常见错误, 代码异味, 最佳实践, 地道写法"</description>
<location>project</location>
</skill>

<skill>
<name>meta-cognition-parallel</name>
<description>"EXPERIMENTAL: Three-layer parallel meta-cognition analysis. Triggers on: /meta-parallel, 三层分析, parallel analysis, 并行元认知"</description>
<location>project</location>
</skill>

<skill>
<name>playwright-skill</name>
<description>Complete browser automation with Playwright. Auto-detects dev servers, writes clean test scripts to /tmp. Test pages, fill forms, take screenshots, check responsive design, validate UX, test login flows, check links, automate any browser task. Use when user wants to test websites, automate browser interactions, validate web functionality, or perform any browser-based testing.</description>
<location>project</location>
</skill>

<skill>
<name>rust-call-graph</name>
<description>"Visualize Rust function call graphs using LSP. Triggers on: /call-graph, call hierarchy, who calls, what calls, 调用图, 调用关系, 谁调用了, 调用了谁"</description>
<location>project</location>
</skill>

<skill>
<name>rust-code-navigator</name>
<description>"Navigate Rust code using LSP. Triggers on: /navigate, go to definition, find references, where is defined, 跳转定义, 查找引用, 定义在哪, 谁用了这个"</description>
<location>project</location>
</skill>

<skill>
<name>rust-daily</name>
<description>|</description>
<location>project</location>
</skill>

<skill>
<name>rust-deps-visualizer</name>
<description>"Visualize Rust project dependencies as ASCII art. Triggers on: /deps-viz, dependency graph, show dependencies, visualize deps, 依赖图, 依赖可视化, 显示依赖"</description>
<location>project</location>
</skill>

<skill>
<name>rust-learner</name>
<description>"Use when asking about Rust versions or crate info. Keywords: latest version, what's new, changelog, Rust 1.x, Rust release, stable, nightly, crate info, crates.io, lib.rs, docs.rs, API documentation, crate features, dependencies, which crate, what version, Rust edition, edition 2021, edition 2024, cargo add, cargo update, 最新版本, 版本号, 稳定版, 最新, 哪个版本, crate 信息, 文档, 依赖, Rust 版本, 新特性, 有什么特性"</description>
<location>project</location>
</skill>

<skill>
<name>rust-refactor-helper</name>
<description>"Safe Rust refactoring with LSP analysis. Triggers on: /refactor, rename symbol, move function, extract, 重构, 重命名, 提取函数, 安全重构"</description>
<location>project</location>
</skill>

<skill>
<name>rust-router</name>
<description>"CRITICAL: Use for ALL Rust questions including errors, design, and coding.</description>
<location>project</location>
</skill>

<skill>
<name>rust-skill-creator</name>
<description>"Use when creating skills for Rust crates or std library documentation. Keywords: create rust skill, create crate skill, create std skill, 创建 rust skill, 创建 crate skill, 创建 std skill, 动态 rust skill, 动态 crate skill, skill for tokio, skill for serde, skill for axum, generate rust skill, rust 技能, crate 技能, 从文档创建skill, from docs create skill"</description>
<location>project</location>
</skill>

<skill>
<name>rust-symbol-analyzer</name>
<description>"Analyze Rust project structure using LSP symbols. Triggers on: /symbols, project structure, list structs, list traits, list functions, 符号分析, 项目结构, 列出所有, 有哪些struct"</description>
<location>project</location>
</skill>

<skill>
<name>rust-trait-explorer</name>
<description>"Explore Rust trait implementations using LSP. Triggers on: /trait-impl, find implementations, who implements, trait 实现, 谁实现了, 实现了哪些trait"</description>
<location>project</location>
</skill>

<skill>
<name>unsafe-checker</name>
<description>"CRITICAL: Use for unsafe Rust code review and FFI. Triggers on: unsafe, raw pointer, FFI, extern, transmute, *mut, *const, union, #[repr(C)], libc, std::ffi, MaybeUninit, NonNull, SAFETY comment, soundness, undefined behavior, UB, safe wrapper, memory layout, bindgen, cbindgen, CString, CStr, 安全抽象, 裸指针, 外部函数接口, 内存布局, 不安全代码, FFI 绑定, 未定义行为"</description>
<location>project</location>
</skill>

</available_skills>
<!-- SKILLS_TABLE_END -->

</skills_system>
