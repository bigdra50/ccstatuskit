# 設定リファレンス

[English](configuration.md) | 日本語 | [简体中文](configuration.zh-CN.md)

ccstatuskit は `$XDG_CONFIG_HOME/ccstatuskit/config.toml`
（既定では `~/.config/ccstatuskit/config.toml`）を読む。
環境変数 `CCSTATUSKIT_CONFIG` で別のパスを指定できる。

設定ファイルがなければ組み込みのデフォルトで動く。
設定が*壊れている*場合もデフォルトにフォールバックする。
設定ミスで statusline が消えることはないが、TOML が直るまでレイアウトはデフォルトに戻る（stderr に警告が出る）。

## レイアウト: `format` テンプレート

```toml
format = """
$model $directory $git
$ctx $usage
"""
```

- `$name` と `${name}` は builtin モジュールを、`${custom.x}` は
  `[custom.x]` エントリを参照する（ドットには波括弧が必要）。
- テンプレートの各行が statusline の行になる。行は宣言するものであり、
  位置指定するものではない。行番号は存在しない。
- 非表示のモジュールは行から詰めて消える。全モジュールが隠れた行
  （リテラルが空白のみの行）は行ごと消える。
- `\$` はリテラルのドル記号、`\\` はリテラルのバックスラッシュになる。
  未知のモジュール名は黙って非表示になるため、新しいバージョン向けの
  設定でも古いバイナリで安全に動く。
- 未設定時のデフォルト:
  `$model $directory $memory $ctx $time $git\n$project $usage`

## builtin モジュール

| モジュール | セグメント | 補足 |
| --- | --- | --- |
| `model` | モデル名（系統別の色付き） | stdin にモデル情報がなければ非表示 |
| `directory` | プロジェクト相対パス | なければ cwd の basename |
| `git` | ブランチ、変更状態、merge/rebase、ahead/behind | `GIT_OPTIONAL_LOCKS=0` で `git` を実行。repo 外では非表示 |
| `memory` | システムメモリ使用量（GB） | >60% / >80% で色が変わる |
| `ctx` | コンテキストウィンドウ使用率 | ≥40% / ≥60% で色が変わる。Claude Code が報告するまで非表示 |
| `time` | 現在時刻とセッション経過時間 | 経過時間は `cost.total_duration_ms` から |
| `project` | プロジェクト種別アイコンとバージョン | Unity、Node/React/Vue/Next、Rust、Go、Python、.NET、Ruby、Java、Kotlin、PHP、Swift |
| `usage` | Claude 利用枠 | 後述 |

どの builtin も共通オプションを2つ受け付ける。

```toml
[git]
disabled = true              # モジュールを外す

[model]
style = "bold fg:#FF79C6"    # デフォルトの色を上書き
```

## style と palette

style 文字列は starship と同じ形式で、空白区切りのトークンを並べる。

- 属性: `bold`、`dimmed`、`italic`、`underline`、`inverted`
- 色: `fg:` / `bg:` に `#rrggbb`、0〜255 の ANSI 番号、または色名
  （`black`、`red`、`green`、`yellow`、`blue`、`purple`、`cyan`、`white` と
  `bright-` 系。`magenta` は `purple` の別名）

名前付きの色は `[palette]` テーブルで定義する。

```toml
[palette]
hot = "#FF5555"

[ctx]
style = "bold fg:hot"
```

## 利用枠の表示

```toml
[usage]
scoped = true            # モデル別枠も取得（非公式API、opt-in）
only = ["fable"]         # 表示する枠を絞る
cache_ttl = 120          # scoped キャッシュの鮮度（秒）
```

`scoped` なしでは、Claude Code が stdin に渡す公式の `rate_limits`
（5時間枠と7日枠、Pro/Max プラン）を表示し、ネットワークには一切触れない。

`scoped = true` にすると、モデル別の週次枠（Fable など）を
Claude Code 自身が使うのと同じ非公式エンドポイントから取得する。
切り離された `ccstatuskit --refresh-usage` プロセスが
stale-while-revalidate 方式のディスクキャッシュを更新し、
描画はディスクを読むだけである。
**scoped の取得元は非公式であり、予告なく壊れることがある。**

`only` に指定できるのは、`"five_hour"`（別名 `"5h"`、`"session"`）、
`"seven_day"`（別名 `"wk"`、`"weekly"`）、または `"fable"` のような
モデル名の部分一致（大文字小文字を区別しない）。空リストは全表示。

高度な設定: `auto_refresh = false` でバックグラウンド更新の起動を止め、
`cache_dir` でキャッシュの場所を変えられる
（既定は `$XDG_CACHE_HOME/ccstatuskit`）。

## カスタムモジュール

[モジュール契約](module-contract.ja.md)に従えば、任意の実行ファイルがモジュールになる。

```toml
format = "$model ${custom.pr}"

[custom.pr]
command = "jq -r '.pr.number // empty' | sed 's/^/#/'"
style = "fg:cyan"
when_dir = [".git"]      # このパスが project dir にあるときだけ実行
# shell = ["bash", "-c"] # 既定: ["sh", "-c"] (unix) / ["cmd", "/C"] (windows)
# disabled = true
```

子プロセスは stdin の JSON と `CCSK_*` 環境変数を受け取り、
stdout の1行目がセグメントになる。空出力は非表示を意味する。
副作用だけが目的のモジュール（マーカーファイルの書き込みなど)も正当な使い方で、
何も出力しなければ画面には一切現れない。

## グローバルオプション

```toml
command_timeout = 250    # 全モジュール共通の実時間予算（ms）
```

環境変数 `NO_COLOR` を設定するとすべての装飾が消える。

## エラー時の挙動

| 事象 | 結果 |
| --- | --- |
| モジュールの panic、エラー、timeout | そのセグメントだけ省略され、行は生き残る |
| stdin の JSON が不正 | パースできた分だけ描画し、exit 0 を維持 |
| 設定 TOML が壊れている | デフォルトで動作し、stderr に警告 |
| `format` 内の未知のモジュール名 | 黙って非表示 |

## 設定例（全体）

```toml
# 1行目: セッション状態 / 2行目: Claude 情報 / 3行目: 開発環境。
# 認識できるプロジェクトの外では3行目が自動で消える。
format = """
$directory $memory $time $git
$model $ctx $usage
$project ${custom.unilyze}
"""

[usage]
scoped = true

[custom.unilyze]
command = "unilyze statusline"
when_dir = ["Assets"]
```

変更は次の描画から反映される。
描画のたびに新しいプロセスが起動するので、再起動の類いは不要である。
