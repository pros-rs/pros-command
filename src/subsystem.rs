use alloc::{boxed::Box, rc::Rc, vec};
use core::{cell::RefCell, fmt::Debug};
use pros::prelude::*;

use crate::{command::{Command, FunctionalCommand}, AnyCommand, CommandScheduler, AnySubsystem, run_once, run, start_end, run_end};

/// A collection of robot parts and other hardware that act together as a whole.
pub trait Subsystem: Debug {
    /// This method will be called once per scheduler run
    fn periodic(&mut self, ctx: AnySubsystem) {}
    /// This method will be called once per scheduler run, but only during simulation
    fn sim_periodic(&mut self, ctx: AnySubsystem) {}
    fn default_command(&self, ctx: AnySubsystem) -> Option<AnyCommand> {
        None
    }

    fn register(self) -> Rc<RefCell<Self>>
    where
        Self: Sized + 'static,
    {
        CommandScheduler::register(self)
    }
}

pub trait SubsystemRefExt {
    fn run_once(&self, action: impl FnMut() -> Result + 'static) -> FunctionalCommand;
    fn run(&self, action: impl FnMut() -> Result + 'static) -> FunctionalCommand;
    fn start_end(
        &self,
        start: impl FnMut() -> Result + 'static,
        end: impl FnMut() -> Result + 'static,
    ) -> FunctionalCommand;
    fn run_end(
        &self,
        run: impl FnMut() -> Result + 'static,
        end: impl FnMut() -> Result + 'static,
    ) -> FunctionalCommand;
}

impl<T> SubsystemRefExt for Rc<RefCell<T>>
    where
        T: Subsystem + 'static,
{
    fn run_once(&self, mut action: impl FnMut() -> Result + 'static) -> FunctionalCommand {
        run_once!({ action() }, AnySubsystem(self.clone()))
    }
    fn run(&self, mut action: impl FnMut() -> Result + 'static) -> FunctionalCommand {
        run!({ action() }, AnySubsystem(self.clone()))
    }
    fn start_end(
        &self,
        mut start: impl FnMut() -> Result + 'static,
        mut end: impl FnMut() -> Result + 'static,
    ) -> FunctionalCommand {
        start_end!({ start() }, { end() }, AnySubsystem(self.clone()))
    }
    fn run_end(
        &self,
        mut run: impl FnMut() -> Result + 'static,
        mut end: impl FnMut() -> Result + 'static,
    ) -> FunctionalCommand {
        run_end!({ run() }, { end() }, AnySubsystem(self.clone()))
    }
}

impl SubsystemRefExt for AnySubsystem {
    fn run_once(&self, mut action: impl FnMut() -> Result + 'static) -> FunctionalCommand {
        run_once!({ action() }, self.clone())
    }
    fn run(&self, mut action: impl FnMut() -> Result + 'static) -> FunctionalCommand {
        run!({ action() }, self.clone())
    }
    fn start_end(
        &self,
        mut start: impl FnMut() -> Result + 'static,
        mut end: impl FnMut() -> Result + 'static,
    ) -> FunctionalCommand {
        start_end!({ start() }, { end() }, self.clone())
    }
    fn run_end(
        &self,
        mut run: impl FnMut() -> Result + 'static,
        mut end: impl FnMut() -> Result + 'static,
    ) -> FunctionalCommand {
        run_end!({ run() }, { end() }, self.clone())
    }
}
