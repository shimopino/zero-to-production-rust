# Zero to Production in Rust

[Zero To Production In Rust](https://www.zero2prod.com/index.html?country=Japan&discount_code=VAT20) のコード写経する

- [Zero to Production in Rust](#zero-to-production-in-rust)
  - [環境設定](#環境設定)
  - [API の仕様](#api-の仕様)
  - [CI の検証](#ci-の検証)

## 環境設定

本リポジトリでの環境設定は下記の状態である

```bash
>> rustc --version
rustc 1.66.0 (69f9c33d7 2022-12-12)

>> cargo --version
cargo 1.66.0 (d65d197ad 2022-11-15)
```

本書では IDE として「IntelliJ Rust」を推奨しているが、本リポジトリでは VSCode と rust-analyzer の拡張機能を使用する

ソースコードが変更されるたびにコンパイルなどを実行するために `cargo-watch` を使用する

```bash
# https://crates.io/crates/cargo-watch
cargo install cargo-watch

# ソースコードが変更された時にどのコマンドを順番に実行するのか指定する
cargo watch -x check -x test -x run
```

テストコードのコードカバレッジを計算するために `cargo-tarpaulin` を使用する

```bash
# https://crates.io/crates/cargo-tarpaulin
cargo install cargo-tarpaulin

# テストコードを無視して、アプリケーションコードのみのカバレッジを計算する
cargo tarpaulin --ignore-tests
```

クレートの脆弱性を検査するために `cargo-audit` を使用する

```bash
# https://crates.io/crates/cargo-audit
cargo install cargo-audit

# 脆弱性を検査する
cargo audit
```

[`cargo-deny`](https://github.com/EmbarkStudios/cargo-deny) だと脆弱性の検査に加えて、ライセンスの制限なども含めてさまざまなことを実施できる

## API の仕様

本書では実装する対象をユーザーストーリーという形式で提供している

- ブログの訪問者として、ニュースレターを購読したい、なぜなら、ブログに新しい記事が投稿されたことを知りたいからだ
- ブログの著者として、購読者に対してメールを送信したい、なぜなら、新しい記事を執筆したことを購読者に教えたいからだ

はじめから上記の仕様を満たした上で非機能的な内容も実装するのではなく、最初は仕様をある程度満たすようにサービスを構築していき、徐々に耐障害性やリトライ機能の追加、新規購読者への確認メールなどを追加していく

## CI の検証

CI パイプラインで使用する Github Actions をローカルで検証するために [nektos/act](https://github.com/nektos/act) を使用する

```bash
# nektos/act をインストールする
brew install act

# 実行できるパイプラインの一覧を表示する
act -l

# 実行
# 何も指定しなければ push イベントで実行する
act

# 特定のイベントで実行
act pull_request

# 特定のジョブを実行
act -j test
```
