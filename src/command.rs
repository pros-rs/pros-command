use alloc::{boxed::Box, rc::Rc, vec::Vec};
use core::cell::RefCell;
use pros::prelude::*;

use crate::{CommandScheduler, AnySubsystem};

/// An action the robot can perform. Runs when scheduled, until it is interrupted or it finishes.
pub trait Command {
    fn get_requirements(&self) -> &[AnySubsystem];

    /// The initial subroutine of a command. Called once when the command is initially scheduled.
    fn initialize(&mut self) -> Result {
        Ok(())
    }
    fn execute(&mut self) -> Result {
        Ok(())
    }
    #[allow(unused_variables)]
    fn end(&mut self, interrupted: bool) -> Result {
        Ok(())
    }

    fn is_finished(&self) -> Result<bool> {
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
    fn schedule(&self) -> Result;
    fn cancel(&self) -> Result;
    fn is_scheduled(&self) -> bool;
}

impl CommandRefExt for Rc<RefCell<dyn Command>> {
    fn schedule(&self) -> Result {
        CommandScheduler::schedule(self.clone())
    }

    fn cancel(&self) -> Result {
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
    on_init: Box<dyn FnMut() -> Result>,
    on_execute: Box<dyn FnMut() -> Result>,
    on_end: Box<dyn FnMut(bool) -> Result>,
    is_finished: Box<dyn Fn() -> Result<bool>>,
    requirements: Vec<AnySubsystem>,
}

impl FunctionalCommand {
    pub fn new(
        on_init: impl FnMut() -> Result + 'static,
        on_execute: impl FnMut() -> Result + 'static,
        on_end: impl FnMut(bool) -> Result + 'static,
        is_finished: impl Fn() -> Result<bool> + 'static,
        requirements: Vec<AnySubsystem>,
    ) -> Self {
        Self {
            on_init: Box::new(on_init),
            on_execute: Box::new(on_execute),
            on_end: Box::new(on_end),
            is_finished: Box::new(is_finished),
            requirements,
        }
    }
}

impl Command for FunctionalCommand {
    fn get_requirements(&self) -> &[AnySubsystem] {
        &self.requirements
    }

    fn initialize(&mut self) -> Result {
        (self.on_init)()
    }

    fn execute(&mut self) -> Result {
        (self.on_execute)()
    }

    fn end(&mut self, interrupted: bool) -> Result {
        (self.on_end)(interrupted)
    }

    fn is_finished(&self) -> Result<bool> {
        (self.is_finished)()
    }
}

#[macro_export]
macro_rules! run_once {
    ($on_init:block) => {
        FunctionalCommand::new(move || $on_init, || Ok(()), |_| Ok(()), || Ok(true), vec![])
    };
    ($on_init:block, $($requirement:expr),+ $(,)?) => {
        FunctionalCommand::new(move || $on_init, || Ok(()), |_| Ok(()), || Ok(true), vec![$($requirement),+])
    };
}

#[macro_export]
macro_rules! run {
    ($on_execute:block) => {
        FunctionalCommand::new(
            || Ok(()),
            move || $on_execute,
            |_| Ok(()),
            || Ok(false),
            vec![],
        )
    };
    ($on_execute:block, $($requirement:expr),+ $(,)?) => {
        FunctionalCommand::new(
            || Ok(()),
            move || $on_execute,
            |_| Ok(()),
            || Ok(false),
            vec![$($requirement),+],
        )
    };
}

#[macro_export]
macro_rules! start_end {
    ($start:block, $end:block) => {
        FunctionalCommand::new(move || $start, || Ok(()), move || $end, || Ok(false), vec![])
    };
    ($start:block, $end:block, $($requirement:expr),+ $(,)?) => {
        FunctionalCommand::new(
            move || $start,
            || Ok(()),
            move |_| $end,
            || Ok(false), 
            vec![$($requirement),+],
        )
    };
}

#[macro_export]
macro_rules! run_end {
    ($execute:block, $end:block) => {
        FunctionalCommand::new(|| Ok(()), move || $execute, move |_| $end, || Ok(false), vec![])
    };
    ($execute:block, $end:block, $($requirement:expr),+ $(,)?) => {
        FunctionalCommand::new(|| Ok(()), move || $execute, move |_| $end, || Ok(false), vec![$($requirement),+],)
    };
}