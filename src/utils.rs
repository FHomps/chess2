use std::fmt::Display;

pub trait ResultUtils {
    fn or_print(self) -> Self;
}

impl<T, E: Display> ResultUtils for Result<T, E> {
    fn or_print(self) -> Self {
        if let Err(ref e) = self {
            eprintln!("{}", e);
        }
        self
    }
}
