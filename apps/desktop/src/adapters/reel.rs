//! The reel port, unfilled. A brand film needs a picture, and a picture needs a surface.

use crate::ports::Reel;

pub struct NoReel;

impl Reel for NoReel {
    fn plays(&self) -> bool {
        false
    }
}
