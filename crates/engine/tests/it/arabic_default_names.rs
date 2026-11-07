//! A new workbook in an Arabic (right-to-left) interface is named the way Arabic spreadsheets
//! name it: مصنف1 with sheet ورقة1, and the next sheet ورقة2, which formulas can refer to.

use gridcraft_engine::Session;
use gridcraft_engine::core::{CellRef, Value};
use serde_json::json;

#[test]
fn an_arabic_interface_names_new_workbooks_and_sheets_in_arabic() {
    let mut s = Session::new();
    s.prefs.default_right_to_left = true;
    s.new_workbook();
    let d = s.doc().unwrap();
    assert_eq!(d.title, "مصنف1");
    assert_eq!(d.wb.sheets[0].name, "ورقة1");
    assert!(d.wb.sheets[0].right_to_left);

    s.execute("home.insertSheet", json!({})).unwrap();
    assert_eq!(s.doc().unwrap().wb.sheets.iter().map(|sh| sh.name.as_str()).collect::<Vec<_>>(), ["ورقة1", "ورقة2"]);
    s.execute("cell.set", json!({"cell": "A1", "input": "7"})).unwrap();
    s.execute("sheet.activate", json!({"sheet": "ورقة1"})).unwrap();
    s.execute("cell.set", json!({"cell": "A1", "input": "=ورقة2!A1*2"})).unwrap();
    assert_eq!(s.doc().unwrap().wb.active().unwrap().cell(CellRef::parse("A1").unwrap()).unwrap().value, Value::Number(14.0));

    s.new_workbook();
    assert_eq!(s.doc().unwrap().title, "مصنف2");
}

#[test]
fn an_english_interface_keeps_book_and_sheet() {
    let mut s = Session::new();
    s.new_workbook();
    assert_eq!(s.doc().unwrap().title, "Book1");
    s.execute("home.insertSheet", json!({})).unwrap();
    assert_eq!(s.doc().unwrap().wb.sheets[1].name, "Sheet2");
}
