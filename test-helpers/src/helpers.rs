use std::fmt::Debug;

/// An enum to choose which type of unwrap to use. When using `Unwrap::Err`, the
/// result must be an `Err` or the test will panic. If the result contains an
/// `Err`, the test will pass only if the error message contains the provided
/// string.
pub enum Unwrap {
    Ok,
    Err(&'static str),
}

impl Unwrap {
    pub fn unwrap<T: Debug, E: Debug>(self, result: Result<T, E>) {
        match self {
            Unwrap::Ok => {
                result.unwrap();
            }
            Unwrap::Err(s) => {
                let err = result.unwrap_err();
                assert!(
                    format!("{:?}", err).contains(&s),
                    "Expected error message to contain {:?}, got {:?}",
                    s,
                    err
                );
            }
        }
    }
}
