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
    pub fn new(character: Character, modifiers: ModifierState) -> Self {
        Self { character, modifiers }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Layout {
    map: fn(key: Key, modifiers: ModifierState) -> Option<CharacterFull>,
}

impl Layout {
    pub fn new(map: fn(key: Key, modifiers: ModifierState) -> Option<CharacterFull>) -> Self {
        Self { map }
    }

    pub fn map(&self, key: Key, modifiers: ModifierState) -> Option<CharacterFull> {
        (self.map)(key, modifiers)
    }
}

pub fn map_qwerty(key: Key, modifiers: ModifierState) -> Option<CharacterFull> {
    use Key::*;
    let c = match key {
        // Now list all keys unaffected by shift
        Escape => Some(Character::Escape),
        Tab => Some(Character::Tab),
        Enter => Some(Character::Enter),
        ArrowUp => Some(Character::ArrowUp),
        Backspace => Some(Character::Backspace),
        ArrowDown => Some(Character::ArrowDown),
        ArrowLeft => Some(Character::ArrowLeft),
        ArrowRight => Some(Character::ArrowRight),
        CapsLock => Some(Character::CapsLock),
        NumLock => Some(Character::NumLock),
        ScrollLock => Some(Character::ScrollLock),
        F1 => Some(Character::F1),
        F2 => Some(Character::F2),
        F3 => Some(Character::F3),
        F4 => Some(Character::F4),
        F5 => Some(Character::F5),
        F6 => Some(Character::F6),
        F7 => Some(Character::F7),
        F8 => Some(Character::F8),
        F9 => Some(Character::F9),
        F10 => Some(Character::F10),
        F11 => Some(Character::F11),
        F12 => Some(Character::F12),
        KeypadAdd => Some(Character::KeypadAdd),
        KeypadSub => Some(Character::KeypadSub),
        KeypadMul => Some(Character::KeypadMul),
        KeypadDiv => Some(Character::KeypadDiv),
        KeypadEnter => Some(Character::KeypadEnter),
        KeypadN0 => Some(Character::KeypadN0),
        KeypadN1 => Some(Character::KeypadN1),
        KeypadN2 => Some(Character::KeypadN2),
        KeypadN3 => Some(Character::KeypadN3),
        KeypadN4 => Some(Character::KeypadN4),
        KeypadN5 => Some(Character::KeypadN5),
        KeypadN6 => Some(Character::KeypadN6),
        KeypadN7 => Some(Character::KeypadN7),
        KeypadN8 => Some(Character::KeypadN8),
        KeypadN9 => Some(Character::KeypadN9),
        KeypadComma => Some(Character::KeypadComma),

        LeftAlt => None,
        RightAlt => None,
        LeftShift => None,
        RightShift => None,
        LeftCtrl => None,
        RightCtrl => None,
        LeftGui => None,
        RightGui => None,
        CapsLock => None,
        NumLock => None,
        ScrollLock => None,

        A => Some(Character::Char(if modifiers.shift_pressed { 'A' } else { 'a' })),
        B => Some(Character::Char(if modifiers.shift_pressed { 'B' } else { 'b' })),
        C => Some(Character::Char(if modifiers.shift_pressed { 'C' } else { 'c' })),
        D => Some(Character::Char(if modifiers.shift_pressed { 'D' } else { 'd' })),
        E => Some(Character::Char(if modifiers.shift_pressed { 'E' } else { 'e' })),
        F => Some(Character::Char(if modifiers.shift_pressed { 'F' } else { 'f' })),
        G => Some(Character::Char(if modifiers.shift_pressed { 'G' } else { 'g' })),
        H => Some(Character::Char(if modifiers.shift_pressed { 'H' } else { 'h' })),
        I => Some(Character::Char(if modifiers.shift_pressed { 'I' } else { 'i' })),
        J => Some(Character::Char(if modifiers.shift_pressed { 'J' } else { 'j' })),
        K => Some(Character::Char(if modifiers.shift_pressed { 'K' } else { 'k' })),
        L => Some(Character::Char(if modifiers.shift_pressed { 'L' } else { 'l' })),
        M => Some(Character::Char(if modifiers.shift_pressed { 'M' } else { 'm' })),
        N => Some(Character::Char(if modifiers.shift_pressed { 'N' } else { 'n' })),
        O => Some(Character::Char(if modifiers.shift_pressed { 'O' } else { 'o' })),
        P => Some(Character::Char(if modifiers.shift_pressed { 'P' } else { 'p' })),
        Q => Some(Character::Char(if modifiers.shift_pressed { 'Q' } else { 'q' })),
        R => Some(Character::Char(if modifiers.shift_pressed { 'R' } else { 'r' })),
        S => Some(Character::Char(if modifiers.shift_pressed { 'S' } else { 's' })),
        T => Some(Character::Char(if modifiers.shift_pressed { 'T' } else { 't' })),
        U => Some(Character::Char(if modifiers.shift_pressed { 'U' } else { 'u' })),
        V => Some(Character::Char(if modifiers.shift_pressed { 'V' } else { 'v' })),
        W => Some(Character::Char(if modifiers.shift_pressed { 'W' } else { 'w' })),
        X => Some(Character::Char(if modifiers.shift_pressed { 'X' } else { 'x' })),
        Y => Some(Character::Char(if modifiers.shift_pressed { 'Y' } else { 'y' })),
        Z => Some(Character::Char(if modifiers.shift_pressed { 'Z' } else { 'z' })),
        N0 => Some(Character::Char(if modifiers.shift_pressed { ')' } else { '0' })),
        N1 => Some(Character::Char(if modifiers.shift_pressed { '!' } else { '1' })),
        N2 => Some(Character::Char(if modifiers.shift_pressed { '@' } else { '2' })),
        N3 => Some(Character::Char(if modifiers.shift_pressed { '#' } else { '3' })),
        N4 => Some(Character::Char(if modifiers.shift_pressed { '$' } else { '4' })),
        N5 => Some(Character::Char(if modifiers.shift_pressed { '%' } else { '5' })),
        N6 => Some(Character::Char(if modifiers.shift_pressed { '^' } else { '6' })),
        N7 => Some(Character::Char(if modifiers.shift_pressed { '&' } else { '7' })),
        N8 => Some(Character::Char(if modifiers.shift_pressed { '*' } else { '8' })),
        N9 => Some(Character::Char(if modifiers.shift_pressed { '(' } else { '9' })),
        Point => Some(Character::Char(if modifiers.shift_pressed { '>' } else { '.' })),
        Equal => Some(Character::Char(if modifiers.shift_pressed { '+' } else { '=' })),
        Minus => Some(Character::Char(if modifiers.shift_pressed { '_' } else { '-' })),
        Comma => Some(Character::Char(if modifiers.shift_pressed { '<' } else { ',' })),
        Backtick => Some(Character::Char(if modifiers.shift_pressed { '~' } else { '`' })),
        Semicolon => Some(Character::Char(if modifiers.shift_pressed { ':' } else { ';' })),
        Slash => Some(Character::Char(if modifiers.shift_pressed { '?' } else { '/' })),
        Backslash => Some(Character::Char(if modifiers.shift_pressed { '|' } else { '\\' })),
        SingleQuote => Some(Character::Char(if modifiers.shift_pressed { '\"' } else { '\'' })),
        SquareBracketsOpen => Some(Character::Char(if modifiers.shift_pressed { '{' } else { '}' })),
        SquareBracketsClosed => Some(Character::Char(if modifiers.shift_pressed { '}' } else { ']' })),
        Space => Some(Character::Char(' ')),
    }?;

    Some(CharacterFull::new(c, modifiers))
}
