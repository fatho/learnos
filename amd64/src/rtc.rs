use crate::cmos;



pub struct ClockTime {
    seconds: u8,
    minutes: u8,
    hours: u8,
    weekday: u8,
    day_of_month: u8,
    month: u8,
    year: u32,
}


pub fn wait_for_update() {

}