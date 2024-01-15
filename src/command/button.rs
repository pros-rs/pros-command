use alloc::rc::Rc;
use core::cell::RefCell;

use pros::controller::{self, Controller, ControllerButton};

use super::{Command, CommandRefExt};
use crate::{event::EventLoop, CommandRef, CommandScheduler};

pub struct Trigger {
    event_loop: Rc<RefCell<EventLoop>>,
    condition: Rc<dyn Fn() -> bool>,
}

impl Trigger {
    pub fn new_with_loop(
        event_loop: Rc<RefCell<EventLoop>>,
        condition: impl Fn() -> bool + 'static,
    ) -> Self {
        Self {
            event_loop,
            condition: Rc::new(condition),
        }
    }

    pub fn new(condition: impl Fn() -> bool + 'static) -> Self {
        Self {
            event_loop: CommandScheduler::button_event_loop(),
            condition: Rc::new(condition),
        }
    }

    pub fn on_true(self, command: impl Into<CommandRef>) -> Self {
        let command = command.into();
        let condition = self.condition.clone();
        let mut pressed_last = condition();
        self.event_loop.borrow_mut().bind(move || {
            let pressed = condition();
            if !pressed_last && pressed {
                command.schedule().unwrap();
            }
            pressed_last = pressed;
        });
        self
    }

    pub fn on_false(self, command: impl Into<CommandRef>) -> Self {
        let command = command.into();
        let condition = self.condition.clone();
        let mut pressed_last = condition();
        self.event_loop.borrow_mut().bind(move || {
            let pressed = condition();
            if pressed_last && !pressed {
                command.schedule().unwrap();
            }
            pressed_last = pressed;
        });
        self
    }

    pub fn while_true(self, command: impl Into<CommandRef>) -> Self {
        let command = command.into();
        let condition = self.condition.clone();
        let mut pressed_last = condition();

        self.event_loop.borrow_mut().bind(move || {
            let pressed = condition();
            if !pressed_last && pressed {
                command.schedule().unwrap();
            } else if pressed_last && !pressed {
                command.cancel().unwrap();
            }
            pressed_last = pressed;
        });
        self
    }

    pub fn while_false(self, command: impl Into<CommandRef>) -> Self {
        let command = command.into();
        let condition = self.condition.clone();
        let mut pressed_last = condition();

        self.event_loop.borrow_mut().bind(move || {
            let pressed = condition();
            if pressed_last && !pressed {
                command.schedule().unwrap();
            } else if !pressed_last && pressed {
                command.cancel().unwrap();
            }
            pressed_last = pressed;
        });
        self
    }

    pub fn toggle_on_true(self, command: impl Into<CommandRef>) -> Self {
        let command = command.into();
        let condition = self.condition.clone();
        let mut pressed_last = condition();

        self.event_loop.borrow_mut().bind(move || {
            let pressed = condition();
            if !pressed_last && pressed {
                if command.is_scheduled() {
                    command.cancel().unwrap();
                } else {
                    command.schedule().unwrap();
                }
            }
            pressed_last = pressed;
        });
        self
    }

    pub fn toggle_on_false(self, command: impl Into<CommandRef>) -> Self {
        let command = command.into();
        let condition = self.condition.clone();
        let mut pressed_last = condition();

        self.event_loop.borrow_mut().bind(move || {
            let pressed = condition();
            if pressed_last && !pressed {
                if command.is_scheduled() {
                    command.cancel().unwrap();
                } else {
                    command.schedule().unwrap();
                }
            }
            pressed_last = pressed;
        });
        self
    }

    pub fn is_active(&self) -> bool {
        (self.condition)()
    }

    pub fn and(&self, other: &Self) -> Self {
        let condition = self.condition.clone();
        let other_condition = other.condition.clone();
        Self::new(move || condition() && other_condition())
    }

    pub fn or(&self, other: &Self) -> Self {
        let condition = self.condition.clone();
        let other_condition = other.condition.clone();
        Self::new(move || condition() || other_condition())
    }

    pub fn negate(&self) -> Self {
        let condition = self.condition.clone();
        Self::new(move || !condition())
    }

    pub fn button(controller: Controller, button: ControllerButton) -> Self {
        Self::new(move || controller.button(button))
    }
}
