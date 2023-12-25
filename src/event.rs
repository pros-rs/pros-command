use alloc::{boxed::Box, rc::Rc, vec::Vec};
use core::cell::{Cell, RefCell};

use crate::command::button::Trigger;

#[derive(Default)]
pub struct EventLoop {
    events: Vec<Box<dyn FnMut()>>,
}

impl EventLoop {
    /// Add an event to run when the loop is polled.
    pub fn bind(&mut self, action: impl FnMut() + 'static) {
        self.events.push(Box::new(action));
    }

    pub fn poll(&mut self) {
        for event in self.events.iter_mut() {
            event();
        }
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
        mut signal: impl FnMut() -> bool + 'static,
    ) -> Self {
        let state = Rc::new(Cell::new(signal()));
        event_loop.borrow_mut().bind({
            let state = state.clone();
            move || {
                state.set(signal());
            }
        });
        Self { event_loop, state }
    }

    pub fn current_state(&self) -> bool {
        self.state.get()
    }

    pub fn if_high(&self, mut action: impl FnMut() + 'static) {
        let state = self.state.clone();
        self.event_loop.borrow_mut().bind(move || {
            if state.get() {
                action();
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
            is_rising
        })
    }

    pub fn falling(&self) -> Self {
        let mut previous = self.state.get();
        let state = self.state.clone();

        Self::new(self.event_loop.clone(), move || {
            let present = state.get();
            let is_rising = previous && !present;
            previous = present;
            is_rising
        })
    }

    pub fn negate(&self) -> Self {
        let state = self.state.clone();
        Self::new(self.event_loop.clone(), move || !state.get())
    }

    pub fn and(&self, other: &Self) -> Self {
        let state = self.state.clone();
        let other_state = other.state.clone();
        Self::new(self.event_loop.clone(), move || {
            state.get() && other_state.get()
        })
    }

    pub fn or(&self, other: &Self) -> Self {
        let state = self.state.clone();
        let other_state = other.state.clone();
        Self::new(self.event_loop.clone(), move || {
            state.get() || other_state.get()
        })
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
