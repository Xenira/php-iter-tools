use ext_php_rs::{
    exception::PhpException,
    zend::{ce, ClassEntry},
};
use thiserror::Error;

const CONSUMED_CODE: i32 = 1;
const ARGUMENT_ERROR_CODE: i32 = 2;
const UNSUPPORTED_CODE: i32 = 3;
const INT_OVERFLOW_CODE: i32 = 4;

#[derive(Error, Debug)]
pub enum IterError {
    #[error("The iterator has already been moved.")]
    Moved,
    #[error("The iterator has already been consumed")]
    Consumed,
    #[error("Invalid argument: {0}")]
    ArgumentError(String),
    #[error("Unsupported operation. {0} is only supported on {1} iterators.")]
    Unsupported(String, String),
    #[error(transparent)]
    ExtPhpError(#[from] ext_php_rs::error::Error),
    #[error(transparent)]
    IntOverflow(#[from] std::num::TryFromIntError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<IterError> for PhpException {
    fn from(val: IterError) -> Self {
        match val {
            IterError::Moved | IterError::Consumed => PhpException::new(
                format!("{val:#}"),
                CONSUMED_CODE,
                ClassEntry::try_find("\\LogicException").unwrap_or(ce::exception()),
            ),
            IterError::ArgumentError(_) => PhpException::new(
                format!("{val:#}"),
                ARGUMENT_ERROR_CODE,
                ClassEntry::try_find("\\ValueError").unwrap_or(ce::exception()),
            ),
            IterError::Unsupported(_, _) => PhpException::new(
                format!("{val:#}"),
                UNSUPPORTED_CODE,
                ClassEntry::try_find("\\DomainException").unwrap_or(ce::exception()),
            ),
            IterError::ExtPhpError(e) => e.into(),
            IterError::IntOverflow(_) => PhpException::new(
                format!("{val:#}"),
                INT_OVERFLOW_CODE,
                ce::arithmetic_error(),
            ),
            IterError::Other(e) => PhpException::default(e.to_string()),
        }
    }
}
