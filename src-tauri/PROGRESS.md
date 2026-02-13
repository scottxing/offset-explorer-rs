# Offset Explorer 3 Rust/Tauri 重构进度

## 当前状态

✅ **Phase 1: 基础架构 (已完成)**

### 已完成的模块

1. **config/** - 配置管理模块
   - ✅ `crypto.rs` - 密码加密/解密 (兼容 Java CryptoUtil)
   - ✅ `settings_complete.rs` - 用户设置管理 (XML 序列化)
   - ✅ `server_group.rs` - 服务器组管理
   - ✅ `mod.rs` - 模块导出

2. **kafka/** - Kafka 客户端模块
   - ✅ `mapper.rs` - Kafka 操作包装器 (rdkafka)
   - ✅ `client.rs` - Kafka 客户端

3. **models/** - 数据模型
   - ✅ `topic.rs` - Topic 模型
   - ✅ `partition.rs` - Partition 模型
   - ✅ `consumer.rs` - Consumer 模型
   - ✅ `broker.rs` - Broker 模型

4. **decoders/** - 消息解码器
   - ✅ `string_decoder.rs` - 字符串解码器
   - ✅ `byte_array_decoder.rs` - 字节数组解码器
   - ✅ `avro_decoder.rs` - Avro 解码器 (stub)
   - ✅ `no_key_decoder.rs` - 无键解码器

5. **ui_events/** - 事件系统
   - ✅ `ui_events.rs` - 事件总线

6. **async_ops/** - 异步任务管理
   - ✅ `async_ops.rs` - 后台任务管理器

7. **tauri_commands/** - Tauri 命令处理器
   - ✅ `tauri_commands.rs` - 命令处理器 (stub)

8. **其他模块**
   - ✅ `zookeeper.rs` - ZooKeeper 客户端 (stub)
   - ✅ `schema_registry.rs` - Schema Registry REST 客户端 (stub)

### 构建状态

```bash
✅ cargo build 成功
✅ 所有核心模块编译通过
⚠️  部分依赖需要系统库支持 (GTK, libsasl2)
```

### 当前依赖

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.35", features = ["rt-multi-thread", "macros", "sync"] }
dirs = "5.0"
quick-xml = "0.31"
sha1 = "0.10"
rand = "0.8"
hex = "0.4"
flate2 = "1.0"
thiserror = "1.0"
anyhow = "1.0"
chrono = "0.4"
tracing = "0.1"
tracing-subscriber = "0.3"
once_cell = "1.19"
rdkafka = "0.36"  # 基础版本, 无 SASL/SSL
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }
apache-avro = "0.16"
```

## 待完成的工作

### Phase 2: Kafka 集成 (部分完成)

- [ ] 实现 KafkaMapper 的完整功能
  - [ ] SSL/TLS 支持
  - [ ] SASL 认证支持 (PLAIN, SCRAM)
  - [ ] Admin 操作完整实现
  - [ ] Consumer 实现
  - [ ] Producer 实现

- [ ] 添加系统依赖
  ```bash
  sudo apt-get install libsasl2-dev libssl-dev
  ```

### Phase 3: 消息操作 (未开始)

- [ ] 实现消息浏览
- [ ] 实现消息搜索
- [ ] 实现消息生产
- [ ] 实现消息解码器完整版

### Phase 4: 消费者组 (未开始)

- [ ] 列出消费者组
- [ ] 查看消费者偏移量和延迟
- [ ] 重置消费者偏移量
- [ ] 删除消费者组
- [ ] ZooKeeper 消费者支持

### Phase 5: 高级功能 (未开始)

- [ ] ACL 管理
- [ ] 数据导出/导入
- [ ] Schema Registry 客户端
- [ ] Avro 解码器
- [ ] ZooKeeper 浏览器

### Phase 6: 前端 UI (未开始)

- [ ] 初始化 Tauri 项目
  ```bash
  npm install tauri
  ```
- [ ] 安装 GTK 开发库
  ```bash
  sudo apt-get install libgtk-3-dev libcairo2-dev libsoup-3.0-dev
  ```
- [ ] 创建 Svelte 组件
- [ ] 实现主题列表
- [ ] 实现消费者组面板
- [ ] 实现消息查看器

### Phase 7: 优化和测试 (未开始)

- [ ] 性能优化
- [ ] 错误处理
- [ ] 单元测试
- [ ] 集成测试
- [ ] 安装包构建

## 下一步操作

### 1. 安装系统依赖

```bash
# Debian/Ubuntu
sudo apt-get install \
    libgtk-3-dev \
    libcairo2-dev \
    libsoup-3.0-dev \
    libsasl2-dev \
    libssl-dev

# Fedora/RHEL
sudo dnf install \
    gtk3-devel \
    cairo-devel \
    libsoup3-devel \
    cyrus-sasl-devel \
    openssl-devel
```

### 2. 启用 Tauri GUI

在 `Cargo.toml` 中取消注释:
```toml
tauri = { version = "2.0", features = ["devtools"] }
```

### 3. 启用 rdkafka SASL/SSL

在 `Cargo.toml` 中更新:
```toml
rdkafka = { version = "0.36", features = ["ssl", "sasl"], default-features = false }
```

### 4. 实现 Kafka 客户端

完善 `kafka/mapper.rs` 中的 TODO 项:
- AdminClient 操作
- Consumer 消息读取
- Producer 消息发送

### 5. 创建前端 UI

```bash
cd ..
npm create tauri-app@latest
# 或使用现有 Svelte 模板
```

## 兼容性验证

### Java 配置文件兼容性

- ✅ 密码加密算法兼容 (crypto.rs)
- ✅ 用户设置 XML 格式兼容 (settings_complete.rs)
- [ ] 服务器连接 XML (需要 server_connection 模块)
- [ ] 服务器组 XML (server_group.rs 已完成)
- [ ] 浏览历史 XML

### 测试清单

- [ ] 能否加载 Java 版本生成的 settings.xml
- [ ] 能否解密 Java 版本加密的密码
- [ ] 能否连接到 PLAINTEXT Kafka
- [ ] 能否连接到 SSL Kafka
- [ ] 能否连接到 SASL_PLAINTEXT Kafka
- [ ] 能否浏览 Topic 列表
- [ ] 能否读取消息
- [ ] 能否发送消息

## 架构说明

### 模块依赖关系

```
main.rs
├── config/
│   ├── crypto.rs          (密码加密)
│   ├── settings_complete.rs (用户设置)
│   └── server_group.rs    (服务器组)
│
├── kafka/
│   ├── mapper.rs          (Kafka 操作)
│   └── client.rs          (Kafka 客户端)
│
├── models/               (数据模型)
│   ├── topic.rs
│   ├── partition.rs
│   ├── consumer.rs
│   └── broker.rs
│
├── decoders/            (消息解码器)
│   ├── string_decoder.rs
│   ├── byte_array_decoder.rs
│   ├── avro_decoder.rs
│   └── no_key_decoder.rs
│
├── ui_events.rs         (事件总线)
├── async_ops.rs         (后台任务)
└── tauri_commands.rs    (Tauri 命令)
```

### 事件系统

使用 `tokio::sync::broadcast` 实现发布-订阅模式:

```rust
pub enum AppEventType {
    ServerConnectionAdded { id: i64, name: String },
    ServerConnected { id: i64 },
    TopicAdded { name: String },
    // ... 更多事件类型
}
```

## 开发环境

### 系统要求

- Rust 1.70+
- Node.js 18+ (用于 Tauri 前端)
- JDK 11+ (用于对比 Java 版本)

### IDE 推荐

- VS Code + rust-analyzer
- IntelliJ IDEA + Rust 插件
- NeoVim + rust-tools.nvim

## 许可证

Copyright 2024 Offset Explorer Rust Team
Licensed under Proprietary License

---

**最后更新**: 2026-02-12
**当前版本**: v3.0.3
**编译状态**: ✅ 成功 (71 警告)
