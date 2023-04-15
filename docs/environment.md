# 環境構築

## 前提条件

```bahs
$ rustc --version
rustc 1.68.2 (9eb3afe9e 2023-03-27)

$ cargo --version
cargo 1.68.2 (6feb7c9cf 2023-03-26)
```

`Rust` のバージョンアップを行いたい場合は以下のコマンドを実行する。

```bash
$ rustup update
```

まずは Cargo を使用してプロジェクトをセットアップする。

```bash
$ cargo new zero2prod
```

## VSCode 設定

拡張機能とフォーマッター周りの設定を行なっておく。

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "esbenp.prettier-vscode",
    "foxundermoon.shell-format",
    "adpyke.vscode-sql-formatter",
    "IronGeek.vscode-env",
    "ms-azuretools.vscode-docker",
    "github.vscode-github-actions"
  ]
}
```

```json
{
  // 各言語に合わせてフォーマッター設定
  "editor.defaultFormatter": "esbenp.prettier-vscode",
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "[shellscript]": {
    "editor.defaultFormatter": "foxundermoon.shell-format"
  },
  "[sql]": {
    "editor.defaultFormatter": "clarkyu.vscode-sql-beautify"
  },
  "[dockerfile]": {
    "editor.defaultFormatter": "ms-azuretools.vscode-docker"
  }
  // rust-analyzerに関する設定
}
```

## Rust 周りのツール設定

### ファイルの変更検知

ファイルの変更を検知して自動的に指定したコマンドを実行するツールを導入する。

```bash
$ cargo install cargo-watch

# ファイルの変更を検知したタイミングで①静的解析、②テスト、③コード実行を行う
$ cargo watch -x check -x test -x run
```

### コードカバレッジ

Rust でテストを実行した時にコードカバレッジを計算するためのツールを導入する。

```bash
$ cargo install cargo-tarpaulin

# テストコードを省略してプロダクションコードのカバレッジを計算する
$ cargo tarpaulin --ignore-tests
```

### 静的解析ツール

```bash
$ rustup component add Clippy

# 静的解析を実行
$ cargo clippy

# 警告がはかれた場合には失敗させる
$ cargo clippy -- -D warnings
```

### フォーマッター

```bash
$ rustup component add rustfmt

# フォーマッターを実行
$ cargo fmt

# フォーマッターによるチェックのみを行う
$ cargo fmt -- --check
```

### 脆弱性検査

Cargo で管理しているクレーとに対して脆弱性検査を行う。

```bash
$ cargo install cargo-audit

# 脆弱性検査を実行
$ cargo audit
```

## git 管理用ファイル

`.gitignore` ファイルに関しては [gitignore.io](https://www.toptal.com/developers/gitignore) を利用して取得したものをそのまま利用する。

ただし、デフォルト設定では `Cargo.lock` は管理外のファイルとして識別されてしまうため、対象箇所に関してコメントアウトを行う。

- [Why do binaries have Cargo.lock in version control, but not libraries?](https://doc.rust-lang.org/cargo/faq.html#why-do-binaries-have-cargolock-in-version-control-but-not-libraries)

> 他の Cargo パッケージなどから利用されるライブラリの場合、対象ライブラリを利用しているユーザーに対して、決定論的に再コンパイルできるように `Cargo.lock` は管理対象外にする必要がある
>
> 実際に `serde` のようなライブラリを見ると、 `Cargo.lock` が管理対象に含まれていないことがわかる
> https://github.com/serde-rs/serde
