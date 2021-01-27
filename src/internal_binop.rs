use crate::variant::*;

macro_rules! op {
    ($ident:ident, $fn:ident, $lhs:expr, $rhs:expr, $op:expr) => {
        {
            match $rhs.$fn() {
                Some(rhs) => match $op.as_str() {
                    "+" => Some(Union::$ident($lhs + rhs)),
                    "-" => Some(Union::$ident($lhs - rhs)),
                    "*" => Some(Union::$ident($lhs * rhs)),
                    "/" => Some(Union::$ident($lhs / rhs)),
                    "%" => Some(Union::$ident($lhs % rhs)),
                    ">" => Some(Union::Bool($lhs > rhs)),
                    "<" => Some(Union::Bool($lhs < rhs)),
                    ">=" => Some(Union::Bool($lhs >= rhs)),
                    "<=" => Some(Union::Bool($lhs <= rhs)),
                    "==" => Some(Union::Bool($lhs == rhs)),
                    _ => None,
                },
                None => None,
            }
        }
    };
}

#[inline(always)]
pub fn internal_binop(lhs: Union, rhs: Union, op: &String) -> Option<Union> {
    match lhs {
        Union::Int(lhs) => op!(Int, as_int, lhs, rhs, op),
        Union::Float(lhs) => op!(Float, as_float, lhs, rhs, op),
        _ => None,
    }
}