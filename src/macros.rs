macro_rules! match_iter_type {
    ($iter:ident, $($code:expr, $($iter_type: path)|*),*, _ => $fallback:expr) => {
        match $iter {
            $($(
                $iter_type($iter) => {
                    Ok($code)
                }
            )*)*
            _ => {
                $fallback
            }
        }
    };
    ($iter:ident, $($code:expr, $($iter_type: path)|*),*) => {
        match $iter {
            $($(
                $iter_type($iter) => {
                    $code
                }
            )*)*
        }
    };

}

macro_rules! match_iter_same_type {
    ($iter:ident, $($code:expr, $($iter_type: path)|*),*, _ => $fallback:expr) => {
        match $iter {
            $($(
                $iter_type($iter) => {
                    Ok($iter_type($code))
                }
            )*)*
            _ => {
                Err($fallback)
            }
        }
    };
    ($iter:ident, $($code:expr, $($iter_type: path)|*),*) => {
        match $iter {
            $($(
                $iter_type($iter) => {
                    $iter_type($code)
                }
            )*)*
        }
    };

}

macro_rules! match_iter_result_type {
    ($iter:ident, $code:expr, $($($iter_type: path)|* => $result_iter_type: path),*; _ => $fallback:expr) => {
        match $iter {
            $($(
                $iter_type($iter) => {
                    Ok($result_iter_type($code))
                }
            )*)*
            _ => {
                $fallback
            }
        }
    };
    ($iter:ident, $code:expr, $($($iter_type: path)|* => $result_iter_type: path),*) => {
        match $iter {
            $($(
                $iter_type($iter) => {
                    $result_iter_type($code)
                }
            )*)*
        }
    };

}

macro_rules! match_nested_iter_type {
    ($iter: ident, $outer: ident, $transform: expr, $code: expr, $($($outer_iter_type: path)|* : $($($inner_iter_type: path)|* => $result_iter_type: path),*, _ => $inner_fallback:expr);*$(, _ => $outer_fallback:expr)?) => {
        match_iter_type!(
            $iter,
            $({
                let $outer = $transform;
                match_iter_type!(
                    $outer,
                    $(
                        $result_iter_type($code),
                        $($inner_iter_type)|*
                    ),*
                    $(, _ => $inner_fallback)?
                )?
            },
            $($outer_iter_type)|*
            ),*
            $(, _ => $outer_fallback)?
        )
    };
    ($iter: ident, $outer: ident, $transform: expr, $code: expr, $($($outer_iter_type: path)|* : $($($inner_iter_type: path)|* => $result_iter_type: path),*);*$(, _ => $outer_fallback:expr)?) => {
        match_iter_type!(
            $iter,
            $({
                let $outer = $transform;
                match_iter_type!(
                    $outer,
                    $(
                        $result_iter_type($code),
                        $($inner_iter_type)|*
                    ),*
                )
            },
            $($outer_iter_type)|*
            ),*
            $(, _ => $outer_fallback)?
        )
    };

}

pub(crate) use match_iter_result_type;
pub(crate) use match_iter_same_type;
pub(crate) use match_iter_type;
pub(crate) use match_nested_iter_type;
