fn main() {
    let result = early_return();
    assert_eq!(result, Err("Divide By 0".to_string()));
}

fn early_return() -> Result<(), String> {
    let value = divide(10, 0)?;
    assert_eq!(value, 2);

    Ok(())
}

#[derive(Debug)]
struct DivideByZero;

impl std::error::Error for DivideByZero {}

impl std::fmt::Display for DivideByZero {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[DividedByZero] Divided by 0")
    }
}

impl From<DivideByZero> for String {
    fn from(value: DivideByZero) -> Self {
        println!("Display: {}, Debug: {:?}", value, value);

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
