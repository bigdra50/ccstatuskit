# 配置参考

[English](configuration.md) | [日本語](configuration.ja.md) | 简体中文

ccstatuskit 读取 `$XDG_CONFIG_HOME/ccstatuskit/config.toml`
（默认为 `~/.config/ccstatuskit/config.toml`）。
可通过环境变量 `CCSTATUSKIT_CONFIG` 指定其他路径。

没有配置文件时使用内置默认值。
配置文件*损坏*时同样回退到默认值。
配置错误绝不会让 statusline 消失，但在 TOML 修好之前布局会退回默认状态（stderr 会输出警告）。

## 布局：`format` 模板

```toml
format = """
$model $directory $git
$ctx $usage
"""
```

- `$name` 与 `${name}` 引用内置模块；`${custom.x}` 引用
  `[custom.x]` 条目（带点的名字必须用花括号）。
- 模板的每一行就是 statusline 的一行。行是声明出来的，不是定位出来的，
  不存在行号。
- 隐藏的模块会从行内收拢消失；整行模块都隐藏（且字面文本只有空白）时，
  该行整体消失。
- `\$` 表示字面美元符号，`\\` 表示字面反斜杠。未知模块名会静默隐藏，
  因此为新版本编写的配置在旧二进制上也能安全运行。
- 未设置时的默认值：
  `$model $directory $memory $ctx $time $git`、
  `$package $unity $node $rust $go $python $dotnet $ruby $java $kotlin $php $swift`、
  `$usage $cost $lines $agent $pr $worktree`

## 内置模块

| 模块 | Segment | 备注 |
| --- | --- | --- |
| `model` | 模型名称（按系列着色） | stdin 无模型信息时隐藏 |
| `directory` | 相对于项目的路径 | 否则回退到 cwd 的 basename |
| `git` | 分支、工作区状态（区分 untracked/rename）、冲突、stash、merge/rebase、ahead/behind | 以 `GIT_OPTIONAL_LOCKS=0` 运行 `git`；仓库外隐藏 |
| `memory` | 系统内存用量（GB） | >60% / >80% 时变色 |
| `ctx` | 上下文窗口使用率 | ≥40% / ≥60% 时变色；Claude Code 上报之前隐藏 |
| `time` | 当前时间与会话时长 | 时长来自 `cost.total_duration_ms` |
| `usage` | Claude 配额窗口 | 见下文 |
| `cost` | 本次会话花费（USD） | 来自 `cost.total_cost_usd`；≥$1 / ≥$5 时变色 |
| `lines` | 本次会话变更行数（`+120 -45`） | 来自 `cost.total_lines_added`/`total_lines_removed`；两者都为 0 时隐藏 |
| `agent` | 当前运行的子代理名称 | 来自 `agent.name`；不在子代理运行中时隐藏 |
| `pr` | 打开的 PR 编号 | 来自 `pr.number`；按 `pr.review_state`（approved/changes_requested/pending）变色 |
| `worktree` | worktree 名称（分支不同则一并显示） | 来自 `worktree.name`/`worktree.branch`；在主 checkout 中隐藏 |
| `package` | 项目自身声明的版本 | starship 方式：读取 `Cargo.toml`、`package.json`、`pyproject.toml`、`composer.json` 或 `pom.xml` 的 `version` 字段（取第一个匹配） |
| `unity`、`node`、`rust`、`go`、`python`、`dotnet`、`ruby`、`java`、`kotlin`、`php`、`swift` | 语言图标 + 工具链版本 | 一种语言一个模块，各自独立根据标记文件判定；`Cargo.toml` 与 `package.json` 同时存在时 `$rust` 与 `$node` 会同时显示。版本指*工具链*的版本，从真实二进制获取（`rustc --version`、`go version`、`python3 --version` 等），二进制不可用时回退到项目文件（`go.mod` 指令、`.python-version` 等）。例外：`unity` 显示编辑器版本，`dotnet` 显示目标框架，`node` 检测到 React/Vue/Next/TypeScript 时显示该框架的依赖版本。`dotnet` 在 Unity 项目内隐藏（Unity 会自动生成 `.csproj`，那里请用 `$unity`） |

每个内置模块都支持两个通用选项：

```toml
[git]
disabled = true              # 移除该模块

[model]
style = "bold fg:#FF79C6"    # 覆盖默认颜色
```

## 样式与调色板

样式字符串沿用 starship 的格式：以空白分隔的 token。

- 属性：`bold`、`dimmed`、`italic`、`underline`、`inverted`
- 颜色：`fg:` / `bg:` 后接 `#rrggbb`、0-255 的 ANSI 索引，或颜色名
  （`black`、`red`、`green`、`yellow`、`blue`、`purple`、`cyan`、`white`
  及 `bright-` 系列；`magenta` 是 `purple` 的别名）

命名颜色在 `[palette]` 表中定义：

```toml
[palette]
hot = "#FF5555"

[ctx]
style = "bold fg:hot"
```

## 配额显示

```toml
[usage]
scoped = true            # 同时获取按模型配额（非官方 API，需显式开启）
only = ["fable"]         # 筛选要显示的窗口
cache_ttl = 120          # scoped 缓存的新鲜度（秒）
```

不开启 `scoped` 时，模块只显示 Claude Code 通过 stdin 传入的官方
`rate_limits` 数据（5 小时与 7 天窗口，Pro/Max 订阅），完全不碰网络。

开启 `scoped = true` 后，按模型的每周配额（如 Fable）来自
Claude Code 自身使用的同一非官方端点。
独立的 `ccstatuskit --refresh-usage` 进程负责刷新
stale-while-revalidate 磁盘缓存，渲染只读磁盘。
**该数据源属于非官方接口，可能随时失效。**

`only` 可填：`"five_hour"`（别名 `"5h"`、`"session"`）、
`"seven_day"`（别名 `"wk"`、`"weekly"`），或如 `"fable"` 的
模型名子串（不区分大小写）。空列表显示全部。

高级选项：`auto_refresh = false` 关闭后台刷新，
`cache_dir` 覆盖缓存位置（默认 `$XDG_CACHE_HOME/ccstatuskit`）。

## 自定义模块

依照[模块契约](module-contract.zh-CN.md)，任何可执行文件都能成为模块：

```toml
format = "$model ${custom.pr}"

[custom.pr]
command = "jq -r '.pr.number // empty' | sed 's/^/#/'"
style = "fg:cyan"
when_dir = [".git"]      # 仅当项目目录中存在该路径时运行
# shell = ["bash", "-c"] # 默认: ["sh", "-c"] (unix) / ["cmd", "/C"] (windows)
# disabled = true
```

子进程通过 stdin 收到原始 JSON，并获得 `CCSK_*` 环境变量；
stdout 的第一行成为 segment，空输出表示隐藏。
只为副作用而存在的模块（写标记文件、通知 socket 等）也是正当用法：
什么都不输出，它就永远不会出现在屏幕上。

## 全局选项

```toml
command_timeout = 500    # 所有模块共享的实时间预算（毫秒）
```

设置环境变量 `NO_COLOR` 可去掉全部样式。

## 诊断问题

渲染路径按设计隐藏错误。
当某些内容不显示时，去问 doctor：

```sh
ccstatuskit doctor
```

它报告解析出的配置路径、TOML 与 format 模板错误、非法的 style 与 palette 条目、
未知模块名、缺少表的 custom 引用，以及 scoped usage 的凭据与缓存状态。
错误以 exit 1 退出；警告（只是被隐藏的内容）以 exit 0 退出。

## 出错时的行为

| 事件 | 结果 |
| --- | --- |
| 模块 panic、报错或超时 | 只省略它的 segment，整行保留 |
| stdin JSON 损坏 | 能解析多少渲染多少，退出码保持 0 |
| 配置 TOML 损坏 | 使用默认值运行，stderr 输出警告 |
| `format` 中出现未知模块名 | 静默隐藏 |

## 完整示例

```toml
# 第 1 行：会话状态 / 第 2 行：Claude 信息 / 第 3 行：开发环境。
# 在无法识别的项目之外，第 3 行会自动消失。
format = """
$directory $memory $time $git
$model $ctx $usage
$rust $unity ${custom.unilyze}
"""

[usage]
scoped = true

[custom.unilyze]
command = "unilyze statusline"
when_dir = ["Assets"]
```

修改在下一次渲染时生效。
每次渲染都是全新进程，因此无需重启任何东西。
