pub const MOUSE_LEFT: u8 = 0x01;
pub const MOUSE_UNKNOWN: u8 = 0x00;

pub fn mouse_btn_to_vnc(button: Option<i32>) -> Option<u8> {
    if button.is_some() {
        let btn = match button.unwrap() {
            1 => MOUSE_LEFT,
            _ => MOUSE_UNKNOWN,
        };
        return Some(btn);
    }
    return None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mouse_adapt() {
        assert_eq!(mouse_btn_to_vnc(Some(1)).unwrap(), MOUSE_LEFT, "left mouse down");
        assert_eq!(mouse_btn_to_vnc(Some(0)).unwrap(), MOUSE_UNKNOWN, "all mouse keys up");
        assert_eq!(mouse_btn_to_vnc(Some(123)).unwrap(), MOUSE_UNKNOWN, "compliant with unknown");
    }
}