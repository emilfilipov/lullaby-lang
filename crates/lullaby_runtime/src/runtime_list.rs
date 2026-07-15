//! Homogeneous-list aggregate operations (sum, min/max extreme, scalar sort)
//! shared by every interpreter backend. Split out of `lib.rs` as a
//! behavior-preserving code move; `Value` and `RuntimeError` (in sibling
//! modules) are reached through `crate::` paths.

use crate::{RuntimeError, Value};

pub fn list_sum_values(name: &str, values: Vec<Value>) -> Result<Value, RuntimeError> {
    let mut iter = values.into_iter();
    let Some(first) = iter.next() else {
        return Ok(Value::I64(0));
    };
    match first {
        Value::I64(mut acc) => {
            for value in iter {
                match value {
                    Value::I64(n) => acc = acc.wrapping_add(n),
                    other => return Err(mixed_numeric_list_error(name, &other)),
                }
            }
            Ok(Value::I64(acc))
        }
        Value::F64(mut acc) => {
            for value in iter {
                match value {
                    Value::F64(n) => acc += n,
                    other => return Err(mixed_numeric_list_error(name, &other)),
                }
            }
            Ok(Value::F64(acc))
        }
        other => Err(RuntimeError::new(
            "L0417",
            format!("{name} expects a list<i64> or list<f64> but found `{other}`"),
        )),
    }
}

/// Find the extreme (min when `want_max` is false, max otherwise) element of a
/// numeric list, returning `None` for an empty list. f64 comparisons use total
/// ordering so `NaN` participates deterministically. A non-numeric or mixed
/// element is a runtime type error (`L0417`).
pub fn list_extreme(
    name: &str,
    values: Vec<Value>,
    want_max: bool,
) -> Result<Option<Value>, RuntimeError> {
    let mut iter = values.into_iter();
    let Some(first) = iter.next() else {
        return Ok(None);
    };
    match first {
        Value::I64(mut best) => {
            for value in iter {
                match value {
                    Value::I64(n) => {
                        if (want_max && n > best) || (!want_max && n < best) {
                            best = n;
                        }
                    }
                    other => return Err(mixed_numeric_list_error(name, &other)),
                }
            }
            Ok(Some(Value::I64(best)))
        }
        Value::F64(mut best) => {
            for value in iter {
                match value {
                    Value::F64(n) => {
                        let ordering = n.total_cmp(&best);
                        let replace = if want_max {
                            ordering == std::cmp::Ordering::Greater
                        } else {
                            ordering == std::cmp::Ordering::Less
                        };
                        if replace {
                            best = n;
                        }
                    }
                    other => return Err(mixed_numeric_list_error(name, &other)),
                }
            }
            Ok(Some(Value::F64(best)))
        }
        other => Err(RuntimeError::new(
            "L0417",
            format!("{name} expects a list<i64> or list<f64> but found `{other}`"),
        )),
    }
}

/// Sort a scalar list ascending, dispatching on the element type. Supports
/// `i64`, `f64` (total order via `total_cmp`, so `NaN` sorts deterministically),
/// and `string` (lexicographic by Rust `str` ordering). The list must be
/// homogeneous; a mixed or unsupported element type yields `L0417`.
pub fn sort_scalar_list(name: &str, values: Vec<Value>) -> Result<Value, RuntimeError> {
    let Some(first) = values.first() else {
        return Ok(Value::Array((Vec::new()).into()));
    };
    match first {
        Value::I64(_) => {
            let mut nums: Vec<i64> = Vec::with_capacity(values.len());
            for value in values {
                match value {
                    Value::I64(n) => nums.push(n),
                    other => return Err(mixed_numeric_list_error(name, &other)),
                }
            }
            nums.sort();
            Ok(Value::Array(nums.into_iter().map(Value::I64).collect()))
        }
        Value::F64(_) => {
            let mut nums: Vec<f64> = Vec::with_capacity(values.len());
            for value in values {
                match value {
                    Value::F64(n) => nums.push(n),
                    other => return Err(mixed_numeric_list_error(name, &other)),
                }
            }
            nums.sort_by(|a, b| a.total_cmp(b));
            Ok(Value::Array(nums.into_iter().map(Value::F64).collect()))
        }
        Value::String(_) => {
            let mut strs: Vec<String> = Vec::with_capacity(values.len());
            for value in values {
                match value {
                    Value::String(s) => strs.push((s).into()),
                    other => return Err(mixed_numeric_list_error(name, &other)),
                }
            }
            strs.sort();
            Ok(Value::Array(
                strs.into_iter().map(|s| Value::String(s.into())).collect(),
            ))
        }
        other => Err(RuntimeError::new(
            "L0417",
            format!("{name} expects a list<i64>, list<f64>, or list<string> but found `{other}`"),
        )),
    }
}

fn mixed_numeric_list_error(name: &str, value: &Value) -> RuntimeError {
    RuntimeError::new(
        "L0417",
        format!("{name} expects a homogeneous numeric list but found `{value}`"),
    )
}
