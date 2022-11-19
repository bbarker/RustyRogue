use bracket_lib::terminal::BTerm;

pub struct DisplayState {
    pub width: u32,
    pub height: u32,
}

impl DisplayState {
    pub fn width_i32(&self) -> i32 {
        self.width as i32
    }
    pub fn height_i32(&self) -> i32 {
        self.height as i32
    }
}

pub fn calc_display_state(ctxt: &BTerm) -> DisplayState {
    DisplayState {
        width: ctxt.get_char_size().0,
        height: ctxt.get_char_size().1,
    }
}
