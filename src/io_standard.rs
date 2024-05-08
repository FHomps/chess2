use std::sync::atomic::{AtomicBool, Ordering};

#[allow(unused)]
pub fn alert(s: &str) {
    println!("Alert! {}", s);
}

#[allow(unused)]
pub fn log(s: &str) {
    println!("{}", s);
}

static SHOULD_RESTART: AtomicBool = AtomicBool::new(true);

pub fn poll_restart() -> bool {
    SHOULD_RESTART.swap(false, Ordering::Relaxed)
}

pub fn get_pieces_string() -> String {
    String::from("\
r___k__r
pppppppp
________
________
________
________
PPPPPPPP
R___K__R")
}

pub fn get_promotions_string() -> String {
    String::from("\
WWWWWWWW
________
________
________
________
________
________
bbbbbbbb")
}

pub fn get_bottom_side() -> String {
    String::from("white")
}