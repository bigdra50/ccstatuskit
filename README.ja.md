# ccstatuskit

[English](README.md) | 日本語 | [简体中文](README.zh-CN.md)

[Claude Code](https://code.claude.com) の statusline を部品から組み立てる、starship 方式のモジュラーキット。
TOML テンプレートでレイアウトを宣言し、builtin モジュールと任意の外部コマンドを組み合わせる。
Rust 製の単一バイナリで、ランタイム依存はない。

```toml
# ~/.config/ccstatuskit/config.toml
format = """
$model $directory $memory $ctx $time $git
$project $usage ${custom.unilyze}
"""

[usage]
scoped = true
only = ["fable"]          # 「Fable 14%」だけを表示するなど、欲しい枠だけを選べる

[custom.unilyze]
command = "unilyze statusline"
when_dir = ["Assets"]     # Unity プロジェクトでのみ実行
```

## なぜ作ったか

Claude Code に登録できる statusline はコマンド1つだけである。
既存の statusline はどれも一枚岩で、誰かの使用量メーターだけが欲しくても、そのスクリプト全体を採用するしかない。
ccstatuskit はこの問題を3つの仕組みで分割する。

- **builtin モジュール**はプロセス内で並列に実行され、それぞれが自分のタイムアウトを持つ。
- **モジュール契約**（`[custom.x]`）は任意の実行ファイルをモジュールとして扱う。
  stdin に Claude Code の JSON が素通しで渡り、`CCSK_*` 環境変数にパース済みの基本情報が入る。
  stdout の1行目がセグメントになり、空出力は「非表示」を意味する。
  bash でも Python でも、どの言語でも書ける。詳細は [docs/module-contract.md](docs/module-contract.md)。
- **fail-soft**：壊れたモジュールや遅いモジュールは自分のセグメントを失うだけで、行は壊れない。
  不正な入力、壊れた設定、スキーマ変更のいずれが起きても、statusline は描画を続け、必ず exit 0 する。

行は絶対位置ではなく、テンプレートの行として宣言する。
非表示になったモジュールは詰めて消え、全モジュールが隠れた行は行ごと消える。

## インストール

```sh
cargo install ccstatuskit          # または GitHub Releases のバイナリを取得
```

`~/.claude/settings.json` で Claude Code に登録する。

```json
{
  "statusLine": { "type": "command", "command": "ccstatuskit" }
}
```

## builtin モジュール

| モジュール | 表示内容 | データ源 |
| --- | --- | --- |
| `model` | モデル名（系統別の色付き） | stdin |
| `directory` | プロジェクト相対パス | stdin |
| `git` | ブランチ、変更状態、merge/rebase、ahead/behind | `git` CLI |
| `memory` | システムメモリ使用量 | procfs / sysctl / WinAPI |
| `ctx` | コンテキストウィンドウ使用率（閾値で色分け） | stdin |
| `time` | 現在時刻とセッション経過時間 | システム時計 |
| `project` | プロジェクト種別アイコンとバージョン（Unity、Node、Rust、Go など） | マーカーファイル |
| `usage` | Claude 利用枠（5h、週次、モデル別） | stdin（+ 任意で API） |

どのモジュールも `disabled = true` と `style = "bold fg:#A6E22E"`（starship 互換の style 文字列）を受け付ける。
名前付きの色は `[palette]` で定義できる。

## 利用枠の表示

デフォルトの `$usage` は、Claude Code が stdin に渡す公式の `rate_limits`（5時間枠と7日枠、Pro/Max プラン）を表示する。
次の設定を加えると、モデル別の週次枠（Fable など）も表示する。

```toml
[usage]
scoped = true
```

モデル別枠は Claude Code 自身が使うのと同じ非公式エンドポイントから取得し、stale-while-revalidate 方式のディスクキャッシュ経由で読む。
描画がネットワークを待つことはなく、OAuth トークンが出力やログに現れることもない。
**この取得元は非公式であり、予告なく壊れることがある。**
`only = [...]` で表示する枠を絞れる。指定できるのは `"five_hour"`、`"seven_day"`、または `"fable"` のようなモデル名の部分一致である。

## 動作要件

- デフォルトのアイコン表示には [Nerd Font](https://www.nerdfonts.com/) が必要（モジュール別の差し替えは今後のリリースで対応予定）
- Windows はそのまま動く。`[custom.x]` モジュールの既定シェルは `cmd /C` なので、Git Bash があるなら `shell = ["sh", "-c"]` を指定する

## ライセンス

[Apache License, Version 2.0](LICENSE-APACHE) と [MIT license](LICENSE-MIT) のデュアルライセンス。どちらかを選択できる。
