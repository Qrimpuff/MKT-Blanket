use crate::compare_screenshot;

#[test]
fn inv_ipad() {
    compare_screenshot("drivers", "inv_ipad.jpg", "inv_ipad.json")
}

#[test]
fn mkt_drivers_1() {
    compare_screenshot("drivers", "mkt_drivers_1.jpg", "mkt_drivers_1.json")
}

#[test]
fn mkt_drivers_2() {
    compare_screenshot("drivers", "mkt_drivers_2.jpg", "mkt_drivers_2.json")
}

#[test]
fn mkt_drivers_3() {
    compare_screenshot("drivers", "mkt_drivers_3.jpg", "mkt_drivers_3.json")
}

#[test]
fn mkt_drivers_4() {
    compare_screenshot("drivers", "mkt_drivers_4.jpg", "mkt_drivers_4.json")
}

#[test]
fn mkt_drivers_5() {
    compare_screenshot("drivers", "mkt_drivers_5.jpg", "mkt_drivers_5.json")
}

#[test]
fn mkt_drivers_6() {
    compare_screenshot("drivers", "mkt_drivers_6.jpg", "mkt_drivers_6.json")
}
