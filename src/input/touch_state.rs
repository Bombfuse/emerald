use miniquad::TouchPhase;

use crate::Position;

#[derive(Debug, Clone, Copy)]
pub struct TouchState {
    pub position: Position,
    pub previous: TouchPhase,
    pub phase: TouchPhase,
}

impl Default for TouchState {
    fn default() -> Self {
        TouchState {
            position: Position::zero(),
            previous: TouchPhase::Cancelled,
            phase: TouchPhase::Cancelled,
        }
    }
}

impl TouchState {
    #[inline]
    pub fn was_pressed(&self) -> bool {
        self.previous == TouchPhase::Started || self.previous == TouchPhase::Moved
    }

    #[inline]
    pub fn is_pressed(&self) -> bool {
        self.phase == TouchPhase::Started || self.phase == TouchPhase::Moved
    }

    #[inline]
    pub fn is_just_pressed(&self) -> bool {
        !self.was_pressed() && self.is_pressed()
    }

    #[inline]
    pub fn is_just_released(&self) -> bool {
        self.was_pressed() && !self.is_pressed()
    }

    pub(crate) fn is_outdated(&self) -> bool {
        !self.was_pressed() && !self.is_pressed()
    }

    pub(crate) fn rollover(&mut self) {
        self.previous = self.phase;
    }
}
