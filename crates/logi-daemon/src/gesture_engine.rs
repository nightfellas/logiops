use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GestureDirection {
    None,
    Click,
    Up,
    Down,
    Left,
    Right,
}

pub struct GestureTracker {
    is_holding: bool,
    accumulated_dx: i32,
    accumulated_dy: i32,
    threshold: i32,
    triggered_direction: GestureDirection,
}

impl GestureTracker {
    pub fn new(threshold: i32) -> Self {
        Self {
            is_holding: false,
            accumulated_dx: 0,
            accumulated_dy: 0,
            threshold,
            triggered_direction: GestureDirection::None,
        }
    }

    pub fn on_button_down(&mut self) {
        self.is_holding = true;
        self.accumulated_dx = 0;
        self.accumulated_dy = 0;
        self.triggered_direction = GestureDirection::None;
    }

    /// Feeds relative mouse motion (dx, dy).
    /// Returns Some(GestureDirection) the instant a threshold is reached for the first time in this hold.
    pub fn on_motion(&mut self, dx: i32, dy: i32) -> Option<GestureDirection> {
        if !self.is_holding || self.triggered_direction != GestureDirection::None {
            return None;
        }

        self.accumulated_dx += dx;
        self.accumulated_dy += dy;

        let abs_x = self.accumulated_dx.abs();
        let abs_y = self.accumulated_dy.abs();

        if abs_x >= self.threshold || abs_y >= self.threshold {
            let direction = if abs_x > abs_y {
                if self.accumulated_dx > 0 {
                    GestureDirection::Right
                } else {
                    GestureDirection::Left
                }
            } else {
                if self.accumulated_dy > 0 {
                    GestureDirection::Down
                } else {
                    GestureDirection::Up
                }
            };

            self.triggered_direction = direction;
            return Some(direction);
        }

        None
    }

    /// Called when the gesture button is released.
    /// If no directional gesture was triggered, returns `GestureDirection::Click`.
    pub fn on_button_up(&mut self) -> GestureDirection {
        let result = if self.triggered_direction != GestureDirection::None {
            GestureDirection::None // Already triggered while moving
        } else if self.is_holding {
            GestureDirection::Click
        } else {
            GestureDirection::None
        };

        self.is_holding = false;
        self.accumulated_dx = 0;
        self.accumulated_dy = 0;
        self.triggered_direction = GestureDirection::None;

        result
    }

    pub fn is_holding(&self) -> bool {
        self.is_holding
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_click_without_motion() {
        let mut tracker = GestureTracker::new(50);
        tracker.on_button_down();
        let motion_res = tracker.on_motion(5, 5);
        assert_eq!(motion_res, None);

        let release_res = tracker.on_button_up();
        assert_eq!(release_res, GestureDirection::Click);
    }

    #[test]
    fn test_gesture_right_swipe() {
        let mut tracker = GestureTracker::new(50);
        tracker.on_button_down();
        assert_eq!(tracker.on_motion(30, 5), None);
        // Exceed horizontal threshold
        assert_eq!(tracker.on_motion(25, 5), Some(GestureDirection::Right));
        // Additional motion ignored during same hold
        assert_eq!(tracker.on_motion(100, 0), None);

        let release_res = tracker.on_button_up();
        assert_eq!(release_res, GestureDirection::None);
    }

    #[test]
    fn test_gesture_up_swipe() {
        let mut tracker = GestureTracker::new(50);
        tracker.on_button_down();
        assert_eq!(tracker.on_motion(0, -60), Some(GestureDirection::Up));
        assert_eq!(tracker.on_button_up(), GestureDirection::None);
    }
}
