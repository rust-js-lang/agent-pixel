/*! Shared document rules for the worker and native server. */
pub mod assets;
pub mod bundle;
pub mod hierarchy;

pub mod scene;

use serde::{Deserialize, Serialize};

pub const BOARD_WIDTH: i32 = 480;
pub const BOARD_HEIGHT: i32 = 600;

pub const MIN_ARTBOARD_SIZE: i32 = 64;
pub const MAX_ARTBOARD_SIZE: i32 = 8192;

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Dimensions {
    pub width: i32,
    pub height: i32,
}

impl Default for Dimensions {
    fn default() -> Self {
        Self {
            width: BOARD_WIDTH,
            height: BOARD_HEIGHT,
        }
    }
}

impl Dimensions {
    pub const LARGEST: Self = Self {
        width: MAX_ARTBOARD_SIZE,
        height: MAX_ARTBOARD_SIZE,
    };

    pub fn is_default(&self) -> bool {
        self.width == BOARD_WIDTH && self.height == BOARD_HEIGHT
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if !(MIN_ARTBOARD_SIZE..=MAX_ARTBOARD_SIZE).contains(&self.width)
            || !(MIN_ARTBOARD_SIZE..=MAX_ARTBOARD_SIZE).contains(&self.height)
        {
            return Err("Artboard width and height must be between 64 and 8192 pixels.");
        }
        Ok(())
    }

    pub fn fits(&self, x: i32, y: i32, width: i32, height: i32) -> bool {
        width > 0
            && height > 0
            && x >= 0
            && y >= 0
            && width <= self.width
            && height <= self.height
            && x <= self.width - width
            && y <= self.height - height
    }
}

pub fn fits_board(x: i32, y: i32, width: i32, height: i32) -> bool {
    Dimensions::default().fits(x, y, width, height)
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
