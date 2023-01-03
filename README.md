# Zero to Production in Rust

[Zero To Production In Rust](https://www.zero2prod.com/index.html?country=Japan&discount_code=VAT20) のコード写経する

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
