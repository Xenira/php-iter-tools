use ext_php_rs::prelude::*;

use crate::ZVal;

#[php_class(name = "Result")]
struct PhpResult {
    inner: Result<ZVal, ZVal>,
}

#[php_impl]
impl PhpResult {
    fn ok(&ZVal: value) -> Self {
        Self { inner: Ok(value) }
    }
}
