#![deny(missing_docs)]

//! # Edit Distance
//! A library for fast finding the Levenshtein edit distance between `s` and `t`.

use std::cmp::{max, min};

use pyo3::prelude::*;

const DEFAULT_K: usize = usize::MAX;

#[pyfunction]
#[pyo3(signature = (s1, s2, /, k))]
fn distance(s1: &str, s2: &str, k: usize) -> usize {
    edit_distance_utf8(s1, s2, k).unwrap_or(k)
}

#[pyfunction]
#[pyo3(signature = (s1, s2, /))]
fn distance_unbounded(s1: &str, s2: &str) -> usize {
    edit_distance_utf8(s1, s2, DEFAULT_K).unwrap()
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(distance, m)?)?;
    m.add_function(wrap_pyfunction!(distance_unbounded, m)?)?;

    Ok(())
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
pub fn edit_distance<T: PartialEq>(
    s: impl IntoIterator<Item = T, IntoIter = impl ExactSizeIterator<Item = T> + Clone>,
    t: impl IntoIterator<Item = T, IntoIter = impl ExactSizeIterator<Item = T> + Clone>,
) -> usize {
    edit_distance_bounded(s, t, DEFAULT_K).unwrap()
}

/// If edit distance `d` between `s` and `t` is at most `k`, then returns `Some(d)` otherwise returns `None`.
#[inline(always)]
pub fn edit_distance_bounded<T: PartialEq>(
    s: impl IntoIterator<Item = T, IntoIter = impl ExactSizeIterator<Item = T> + Clone>,
    t: impl IntoIterator<Item = T, IntoIter = impl ExactSizeIterator<Item = T> + Clone>,
    k: usize,
) -> Option<usize> {
    let (s, t) = (s.into_iter(), t.into_iter());
    let (s_length, t_length) = (s.len(), t.len());

    if s_length > t_length {
        return edit_distance_bounded(t, s, k);
    }

    debug_assert!(s_length <= t_length);

    let k = k.min(s_length.max(t_length));

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
                    mismatch(s.clone().skip(r), t.clone().skip(r + i - shift)) + r
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
pub fn mismatch<T: PartialEq>(
    s: impl IntoIterator<Item = T>,
    t: impl IntoIterator<Item = T>,
) -> usize {
    s.into_iter().zip(t).take_while(|(x, y)| x == y).count()
}
