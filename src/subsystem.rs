use alloc::{boxed::Box, rc::Rc, vec};
use core::{any::Any, cell::RefCell, fmt::Debug};

use crate::{
    command::{Command, FunctionalCommand},
    CommandScheduler, SubsystemRef,
};

/// A collection of robot parts and other hardware that act together as a whole.
pub trait Subsystem: Debug {
    /// This method will be called once per scheduler run
    fn periodic(&mut self) {}
    /// This method will be called once per scheduler run, but only during simulation
    fn sim_periodic(&mut self) {}
    fn default_command(&self) -> Option<Box<dyn Command>> {
        None
    }

    fn register(self) -> Rc<RefCell<Self>>
    where
        Self: Sized + 'static,
    {
        CommandScheduler::register(self)
    }
}

pub trait SubsystemRefExt<T>
where
    T: Subsystem,
{
    fn run_once(&self, action: impl FnMut() -> pros::Result + 'static) -> FunctionalCommand;
    fn run(&self, action: impl FnMut() -> pros::Result + 'static) -> FunctionalCommand;
    fn start_end(
        &self,
        start: impl FnMut() -> pros::Result + 'static,
        end: impl FnMut(bool) -> pros::Result + 'static,
    ) -> FunctionalCommand;
    fn run_end(
        &self,
        run: impl FnMut() -> pros::Result + 'static,
        end: impl FnMut(bool) -> pros::Result + 'static,
    ) -> FunctionalCommand;
}

impl<T> SubsystemRefExt<T> for Rc<RefCell<T>>
where
    T: Subsystem + 'static,
{
    fn run_once(&self, action: impl FnMut() -> pros::Result + 'static) -> FunctionalCommand {
        FunctionalCommand::instant(action, vec![SubsystemRef(self.clone())])
    }
    fn run(&self, action: impl FnMut() -> pros::Result + 'static) -> FunctionalCommand {
        FunctionalCommand::run(action, vec![SubsystemRef(self.clone())])
    }
    fn start_end(
        &self,
        start: impl FnMut() -> pros::Result + 'static,
        end: impl FnMut(bool) -> pros::Result + 'static,
    ) -> FunctionalCommand {
        FunctionalCommand::start_end(start, end, vec![SubsystemRef(self.clone())])
    }
    fn run_end(
        &self,
        run: impl FnMut() -> pros::Result + 'static,
        end: impl FnMut(bool) -> pros::Result + 'static,
    ) -> FunctionalCommand {
        FunctionalCommand::run_end(run, end, vec![SubsystemRef(self.clone())])
    }
}
