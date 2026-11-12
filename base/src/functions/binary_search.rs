use std::cmp::Ordering;

use crate::{
    calc_result::CalcResult,
    constants::{LAST_COLUMN, LAST_ROW},
    expressions::types::CellReferenceIndex,
    model::Model,
};

use super::util::compare_values;

// NOTE: We don't know how Excel exactly implements binary search internally.
// This means that if the values on the lookup range are not in order our results and Excel's will differ

// Assumes values are in ascending order, returns matching index or the largest value smaller than target.
// Returns None if target is smaller than the smaller value.
pub(crate) fn binary_search_or_smaller<T: Ord>(target: &T, array: &[T]) -> Option<i32> {
    // We apply binary search leftmost for value in the range
    let n = array.len();
    let mut l = 0;
    let mut r = n;
    while l < r {
        let m = (l + r) / 2;
        if &array[m] < target {
            l = m + 1;
        } else {
            r = m;
        }
    }
    if l == n {
        return Some((l - 1) as i32);
    }
    // Now l points to the leftmost element
    if &array[l] == target {
        return Some(l as i32);
    }
    // If target is less than the minimum return None
    if l == 0 {
        return None;
    }
    Some((l - 1) as i32)
}

// Assumes values are in ascending order, returns matching index or the smaller value larger than target.
// Returns None if target is smaller than the smaller value.
pub(crate) fn binary_search_or_greater<T: Ord>(target: &T, array: &[T]) -> Option<i32> {
    let mut l = 0;
    let mut r = array.len();
    while l < r {
        let mut m = (l + r) / 2;
        match &array[m].cmp(target) {
            Ordering::Less => {
                l = m + 1;
            }
            Ordering::Greater => {
                r = m;
            }
            Ordering::Equal => {
                while m > 1 {
                    if &array[m - 1] == target {
                        m -= 1;
                    } else {
                        break;
                    }
                }
                return Some(m as i32);
            }
        }
    }
    // If target is larger than the maximum return None
    if r == array.len() {
        return None;
    }
    // Now r points to the rightmost element
    Some(r as i32)
}

// Assumes values are in descending order
pub(crate) fn binary_search_descending_or_smaller<T: Ord>(target: &T, array: &[T]) -> Option<i32> {
    let n = array.len();
    let mut l = 0;
    let mut r = n;
    while l < r {
        let m = (l + r) / 2;
        let mut index = n - m - 1;
        match &array[index].cmp(target) {
            Ordering::Less => {
                l = m + 1;
            }
            Ordering::Greater => {
                r = m;
            }
            Ordering::Equal => {
                while index < n - 1 {
                    if &array[index + 1] == target {
                        index += 1;
                    } else {
                        break;
                    }
                }
                return Some(index as i32);
            }
        }
    }
    if l == 0 {
        return None;
    }
    Some((n - l) as i32)
}

// Assumes values are in descending order, returns matching index or the smaller value larger than target.
// Returns None if target is smaller than the smaller value.
pub(crate) fn binary_search_descending_or_greater<T: Ord>(target: &T, array: &[T]) -> Option<i32> {
    let n = array.len();
    let mut l = 0;
    let mut r = n;
    while l < r {
        let m = (l + r) / 2;
        let mut index = n - m - 1;
        match &array[index].cmp(target) {
            Ordering::Less => {
                l = m + 1;
            }
            Ordering::Greater => {
                r = m;
            }
            Ordering::Equal => {
                while index < n - 1 {
                    if &array[index + 1] == target {
                        index += 1;
                    } else {
                        break;
                    }
                }
                return Some(index as i32);
            }
        }
    }
    if r == n {
        return None;
    }
    Some((n - r - 1) as i32)
}

impl<'a> Model<'a> {
    /// Where a vector is read to. A column that runs to the last row, or a row
    /// that runs to the last column, ends at its last cell that holds
    /// something: the cells past it are empty, and Excel leaves them out too
    /// (an approximate MATCH over `A:A` finds the last value, not row
    /// 1,048,576). They go on record as read empty, so that a spill into them
    /// restarts the pass (see `clip_to_used_area`). XLOOKUP and XMATCH do not
    /// use it: they clip to the sheet's used area, because Excel's binary
    /// search there does see the blanks after the last value.
    fn vector_end(
        &mut self,
        left: &CellReferenceIndex,
        right: &CellReferenceIndex,
        is_row_vector: bool,
    ) -> CellReferenceIndex {
        let Ok(worksheet) = self.workbook.worksheet(left.sheet) else {
            return *right;
        };
        let data = &worksheet.sheet_data;
        let end = if is_row_vector && right.row == LAST_ROW {
            let last = data.last_row_in_column(left.column).unwrap_or(0);
            CellReferenceIndex {
                row: last.max(left.row - 1),
                ..*right
            }
        } else if !is_row_vector && right.column == LAST_COLUMN {
            let last = data.last_column_in_row(left.row).unwrap_or(0);
            CellReferenceIndex {
                column: last.max(left.column - 1),
                ..*right
            }
        } else {
            return *right;
        };
        let (row1, column1) = if is_row_vector {
            (end.row + 1, left.column)
        } else {
            (left.row, end.column + 1)
        };
        self.record_seen_empty_range(left.sheet, row1, column1, right.row, right.column);
        end
    }

    /// The search vector of MATCH, VLOOKUP, HLOOKUP and LOOKUP: like
    /// `prepare_array`, but read only to `vector_end`.
    pub(crate) fn prepare_lookup_vector(
        &mut self,
        left: &CellReferenceIndex,
        right: &CellReferenceIndex,
        is_row_vector: bool,
    ) -> Vec<CalcResult> {
        let right = self.vector_end(left, right, is_row_vector);
        self.prepare_array(left, &right, is_row_vector)
    }

    /// Returns an array with the list of cell values in the range
    pub(crate) fn prepare_array(
        &mut self,
        left: &CellReferenceIndex,
        right: &CellReferenceIndex,
        is_row_vector: bool,
    ) -> Vec<CalcResult> {
        let n = if is_row_vector {
            right.row - left.row
        } else {
            right.column - left.column
        } + 1;
        let mut result = vec![];
        for index in 0..n {
            let row;
            let column;
            if is_row_vector {
                row = left.row + index;
                column = left.column;
            } else {
                column = left.column + index;
                row = left.row;
            }
            let value = self.evaluate_cell(CellReferenceIndex {
                sheet: left.sheet,
                row,
                column,
            });
            result.push(value);
        }
        result
    }
}

/// Old style binary search over an already materialized vector of values.
/// Returns the index of the matching element or the largest element smaller
/// than `target`. Returns `-2` if `target` is smaller than every element.
pub(crate) fn binary_search_on_array(target: &CalcResult, array: &[CalcResult]) -> i32 {
    // We apply binary search leftmost for value in the array
    let mut l = 0;
    let mut r = array.len();
    while l < r {
        let m = (l + r) / 2;
        match compare_values(&array[m], target) {
            -1 => {
                l = m + 1;
            }
            1 => {
                r = m;
            }
            _ => {
                return m as i32;
            }
        }
    }
    // If target is less than the minimum return #N/A
    if l == 0 {
        return -2;
    }
    // Now l points to the leftmost element
    (l - 1) as i32
}
