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
    character: Character,
    modifiers: ModifierState,
}

impl CharacterFull {
    pub fn new(character: Character, modifiers: ModifierState) -> Self {
        Self { character, modifiers }
    }
}

pub struct Layout {
    layer_base: [Option<Character>; 256],
    layer_shift: [Character<Character>; 256],
}

impl Layout {
    pub fn map(&self, key: Key, modifiers: ModifierState) -> Option<CharacterFull> {
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

            A => match modifiers.shift_pressed {
                true => Some(Character::Char('A')),
                false => Some(Character::Char('a')),
            },
            B => match modifiers.shift_pressed {
                true => Some(Character::Char('B')),
                false => Some(Character::Char('b')),
            },
            C => match modifiers.shift_pressed {
                true => Some(Character::Char('C')),
                false => Some(Character::Char('c')),
            },
            D => match modifiers.shift_pressed {
                true => Some(Character::Char('D')),
                false => Some(Character::Char('d')),
            },
            E => match modifiers.shift_pressed {
                true => Some(Character::Char('E')),
                false => Some(Character::Char('e')),
            },
            F => match modifiers.shift_pressed {
                true => Some(Character::Char('F')),
                false => Some(Character::Char('f')),
            },
            G => match modifiers.shift_pressed {
                true => Some(Character::Char('G')),
                false => Some(Character::Char('g')),
            },
            H => match modifiers.shift_pressed {
                true => Some(Character::Char('H')),
                false => Some(Character::Char('h')),
            },
            I => match modifiers.shift_pressed {
                true => Some(Character::Char('I')),
                false => Some(Character::Char('i')),
            },
            J => match modifiers.shift_pressed {
                true => Some(Character::Char('J')),
                false => Some(Character::Char('j')),
            },
            K => match modifiers.shift_pressed {
                true => Some(Character::Char('K')),
                false => Some(Character::Char('k')),
            },
            L => match modifiers.shift_pressed {
                true => Some(Character::Char('L')),
                false => Some(Character::Char('l')),
            },
            M => match modifiers.shift_pressed {
                true => Some(Character::Char('M')),
                false => Some(Character::Char('m')),
            },
            N => match modifiers.shift_pressed {
                true => Some(Character::Char('N')),
                false => Some(Character::Char('n')),
            },
            O => match modifiers.shift_pressed {
                true => Some(Character::Char('O')),
                false => Some(Character::Char('o')),
            },
            P => match modifiers.shift_pressed {
                true => Some(Character::Char('P')),
                false => Some(Character::Char('p')),
            },
            Q => match modifiers.shift_pressed {
                true => Some(Character::Char('Q')),
                false => Some(Character::Char('q')),
            },
            R => match modifiers.shift_pressed {
                true => Some(Character::Char('R')),
                false => Some(Character::Char('r')),
            },
            S => match modifiers.shift_pressed {
                true => Some(Character::Char('S')),
                false => Some(Character::Char('s')),
            },
            T => match modifiers.shift_pressed {
                true => Some(Character::Char('T')),
                false => Some(Character::Char('t')),
            },
            U => match modifiers.shift_pressed {
                true => Some(Character::Char('U')),
                false => Some(Character::Char('u')),
            },
            V => match modifiers.shift_pressed {
                true => Some(Character::Char('V')),
                false => Some(Character::Char('v')),
            },
            W => match modifiers.shift_pressed {
                true => Some(Character::Char('W')),
                false => Some(Character::Char('w')),
            },
            X => match modifiers.shift_pressed {
                true => Some(Character::Char('X')),
                false => Some(Character::Char('x')),
            },
            Y => match modifiers.shift_pressed {
                true => Some(Character::Char('Y')),
                false => Some(Character::Char('y')),
            },
            Z => match modifiers.shift_pressed {
                true => Some(Character::Char('Z')),
                false => Some(Character::Char('z')),
            },
            N0 => match modifiers.shift_pressed {
                true => Some(Character::Char(')')),
                false => Some(Character::Char('0')),
            },
            N1 => match modifiers.shift_pressed {
                true => Some(Character::Char('!')),
                false => Some(Character::Char('1')),
            },
            N2 => match modifiers.shift_pressed {
                true => Some(Character::Char('@')),
                false => Some(Character::Char('2')),
            },
            N3 => match modifiers.shift_pressed {
                true => Some(Character::Char('#')),
                false => Some(Character::Char('3')),
            },
            N4 => match modifiers.shift_pressed {
                true => Some(Character::Char('$')),
                false => Some(Character::Char('4')),
            },
            N5 => match modifiers.shift_pressed {
                true => Some(Character::Char('%')),
                false => Some(Character::Char('5')),
            },
            N6 => match modifiers.shift_pressed {
                true => Some(Character::Char('^')),
                false => Some(Character::Char('6')),
            },
            N7 => match modifiers.shift_pressed {
                true => Some(Character::Char('&')),
                false => Some(Character::Char('7')),
            },
            N8 => match modifiers.shift_pressed {
                true => Some(Character::Char('*')),
                false => Some(Character::Char('8')),
            },
            N9 => match modifiers.shift_pressed {
                true => Some(Character::Char('(')),
                false => Some(Character::Char('9')),
            },
            Point => match modifiers.shift_pressed {
                true => Some(Character::Char('>')),
                false => Some(Character::Char('.')),
            },
            Space => Some(Character::Char(' ')),
            Equal => match modifiers.shift_pressed {
                true => Some(Character::Char('+')),
                false => Some(Character::Char('=')),
            },
            Minus => match modifiers.shift_pressed {
                true => Some(Character::Char('_')),
                false => Some(Character::Char('-')),
            },
            Comma => match modifiers.shift_pressed {
                true => Some(Character::Char('<')),
                false => Some(Character::Char(',')),
            },
            Backtick => match modifiers.shift_pressed {
                true => Some(Character::Char('~')),
                false => Some(Character::Char('`')),
            },
            Semicolon => match modifiers.shift_pressed {
                true => Some(Character::Char(':')),
                false => Some(Character::Char(';')),
            },
            Slash => match modifiers.shift_pressed {
                true => Some(Character::Char('?')),
                false => Some(Character::Char('/')),
            },
            Backslash => match modifiers.shift_pressed {
                true => Some(Character::Char('|')),
                false => Some(Character::Char('\\')),
            },
            SingleQuote => match modifiers.shift_pressed {
                true => Some(Character::Char('\"')),
                false => Some(Character::Char('\'')),
            },
            SquareBracketsOpen => match modifiers.shift_pressed {
                true => Some(Character::Char('{')),
                false => Some(Character::Char('[')),
            },
            SquareBracketsClosed => match modifiers.shift_pressed {
                true => Some(Character::Char('}')),
                false => Some(Character::Char(']')),
            },
        }?;

        Some(CharacterFull::new(c, modifiers))
    }
}
