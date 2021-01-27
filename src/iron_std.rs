use crate::variant::*;
use crate::*;

def_module! {
    pub mod iron_std {
        mod fs {

        }

        mod sys {

        }
    }
}

merge_modules! {
    pub mod global_full {
        string;
        ty;
        option;
        array;
        range;
    }
}

#[derive(Clone)]
pub struct Range {
    pub start: i32,
    pub end: i32,
}

#[derive(Clone)]
pub struct RangeIter {
    pub start: i32,
    pub end: i32,
    pub position: i32,
}

def_module! {
    pub mod range {
        fn "range"(start: i32, end: i32) {
            Range {
                start,
                end,
            }
        }

        fn "into_iter"(range: Range) {
            RangeIter {
                start: range.start,
                end: range.end,
                position: range.start,
            }
        }

        fn "iter_next"(range_iter: &mut RangeIter) {
            range_iter.position += 1;

            if range_iter.position <= range_iter.end {
                Some(Union::from(range_iter.position - 1))
            } else {
                None
            }
        }
    }
}

def_module! {
    pub mod array {
        fn "arr"() {
            vec![UnionCell::new(4), UnionCell::new(5)]
        }

        fn "push"(arr: &mut Vec<UnionCell>, item: Union) {
            arr.push(UnionCell::from(item));
        }

        fn "[]"(arr: Mut<Vec<UnionCell>>, index: i32) {
            arr.map_mut(|u| u[index as usize].get_shared())
        }

        fn "iter_next"(arr: &mut Vec<UnionCell>) {
            if arr.len() > 0 {
                Some(arr.remove(0))
            } else {
                None
            }
        }
    }
}

def_module! {
    pub mod option {
        fn "some"(s: Union) {
            Some(s)
        }

        fn "none"() {
            None::<Union>
        }

        fn "is_some"(o: Option<Union>) {
            o.is_some()
        }

        fn "is_none"(o: Option<Union>) {
            o.is_none()
        }

        fn "unwrap"(o: Option<Union>) {
            o.unwrap()
        }
    }
}

def_module! {
    pub mod ty {
        fn "type_of"(t: Union) {
            t.ty()
        }
    }
}

def_module! {
    pub mod string {
        fn "+"(a: &str, b: Union) {
            format!("{}{}", a, b)
        }

        fn "=="(lhs: &str, rhs: &str) {
            lhs == rhs
        }
    }
}
