use crate::{
    keyboard::layout::{Character, CharacterFull, Layout},
    ps2::{self, Key},
};

pub mod layout;

#[derive(Clone, Copy, Debug)]
pub struct ModifierState {
    shift_pressed: bool,
    ctrl_pressed: bool,
    alt_pressed: bool,
    gui_pressed: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Keyboard {
    modifier: ModifierState,
    layout: Layout,
}

impl Keyboard {
    #[must_use]
    pub fn new(layout: Layout) -> Self {
        Self {
            modifier: ModifierState {
                shift_pressed: false,
                ctrl_pressed: false,
                alt_pressed: false,
                gui_pressed: false,
            },
            layout,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<Character> {
        let c = self.next_full()?;
        Some(c.character)
    }

    pub fn next_full(&mut self) -> Option<CharacterFull> {
        while let Some(key_event) = ps2::read_key_event() {
            use Key::*;
            use ps2::Event::*;
            match key_event.key {
                LeftShift | RightShift => match key_event.event {
                    Pressed => self.modifier.shift_pressed = true,
                    Released => self.modifier.shift_pressed = false,
                },
                LeftCtrl | RightCtrl => match key_event.event {
                    Pressed => self.modifier.ctrl_pressed = true,
                    Released => self.modifier.ctrl_pressed = false,
                },
                LeftAlt | RightAlt => match key_event.event {
                    Pressed => self.modifier.alt_pressed = true,
                    Released => self.modifier.alt_pressed = false,
                },
                LeftGui | RightGui => match key_event.event {
                    Pressed => self.modifier.gui_pressed = true,
                    Released => self.modifier.gui_pressed = false,
                },
                _ => match key_event.event {
                    Pressed => {
                        return self.layout.map(key_event.key, self.modifier);
                    }
                    Released => {}
                },
            }
        }
        None
    }
}
