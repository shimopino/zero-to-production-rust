# エラーハンドリング

- [エラーハンドリング](#エラーハンドリング)
  - [Rust におけるエラーハンドリングの基本](#rust-におけるエラーハンドリングの基本)
    - [Result 型とは](#result-型とは)
    - [Err 型で自作型を返却する](#err-型で自作型を返却する)
    - [Error トレイトを実装する](#error-トレイトを実装する)
    - [複数のエラー型の組み合わせ](#複数のエラー型の組み合わせ)
  - [thiserror クレート](#thiserror-クレート)
  - [anyhow クレート](#anyhow-クレート)
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

https://docs.rs/thiserror/latest/thiserror/

## anyhow クレート

https://docs.rs/anyhow/latest/anyhow/

## axum との組み合わせ

https://docs.rs/axum/latest/axum/error_handling/index.html

## 参考資料

- [std::result](https://doc.rust-lang.org/std/result/index.html)
- [std::error](https://doc.rust-lang.org/std/error/index.html)
