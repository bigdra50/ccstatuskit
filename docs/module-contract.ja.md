# 外部モジュール契約 (v1)

[English](module-contract.md) | 日本語 | [简体中文](module-contract.zh-CN.md)

ccstatuskit のモジュールは任意の実行ファイルである。
`[custom.<name>]` で宣言し、`format` テンプレートに `${custom.<name>}` を置く。

```toml
format = "$model $git ${custom.unilyze}"

[custom.unilyze]
command = "unilyze statusline"
when_dir = ["Assets"]          # プロジェクトディレクトリにこのパスがあるときだけ実行
# shell = ["bash", "-c"]       # 既定: ["sh", "-c"] (unix) / ["cmd", "/C"] (windows)
# style = "bold fg:#A6E22E"    # 任意。プレーンテキスト出力を包む
# disabled = true
```

## 入力

| チャネル | 内容 |
| --- | --- |
| stdin | Claude Code の statusline JSON。ccstatuskit が受け取ったバイト列をそのまま渡す。未知のフィールドが現れても無視すること。 |
| `CCSK_CONTRACT` | 契約バージョン。現在は `1`。この値が上がるのは破壊的変更のとき。 |
| `CCSK_MODEL` | モデル表示名（例 `Opus`）。不明なら未設定。 |
| `CCSK_MODEL_ID` | モデル ID（例 `claude-opus-4-8`）。 |
| `CCSK_CWD` | Claude Code のカレントディレクトリ。 |
| `CCSK_CURRENT_DIR` | `workspace.current_dir`。 |
| `CCSK_PROJECT_DIR` | `workspace.project_dir`。 |
| `CCSK_SESSION_ID` | セッションごとに安定した ID。 |
| `CCSK_TRANSCRIPT_PATH` | セッションの transcript JSONL のパス。 |
| `CCSK_VERSION` | Claude Code のバージョン。 |
| `CCSK_CTX_PCT` | コンテキストウィンドウ使用率（整数パーセント）。 |

どの `CCSK_*` 変数も、元の JSON フィールドが存在するときだけ設定される。
簡単なモジュールは環境変数で済ませ、それ以上が必要なときに stdin をパースする。

## 出力

- **stdout の1行目がセグメントになる。** 2行目以降は無視される。
- **空出力は「非表示」を意味する。** 表示するものがないときに何も出力しないのが、条件付き表示の作法である。
- ANSI エスケープは使ってよい。設定側に `style` があると ccstatuskit が（プレーンな）出力を包むので、両方は併用しない。
- UTF-8 で書く。statusline に載るものなので、短い1セグメントに収める。

## 失敗時の扱い (fail-soft)

| 事象 | 結果 |
| --- | --- |
| 非0 exit | セグメント非表示（stdout は破棄） |
| `command_timeout`（既定 250ms）以内に出力なし | プロセスを kill してセグメント非表示 |
| クラッシュ、起動不能なコマンド | セグメント非表示 |

モジュールが失うのは自分のセグメントだけである。
statusline を壊すことも、deadline を超えてプロセスが生き残ることもない。
データ源が遅い場合はモジュール側でキャッシュし（stale-while-revalidate が向いている）、キャッシュ値を即座に返すこと。

## 互換性のルール

1. stdin の JSON は無加工で渡される。Claude Code 側のスキーマ変更はそのまま届くので、防御的にパースすること。
2. 契約 v1 の範囲内で `CCSK_*` 変数が追加されることはあるが、既存の変数の意味が変わったり消えたりすることはない。
3. 上記のルールに破壊的変更が入るときは `CCSK_CONTRACT` が上がる。

## 例: 最小の bash モジュール

```bash
#!/usr/bin/env bash
# セッションに open な PR があるときだけ PR 番号を表示する。
number=$(jq -r '.pr.number // empty' 2>/dev/null)   # stdin を読む
[ -n "$number" ] && printf '#%s' "$number"           # 空出力 = 非表示
```

どの言語でも書き方は同じで、stdin か `CCSK_*` を読み、1行出力する。
