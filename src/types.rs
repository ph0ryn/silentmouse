use crate::error::SilentMouseError;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenPoint {
    pub x: f64,
    pub y: f64,
}

impl ScreenPoint {
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
    pub fn local_point(&self, screen_point: ScreenPoint) -> ScreenPoint {
        ScreenPoint {
            x: screen_point.x - self.x,
            y: screen_point.y - self.y,
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
    fn computes_window_local_point() {
        let rect = Rect {
            x: 100.0,
            y: 200.0,
            width: 500.0,
            height: 300.0,
        };

        assert_eq!(
            rect.local_point(ScreenPoint { x: 125.5, y: 250.0 }),
            ScreenPoint { x: 25.5, y: 50.0 }
        );
    }

    #[test]
    fn rejects_non_finite_coordinates() {
        assert!(matches!(
            ScreenPoint::new(f64::NAN, 1.0),
            Err(SilentMouseError::InvalidCoordinate { axis: "x", .. })
        ));
        assert!(matches!(
            ScreenPoint::new(1.0, f64::INFINITY),
            Err(SilentMouseError::InvalidCoordinate { axis: "y", .. })
        ));
    }
}
