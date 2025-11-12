本仓库提供面向 Rust 的 Polymarket SDK，整合了 TypeScript SDK 对应的 clob-client、builder-signing、order-utils 与 builder-relayer的redeem/merge 客户端相关模块，
以一体化（集合式）的方式对外提供统一的依赖与文档，便于在同一代码库内完成订单构建/签名、builder-relayer相关能力以及常用工具方法的集成与调用。

clob-client-rust
=================

Rust 版本的 Polymarket CLOB 客户端，移植自原 TypeScript `clob-client`，提供：

* EIP-712 确定性签名（与 TS SDK 结果保持一致）
* 订单构建 / 签名（限价、市价）与便捷创建+提交方法
* L1 派生 / 创建 API Key、Builder API Key 支持
* Tick Size 可显式传入；未提供则使用默认值（当前为 "0.01"），创建阶段不再隐式发起后端请求
* 订单取消（单个、批量、全部）、订单评分状态查询（scoring）
* 市场 / 价格 / 成交 / 奖励 等公共与鉴权端点访问

目录速览
--------
```
src/                核心库实现
examples/           使用示例（与 TS 常见调用场景对齐）
tests/              单元/集成测试（待补充更多 EIP-712 & 流程用例）
```

功能列表
--------
说明：以下为模块化功能清单。已通过测试或人工验证的项已在前方用 ✅ 标注；未标注项仍在计划中或待补充。

签名与密钥
- [✅] EIP-712 确定性签名（与 TS SDK 完全对齐）
- [✅] EOA 签名类型支持（SignatureType: EOA 等）
- [✅] L1 私钥派生 / 创建 L2 API Key（Create / Derive）
- [✅] Builder API Key 认证头生成（HMAC / Base64）
- [✅] 请求头封装：L1/L2/Builder 混用与降级

订单相关
- [✅] 限价单构建/签名/提交（create + post）
- [✅] 市价单构建/签名/提交（包含价格由外部订单簿计算）
- [✅] Tick Size 显式传入；默认 Tick（"0.01"）兜底
- [ ] 订单取消：单个 / 批量 / 全部开放订单
- [✅] FOK / FAK 订单类型支持（含边界校验）
- [ ] 订单评分状态（scoring）查询

市场与行情 API
- [ ] 获取市场列表（get_markets）
- [ ] 获取订单簿（get_order_book）
- [ ] 价格类端点（中点价 / spreads / prices）
- [ ] 成交列表（get_trades）
- [ ] 通知（get_notifications）
- [ ] 奖励数据（rewards，按日聚合）

账户与权限
- [ ] 余额/授权查询与可选更新（balance_allowance）
- [✅] API Keys 管理（get_api_keys 等）

Builder Redeem / Merge（资产赎回与份额合并）
- [✅] 创建 Redeem 请求（单市场或多市场）
- [ ] 批量 Redeem（Batch Redeem）
- [ ] Merge / Unmerge 份额操作
- [✅] Gas / Fee 预估（Estimate Gas/Fee）
- [✅] 状态查询与结果轮询（Redeem/Merge 状态 & Receipt）
- [ ] 幂等键与去重控制（Idempotency Key）
- [✅] 失败重试与错误分类（Retry / Error Categorization）
- [ ] Dry-run / Simulation（不广播链上交易）

WebSocket / 实时
- [ ] 实时订阅（价差 / 成交 / 订单事件）（规划中）

工程与类型
- [✅] 强类型数据结构（types），避免直接使用 `serde_json::Value`
- [✅] HTTP 帮助方法（typed 反序列化 / 基础重试）
- [✅] 别名方法（camelCase 与 snake_case 并存，兼容 TS 迁移）

示例与脚本
- [✅] `sign_order` / `post_order` 示例
- [✅] `create_and_post_order` / `create_and_post_market_order`
- [✅] `market_buy_order` / `market_sell_order`
- [✅] `get_markets` / `get_order_book` / `get_open_orders`
- [ ] 取消类：`cancel_order` / `cancel_orders` / `cancel_all`
- [✅] Key 管理：`create_api_key` / `derive_api_key` / `get_api_keys`
- [ ] 其他：`get_trades` / `get_notifications` / `rewards` / `balance_allowance` / `get_prices`

兼容性 / Parity
- [✅] 与 TypeScript 版本签名对齐（盐、signature、maker/taker 金额）
- [✅] TickSize 策略与 TS 一致（不隐式请求，默认/显式二选一）

安装与构建
----------
```bash
git clone <repo-url>
cd clob-client-rust
cargo build
```

**在你的项目中使用**（详见 [INTEGRATION.md](INTEGRATION.md)）：

```toml
```

环境变量约定
------------
按需设置（不同示例会读取）：

| 名称 | 说明 | 示例 |
|------|------|------|
| PK | L1 私钥（0x 开头 64 hex） | 0x... |
| CLOB_API_URL | CLOB 后端地址 | https://api.polymarket.com |
| CHAIN_ID | 链 ID | 80002 / 137 |
| CLOB_API_KEY / CLOB_SECRET / CLOB_PASS_PHRASE | L2 API 访问凭据 | 根据后端分配 |
| BUILDER_API_KEY / BUILDER_SECRET_B64 / BUILDER_PASSPHRASE | Builder 认证（可选） | 根据后端分配 |

快速示例
--------
签名但不提交（无 API 凭据时仅构建）：
```bash
cargo run --example sign_order
```

示例索引
--------
| 示例 | 功能 | 需环境变量 |
|------|------|-----------|
| `sign_order` | 构建并签名限价单 | PK |
| `post_order` | 构建后（示例中跳过真实提交） | PK |
| `create_and_post_order` | 限价单：创建+提交 | PK, CLOB_API_KEY/SECRET/PASS_PHRASE |
| `create_and_post_market_order` | 市价单：创建+提交（需外部先计算价格） | PK, CLOB_API_KEY/SECRET/PASS_PHRASE |
| `market_buy_order` / `market_sell_order` | 构建市价买/卖单 | PK |
| `get_markets` / `get_order_book` | 获取市场列表 / 订单簿 | (可选)CLOB_API_URL |
| `get_open_orders` | 查询当前开放订单 | (若需鉴权端点则需 L2) |
| `cancel_order` | 取消单个订单 | PK + L2 凭据 |
| `cancel_orders` | 批量取消 | PK + L2 凭据 |
| `cancel_all` | 取消全部开放订单 | PK + L2 凭据 |
| `create_api_key` / `derive_api_key` | 创建或派生 L2 API Key | PK |
| `get_api_keys` | 列出现有 API Keys | PK + L2 凭据 |
| `get_trades` | 获取成交列表（分页首页） | PK + L2 凭据 |
| `get_notifications` | 获取通知 | PK + L2 凭据 |
| `rewards` | 奖励数据（按日） | (视后端要求) |
| `scoring` | 订单评分状态 | (可选 Builder 认证) |
| `balance_allowance` | 查询并可选更新余额/授权 | PK + L2 凭据 |
| `get_prices` | 中点价 / spreads / prices | (公共或半公共) |

TypeScript Parity 说明
----------------------
* 通过添加 camelCase alias（如 `getMarkets`）与 snake_case 原始方法共存，便于迁移现有 TS 代码。
* EIP-712 签名盐/结构确保与 TS 结果一致（测试中已对比）。
* TickSize 行为：不再自动请求后端。可在外部调用 `get_tick_size` 获取真实最小值再传入；若未传则使用默认 "0.01"。
* 订单构建使用 `OrderBuilder`；支持限价与市价、EOA SignatureType，目前扩展点预留。

特性与结构
-----------
| 模块 | 作用 |
|------|------|
| `order_builder` | 构建/签名订单、处理价格与数量换算 |
| `client` | 封装 HTTP 调用、缓存 tick/fee/neg_risk、别名与鉴权头生成 |
| `headers` | L1/L2/Builder 请求头生成（HMAC / EIP-712 派生） |
| `types` | Typed 数据结构，避免直接使用 `serde_json::Value` |
| `http_helpers` | 基础请求封装 + typed 反序列化支持 |

开发调试建议
------------
1. 使用本地 mock / dev 节点运行后端，设置 `CLOB_API_URL` 指向对应地址。
2. 若需对比 TS 结果，可并行运行原 SDK 并抓取签名输出比对盐与最终 signature。
3. 对关键路径添加 `RUST_LOG=debug`（可在后续引入 tracing 以便更细粒度日志）。

统一字段要求（破坏性更新）
--------------------------
以下字段已改为必填，消除隐式默认与内部网络请求：
| 结构 | 字段 | 说明 |
|------|------|------|
| `UserOrder` | `fee_rate_bps: f64` | 限价单费率（bps） |
| `UserMarketOrder` | `price: f64` | 市价单价格（外部基于订单簿计算） |
| `UserMarketOrder` | `fee_rate_bps: f64` | 市价单费率 |
| `UserMarketOrder` | `order_type: OrderType` | FOK / FAK 必填 |

签名对齐测试（Rust vs TypeScript）
---------------------------------
1. 使用相同私钥与输入字段在 TS SDK 与本库各生成一个 `SignedOrder`。
2. 比较字段：`maker_amount` / `taker_amount` / `salt` / `signature`。
3. Builder 认证与否只影响提交请求头，不影响上述签名本体。

市价价格算法摘要：
* BUY：按卖单（asks）从高价向低价累加金额 (size * price)，满足目标 USDC 金额。FOK 若不足报错，FAK 若不足使用最优价。
* SELL：按买单（bids）从高价向低价累加 shares 数量。FOK 不足报错，FAK 不足使用最优价。


Roadmap / TODO
--------------
* [ ] 更多 Builder 功能（批量评分查询丰富字段）
* [ ] WebSocket / 实时订阅（价差 / 成交 / 订单事件）
* [ ] 更完善的错误分类（HTTP code -> Domain Error）
* [ ] 可插拔签名适配器 trait
* [ ] CI: GitHub Actions（build + test + clippy + fmt）

示例常见问题 (FAQ)
------------------
**Q:** 为什么某些字段是 Option<String>？
**A:** 后端不同路径返回的字段集合不完全一致，保持宽松结构防止反序列化失败。

**Q:** 为什么既有 camelCase 又有 snake_case 方法？
**A:** 兼容 TS 迁移，内部统一调用 snake_case，camelCase 为轻量包装。

**Q:** Tick Size 没传会怎样？
**A:** 使用默认 "0.01"；如需真实市场最小 tick，请外部调用 `get_tick_size` 后再传入。

许可
----
本项目沿用与原始 SDK 相同的许可证。暂未内置正式 LICENSE，请在引入生产使用前确认。

贡献
----
欢迎提交 PR：保持最小变更原则、补充对应测试、确保 `cargo clippy -D warnings` 通过。
