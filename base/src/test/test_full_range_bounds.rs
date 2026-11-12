#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// Whole-column and whole-row references (A:A, 1:1) are read only as far as
// the cells that hold something. These tests pin the results that must not
// change because of that.

fn model_with_column_data<'a>() -> crate::model::Model<'a> {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("A2", "20");
    model._set("A3", "30");
    model._set("B1", "a");
    model._set("B2", "b");
    model._set("B3", "c");
    model._set("C1", "30");
    model._set("C2", "20");
    model._set("C3", "10");
    model
}

#[test]
fn approximate_match_over_whole_column_matches_excel() {
    let mut model = model_with_column_data();
    model._set("E1", "=MATCH(99,A:A,1)");
    model._set("E2", "=MATCH(-99,C:C,-1)");
    model._set("E3", "=MATCH(25,A:A,1)");
    model._set("E4", "=MATCH(25,C:C,-1)");
    model._set("E5", "=MATCH(99,A:A)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "3");
    assert_eq!(model._get_text("E2"), "3");
    assert_eq!(model._get_text("E3"), "2");
    assert_eq!(model._get_text("E4"), "1");
    assert_eq!(model._get_text("E5"), "3");

    model._set("H1000", "1");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "3");
    assert_eq!(model._get_text("E2"), "3");

    model._set("D2000", "2");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "3");
    assert_eq!(model._get_text("E2"), "3");
}

#[test]
fn approximate_match_over_whole_row_ignores_far_columns() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("B1", "20");
    model._set("C1", "30");
    model._set("A2", "30");
    model._set("B2", "20");
    model._set("C2", "10");
    model._set("D3", "=MATCH(99,1:1,1)");
    model._set("E3", "=MATCH(-99,2:2,-1)");
    model._set("XFD100", "1");
    model.evaluate();
    assert_eq!(model._get_text("D3"), "3");
    assert_eq!(model._get_text("E3"), "3");
}

#[test]
fn vlookup_whole_column_exact_and_approximate() {
    let mut model = model_with_column_data();
    model._set("E1", "=VLOOKUP(20,A:B,2,0)");
    model._set("E2", "=VLOOKUP(20,$A:$B,2,FALSE)");
    model._set("E3", "=VLOOKUP(99,A:B,2)");
    model._set("E4", "=VLOOKUP(25,A:B,2,TRUE)");
    model._set("E5", "=VLOOKUP(5,A:B,2,TRUE)");
    model._set("E6", "=VLOOKUP(40,A:B,2,0)");
    model._set("E7", "=VLOOKUP(20,A:B,3,0)");
    model._set("H1000", "1");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "b");
    assert_eq!(model._get_text("E2"), "b");
    assert_eq!(model._get_text("E3"), "c");
    assert_eq!(model._get_text("E4"), "b");
    assert_eq!(model._get_text("E5"), "#N/A");
    assert_eq!(model._get_text("E6"), "#N/A");
    assert_eq!(model._get_text("E7"), "#REF!");
}

#[test]
fn vlookup_whole_column_on_other_sheet() {
    let mut model = new_empty_model();
    model.add_sheet("Data").unwrap();
    model._set("Data!F2", "k1");
    model._set("Data!F3", "k2");
    model._set("Data!I2", "100");
    model._set("Data!I3", "200");
    model._set("A1", "=VLOOKUP(\"k2\",Data!$F:$I,4,0)");
    model._set("A2", "=VLOOKUP(\"k9\",Data!$F:$I,4,0)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "200");
    assert_eq!(model._get_text("A2"), "#N/A");
}

#[test]
fn hlookup_whole_row_exact_and_approximate() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("B1", "20");
    model._set("C1", "30");
    model._set("A2", "a");
    model._set("B2", "b");
    model._set("C2", "c");
    model._set("A5", "=HLOOKUP(20,1:2,2,0)");
    model._set("A6", "=HLOOKUP(99,1:2,2)");
    model._set("A7", "=HLOOKUP(25,1:2,2,TRUE)");
    model._set("A8", "=HLOOKUP(40,1:2,2,0)");
    model._set("XFD100", "1");
    model.evaluate();
    assert_eq!(model._get_text("A5"), "b");
    assert_eq!(model._get_text("A6"), "c");
    assert_eq!(model._get_text("A7"), "b");
    assert_eq!(model._get_text("A8"), "#N/A");
}

#[test]
fn value_far_down_the_column_is_found() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A1000000", "99");
    model._set("B1000000", "far");
    model._set("C1", "=MATCH(99,A:A,0)");
    model._set("C2", "=VLOOKUP(99,A:B,2,0)");
    model._set("C3", "=INDEX(B:B,MATCH(99,A:A,0))");
    model._set("C4", "=XLOOKUP(99,A:A,B:B)");
    model._set("C5", "=MATCH(1000,A:A,1)");
    model._set("C6", "=LOOKUP(1000,A:A,B:B)");
    model._set("C7", "=COUNTIF(A:A,99)");
    model._set("C8", "=SUMIF(A:A,\">50\")");
    model._set("C9", "=MAX(A:A)");
    model._set("C10", "=COUNT(A:A)");
    model._set("C11", "=COUNTA(B:B)");
    model._set("C12", "=AVERAGE(A:A)");
    model._set("C13", "=SUMIFS(A:A,B:B,\"far\")");
    model._set("C14", "=AVERAGEIF(B:B,\"far\",A:A)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "1000000");
    assert_eq!(model._get_text("C2"), "far");
    assert_eq!(model._get_text("C3"), "far");
    assert_eq!(model._get_text("C4"), "far");
    assert_eq!(model._get_text("C5"), "1000000");
    assert_eq!(model._get_text("C6"), "far");
    assert_eq!(model._get_text("C7"), "1");
    assert_eq!(model._get_text("C8"), "99");
    assert_eq!(model._get_text("C9"), "99");
    assert_eq!(model._get_text("C10"), "2");
    assert_eq!(model._get_text("C11"), "1");
    assert_eq!(model._get_text("C12"), "50");
    assert_eq!(model._get_text("C13"), "99");
    assert_eq!(model._get_text("C14"), "99");
}

#[test]
fn value_far_along_the_row_is_found() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("XFA1", "99");
    model._set("XFA2", "far");
    model._set("A5", "=MATCH(99,1:1,0)");
    model._set("A6", "=HLOOKUP(99,1:2,2,0)");
    model._set("A7", "=INDEX(2:2,MATCH(99,1:1,0))");
    model._set("A8", "=XLOOKUP(99,1:1,2:2)");
    model._set("A9", "=COUNTIF(1:1,99)");
    model._set("A10", "=SUMIF(2:2,\"far\",1:1)");
    model._set("A11", "=MIN(1:1)");
    model._set("A12", "=COUNTA(2:2)");
    model.evaluate();
    assert_eq!(model._get_text("A5"), "16381");
    assert_eq!(model._get_text("A6"), "far");
    assert_eq!(model._get_text("A7"), "far");
    assert_eq!(model._get_text("A8"), "far");
    assert_eq!(model._get_text("A9"), "1");
    assert_eq!(model._get_text("A10"), "99");
    assert_eq!(model._get_text("A11"), "1");
    assert_eq!(model._get_text("A12"), "1");
}

#[test]
fn empty_column_and_row() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "=MATCH(1,C:C,0)");
    model._set("B2", "=VLOOKUP(1,C:D,2,0)");
    model._set("B3", "=COUNTIF(C:C,\">0\")");
    model._set("B4", "=SUMIF(C:C,\">0\")");
    model._set("B5", "=COUNT(C:C)");
    model._set("B6", "=COUNTA(C:C)");
    model._set("B7", "=MAX(C:C)");
    model._set("B8", "=MIN(C:C)");
    model._set("B9", "=AVERAGE(C:C)");
    model._set("B10", "=AND(C:C)");
    model._set("B11", "=AND(C1:C10)");
    model._set("B12", "=HLOOKUP(1,500:501,2,0)");
    model._set("B13", "=MATCH(1,500:500,1)");
    model._set("B14", "=LOOKUP(1,C:C,D:D)");
    model._set("B15", "=COUNTA(500:500)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "#N/A");
    assert_eq!(model._get_text("B2"), "#N/A");
    assert_eq!(model._get_text("B3"), "0");
    assert_eq!(model._get_text("B4"), "0");
    assert_eq!(model._get_text("B5"), "0");
    assert_eq!(model._get_text("B6"), "0");
    assert_eq!(model._get_text("B7"), "0");
    assert_eq!(model._get_text("B8"), "0");
    assert_eq!(model._get_text("B9"), "#DIV/0!");
    assert_eq!(model._get_text("B10"), model._get_text("B11"));
    assert_eq!(model._get_text("B12"), "#N/A");
    assert_eq!(model._get_text("B13"), "#N/A");
    assert_eq!(model._get_text("B14"), "#N/A");
    assert_eq!(model._get_text("B15"), "0");
}

#[test]
fn conditional_aggregates_over_whole_columns_and_rows() {
    let mut model = model_with_column_data();
    model._set("E1", "=COUNTIF(A:A,\">15\")");
    model._set("E2", "=SUMIF(A:A,\">15\")");
    model._set("E3", "=SUMIF(B:B,\"b\",A:A)");
    model._set("E4", "=AVERAGEIF(A:A,\">15\")");
    model._set("E5", "=COUNTIFS(A:A,\">15\",B:B,\"c\")");
    model._set("E6", "=SUMIFS(A:A,B:B,\"<>a\")");
    model._set("E7", "=AVERAGEIFS(C:C,A:A,\">15\")");
    model._set("E8", "=COUNTIF(A:A,\"\")");
    model._set("E9", "=COUNTIF(1:1,\">15\")");
    model._set("E10", "=SUMIF(1:1,\">15\")");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "2");
    assert_eq!(model._get_text("E2"), "50");
    assert_eq!(model._get_text("E3"), "20");
    assert_eq!(model._get_text("E4"), "25");
    assert_eq!(model._get_text("E5"), "1");
    assert_eq!(model._get_text("E6"), "50");
    assert_eq!(model._get_text("E7"), "15");
    assert_eq!(model._get_text("E8"), "1048573");
    assert_eq!(model._get_text("E9"), "1");
    assert_eq!(model._get_text("E10"), "30");
}

#[test]
fn aggregates_over_whole_columns_and_rows() {
    let mut model = new_empty_model();
    model._set("A2", "-5");
    model._set("A3", "12");
    model._set("A4", "TRUE");
    model._set("A5", "x");
    model._set("B1", "=MIN(A:A)");
    model._set("B2", "=MAX(A:A)");
    model._set("B3", "=COUNT(A:A)");
    model._set("B4", "=COUNTA(A:A)");
    model._set("B5", "=AVERAGE(A:A)");
    model._set("B6", "=AVERAGEA(A:A)");
    model._set("B7", "=OR(A:A)");
    model._set("B8", "=AND(A:A)");
    model._set("B9", "=MINA(A:A)");
    model._set("B10", "=MAXA(A:A)");
    model._set("C30", "7");
    model._set("D30", "3");
    model._set("E40", "=MIN(30:30)");
    model._set("E41", "=COUNT(30:30)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "-5");
    assert_eq!(model._get_text("B2"), "12");
    assert_eq!(model._get_text("B3"), "2");
    assert_eq!(model._get_text("B4"), "4");
    assert_eq!(model._get_text("B5"), "3.5");
    assert_eq!(model._get_text("B6"), "2");
    assert_eq!(model._get_text("B7"), "TRUE");
    assert_eq!(model._get_text("B8"), "TRUE");
    assert_eq!(model._get_text("B9"), "-5");
    assert_eq!(model._get_text("B10"), "12");
    assert_eq!(model._get_text("E40"), "3");
    assert_eq!(model._get_text("E41"), "2");
}

#[test]
fn countblank_still_counts_the_whole_column_and_row() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("C1", "=COUNTBLANK(A:A)");
    model._set("C5", "=COUNTBLANK(2:2)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "1048574");
    assert_eq!(model._get_text("C5"), "16383");
}

#[test]
fn whole_column_array_keeps_its_shape() {
    let mut model = model_with_column_data();
    model._set("E1", "=INDEX(A:A+0,4)");
    model._set("E2", "=INDEX(A1:A4+0,4)");
    model._set("E3", "=ROWS(A:A)");
    model._set("E4", "=INDEX(A:A,1000)");
    model._set("E5", "=COLUMNS(1:1)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "0");
    assert_eq!(model._get_text("E2"), "0");
    assert_eq!(model._get_text("E3"), "1048576");
    assert_eq!(model._get_text("E4"), "0");
    assert_eq!(model._get_text("E5"), "16384");
}

#[test]
fn index_match_and_xlookup_over_whole_columns() {
    let mut model = model_with_column_data();
    model._set("E1", "=INDEX(B:B,MATCH(30,A:A,0))");
    model._set("E2", "=XLOOKUP(20,A:A,B:B)");
    model._set("E3", "=XMATCH(30,A:A)");
    model._set("E4", "=XLOOKUP(25,A:A,B:B,\"none\",-1)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "c");
    assert_eq!(model._get_text("E2"), "b");
    assert_eq!(model._get_text("E3"), "3");
    assert_eq!(model._get_text("E4"), "b");
}

#[test]
fn lookup_vector_result_on_other_sheet_beyond_its_used_area() {
    let mut model = new_empty_model();
    model.add_sheet("Other").unwrap();
    model._set("A1", "10");
    model._set("A2", "20");
    model._set("A3", "30");
    model._set("Other!B1", "x");
    model._set("C1", "=LOOKUP(99,A:A,Other!B:B)");
    model._set("C2", "=LOOKUP(10,A:A,Other!B:B)");
    model._set("C3", "=LOOKUP(25,A:A,{1;2;3})");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "0");
    assert_eq!(model._get_text("C2"), "x");
    assert_eq!(model._get_text("C3"), "2");
}

#[test]
fn spill_into_a_whole_column_is_seen() {
    let mut model = new_empty_model();
    model._set("A1", "=MATCH(4,C:C,0)");
    model._set("A2", "=VLOOKUP(5,C:C,1,0)");
    model._set("A3", "=COUNT(C:C)");
    model._set("A4", "=MAX(C:C)");
    model._set("A5", "=MATCH(99,C:C,1)");
    model._set("C1", "=SEQUENCE(5)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "4");
    assert_eq!(model._get_text("A2"), "5");
    assert_eq!(model._get_text("A3"), "5");
    assert_eq!(model._get_text("A4"), "5");
    assert_eq!(model._get_text("A5"), "5");
}
