use alloc::{boxed::Box, rc::Rc, vec::Vec};
use core::cell::RefCell;

use crate::{CommandScheduler, SubsystemRef};

pub mod button;

/// An action the robot can perform. Runs when scheduled, until it is interrupted or it finishes.
pub trait Command {
    fn get_requirements(&self) -> &[SubsystemRef];

    /// The initial subroutine of a command. Called once when the command is initially scheduled.
    fn initialize(&mut self) -> pros::Result {
        Ok(())
    }
    fn execute(&mut self) -> pros::Result {
        Ok(())
    }
    #[allow(unused_variables)]
    fn end(&mut self, interrupted: bool) -> pros::Result {
        Ok(())
    }

    fn is_finished(&mut self) -> pros::Result<bool> {
        Ok(false)
    }

    fn runs_when_disabled(&self) -> bool {
        false
    }

    fn get_interruption_behavior(&self) -> InterruptionBehavior {
        InterruptionBehavior::default()
    }
}

pub trait CommandRefExt {
    fn schedule(&self) -> pros::Result;
    fn cancel(&self) -> pros::Result;
    fn is_scheduled(&self) -> bool;
}

impl CommandRefExt for Rc<RefCell<dyn Command>> {
    fn schedule(&self) -> pros::Result {
        CommandScheduler::schedule(self.clone())
    }

    fn cancel(&self) -> pros::Result {
        CommandScheduler::cancel(self.clone())
    }

    fn is_scheduled(&self) -> bool {
        CommandScheduler::is_scheduled(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InterruptionBehavior {
    #[default]
    CancelSelf,
    CancelIncoming,
}

pub struct FunctionalCommand {
    on_init: Box<dyn FnMut() -> pros::Result>,
    on_execute: Box<dyn FnMut() -> pros::Result>,
    on_end: Box<dyn FnMut(bool) -> pros::Result>,
    is_finished: Box<dyn FnMut() -> pros::Result<bool>>,
    requirements: Vec<SubsystemRef>,
}

impl FunctionalCommand {
    pub fn new(
        on_init: impl FnMut() -> pros::Result + 'static,
        on_execute: impl FnMut() -> pros::Result + 'static,
        on_end: impl FnMut(bool) -> pros::Result + 'static,
        is_finished: impl FnMut() -> pros::Result<bool> + 'static,
        requirements: Vec<SubsystemRef>,
    ) -> Self {
        Self {
            on_init: Box::new(on_init),
            on_execute: Box::new(on_execute),
            on_end: Box::new(on_end),
            is_finished: Box::new(is_finished),
            requirements,
        }
    }

    pub fn instant(
        on_init: impl FnMut() -> pros::Result + 'static,
        requirements: Vec<SubsystemRef>,
    ) -> Self {
        Self::new(on_init, || Ok(()), |_| Ok(()), || Ok(true), requirements)
    }

    pub fn run(
        on_execute: impl FnMut() -> pros::Result + 'static,
        requirements: Vec<SubsystemRef>,
    ) -> Self {
        Self::new(
            || Ok(()),
            on_execute,
            |_| Ok(()),
            || Ok(false),
            requirements,
        )
    }

    pub fn start_end(
        on_init: impl FnMut() -> pros::Result + 'static,
        on_end: impl FnMut(bool) -> pros::Result + 'static,
        requirements: Vec<SubsystemRef>,
    ) -> Self {
        Self::new(on_init, || Ok(()), on_end, || Ok(false), requirements)
    }

    pub fn run_end(
        on_execute: impl FnMut() -> pros::Result + 'static,
        on_end: impl FnMut(bool) -> pros::Result + 'static,
        requirements: Vec<SubsystemRef>,
    ) -> Self {
        Self::new(|| Ok(()), on_execute, on_end, || Ok(false), requirements)
    }
}

impl Command for FunctionalCommand {
    fn get_requirements(&self) -> &[SubsystemRef] {
        &self.requirements
    }

    fn initialize(&mut self) -> pros::Result {
        (self.on_init)()
    }

    fn execute(&mut self) -> pros::Result {
        (self.on_execute)()
    }

    fn end(&mut self, interrupted: bool) -> pros::Result {
        (self.on_end)(interrupted)
    }

    fn is_finished(&mut self) -> pros::Result<bool> {
        (self.is_finished)()
    }
}
