# エラーハンドリング

- [エラーハンドリング](#エラーハンドリング)
  - [Rust におけるエラーハンドリングの基本](#rust-におけるエラーハンドリングの基本)
    - [Result 型とは](#result-型とは)
    - [Err 型で自作型を返却する](#err-型で自作型を返却する)
    - [Error トレイトを実装する](#error-トレイトを実装する)
    - [複数のエラー型の組み合わせ](#複数のエラー型の組み合わせ)
  - [thiserror クレート](#thiserror-クレート)
  - [anyhow クレート](#anyhow-クレート)
  - [各種クレートでのエラーハンドリング](#各種クレートでのエラーハンドリング)
    - [sqlx での使い方](#sqlx-での使い方)
    - [reqwest での使い方](#reqwest-での使い方)
  - [axum との組み合わせ](#axum-との組み合わせ)
  - [参考資料](#参考資料)

## Rust におけるエラーハンドリングの基本

### Result 型とは

Rust ではエラー処理を取り扱いたい時に利用できる `Result` 型が標準ライブラリから提供されている。

標準ライブラリの `std::result` モジュールでは `Result` トレイトが定義されており、以下のように成功した時の返り値と失敗した時の返り値が表現されている。

```rs
// https://doc.rust-lang.org/std/result/index.html
enum Result<T, E> {
   Ok(T),
   Err(E),
}
```

例えば整数の割り算を行う関数を考えると、0 で割り算を行おうとした場合には失敗を表現し、それ以外の場合では成功を表現することで、この関数が返しうる全ての範囲を表現することが可能となる。

```rs
fn divide(numerator: i32, denominator: i32) -> Result<i32, String> {
    if denominator == 0 {
        Err("Divide by 0".to_string())
    } else {
        Ok(numerator / denominator)
    }
}
```

実際に以下のように成功時の型と失敗時の型で計算結果を取得していることがわかる。

```rs
fn main() {
    // Result<i32, String>
    let success = divide(10, 2);
    assert_eq!(success, Ok(5));

    // Result<i32, String>
    let failure = divide(10, 0);
    assert_eq!(failure, Err("Divide by 0".to_string()));
}
```

Rust のパターンマッチングの機能と早期 return の機能を利用すれば、以下のように成功時には中身の値を取り出して変数に代入し、失敗時には即座に結果を関数から返却することが可能となる。

```rs
fn early_return() -> Result<(), String> {
    let value = match divide(10, 5) {
        // 成功時には中身を取り出して変数に代入する
        Ok(value) => value,
        // 失敗時にはこの時点で、結果を関数から返却する
        Err(e) => return Err(e),
    };

    // 値は 2 であり中身が取り出されている
    println!("値は {} であり中身が取り出されている", value);

    Ok(())
}
```

Rust には [`?`](https://doc.rust-lang.org/std/result/index.html#the-question-mark-operator-) というシンタックスシュガーが用意されており、上記で実行した内容をよりシンプルな構文で再現することができる

```rs
fn early_return() -> Result<(), String> {
    // 成功時には中身を取り出して変数に代入する
    // 失敗時にはこの時点で、結果を関数から返却する
    let value = divide(10, 5)?;
    assert_eq!(value, 2);

    // 後続処理では value: i32 のままで値を利用することができる
    // ...

    Ok(())
}
```

### Err 型で自作型を返却する

作成した `divide` 関数の返却値の型は `Result<i32, String>` となっているが、全ての関数の返り値をこのように設計した場合、呼び出し元では型を見てもどのようなエラーが発生する可能性があるのか把握することができない。

そのため以下のような失敗時の専用の型を用意して、明確に他の返り値を分離させることができる。

```rs
struct DivideByZero;
```

この型を使用すれば以下のように返り値の型を明確に表現することが可能となる

```rs
/// 呼び出し元は DivideByZero という型からどのようなエラーが発生する可能性があるのか把握できる
fn divide(numerator: i32, denominator: i32) -> Result<i32, DivideByZero> {
    if denominator == 0 {
        Err(DivideByZero)
    } else {
        Ok(numerator / denominator)
    }
}
```

しかし元々この関数を呼び出していた `early_return` 関数は、返り値の型と関数が返す型が合わない状態になってしまうためコンパイルエラーが発生してしまう。

```rs
fn early_return() -> Result<(), String> {
    // 型が合わない
    let value = divide(10, 5)?;
    assert_eq!(value, 2);

    Ok(())
}
```

実際にエラーを確認すると、以下のように型変換ができないためコンパイルエラーが発生していることがわかる。

```bash
error[E0277]: `?` couldn't convert the error to `String`
 --> examples/main.rs:6:30
  |
5 | fn early_return() -> Result<(), String> {
  |                      ------------------ expected `String` because of this
6 |     let value = divide(10, 5)?;
  |                              ^ the trait `From<DivideByZero>` is not implemented for `String`
```

ここでは `early_return` 関数の返り値の型を修正することでも対応できるが、ここでは 1 歩このコンパイルエラーを噛み砕いてみる。

`From` トレイトを実装していないと表示されているが、シンタックスシュガーである `?` を使用すると、型推論をもとに暗黙的に [`From`](https://doc.rust-lang.org/std/convert/trait.From.html) トレイトの実装を呼び出している。

このトレイトを活用して型変換を行うことで、さまざまな関数を組み合わせることが可能となる。

今回は `DivideByZero` という自作した型を `String` 型に変換する実装を以下のように追加する。

```rs
struct DivideByZero;

impl From<DivideByZero> for String {
    // 値を consume する
    fn from(_value: DivideByZero) -> Self {
        println!("convert DivideByZero to 'Divide by 0' String");
        "Divide By 0".to_string()
    }
}
```

こうすることで暗黙的に `from` メソッドが実行され、以下のように呼び出しもとに変換されたエラーが返却されていることがわかる。

```rs
fn main() {
    let result = early_return();
    // from によって変換された値が返ってきていることがわかる
    assert_eq!(result, Err("Divide By 0".to_string()));
}

fn early_return() -> Result<(), String> {
    // 暗黙的に DivideByZero -> String に変換するための from メソッドが呼ばれる
    let value = divide(10, 0)?;
    assert_eq!(value, 2);

    Ok(())
}
```

なお `From` トレイトを実装することで自動的に `Into` トレイトも実装されるため、以下のように型変換を行うことも可能になっている。

```rs
let sample: String = DivideByZero.into();
```

これは以下のように `from` が呼ばれていることと同義である。

```rs
fn early_return() -> Result<(), String> {
    let value = match divide(10, 0) {
        Ok(value) => value,
        // e: DivideByZero と型推論される
        // そのため自動的に DivideByZero の from 実装が呼び出される
        Err(e) => return Err(From::from(e)),
    };

    Ok(())
}
```

これで自作した型を Result 型に適用したり、異なる型同士で型変換を行う方法がわかった。

### Error トレイトを実装する

Result 型の `E` で指定されている型には文字列や自作した型を指定することもできるが、標準ライブラリが提供している [`Error`](https://doc.rust-lang.org/std/error/trait.Error.html) トレイトを実装したものを使用するのが一般的な慣習である。

このトレイトは以下のように定義されており、 `Debug` トレイトや `Display` トレイトが境界として定義されているため実装が必要となる。

また `source` メソッドが定義されており、このメソッドを使用することでエラーの原因をつ移籍することが可能となる。デフォルト実装が提供されているため実装する必要はないが、内部エラーをラップしている場合にはオーバーライドすることが推奨されている。

```rs
pub trait Error: Debug + Display {
    // Provided methods
    fn source(&self) -> Option<&(dyn Error + 'static)> { ... }
    fn description(&self) -> &str { ... }
    fn cause(&self) -> Option<&dyn Error> { ... }
    fn provide<'a>(&'a self, demand: &mut Demand<'a>) { ... }
}
```

この `Error` トレイトを利用することで `println!("{}", e)` や `println!("{:?}", e)` などの出力を利用することができ、利用するユーザーにとって使いやすいものとなる。

```rs
#[derive(Debug)]
struct DivideByZero;

impl std::error::Error for DivideByZero {}

impl std::fmt::Display for DivideByZero {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Divided by 0")
    }
}
```

これで以下のように `From` トレイトの実装を変更して実際に標準出力にエラーを表示してみると、実装した `Display` トレイトの内容が反映されていることがわかる。

```rs
impl From<DivideByZero> for String {
    fn from(value: DivideByZero) -> Self {
        // これは以下のように出力される
        // Display: [DividedByZero] Divided by 0, Debug: DivideByZero
        println!("Display: {}, Debug: {:?}", value, value);

        "Divide By 0".to_string()
    }
}
```

### 複数のエラー型の組み合わせ

アプリケーション全体でエラー型を作成する時には、サードパーティのクレートで定義されているエラー型なども `enum` で一部のエラーとして表現する場合もある。

その場合には `From` トレイトなどを使用してアプリケーション全体の型に変換することもできる。

```rs
// 例えば以下で定義しているErrorが、sqlx::Error だったり reqwest::Error だったりする
#[derive(Debug)]
struct CustomErrorType1;

#[derive(Debug)]
struct CustomErrorType2;

#[derive(Debug)]
enum ApplicationError {
    Type1(CustomErrorType1),
    Type2(CustomErrorType2),
}

impl std::error::Error for ApplicationError {}

impl Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplicationError::Type1(_) => write!(f, "Error type 1"),
            ApplicationError::Type2(_) => write!(f, "Error type 2"),
        }
    }
}
```

例えば関数の中では以下のように `CustomErrorType1` を返すようなものもあるかもしれない。

```rs
fn some_function_custom_error_1() -> Result<i32, CustomErrorType1> {
    // ...
}
```

この関数を以下のように利用してもそのままでは型変換できずにコンパイルエラーになってしまう。

```rs
fn main() -> Result<(), ApplicationError> {
    // 以下の関数では CustomErrorType1 がエラーとして返却される
    let result = some_function_custom_error_1()?;

    // ...
}
```

このような場合にはそれぞれの型に対して `From` トレイトを実装して型推論から暗黙的に型変換のための関数を呼び出すようにすればいい。

```rs
impl From<CustomErrorType1> for ApplicationError {
    fn from(error: CustomErrorType1) -> Self {
        ApplicationError::Type1(error)
    }
}

impl From<CustomErrorType2> for ApplicationError {
    fn from(error: CustomErrorType2) -> Self {
        ApplicationError::Type2(error)
    }
}
```

こうすればコンパイルエラーが発生することなく、エラーの型を変換することができる。

## thiserror クレート

独自に型を定義したエラー型を作成する際には、各種トレイトなどのボイラープレートを記述する必要があり、アプリケーションの規模が拡大していくとエラー型の管理が大変になってしまう。

`thiserror` クレートはこうしたボイラープレートの実装の手間を減らし、失敗した時に呼び出し元が選択した情報を正確に受け取れるようにすることを重視する時に利用できる。ライブラリなどの呼び出し元が不特定多数であり、可能な限り失敗した原因をユーザーに伝えたい場合などで使用する。

公式ページに記載されている以下のサンプルコードを見てみる。

```rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataStoreError {
    #[error("data store disconnected")]
    Disconnect(#[from] std::io::Error),
    #[error("the data for key `{0}` is not available")]
    Redaction(String),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader {
        expected: String,
        found: String,
    },
    #[error("unknown data store error")]
    Unknown,
}
```

マクロでさまざまな定義を行なっているが [cargo-expand](https://github.com/dtolnay/cargo-expand) を利用してどのようなコードが展開されているのか確認する。

展開した内容をみると以下のように今まで自前で定義していた `Error` トレイトの実装が自動的に追加されていることがわかる。

```rs
impl std::error::Error for DataStoreError {
    fn source(&self) -> std::option::Option<&(dyn std::error::Error + 'static)> {
        use thiserror::__private::AsDynError;
        #[allow(deprecated)]
        match self {
            DataStoreError::Disconnect { 0: source, .. } => {
                std::option::Option::Some(source.as_dyn_error())
            }
            DataStoreError::Redaction { .. } => std::option::Option::None,
            DataStoreError::InvalidHeader { .. } => std::option::Option::None,
            DataStoreError::Unknown { .. } => std::option::Option::None,
        }
    }
}
```

`Error` トレイトでは `source()` メソッドは `#[source]` 属性を有するフィールドを、下位レベルのエラーとして指定する。

今回 `#[source]` 属性を指定していないが、 `#[from]` 属性を付与すると `From` トレイトの実装だけではなく、暗黙的に `#[source]` と同じフィールドだと識別される。

実際に以下のように指定した属性に対して `From` トレイトが実装されていることがわかる。

```rs
impl std::convert::From<std::io::Error> for DataStoreError {
    #[allow(deprecated)]
    fn from(source: std::io::Error) -> Self {
        DataStoreError::Disconnect {
            0: source,
        }
    }
}
```

`#[error("...")]` では `Display` トレイトに対してどのような実装を行うのかを指定することができ、今回では以下のようにタプルで指定した値を表示したり、指定した属性の値を `Debug` で出力するような設定が組み込まれていることがわかる

```rs
impl std::fmt::Display for DataStoreError {
    fn fmt(&self, __formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        #[allow(unused_imports)]
        use thiserror::__private::{DisplayAsDisplay, PathAsDisplay};
        #[allow(unused_variables, deprecated, clippy::used_underscore_binding)]
        match self {
            DataStoreError::Disconnect(_0) => {
                __formatter.write_fmt(format_args!("data store disconnected"))
            }
            DataStoreError::Redaction(_0) => {
                __formatter
                    .write_fmt(
                        format_args!(
                            "the data for key `{0}` is not available", _0.as_display()
                        ),
                    )
            }
            DataStoreError::InvalidHeader { expected, found } => {
                __formatter
                    .write_fmt(
                        format_args!(
                            "invalid header (expected {0:?}, found {1:?})", expected,
                            found
                        ),
                    )
            }
            DataStoreError::Unknown {} => {
                __formatter.write_fmt(format_args!("unknown data store error"))
            }
        }
    }
}
```

このように `thiserror` クレートを利用することでエラー型を定義する時のボイラープレートを大幅に削減することができる。

また `Display` の実装は他の型で既に実装されているものを `#[error(transparent)]` で利用することができる。

通常は以下のように `#[error("...")]` を付与すると出力する文字列を調整することができる。

```rs
#[derive(Error, Debug)]
pub enum DataStoreError {
    #[error("data store disconnected")]
    Disconnect(#[from] std::io::Error),
}

impl std::fmt::Display for DataStoreError {
    fn fmt(&self, __formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DataStoreError::Disconnect(_0) => {
                __formatter.write_fmt(format_args!("data store disconnected"))
            },
            // ...
        }
    }
}
```

`#[error(transparent)]` を利用することで `Disconnect` が値として受け取ったものに対してそのまま `fmt` を呼び出してエラーメッセージの表示の機能を委譲していることがわかる。

```rs
#[derive(Error, Debug)]
pub enum DataStoreError {
    #[error(transparent)]
    Disconnect(#[from] std::io::Error),
}

impl std::fmt::Display for DataStoreError {
    fn fmt(&self, __formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DataStoreError::Disconnect(_0) => std::fmt::Display::fmt(_0, __formatter),
            // ...
        }
    }
}
```

- https://docs.rs/thiserror/latest/thiserror/
- https://github.com/dtolnay/thiserror

## anyhow クレート

[`anyhow`](https://docs.rs/anyhow/latest/anyhow/) を利用することで `std::error::Error` トレイトを実装しているエラーを簡単に伝播させることができる。

```rs
fn run() -> anyhow::Result<()> {
    let path = "./sample.txt";
    // std::io::Result<String> が返却される
    // type Result<T> = Result<T, std::error::Error>; で定義されている
    // https://doc.rust-lang.org/std/io/type.Result.html
    let data = std::fs::read_to_string(path)?;
    println!("File contents: {}", data);
    Ok(())
}
```

この関数を以下のように呼び出す。

```rs
fn main() {
    if let Err(e) = run() {
        println!("Error: {:?}", e);
    }
}
```

その結果出力されるエラー情報は以下のように簡素なものとなっており、これだけではエラーに関する情報が不足していることがわかる。

```rs
Error: No such file or directory (os error 2)
```

`anyhow` は他にも `Context` が提供されており、エラーにコンテキスト情報 w 含めることでより豊かなエラーメッセージを表示することが可能となる。

例えば以下のように `context` メソッドを利用すれば失敗した時のエラーメッセージを変化させることができる

```rs
fn run() -> anyhow::Result<()> {
    let path = "./sample.txt";
    let data = std::fs::read_to_string(path)
        .context(format!("Failed to read file: {}", path))?;
    println!("File contents: {}", data);
    Ok(())
}
```

この処理を実行してエラーを標準出力に出せば、以下のように指定したメッセージが表示されていることがわかる。

```bash
Error: Failed to read file: ./sample.txt
```

`context` メソッドは即時評価であるためコンテキストメッセージがすぐに作成されるが、 `with_context` メソッドを使用してクロージャを利用してメッセージを表示すると、遅延評価になるためメッセージ作成が高いコストの場合に利用できる。

```rs
fn run() -> anyhow::Result<()> {
    let path = "./sample.txt";
    let data = std::fs::read_to_string(path)
        // 以下のクロージャは遅延評価される
        .with_context(|| format!("Failed to read file: {}", path))?;
    println!("File contents: {}", data);
    Ok(())
}
```

標準ライブラリが提供している `Error` トレイトには `source` メソッドがありエラーがどこで発生したのかを辿ることができたが、 `anyhow` にも [`Chain`](https://docs.rs/anyhow/latest/anyhow/struct.Chain.html) という機能があり、以下のようにエラーを辿っていくことができる。

```rs
fn main() {
    if let Err(e) = run() {
        println!("Error: {}", e);
        // エラーを辿って出力することができる
        for cause in e.chain().skip(1) {
            println!("Caused by {}", cause);
        }
    }
}

fn run() -> anyhow::Result<()> {
    let path = "./sample.txt";
    let data = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path))?;
    println!("File contents: {}", data);
    Ok(())
}
```

これは以下のように `with_context` で指定したメッセージに加えて、元々のエラーで出力されるメッセージも表示できていることがわかる。

```bash
Error: Failed to read file: ./sample.txt
Caused by No such file or directory (os error 2)
```

`thiserror` で既に定義しているエラーが存在していた場合でも、以下のように `anyhow!` マクロを使用したり、 `anyhow::Error` に対して実装されている `into` メソッドを呼び出したりすることで、コンパイルエラーを発生させることなく関数を定義できる。

```rs
#[derive(Debug, thiserror::Error)]
enum ApplicationError {
    #[error("Divided By 0.")]
    DivivedByZero,
    #[error("Arguments are negative")]
    NegativeNumber,
}

fn calc(a: i32, b: i32) -> anyhow::Result<i32> {
    if b == 0 {
        Err(anyhow!(ApplicationError::DivivedByZero))
    } else if a < 0 {
        Err(ApplicationError::NegativeNumber.into())
    } else {
        Ok(a + b)
    }
}
```

ただしこのように定義すると呼び出し元では、型からどのようなエラーが発生するのか把握することはできなくなるため、エラー種別に応じてエラーハンドリングを行いたい場合は明示的に型定義を行う必要がある。

`anyhow!` マクロ以外にもより簡単に記述を行うことが可能な [`bail!`](https://docs.rs/anyhow/latest/anyhow/macro.bail.html) マクロも用意されている。このマクロは `return Err(anyhow!($args...))` を記述した時と同じ挙動となる。

```rs
fn calc(a: i32, b: i32) -> anyhow::Result<i32> {
    if b == 0 {
        bail!(ApplicationError::DivivedByZero)
    } else if a < 0 {
        bail!(ApplicationError::NegativeNumber)
    } else {
        Ok(a + b)
    }
}
```

条件文と返却するエラーを同時に指定することのできる [`ensure!`](https://docs.rs/anyhow/latest/anyhow/macro.ensure.html) マクロも用意されており、 `assert!` マクロにも似た機能を提供している。

```rs
fn calc(a: i32, b: i32) -> anyhow::Result<i32> {
    ensure!(b == 0, ApplicationError::DivivedByZero);
    ensure!(a < 0, ApplicationError::NegativeNumber);

    Ok(a + b)
}
```

`anyhow::Error` ではトレイト境界に `Send + Sync + 'static` が設定されているため、標準ライブラリの `Error` トレイトと同じように `is` や `downcast_ref` などのメソッドを使用することができ、呼び出し元で柔軟にエラーハンドリングすることが可能となる。

```rs
fn main() {
    let error = calc(10, -5).unwrap_err();
    assert!(error.is::<ApplicationError>());

    match error.downcast_ref::<ApplicationError>() {
        Some(ApplicationError::DivivedByZero) => {
            println!("Error is [DivivedByZero]")
        }
        Some(ApplicationError::NegativeNumber) => {
            println!("Error is [NegativeNumber]")
        }
        None => println!("not [ApplicationError]"),
    }
}

fn calc(a: i32, b: i32) -> anyhow::Result<i32> {
    ensure!(b == 0, ApplicationError::DivivedByZero);
    ensure!(a < 0, ApplicationError::NegativeNumber);

    Ok(a + b)
}
```

## 各種クレートでのエラーハンドリング

`thiserror` や `anyhow` を利用しているクレートを探したが、案外これらのクレートは使っておらずエラー型を自作して対応しているものが多かった。

### sqlx での使い方

sqlx では [sqlx-core/src/error.rs](https://github.com/launchbadge/sqlx/blob/main/sqlx-core/src/error.rs) で以下のように専用の Result 型や Error 型を定義している。

```rs
/// A specialized `Result` type for SQLx.
pub type Result<T, E = Error> = ::std::result::Result<T, E>;

// Convenience type alias for usage within SQLx.
// Do not make this type public.
pub type BoxDynError = Box<dyn StdError + 'static + Send + Sync>;

/// Represents all the ways a method can fail within SQLx.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Error occurred while parsing a connection string.
    #[error("error with configuration: {0}")]
    Configuration(#[source] BoxDynError),

    /// Error returned from the database.
    #[error("error returned from database: {0}")]
    Database(#[source] Box<dyn DatabaseError>),

    /// Error communicating with the database backend.
    #[error("error communicating with database: {0}")]
    Io(#[from] io::Error),

    // ...
}
```

こうしたエラーに関しては以下のように 1 つのエラー型しか取り扱わない場合には明示的にエラー型を `MigrateError` のように指定している。

```rs
fn validate_applied_migrations(
    applied_migrations: &[AppliedMigration],
    migrator: &Migrator,
    ignore_missing: bool,
) -> Result<(), MigrateError> {
    if ignore_missing {
        return Ok(());
    }

    let migrations: HashSet<_> = migrator.iter().map(|m| m.version).collect();

    for applied_migration in applied_migrations {
        if !migrations.contains(&applied_migration.version) {
            return Err(MigrateError::VersionMissing(applied_migration.version));
        }
    }

    Ok(())
}
```

あるいは以下のように `anyhow::Result` を利用して複数の関数を組み合わせるときに発生するであろう、コンパイルエラーを防ぐようにしている。この関数では `bail` マクロを利用して `MigrateError` を返却したり、 `sqlx-core` ライブラリで定義している `Error` を返却する関数を利用している。

```rs
pub async fn revert(
    migration_source: &str,
    connect_opts: &ConnectOpts,
    dry_run: bool,
    ignore_missing: bool,
) -> anyhow::Result<()> {
    let migrator = Migrator::new(Path::new(migration_source)).await?;
    let mut conn = crate::connect(&connect_opts).await?;

    conn.ensure_migrations_table().await?;

    let version = conn.dirty_version().await?;
    if let Some(version) = version {
        bail!(MigrateError::Dirty(version));
    }

    // ...
}
```

基本的には `core` で自作している `Error` 型を他の feature でも利用するような形式となっており、それ以外の型も組み合わせる時には `anyhow` を利用している。

### reqwest での使い方

reqwest では [Error](https://github.com/seanmonstar/reqwest/blob/master/src/error.rs) をカスタマイズしており、エラーの種別やリクエスト時に指定した URL などを取得できるように定義されている。

```rs
pub struct Error {
    inner: Box<Inner>,
}

pub(crate) type BoxError = Box<dyn StdError + Send + Sync>;

struct Inner {
    kind: Kind,
    source: Option<BoxError>,
    url: Option<Url>,
}

#[derive(Debug)]
pub(crate) enum Kind {
    Builder,
    Request,
    Redirect,
    Status(StatusCode),
    Body,
    Decode,
    Upgrade,
}
```

そして `Debug` 属性や `thiserror` などは使用せずに、以下のようにエラー型で管理している種別に応じてエラーメッセージを変更するようにカスタマイズを行なっている。

```rs
impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // ...
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.inner.kind {
            Kind::Builder => f.write_str("builder error")?,
            Kind::Request => f.write_str("error sending request")?,
            Kind::Body => f.write_str("request or response body error")?,
            // ...
        };

        // ...
    }
}
```

reqewest はそれほど大規模なクレートというわけではないため、おそらく自身でエラーに関する内容を定義し、外部のクレートに依存しないようにしていると考えられる。

## axum との組み合わせ

https://docs.rs/axum/latest/axum/error_handling/index.html

## 参考資料

- [Rust/Anyhow の Tips](https://zenn.dev/yukinarit/articles/b39cd42820f29e#bail!%E3%82%92%E4%BD%BF%E3%81%86)
- [Rust エラー処理 2020](https://cha-shu00.hatenablog.com/entry/2020/12/08/060000#anyhow)
