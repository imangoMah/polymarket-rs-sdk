# Relayer Client 示例

本目录包含了使用 Polymarket Relayer Client 的各种示例代码。

## 前提条件

1. **环境变量配置**

创建 `.env` 文件:

```bash
# RPC 端点
RPC_URL=https://polygon-rpc.com

# 钱包私钥 (不要提交到版本控制!)
PRIVATE_KEY=0x...

# Builder API 凭证
BUILDER_API_KEY=your_api_key
BUILDER_SECRET=your_secret
BUILDER_PASS_PHRASE=your_passphrase
```

2. **依赖安装**

确保 `Cargo.toml` 包含:

```toml
[dependencies]
builder-relayer-client-rust = { path = ".." }
ethers = "2.0"
tokio = { version = "1", features = ["full"] }
dotenv = "0.15"
hex = "0.4"
```

## 示例列表

### 1. 快速开始 (`quick_start.rs`)

最简单的入门示例,演示如何初始化客户端并部署 Safe 钱包。

**运行:**
```bash
cargo run --example quick_start
```

**功能:**
- ✅ 初始化 Relayer Client
- ✅ 部署 Safe 钱包
- ✅ 等待交易确认

**输出示例:**
```
🚀 快速开始: Polymarket Relayer Client

钱包地址: 0x...
✅ Relayer Client 初始化成功!

部署 Safe 钱包...
✅ Safe 部署成功!
   交易哈希: 0x...
   Safe 地址: 0x...
```

---

### 2. 代币授权 (`approve_tokens.rs`)

演示如何授权 ERC20 代币,包括单个授权和批量授权。

**运行:**
```bash
cargo run --example approve_tokens
```

**功能:**
- ✅ 授权 USDC 给 CTF
- ✅ 授权 USDC 给 Exchange
- ✅ 批量授权多个代币

**输出示例:**
```
💰 代币授权示例

1️⃣ 授权 USDC 给 CTF...
   代币: 0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174
   授权给: 0x4d97dcd97ec945f40cf65f87097ace5ea0476045
   交易哈希: 0x...
✅ 授权完成

2️⃣ 授权 USDC 给 CTF Exchange...
   代币: 0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174
   授权给: 0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E
   交易哈希: 0x...
✅ 授权完成

3️⃣ 批量授权...
   提交 2 个授权交易...
   交易哈希: 0x...
✅ 批量授权完成

🎉 所有授权完成!
```

---

### 3. 交易监控 (`monitor_transactions.rs`)

演示如何监控和追踪 Relayer 交易状态。

**运行:**
```bash
cargo run --example monitor_transactions
```

**功能:**
- ✅ 自动等待交易确认
- ✅ 手动轮询交易状态
- ✅ 带超时的监控

**输出示例:**
```
📊 交易监控示例

1️⃣ 自动等待交易确认...
   部署 Safe 钱包...
   交易已提交,ID: abc123
   ✅ 交易已确认!
      状态: STATE_CONFIRMED
      哈希: 0x...
      Safe 地址: 0x...

2️⃣ 手动轮询交易状态...
   部署 Safe 钱包...
   交易已提交,ID: def456
   开始手动轮询状态...

   [ 1] 状态: STATE_NEW (更新于: 10:30:15)
        → Relayer 已接收交易
   [ 2] 状态: STATE_EXECUTED (更新于: 10:30:18)
        → 交易已在链上执行
   [ 3] 状态: STATE_MINED (更新于: 10:30:21)
        → 交易已被包含在区块中
   [ 4] 状态: STATE_CONFIRMED (更新于: 10:30:24)
        → 交易已确认!

   ✅ 交易成功!
      交易哈希: 0x...
```

---

### 4. 完整示例 (`relayer_client_demo.rs`)

综合演示所有主要功能的完整示例。

**运行:**
```bash
cargo run --example relayer_client_demo
```

**功能:**
- ✅ 初始化客户端
- ✅ 部署 Safe 钱包
- ✅ 授权代币
- ✅ 执行批量交易
- ✅ CTF 操作 (split/merge/redeem)
- ✅ 错误处理和重试

**输出示例:**
```
🚀 Polymarket Relayer Client 示例

1️⃣ 初始化 Relayer Client...
   钱包地址: 0x...
✅ 客户端初始化成功

2️⃣ 部署 Safe 钱包...
   发送部署请求...
   交易已提交,等待确认...
   交易哈希: 0x...
   Safe 地址: 0x...
✅ Safe 部署成功

3️⃣ 授权 USDC 代币...
   代币: 0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174
   授权给: 0x4d97dcd97ec945f40cf65f87097ace5ea0476045
   交易已提交,等待确认...
   交易哈希: 0x...
✅ 代币授权成功

4️⃣ 执行批量交易...
   创建批量交易...
   批量交易包含 2 个操作
   交易已提交,等待确认...
   交易哈希: 0x...
   状态: STATE_CONFIRMED
✅ 批量交易完成

🎉 所有示例执行完成!
```

---

### 5. CTF 操作 (`ctf_operations.rs`)

演示条件代币框架 (CTF) 操作,包括分割、合并和赎回头寸。

**运行:**
```bash
cargo run --example ctf_operations
```

**功能:**
- ✅ Split Position (分割头寸) - 将抵押品拆分为条件代币
- ✅ Merge Position (合并头寸) - 将条件代币合并回抵押品
- ✅ Redeem Position (赎回头寸) - 赎回获胜的条件代币

**输出示例:**
```
🎲 CTF 操作示例

钱包地址: 0x...
✅ 客户端初始化成功

1️⃣ 分割头寸 (Split Position)...
   将抵押品代币分割为条件代币
   抵押品: 0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174
   条件ID: 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef
   分区: [1, 2]
   数量: 1000000
   交易已提交: tx_abc123
   交易哈希: 0x...
✅ 分割完成

2️⃣ 合并头寸 (Merge Position)...
   将条件代币合并回抵押品
   抵押品: 0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174
   条件ID: 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef
   分区: [1, 2]
   数量: 1000000
   交易已提交: tx_def456
   交易哈希: 0x...
✅ 合并完成

3️⃣ 赎回头寸 (Redeem Position)...
   赎回获胜的条件代币换回抵押品
   抵押品: 0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174
   条件ID: 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef
   索引集: [1, 2]
   交易已提交: tx_ghi789
   交易哈希: 0x...
✅ 赎回完成

🎉 所有 CTF 操作完成!
```

**关键概念:**

- **Split Position**: 将 USDC 等抵押品代币分割成代表不同市场结果(如 YES/NO)的条件代币
- **Merge Position**: 反向操作,将完整的条件代币集合合并回原始抵押品
- **Redeem Position**: 在市场解决后,将获胜的条件代币赎回为抵押品

**函数选择器:**
- `splitPosition`: `0x5c382289`
- `mergePositions`: `0xb73f4554`
- `redeemPositions`: `0x6d625a4e`

---

## 交易状态说明

Relayer 交易会经历以下状态:

| 状态 | 说明 | 类型 |
|------|------|------|
| `STATE_NEW` | Relayer 已接收交易 | 进行中 |
| `STATE_EXECUTED` | 交易已在链上执行 | 进行中 |
| `STATE_MINED` | 交易已被包含在区块中 | 进行中 |
| `STATE_CONFIRMED` | 交易已确认 | **最终状态** ✅ |
| `STATE_FAILED` | 交易失败 | **终止状态** ❌ |
| `STATE_INVALID` | 交易被拒绝为无效 | **终止状态** ❌ |

## 合约地址 (Polygon 主网)

示例中使用的合约地址:

```rust
// USDC
const USDC_ADDRESS: &str = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";

// CTF (条件代币框架)
const CTF_ADDRESS: &str = "0x4d97dcd97ec945f40cf65f87097ace5ea0476045";

// CTF Exchange
const CTF_EXCHANGE: &str = "0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E";

// Neg Risk CTF Exchange
const NEG_RISK_EXCHANGE: &str = "0xC5d563A36AE78145C45a50134d48A1215220f80a";
```

## 常见问题

### 1. 如何获取 Builder API 凭证?

联系 Polymarket 团队申请 Builder API 凭证。

### 2. 交易一直处于 STATE_NEW 状态?

- 检查 Builder API 凭证是否正确
- 检查签名逻辑是否正确
- 查看 Relayer 返回的错误信息

### 3. 如何调试交易失败?

```rust
let response = client.execute_safe_transactions(txs, "test").await?;
let result = response.wait().await?;

if let Some(receipt) = result {
    if receipt.state == "STATE_FAILED" {
        println!("失败原因: {:?}", receipt);
        // 检查交易数据、Gas、授权等
    }
}
```

### 4. 如何设置自定义超时?

```rust
use tokio::time::{timeout, Duration};

let response = client.deploy_safe().await?;

// 设置 60 秒超时
match timeout(Duration::from_secs(60), response.wait()).await {
    Ok(Ok(Some(receipt))) => println!("成功: {:?}", receipt),
    Ok(Ok(None)) => println!("失败"),
    Err(_) => println!("超时"),
    _ => println!("错误"),
}
```

## 最佳实践

### 1. 使用批量交易

将多个操作组合成一个批量交易:

```rust
let transactions = vec![
    create_approval_transaction(usdc, ctf),
    create_approval_transaction(usdc, exchange),
];

client.execute_safe_transactions(transactions, "Batch approvals").await?;
```

### 2. 添加有意义的元数据

```rust
let metadata = format!(
    "User: {}, Operation: {}, Amount: {}",
    user_id, operation, amount
);

client.execute_safe_transactions(txs, &metadata).await?;
```

### 3. 实现错误重试

```rust
for attempt in 0..3 {
    match client.execute_safe_transactions(txs.clone(), "tx").await {
        Ok(response) => {
            if let Ok(Some(_)) = response.wait().await {
                return Ok(());
            }
        }
        Err(e) if attempt < 2 => {
            sleep(Duration::from_secs(2u64.pow(attempt))).await;
            continue;
        }
        Err(e) => return Err(e.into()),
    }
}
```

### 4. 正确处理交易状态

```rust
match status.state.as_str() {
    "STATE_CONFIRMED" => {
        // 交易成功,更新数据库
    }
    "STATE_FAILED" | "STATE_INVALID" => {
        // 交易失败,回滚操作
    }
    _ => {
        // 继续等待
    }
}
```

## 更多资源

- [完整文档](../RELAYER_CLIENT_GUIDE.md)
- [TypeScript 参考](https://github.com/Polymarket/builder-relayer-client)
- [Polymarket 文档](https://docs.polymarket.com/developers/builders/relayer-client)

## 支持

如有问题,请联系:
- Email: support@polymarket.com
- GitHub: 提交 Issue

---

**更新时间**: 2025-11-08  
**版本**: v1.0.0
