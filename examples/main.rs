fn main() {
    let result = early_return();
}

fn early_return() -> Result<i32, Box<dyn std::error::Error>> {
    let value = divide(10, 0)?;
    let result = multiply(value)?;

    Ok(result)
}

#[derive(Debug)]
enum CustomError {
    DivideByZero,
    NegativeNumber,
}

impl std::error::Error for CustomError {}

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            CustomError::DivideByZero => write!(f, "Divided by 0"),
            CustomError::NegativeNumber => write!(f, "Negative numbers"),
        }
    }
}

fn multiply(number: i32) -> Result<i32, CustomError> {
    if number < 0 {
        Err(CustomError::NegativeNumber)
    } else {
        Ok(number * 10)
    }
}

fn divide(numerator: i32, denominator: i32) -> Result<i32, CustomError> {
    if denominator == 0 {
        Err(CustomError::DivideByZero)
    } else {
        Ok(numerator / denominator)
    }
}
