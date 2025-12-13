/// Represents a single character entry for the Screen buffer.
///
/// Each `Entry` consists of a character and a color attribute. The color is set
/// to the default color (light gray on black) by default, but it can be
/// customized. Each `Entry` can be converted into a `u16` value, which is the
/// format used for writing to the VGA buffer.
#[derive(Clone, Copy, Debug)]
pub struct Entry {
    color: u8,
    character: u8,
}

impl Entry {
    /// Creates a new `Entry` with the specified character and the default
    /// color.
    ///
    /// The default color is light gray (`0x07`).
    ///
    /// ### Parameters:
    /// - `character`: The character to be storedy.
    #[must_use]
    pub const fn new(character: u8) -> Self {
        Entry {
            color: Color::Default as u8,
            character,
        }
    }

    /// Creates a new `Entry` with the specified character and custom color.
    ///
    /// This function allows the creation of a `Entry` with a specific character
    /// and color, where the color is passed as a parameter. The color is
    /// represented as an 8-bit value, allowing for a wide range of color
    /// codes (e.g., for screen colors). The character is displayed with
    /// this color when rendered to the VGA buffer.
    ///
    /// ### Parameters:
    /// - `character`: The character to be displayed (e.g., an ASCII value representing a letter or
    ///   symbol).
    /// - `color`: The color code for the character (an 8-bit value that determines the character's
    ///   color).
    ///   - The value should correspond to a color in the VGA color palette (for example, `0x0F` for
    ///     white, `0x01` for blue, etc.).
    #[must_use]
    pub fn new_with_color(character: u8, color: u8) -> Self {
        Entry { color, character }
    }

    /// Converts this `Entry` into a `u16` value that can be written to the VGA
    /// buffer.
    ///
    /// The `u16` format stores the color in the upper 8 bits and the character
    /// in the lower 8 bits.
    ///
    /// ### Returns:
    /// A `u16` value representing this `Entry`.
    #[must_use]
    pub const fn to_u16(&self) -> u16 {
        ((self.color as u16) << 8) | (self.character as u16)
    }

    #[must_use]
    pub const fn get_character(&self) -> u8 {
        self.character
    }
}

/// Represents the available color codes for screen entries.
///
/// The colors are defined as `u8` values, where each value corresponds to a
/// particular color. The default color is light gray on black.
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Color {
    /// Light gray on black (default)
    Default = 0x07,
    /// White on Red
    #[allow(unused)]
    Error = 0x4F,
}
