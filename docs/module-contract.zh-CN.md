# 外部模块契约 (v1)

[English](module-contract.md) | [日本語](module-contract.ja.md) | 简体中文

ccstatuskit 的模块可以是任意可执行文件。
在 `[custom.<name>]` 中声明，并在 `format` 模板里放置 `${custom.<name>}`。

```toml
format = "$model $git ${custom.unilyze}"

[custom.unilyze]
command = "unilyze statusline"
when_dir = ["Assets"]          # 仅当项目目录中存在该路径时运行
# shell = ["bash", "-c"]       # 默认: ["sh", "-c"] (unix) / ["cmd", "/C"] (windows)
# style = "bold fg:#A6E22E"    # 可选；包裹纯文本输出
# disabled = true
```

## 输入

| 通道 | 内容 |
| --- | --- |
| stdin | Claude Code 的 statusline JSON，与 ccstatuskit 收到的完全一致（逐字节透传）。可能出现你不认识的字段，请忽略它们。 |
| `CCSK_CONTRACT` | 契约版本，当前为 `1`。版本号提升意味着破坏性变更。 |
| `CCSK_MODEL` | 模型显示名（如 `Opus`）。未知时不设置。 |
| `CCSK_MODEL_ID` | 模型 ID（如 `claude-opus-4-8`）。 |
| `CCSK_CWD` | Claude Code 的当前工作目录。 |
| `CCSK_CURRENT_DIR` | `workspace.current_dir`。 |
| `CCSK_PROJECT_DIR` | `workspace.project_dir`。 |
| `CCSK_SESSION_ID` | 会话内保持稳定的 ID。 |
| `CCSK_TRANSCRIPT_PATH` | 会话 transcript JSONL 的路径。 |
| `CCSK_VERSION` | Claude Code 版本。 |
| `CCSK_CTX_PCT` | 上下文窗口使用率（整数百分比）。 |

每个 `CCSK_*` 变量仅在对应的 JSON 字段存在时才会设置。
简单模块优先使用环境变量；需要更多信息时再解析 stdin。

## 输出

- **stdout 的第一行成为 segment。** 后续行会被忽略。
- **空输出表示「隐藏」。** 没有内容要展示时什么都不输出，这就是条件显示的惯用写法。
- 允许 ANSI 转义码。若配置中设置了 `style`，ccstatuskit 会包裹你的（纯文本）输出，两者不要同时使用。
- 使用 UTF-8。这是 statusline，请保持一个简短的 segment。

## 失败语义 (fail-soft)

| 事件 | 结果 |
| --- | --- |
| 非零退出码 | 隐藏 segment（丢弃 stdout） |
| 在 `command_timeout`（默认 500 ms）内无输出 | 杀死进程并隐藏 segment |
| 崩溃或无法启动的命令 | 隐藏 segment |

模块最多失去自己的 segment。
它既不可能破坏 statusline，也不可能在 deadline 之后留下存活的进程。
如果数据源较慢，请在模块内自行缓存（stale-while-revalidate 很合适），并立刻返回缓存值。

## 兼容性规则

1. stdin 的 JSON 原样透传。Claude Code 的 schema 变动会直接到达你这里，请防御性地解析。
2. 契约 v1 范围内可能新增 `CCSK_*` 变量，但既有变量不会改变含义或消失。
3. 上述规则发生破坏性变更时，`CCSK_CONTRACT` 会提升。

## 示例：最小的 bash 模块

```bash
#!/usr/bin/env bash
# 仅当会话存在打开的 PR 时显示 PR 编号。
number=$(jq -r '.pr.number // empty' 2>/dev/null)   # 读取 stdin
[ -n "$number" ] && printf '#%s' "$number"           # 空输出 = 隐藏
```

任何语言的写法都相同：读取 stdin 或 `CCSK_*`，输出一行。

## 测试你的模块

把命令交给内置 harness，就能看到渲染时的确切行为：

```sh
ccstatuskit test-module './my-module.sh'
```

它以 full context 与 empty context 两个场景执行，stdin 提供 JSON 并设置
`CCSK_*` 环境变量，然后报告得到的 segment（或隐藏的原因）、
相对渲染预算的耗时，以及真实渲染会丢弃的 stderr。
