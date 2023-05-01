fn main() {
    let success = divide(10, 2);
    assert_eq!(success, Ok(5));

    let failure = divide(10, 0);
    assert_eq!(failure, Err("Divide by 0".to_string()));

    early_return();
}

fn early_return() -> Result<(), String> {
    let value = match divide(10, 5) {
        // 成功時には中身を取り出して変数に代入する
        Ok(value) => value,
        // 失敗時にはこの時点で、結果を関数から返却する
        Err(e) => return Err(e),
    };

    //
    println!("値は {} であり中身が取り出されている", value);

    Ok(())
}

fn divide(numerator: i32, denominator: i32) -> Result<i32, String> {
    if denominator == 0 {
        Err("Divide by 0".to_string())
    } else {
        Ok(numerator / denominator)
    }
}
