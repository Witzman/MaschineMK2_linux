pub fn normalize_encoder(raw_status: i32, roller_state: usize) -> u8 {
    let accumulated = raw_status / 4 + roller_state as i32 * 64;
    accumulated.clamp(0, 127) as u8
}

pub fn button_cc_value(is_down: bool) -> u8 {
    if is_down { 127 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoder_zero() {
        assert_eq!(normalize_encoder(0, 0), 0);
    }

    #[test]
    fn encoder_mid_range() {
        assert_eq!(normalize_encoder(128, 1), 96);
    }

    #[test]
    fn encoder_clamps_high() {
        assert_eq!(normalize_encoder(252, 3), 127);
    }

    #[test]
    fn encoder_clamps_low() {
        assert_eq!(normalize_encoder(-100, 0), 0);
    }

    #[test]
    fn button_press_gives_127() {
        assert_eq!(button_cc_value(true), 127);
    }

    #[test]
    fn button_release_gives_0() {
        assert_eq!(button_cc_value(false), 0);
    }
}
