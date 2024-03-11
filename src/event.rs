use alloc::{boxed::Box, rc::Rc, vec::Vec};
use core::cell::{Cell, RefCell};
use crate::controller::Trigger;
use pros::prelude::*;

#[derive(Default)]
pub struct EventLoop {
    events: Vec<Box<dyn FnMut() -> Result>>,
}

impl EventLoop {
    /// Add an event to run when the loop is polled.
    pub fn bind(&mut self, action: impl FnMut() -> Result + 'static) {
        self.events.push(Box::new(action));
    }

    pub fn poll(&mut self) -> Result {
        for event in self.events.iter_mut() {
            event()?;
        }
        Ok(())
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }
}

pub struct BooleanEvent {
    event_loop: Rc<RefCell<EventLoop>>,
    state: Rc<Cell<bool>>,
}

impl BooleanEvent {
    pub fn new(
        event_loop: Rc<RefCell<EventLoop>>,
        mut signal: impl FnMut() -> Result<bool> + 'static,
    ) -> Result<Self> {
        let state = Rc::new(Cell::new(signal()?));
        event_loop.borrow_mut().bind({
            let state = state.clone();
            move || {
                state.set(signal()?);
                Ok(())
            }
        });
        Ok(Self { event_loop, state })
    }

    pub fn current_state(&self) -> bool {
        self.state.get()
    }

    pub fn if_high(&self, mut action: impl FnMut() -> Result + 'static) {
        let state = self.state.clone();
        self.event_loop.borrow_mut().bind(move || {
            if state.get() {
                action()
            } else {
                Ok(())
            }
        });
    }

    pub fn rising(&self) -> Self {
        let mut previous = self.state.get();
        let state = self.state.clone();

        Self::new(self.event_loop.clone(), move || {
            let present = state.get();
            let is_rising = !previous && present;
            previous = present;
            Ok(is_rising)
        }).unwrap()
    }

    pub fn falling(&self) -> Self {
        let mut previous = self.state.get();
        let state = self.state.clone();

        Self::new(self.event_loop.clone(), move || {
            let present = state.get();
            let is_falling = previous && !present;
            previous = present;
            Ok(is_falling)
        }).unwrap()
    }

    pub fn negate(&self) -> Self {
        let state = self.state.clone();
        Self::new(self.event_loop.clone(), move || Ok(!state.get())).unwrap()
    }

    pub fn and(&self, other: &Self) -> Self {
        let state = self.state.clone();
        let other_state = other.state.clone();
        Self::new(self.event_loop.clone(), move || {
            Ok(state.get() && other_state.get())
        }).unwrap()
    }

    pub fn or(&self, other: &Self) -> Self {
        let state = self.state.clone();
        let other_state = other.state.clone();
        Self::new(self.event_loop.clone(), move || {
            Ok(state.get() || other_state.get())
        }).unwrap()
    }

    pub fn as_trigger(&self) -> Trigger {
        let state = self.state.clone();
        Trigger::new_with_loop(self.event_loop.clone(), move || state.get())
    }
}

impl From<BooleanEvent> for Trigger {
    fn from(event: BooleanEvent) -> Self {
        Self::new_with_loop(event.event_loop, move || event.state.get())
    }
}

// pub trait CommandBasedController {
//     
// }