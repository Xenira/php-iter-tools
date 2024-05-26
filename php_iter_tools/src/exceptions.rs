use ext_php_rs::{
    exception::PhpException,
    zend::{ce, ClassEntry},
};
use thiserror::Error;

const CONSUMED_CODE: i32 = 1;

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
    Other(#[from] anyhow::Error),
}

impl Into<PhpException> for IterError {
    fn into(self) -> PhpException {
        match self {
            IterError::Moved => PhpException::new(
                format!("{self:#}"),
                CONSUMED_CODE,
                ClassEntry::try_find("\\LogicException").unwrap_or(ce::exception()),
            ),
            IterError::Consumed => PhpException::new(
                format!("{self:#}"),
                CONSUMED_CODE,
                ClassEntry::try_find("\\LogicException").unwrap_or(ce::exception()),
            ),
            IterError::ArgumentError(_) => PhpException::new(
                format!("{self:#}"),
                CONSUMED_CODE,
                ClassEntry::try_find("\\ValueError").unwrap_or(ce::exception()),
            ),
            IterError::Unsupported(_, _) => PhpException::new(
                format!("{self:#}"),
                CONSUMED_CODE,
                ClassEntry::try_find("\\DomainException").unwrap_or(ce::exception()),
            ),
            IterError::Other(e) => PhpException::default(e.to_string()),
        }
    }
}
