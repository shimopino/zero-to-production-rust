# エラーハンドリング

- [エラーハンドリング](#エラーハンドリング)
  - [Rust におけるエラーハンドリングの基本](#rust-におけるエラーハンドリングの基本)
  - [thiserror クレート](#thiserror-クレート)
  - [anyhow クレート](#anyhow-クレート)
  - [axum との組み合わせ](#axum-との組み合わせ)
  - [参考資料](#参考資料)

## Rust におけるエラーハンドリングの基本

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

## thiserror クレート

https://docs.rs/thiserror/latest/thiserror/

## anyhow クレート

https://docs.rs/anyhow/latest/anyhow/

## axum との組み合わせ

https://docs.rs/axum/latest/axum/error_handling/index.html

## 参考資料

- [std::result](https://doc.rust-lang.org/std/result/index.html)
- [std::error](https://doc.rust-lang.org/std/error/index.html)
