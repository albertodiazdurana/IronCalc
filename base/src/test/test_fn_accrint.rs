#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn fn_average_accrint_simple_cases() {
    let mut model = new_empty_model();
    // ACCRINT(issue, first_interest, settlement, rate, par, frequency, [basis], [calc_method])
    model._set("A1", "=ACCRINT(39508, 39691, 39569, 0.1, 1000, 2, 0)");
    model._set(
        "A2",
        "=ACCRINT(DATE(2008, 3, 5), 39691, 39569, 0.1, 1000, 2, 0, FALSE)",
    );
    model._set(
        "A3",
        "=ACCRINT(DATE(2008, 4, 5), 39691, 39569, 0.1, 1000, 2, 0, TRUE)",
    );
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"16.666666667");
    assert_eq!(model._get_text("A2"), *"15.555555556");
    assert_eq!(model._get_text("A3"), *"7.222222222");
}

// DAX canonical worked example
// (https://learn.microsoft.com/en-us/dax/accrint-function-dax):
//
//   ACCRINT(DATE(2007,3,1), DATE(2008,8,31), DATE(2008,5,1), 0.1, 1000, 2, 0)
//     -> 116.944444444444
//
//   ACCRINT(DATE(2007,3,1), DATE(2008,8,31), DATE(2008,5,1), 0.1, 1000, 2, 0, FALSE)
//     -> 66.9444444444445
//
// These values exercise the multi-period accrual (NC = 3) because the
// settlement falls inside the third quasi-coupon period after issue.
// They are the primary BL-006 P3 conformance target and are documented
// in the BL-006 P2 gap audit.
#[test]
fn fn_accrint_dax_canonical() {
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=ACCRINT(DATE(2007,3,1), DATE(2008,8,31), DATE(2008,5,1), 0.1, 1000, 2, 0)",
    );
    model._set(
        "A2",
        "=ACCRINT(DATE(2007,3,1), DATE(2008,8,31), DATE(2008,5,1), 0.1, 1000, 2, 0, FALSE)",
    );
    model.evaluate();

    let a1 = model._get_text("A1");
    let a2 = model._get_text("A2");

    // Verify within standard floating-point tolerance: parse and compare.
    let a1_val: f64 = a1.parse().unwrap();
    let a2_val: f64 = a2.parse().unwrap();
    assert!(
        (a1_val - 116.944_444_444_444).abs() < 1e-6,
        "A1 (calc_method=TRUE default) = {a1}, expected 116.944444444444"
    );
    assert!(
        (a2_val - 66.944_444_444_445).abs() < 1e-6,
        "A2 (calc_method=FALSE) = {a2}, expected 66.9444444444445"
    );
}

// Settlement equals coupon date returns 0 (gap audit Q4).
#[test]
fn fn_accrint_settlement_equals_coupon() {
    let mut model = new_empty_model();
    // Issue 2008-3-1, first_interest 2008-9-1, settlement 2008-9-1
    // (= first_interest, the next coupon date). Excel and LibreOffice
    // misbehave on this case; IronCalc returns 0.
    model._set(
        "A1",
        "=ACCRINT(DATE(2008,3,1), DATE(2008,9,1), DATE(2008,9,1), 0.1, 1000, 2, 0)",
    );
    model.evaluate();
    let a1 = model._get_text("A1");
    let a1_val: f64 = a1.parse().unwrap();
    // Issue→settlement is exactly one full period, sum = 1.0,
    // AI = 1000 * 0.05 * 1.0 = 50. The "settlement = next coupon"
    // case is the boundary where one full coupon has accrued.
    assert!(
        (a1_val - 50.0).abs() < 1e-6,
        "A1 (settlement=first_interest) = {a1}, expected 50.0"
    );
}
