//! Game colour conversion.

/// A colour with channels from 0.0 to 1.0.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    /// Red channel.
    pub red: f32,
    /// Green channel.
    pub green: f32,
    /// Blue channel.
    pub blue: f32,
    /// Alpha channel.
    pub alpha: f32,
}

impl Color {
    /// An opaque colour.
    pub fn rgb(red: f32, green: f32, blue: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha: 1.0,
        }
    }

    /// A colour with an explicit alpha.
    pub fn rgba(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    /// Reads the game's packed `0xAABBGGRR` form.
    pub fn from_packed(packed: u32) -> Self {
        Self {
            red: channel(packed & 0xff),
            green: channel((packed >> 8) & 0xff),
            blue: channel((packed >> 16) & 0xff),
            alpha: channel(packed >> 24),
        }
    }

    /// Writes the game's packed `0xAABBGGRR` form.
    pub fn packed(self) -> u32 {
        pack(self.alpha) << 24 | pack(self.blue) << 16 | pack(self.green) << 8 | pack(self.red)
    }
}

fn channel(value: u32) -> f32 {
    value as f32 / 255.0
}

fn pack(value: f32) -> u32 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_and_unpacks() {
        assert_eq!(Color::rgb(1.0, 1.0, 1.0).packed(), 0xffff_ffff);
        assert_eq!(Color::rgb(1.0, 0.0, 0.0).packed(), 0xff00_00ff);
        assert_eq!(Color::rgb(0.0, 1.0, 0.0).packed(), 0xff00_ff00);
        assert_eq!(Color::rgb(0.0, 0.0, 1.0).packed(), 0xffff_0000);
        assert_eq!(Color::from_packed(0xff00_00ff), Color::rgb(1.0, 0.0, 0.0));
        assert_eq!(Color::from_packed(0).packed(), 0);
    }

    #[test]
    fn clamps_channels() {
        assert_eq!(Color::rgb(-1.0, 2.0, 0.5).packed(), 0xff80_ff00);
    }
}
