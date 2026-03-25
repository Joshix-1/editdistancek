#![deny(missing_docs)]

//! # Edit Distance
//! A library for fast finding the Levenshtein edit distance between `s` and `t`.

use std::cmp::{max, min};

use pyo3::prelude::*;
use pyo3::types::PyString;

const DEFAULT_K: usize = usize::MAX;

#[pyfunction]
#[pyo3(signature = (s1, s2, /, k))]
fn distance(s1: Bound<'_, PyString>, s2: Bound<'_, PyString>, k: usize) -> PyResult<usize> {
    Ok(edit_distance_python(s1, s2, k)?.unwrap_or(k))
}

#[pyfunction]
#[pyo3(signature = (s1, s2, /))]
fn distance_unbounded(s1: Bound<'_, PyString>, s2: Bound<'_, PyString>) -> PyResult<usize> {
    Ok(edit_distance_python(s1, s2, DEFAULT_K)?.unwrap())
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(distance, m)?)?;
    m.add_function(wrap_pyfunction!(distance_unbounded, m)?)?;

    Ok(())
}

#[inline(always)]
fn edit_distance_python(s1: Bound<'_, PyString>, s2: Bound<'_, PyString>, k: usize) -> PyResult<Option<usize>> {
    let d1 = unsafe { s1.data() }?;
    let d2 = unsafe { s2.data() }?;

    use pyo3::types::PyStringData::*;

    match (d1, d2) {
        (Ucs1(i1), Ucs1(i2)) => Ok(edit_distance_bounded(i1, i2, k)),
        (Ucs2(i1), Ucs2(i2)) => Ok(edit_distance_bounded(i1, i2, k)),
        (Ucs4(i1), Ucs4(i2)) => Ok(edit_distance_bounded(i1, i2, k)),
        (Ucs1(i1), Ucs2(i2)) => Ok(edit_distance_bounded(i1, i2, k)),
        (Ucs1(i1), Ucs4(i2)) => Ok(edit_distance_bounded(i1, i2, k)),
        (Ucs2(i1), Ucs1(i2)) => Ok(edit_distance_bounded(i1, i2, k)),
        (Ucs2(i1), Ucs4(i2)) => Ok(edit_distance_bounded(i1, i2, k)),
        (Ucs4(i1), Ucs1(i2)) => Ok(edit_distance_bounded(i1, i2, k)),
        (Ucs4(i1), Ucs2(i2)) => Ok(edit_distance_bounded(i1, i2, k)),
    }
}

/// Bounded UTF-8 edit-distance
#[inline(always)]
pub fn edit_distance_utf8(s: &str, t: &str, k: usize) -> Option<usize> {
    edit_distance_bounded(
        &s.chars().collect::<Box<_>>(),
        &t.chars().collect::<Box<_>>(),
        k,
    )
}

/// Returns edit distance between `s` and `t`.
#[inline(always)]
pub fn edit_distance<T: OwnEq<T>>(s: &[T], t: &[T]) -> usize {
    edit_distance_bounded(s, t, DEFAULT_K).unwrap()
}

/// Trait
pub trait OwnEq<T> {
    /// eq
    fn equals(&self, rhs: &T) -> bool;
}

macro_rules! generate_self_eq {
    ($typ:ty) => {
        impl OwnEq<$typ> for $typ {
            #[inline(always)]
            fn equals(&self, rhs: &$typ) -> bool {
                self == rhs
            }
        }
    };
}
generate_self_eq!(char);
generate_self_eq!(u8);
generate_self_eq!(u16);
generate_self_eq!(u32);

macro_rules! generate_eq {
    ($typ2:ty, $typ:ty) => {
        impl OwnEq<$typ> for $typ2 {
            #[inline(always)]
            fn equals(&self, rhs: &$typ) -> bool {
                let s: $typ = (*self).into();

                s == *rhs
            }
        }
        impl OwnEq<$typ2> for $typ {
            #[inline(always)]
            fn equals(&self, rhs: &$typ2) -> bool {
                let r: $typ = (*rhs).into();

                *self == r
            }
        }
    };
}
generate_eq!(u8, u16);
generate_eq!(u8, u32);
generate_eq!(u16, u32);


/// If edit distance `d` between `s` and `t` is at most `k`, then returns `Some(d)` otherwise returns `None`.
#[inline(always)]
pub fn edit_distance_bounded<T, K>(s: &[T], t: &[K], k: usize) -> Option<usize>
    where T: OwnEq<K>, K: OwnEq<T>
{
    let (s_length, t_length) = (s.len(), t.len());

    let (s_length, s, t_length, t) = if s_length > t_length {
        return edit_distance_bounded(t, s, k);
    } else {
        (s_length, s, t_length, t)
    };

    debug_assert!(s_length <= t_length);

    let k = min(k, max(s_length, t_length));

    let diff = t_length - s_length;
    if diff > k {
        return None;
    }

    let max_p = 1 + (k - diff) / 2;
    let max_q = 1 + k / 2 + diff;

    let shift = k + 1;
    let helper_arr_len = 2 * k + 3;
    let mut full_helper_arr = vec![-1isize; helper_arr_len * 2];
    let (a, b) = full_helper_arr.split_at_mut(helper_arr_len);

    for h in 0..=k {
        let (a, b) = if (h & 1) == 0 {
            (&*b, &mut *a)
        } else {
            (&*a, &mut *b)
        };
        let (p, q) = (
            shift - min(max_p, h),
            shift + min(max_q, h),
        );
        for i in p..=q {
            b[i] = {
                let r = (max(max(a[i - 1], a[i] + 1), a[i + 1] + 1)) as usize;
                if r >= s_length || r + i - shift >= t_length {
                    r
                } else {
                    mismatch(&s[r..], &t[(r + i - shift)..]) + r
                }
            } as isize;
            if i + s_length == t_length + shift && b[i] as usize >= s_length {
                return Some(h);
            }
        }
    }
    None
}

#[inline(always)]
/// Calculate the mismatch between iteratables.
pub fn mismatch<T: OwnEq<K>, K>(s: &[T], t: &[K]) -> usize {
    s.iter().zip(t).take_while(|(x, y)| x.equals(y)).count()
}
