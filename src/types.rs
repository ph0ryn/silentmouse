use crate::error::SilentMouseError;

pub const DEFAULT_CLICK_DURATION_MS: u64 = 80;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenPoint {
    pub x: f64,
    pub y: f64,
}

impl ScreenPoint {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowPoint {
    pub x: f64,
    pub y: f64,
}

impl WindowPoint {
    pub fn new(x: f64, y: f64) -> Result<Self, SilentMouseError> {
        if !x.is_finite() {
            return Err(SilentMouseError::InvalidCoordinate {
                axis: "x",
                value: x,
            });
        }
        if !y.is_finite() {
            return Err(SilentMouseError::InvalidCoordinate {
                axis: "y",
                value: y,
            });
        }
        Ok(Self { x, y })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn screen_point(&self, window_point: WindowPoint) -> ScreenPoint {
        ScreenPoint {
            x: self.x + window_point.x,
            y: self.y + window_point.y,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowTarget {
    pub window_id: u32,
    pub pid: i32,
    pub bounds: Rect,
    pub is_active: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClickResult {
    pub window_id: u32,
    pub pid: i32,
    pub target_was_active: bool,
    pub used_background_flag: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_screen_point_from_window_point() {
        let rect = Rect {
            x: -611.0,
            y: 254.0,
            width: 500.0,
            height: 532.0,
        };

        assert_eq!(
            rect.screen_point(WindowPoint { x: 250.0, y: 266.0 }),
            ScreenPoint {
                x: -361.0,
                y: 520.0
            }
        );
    }

    #[test]
    fn rejects_non_finite_coordinates() {
        assert!(matches!(
            WindowPoint::new(f64::NAN, 1.0),
            Err(SilentMouseError::InvalidCoordinate { axis: "x", .. })
        ));
        assert!(matches!(
            WindowPoint::new(1.0, f64::INFINITY),
            Err(SilentMouseError::InvalidCoordinate { axis: "y", .. })
        ));
    }
}
