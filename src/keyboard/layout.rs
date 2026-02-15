use crate::{keyboard::ModifierState, ps2::Key};

#[derive(Clone, Copy, Debug)]
pub enum Character {
    Char(char),
    Escape,
    Tab,
    Enter,
    ArrowUp,
    Backspace,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    CapsLock,
    NumLock,
    ScrollLock,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    KeypadAdd,
    KeypadSub,
    KeypadMul,
    KeypadDiv,
    KeypadEnter,
    KeypadN0,
    KeypadN1,
    KeypadN2,
    KeypadN3,
    KeypadN4,
    KeypadN5,
    KeypadN6,
    KeypadN7,
    KeypadN8,
    KeypadN9,
    KeypadComma,
}

#[derive(Clone, Copy, Debug)]
pub struct CharacterFull {
    pub character: Character,
    pub modifiers: ModifierState,
}

impl CharacterFull {
    #[must_use]
    pub fn new(character: Character, modifiers: ModifierState) -> Self {
        Self { character, modifiers }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Layout {
    map: fn(key: Key, modifiers: ModifierState) -> Option<CharacterFull>,
}

impl Layout {
    #[must_use]
    pub fn new(map: fn(key: Key, modifiers: ModifierState) -> Option<CharacterFull>) -> Self {
        Self { map }
    }

    #[must_use]
    pub fn map(&self, key: Key, modifiers: ModifierState) -> Option<CharacterFull> {
        (self.map)(key, modifiers)
    }
}

#[must_use]
pub fn map_qwerty(key: Key, modifiers: ModifierState) -> Option<CharacterFull> {
    let c = match key {
        Key::Escape => Some(Character::Escape),
        Key::Tab => Some(Character::Tab),
        Key::Enter => Some(Character::Enter),
        Key::ArrowUp => Some(Character::ArrowUp),
        Key::Backspace => Some(Character::Backspace),
        Key::ArrowDown => Some(Character::ArrowDown),
        Key::ArrowLeft => Some(Character::ArrowLeft),
        Key::ArrowRight => Some(Character::ArrowRight),
        Key::CapsLock => Some(Character::CapsLock),
        Key::NumLock => Some(Character::NumLock),
        Key::ScrollLock => Some(Character::ScrollLock),
        Key::F1 => Some(Character::F1),
        Key::F2 => Some(Character::F2),
        Key::F3 => Some(Character::F3),
        Key::F4 => Some(Character::F4),
        Key::F5 => Some(Character::F5),
        Key::F6 => Some(Character::F6),
        Key::F7 => Some(Character::F7),
        Key::F8 => Some(Character::F8),
        Key::F9 => Some(Character::F9),
        Key::F10 => Some(Character::F10),
        Key::F11 => Some(Character::F11),
        Key::F12 => Some(Character::F12),
        Key::KeypadAdd => Some(Character::KeypadAdd),
        Key::KeypadSub => Some(Character::KeypadSub),
        Key::KeypadMul => Some(Character::KeypadMul),
        Key::KeypadDiv => Some(Character::KeypadDiv),
        Key::KeypadEnter => Some(Character::KeypadEnter),
        Key::KeypadN0 => Some(Character::KeypadN0),
        Key::KeypadN1 => Some(Character::KeypadN1),
        Key::KeypadN2 => Some(Character::KeypadN2),
        Key::KeypadN3 => Some(Character::KeypadN3),
        Key::KeypadN4 => Some(Character::KeypadN4),
        Key::KeypadN5 => Some(Character::KeypadN5),
        Key::KeypadN6 => Some(Character::KeypadN6),
        Key::KeypadN7 => Some(Character::KeypadN7),
        Key::KeypadN8 => Some(Character::KeypadN8),
        Key::KeypadN9 => Some(Character::KeypadN9),
        Key::KeypadComma => Some(Character::KeypadComma),
        Key::LeftAlt => None,
        Key::RightAlt => None,
        Key::LeftShift => None,
        Key::RightShift => None,
        Key::LeftCtrl => None,
        Key::RightCtrl => None,
        Key::LeftGui => None,
        Key::RightGui => None,
        Key::A => Some(Character::Char(if modifiers.shift_pressed { 'A' } else { 'a' })),
        Key::B => Some(Character::Char(if modifiers.shift_pressed { 'B' } else { 'b' })),
        Key::C => Some(Character::Char(if modifiers.shift_pressed { 'C' } else { 'c' })),
        Key::D => Some(Character::Char(if modifiers.shift_pressed { 'D' } else { 'd' })),
        Key::E => Some(Character::Char(if modifiers.shift_pressed { 'E' } else { 'e' })),
        Key::F => Some(Character::Char(if modifiers.shift_pressed { 'F' } else { 'f' })),
        Key::G => Some(Character::Char(if modifiers.shift_pressed { 'G' } else { 'g' })),
        Key::H => Some(Character::Char(if modifiers.shift_pressed { 'H' } else { 'h' })),
        Key::I => Some(Character::Char(if modifiers.shift_pressed { 'I' } else { 'i' })),
        Key::J => Some(Character::Char(if modifiers.shift_pressed { 'J' } else { 'j' })),
        Key::K => Some(Character::Char(if modifiers.shift_pressed { 'K' } else { 'k' })),
        Key::L => Some(Character::Char(if modifiers.shift_pressed { 'L' } else { 'l' })),
        Key::M => Some(Character::Char(if modifiers.shift_pressed { 'M' } else { 'm' })),
        Key::N => Some(Character::Char(if modifiers.shift_pressed { 'N' } else { 'n' })),
        Key::O => Some(Character::Char(if modifiers.shift_pressed { 'O' } else { 'o' })),
        Key::P => Some(Character::Char(if modifiers.shift_pressed { 'P' } else { 'p' })),
        Key::Q => Some(Character::Char(if modifiers.shift_pressed { 'Q' } else { 'q' })),
        Key::R => Some(Character::Char(if modifiers.shift_pressed { 'R' } else { 'r' })),
        Key::S => Some(Character::Char(if modifiers.shift_pressed { 'S' } else { 's' })),
        Key::T => Some(Character::Char(if modifiers.shift_pressed { 'T' } else { 't' })),
        Key::U => Some(Character::Char(if modifiers.shift_pressed { 'U' } else { 'u' })),
        Key::V => Some(Character::Char(if modifiers.shift_pressed { 'V' } else { 'v' })),
        Key::W => Some(Character::Char(if modifiers.shift_pressed { 'W' } else { 'w' })),
        Key::X => Some(Character::Char(if modifiers.shift_pressed { 'X' } else { 'x' })),
        Key::Y => Some(Character::Char(if modifiers.shift_pressed { 'Y' } else { 'y' })),
        Key::Z => Some(Character::Char(if modifiers.shift_pressed { 'Z' } else { 'z' })),
        Key::N0 => Some(Character::Char(if modifiers.shift_pressed { ')' } else { '0' })),
        Key::N1 => Some(Character::Char(if modifiers.shift_pressed { '!' } else { '1' })),
        Key::N2 => Some(Character::Char(if modifiers.shift_pressed { '@' } else { '2' })),
        Key::N3 => Some(Character::Char(if modifiers.shift_pressed { '#' } else { '3' })),
        Key::N4 => Some(Character::Char(if modifiers.shift_pressed { '$' } else { '4' })),
        Key::N5 => Some(Character::Char(if modifiers.shift_pressed { '%' } else { '5' })),
        Key::N6 => Some(Character::Char(if modifiers.shift_pressed { '^' } else { '6' })),
        Key::N7 => Some(Character::Char(if modifiers.shift_pressed { '&' } else { '7' })),
        Key::N8 => Some(Character::Char(if modifiers.shift_pressed { '*' } else { '8' })),
        Key::N9 => Some(Character::Char(if modifiers.shift_pressed { '(' } else { '9' })),
        Key::Point => Some(Character::Char(if modifiers.shift_pressed { '>' } else { '.' })),
        Key::Equal => Some(Character::Char(if modifiers.shift_pressed { '+' } else { '=' })),
        Key::Minus => Some(Character::Char(if modifiers.shift_pressed { '_' } else { '-' })),
        Key::Comma => Some(Character::Char(if modifiers.shift_pressed { '<' } else { ',' })),
        Key::Backtick => Some(Character::Char(if modifiers.shift_pressed { '~' } else { '`' })),
        Key::Semicolon => Some(Character::Char(if modifiers.shift_pressed { ':' } else { ';' })),
        Key::Slash => Some(Character::Char(if modifiers.shift_pressed { '?' } else { '/' })),
        Key::Backslash => Some(Character::Char(if modifiers.shift_pressed { '|' } else { '\\' })),
        Key::SingleQuote => Some(Character::Char(if modifiers.shift_pressed { '\"' } else { '\'' })),
        Key::SquareBracketsOpen => Some(Character::Char(if modifiers.shift_pressed { '{' } else { '}' })),
        Key::SquareBracketsClosed => Some(Character::Char(if modifiers.shift_pressed { '}' } else { ']' })),
        Key::Space => Some(Character::Char(' ')),
    }?;

    Some(CharacterFull::new(c, modifiers))
}
