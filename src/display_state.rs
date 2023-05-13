use bracket_lib::terminal::BTerm;

use crate::PsnU;

#[derive(Debug)]
pub struct DisplayState {
    pub width: PsnU,
    pub height: PsnU,
}

impl DisplayState {
    pub fn width_i32(&self) -> i32 {
        self.width as i32
    }
    pub fn height_i32(&self) -> i32 {
        self.height as i32
    }
    pub fn default() -> DisplayState {
        DisplayState {
            width: 80,
            height: 50,
        }
    }
}

pub fn calc_display_state(ctxt: &BTerm) -> DisplayState {
    let ctxt_char_size = ctxt.get_char_size();
    DisplayState {
        width: ctxt_char_size.0.try_into().unwrap(),
        height: ctxt_char_size.1.try_into().unwrap(),
    }
}
