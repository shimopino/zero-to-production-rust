fn main() {
    let result = early_return();
    assert_eq!(result, Err("Divide By 0".to_string()));
}

fn early_return() -> Result<(), String> {
    let value = divide(10, 0)?;
    assert_eq!(value, 2);

    Ok(())
}

struct DivideByZero;

impl From<DivideByZero> for String {
    fn from(_value: DivideByZero) -> Self {
        println!("convert DivideByZero to 'Divide by 0' String");
        "Divide By 0".to_string()
    }
}

fn divide(numerator: i32, denominator: i32) -> Result<i32, DivideByZero> {
    if denominator == 0 {
        Err(DivideByZero)
    } else {
        Ok(numerator / denominator)
    }
}
