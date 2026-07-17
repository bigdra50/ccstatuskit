# ccstatuskit

[![CI](https://github.com/bigdra50/ccstatuskit/actions/workflows/ci.yml/badge.svg)](https://github.com/bigdra50/ccstatuskit/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/ccstatuskit.svg)](https://crates.io/crates/ccstatuskit)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-orange.svg)](Cargo.toml)

[English](README.md) | [日本語](README.ja.md) | 简体中文

为 [Claude Code](https://code.claude.com) 打造的模块化 statusline 工具箱，设计取法 starship。
用 TOML 模板声明布局，将内置模块与任意外部命令自由组合。
Rust 编写的单一二进制文件，无运行时依赖。

![在 Rust 仓库与 Unity 项目中渲染的三行 statusline 示例](https://raw.githubusercontent.com/bigdra50/ccstatuskit/main/assets/statusline.png)

```toml
# ~/.config/ccstatuskit/config.toml
format = """
$model $directory $memory $ctx $time $git
$project $usage ${custom.unilyze}
"""

[usage]
scoped = true
only = ["fable"]          # 只显示「Fable 14%」——精确挑选你想要的窗口

[custom.unilyze]
command = "unilyze statusline"
when_dir = ["Assets"]     # 仅在 Unity 项目中运行
```

## 为什么要做这个

Claude Code 只能注册一条 statusline 命令。
现有的 statusline 都是一体式脚本：哪怕只想要别人的用量显示，也得整个搬过来。
ccstatuskit 用三个机制拆分这个问题：

- **内置模块**在进程内并行执行，各自拥有独立的超时。
- **模块契约**（`[custom.x]`）把任意可执行文件当作模块。
  stdin 原样收到 Claude Code 的 JSON，`CCSK_*` 环境变量提供解析好的基础信息。
  stdout 的第一行成为一个 segment，空输出表示隐藏。
  用 bash、Python 或任何语言都能编写。详见 [docs/module-contract.md](docs/module-contract.md)。
- **fail-soft**：模块崩溃或超时只会丢掉自己的 segment，绝不破坏整行。
  无论输入损坏、配置写错还是 schema 变动，statusline 始终渲染，并始终以 exit 0 退出。

行以模板行的形式声明，而非绝对行号。
隐藏的模块自动收拢，整行模块都隐藏时该行整体消失。

## 安装

```sh
cargo install ccstatuskit          # 或从 GitHub Releases 获取二进制文件
```

然后在 `~/.claude/settings.json` 中注册：

```json
{
  "statusLine": { "type": "command", "command": "ccstatuskit" }
}
```

完整配置说明见 [docs/configuration.zh-CN.md](docs/configuration.zh-CN.md)，编写自定义模块见 [docs/module-contract.zh-CN.md](docs/module-contract.zh-CN.md)。

## 内置模块

| 模块 | 显示内容 | 数据来源 |
| --- | --- | --- |
| `model` | 模型名称（按系列着色） | stdin |
| `directory` | 相对于项目的路径 | stdin |
| `git` | 分支、工作区状态、merge/rebase、ahead/behind | `git` CLI |
| `memory` | 系统内存用量 | procfs / sysctl / WinAPI |
| `ctx` | 上下文窗口使用率（按阈值着色） | stdin |
| `time` | 当前时间与会话时长 | 系统时钟 |
| `project` | 项目类型图标与版本（Unity、Node、Rust、Go 等） | 标记文件 |
| `usage` | Claude 配额（5 小时、每周、按模型） | stdin（+ 可选 API） |

所有模块都支持 `disabled = true` 与 `style = "bold fg:#A6E22E"`（starship 风格的样式字符串）。
命名颜色可在 `[palette]` 中定义。

## 配额显示

默认的 `$usage` 显示 Claude Code 通过 stdin 传入的官方 `rate_limits` 数据（5 小时与 7 天窗口，Pro/Max 订阅）。
加上以下配置后，还会显示按模型划分的每周配额（如 Fable）：

```toml
[usage]
scoped = true
```

按模型配额来自 Claude Code 自身使用的同一非官方端点，经由 stale-while-revalidate 磁盘缓存读取。
渲染永远不会等待网络，OAuth token 也不会出现在任何输出或日志中。
**该数据源属于非官方接口，可能随时失效。**
`only = [...]` 可筛选要显示的窗口：`"five_hour"`、`"seven_day"`，或如 `"fable"` 的模型名子串。

## 环境要求

- 默认图标需要 [Nerd Font](https://www.nerdfonts.com/) **v3**。没有它也不会出问题：图标退化为占位方块（□），而所有信息（模型名、路径、百分比、分支名）仍以文本形式可读。✓ ✎ ⚠ 等状态符号是标准 Unicode，任何现代字体都能显示
- Windows 开箱即用；`[custom.x]` 模块默认使用 `cmd /C`，装有 Git Bash 时可指定 `shell = ["sh", "-c"]`

## 许可证

[Apache License, Version 2.0](LICENSE-APACHE) 与 [MIT license](LICENSE-MIT) 双许可，任选其一。
