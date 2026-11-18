/*! Shared document rules for the worker and native server. */
pub mod scene;

pub const BOARD_WIDTH: i32 = 480;
pub const BOARD_HEIGHT: i32 = 600;

pub fn fits_board(x: i32, y: i32, width: i32, height: i32) -> bool {
    width > 0
        && height > 0
        && x >= 0
        && y >= 0
        && width <= BOARD_WIDTH
        && height <= BOARD_HEIGHT
        && x <= BOARD_WIDTH - width
        && y <= BOARD_HEIGHT - height
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds_include_edges_and_reject_overflow() {
        assert!(fits_board(0, 0, 480, 600));
        assert!(fits_board(479, 599, 1, 1));
        assert!(!fits_board(480, 599, 1, 1));
        assert!(!fits_board(0, 0, 0, 1));
        assert!(!fits_board(i32::MAX, 0, 2, 2));
    }
}
