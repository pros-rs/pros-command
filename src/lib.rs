#![no_std]

extern crate alloc;

use alloc::{collections::BTreeSet, rc::Rc, vec, vec::Vec};
use core::{
    borrow::{Borrow, BorrowMut},
    cell::{Cell, RefCell},
    fmt::Formatter,
    hash::Hash,
    ops::Deref,
};

use command::{Command, InterruptionBehavior};
use event::EventLoop;
use hashbrown::{HashMap, HashSet};
use pros::prelude::*;
use snafu::Snafu;
use subsystem::Subsystem;

pub mod command;
pub mod event;
pub mod robot;
pub mod subsystem;

#[derive(Clone)]
pub struct SubsystemRef(Rc<RefCell<dyn Subsystem>>);

impl PartialEq for SubsystemRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for SubsystemRef {}

impl Hash for SubsystemRef {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.0).hash(state);
    }
}

impl From<Rc<RefCell<dyn Subsystem>>> for SubsystemRef {
    fn from(subsystem: Rc<RefCell<dyn Subsystem>>) -> Self {
        Self(subsystem)
    }
}

impl<T: Subsystem + 'static> From<T> for SubsystemRef {
    fn from(subsystem: T) -> Self {
        Self(Rc::new(RefCell::new(subsystem)))
    }
}

impl Deref for SubsystemRef {
    type Target = Rc<RefCell<dyn Subsystem>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
pub struct CommandRef(Rc<RefCell<dyn Command>>);

impl PartialEq for CommandRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for CommandRef {}

impl Hash for CommandRef {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.0).hash(state);
    }
}

impl From<Rc<RefCell<dyn Command>>> for CommandRef {
    fn from(command: Rc<RefCell<dyn Command>>) -> Self {
        Self(command)
    }
}

impl<T: Command + 'static> From<T> for CommandRef {
    fn from(subsystem: T) -> Self {
        Self(Rc::new(RefCell::new(subsystem)))
    }
}

impl Deref for CommandRef {
    type Target = Rc<RefCell<dyn Command>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Snafu)]
pub enum SetDefaultCommandError {
    #[snafu(display("Default commands must require their subsystem."))]
    MustRequireSubsystem,
    #[snafu(display("Cannot set the default command on a subsystem that is not registered."))]
    NotRegistered,
}

#[derive(Default)]
struct CommandSchedulerState {
    subsystems: RefCell<HashMap<SubsystemRef, Option<CommandRef>>>,
    in_run_loop: Cell<bool>,
    to_schedule: RefCell<Vec<CommandRef>>,
    to_cancel: RefCell<Vec<CommandRef>>,
    scheduled_commands: RefCell<HashSet<CommandRef>>,
    requirements: RefCell<HashMap<SubsystemRef, CommandRef>>,
    button_loop: Rc<RefCell<EventLoop>>,
    ending_commands: RefCell<HashSet<CommandRef>>,
}

impl CommandSchedulerState {
    #[inline]
    fn is_scheduled(&self, command: &CommandRef) -> bool {
        self.scheduled_commands.borrow().contains(command)
    }

    fn requiring(&self, subsystem: &SubsystemRef) -> Option<CommandRef> {
        self.requirements.borrow().get(subsystem).cloned()
    }

    fn init_command(
        &self,
        command: CommandRef,
        requirements: HashSet<SubsystemRef>,
    ) -> pros::Result {
        self.requirements
            .borrow_mut()
            .extend(requirements.into_iter().map(|r| (r, command.clone())));

        let mut scheduled_commands = self.scheduled_commands.borrow_mut();
        let command = scheduled_commands.entry(command).insert();
        (*command.get().0).borrow_mut().initialize()?;
        Ok(())
    }

    fn cancel(&self, command: &CommandRef) -> pros::Result {
        if self.ending_commands.borrow().contains(command) {
            return Ok(());
        }

        if self.in_run_loop.get() {
            self.to_cancel.borrow_mut().push(command.clone());
            return Ok(());
        }

        if !self.is_scheduled(command) {
            return Ok(());
        }

        self.ending_commands.borrow_mut().insert(command.clone());
        {
            let mut command = (*command.0).borrow_mut();
            command.end(true)?;
        }
        self.ending_commands.borrow_mut().remove(command);
        self.scheduled_commands.borrow_mut().remove(command);
        {
            let requirements = CommandScheduler::requirements_of(&*(*command.0).borrow());
            for requirement in requirements {
                self.requirements.borrow_mut().remove(&requirement);
            }
        }

        Ok(())
    }

    fn schedule_now(&self, command: CommandRef) -> pros::Result {
        if self.is_scheduled(&command) {
            return Ok(());
        }

        let requirements = CommandScheduler::requirements_of(&*(*command.0).borrow());

        if requirements.is_disjoint(&self.requirements.borrow().keys().cloned().collect()) {
            self.init_command(command, requirements)
        } else {
            let requiring_commands = requirements
                .iter()
                .filter_map(|r| self.requiring(r))
                .collect::<Vec<_>>();

            for requiring in &requiring_commands {
                if (*requiring.0).borrow().get_interruption_behavior()
                    == InterruptionBehavior::CancelIncoming
                {
                    return Ok(());
                }
            }

            for requiring in &requiring_commands {
                self.cancel(requiring)?;
            }

            self.init_command(command, requirements)
        }
    }
}

os_task_local! {
    static STATE: CommandSchedulerState = CommandSchedulerState::default();
}

pub struct CommandScheduler;

impl CommandScheduler {
    /// Register a subsystem with the scheduler.
    pub fn register<S: Subsystem + 'static>(subsystem: S) -> Rc<RefCell<S>> {
        let subsystem = Rc::new(RefCell::new(subsystem));
        STATE.with(|state| {
            state
                .subsystems
                .borrow_mut()
                .insert(SubsystemRef(subsystem.clone()), None);
        });
        subsystem
    }

    /// Schedule a command to run.
    pub fn schedule(command: Rc<RefCell<dyn Command>>) -> pros::Result {
        STATE.with(|state| {
            let command = CommandRef(command);
            if state.in_run_loop.get() {
                state.to_schedule.borrow_mut().push(command);
                return Ok(());
            }

            state.schedule_now(command)
        })
    }

    pub fn cancel(command: Rc<RefCell<dyn Command>>) -> pros::Result {
        STATE.with(|state| state.cancel(&CommandRef(command)))
    }

    pub fn set_default_command<S>(
        subsystem: &Rc<RefCell<S>>,
        command: impl Command + 'static,
    ) -> Result<(), SetDefaultCommandError>
    where
        S: Subsystem + 'static,
    {
        STATE.with(|state| {
            let requirements = CommandScheduler::requirements_of(&command);
            if !requirements.contains(&SubsystemRef(subsystem.clone())) {
                return Err(SetDefaultCommandError::MustRequireSubsystem);
            }

            // if command.get_interruption_behavior() == InterruptionBehavior::CancelIncoming {
            //     weird but ok i guess
            // }

            let command = CommandRef(Rc::new(RefCell::new(command)));
            state
                .subsystems
                .borrow_mut()
                .get_mut(&SubsystemRef(subsystem.clone()))
                .unwrap()
                .replace(command);

            Ok(())
        })
    }

    pub fn remove_default_command<S>(subsystem: &Rc<RefCell<S>>) -> Option<Rc<RefCell<dyn Command>>>
    where
        S: Subsystem + 'static,
    {
        STATE.with(|state| {
            let command = state
                .subsystems
                .borrow_mut()
                .get_mut(&SubsystemRef(subsystem.clone()))?
                .take();
            command.map(|c| c.0)
        })
    }

    pub fn run() -> pros::Result {
        STATE.with(|state| {
            for subsystem in state.subsystems.borrow().keys() {
                let mut subsystem = (*subsystem.0).borrow_mut();
                subsystem.periodic();
                if robot::is_sim() {
                    subsystem.sim_periodic();
                }
            }

            let button_loop = state.button_loop.clone();
            (*button_loop).borrow_mut().poll();

            state.in_run_loop.set(true);
            let disabled = pros::competition::is_disabled();

            let scheduled_commands = state
                .scheduled_commands
                .borrow()
                .iter()
                .cloned()
                .collect::<Vec<_>>();

            for command in scheduled_commands {
                let mut command_ref = (*command.0).borrow_mut();
                if disabled && !command_ref.runs_when_disabled() {
                    state.cancel(&command)?;
                }

                command_ref.execute()?;
                if command_ref.is_finished()? {
                    state.ending_commands.borrow_mut().insert(command.clone());
                    let res = command_ref.end(false);
                    state.ending_commands.borrow_mut().remove(&command);
                    res?;
                    state.scheduled_commands.borrow_mut().remove(&command);
                    let requirements = command_ref.get_requirements();
                    for requirement in requirements {
                        state.requirements.borrow_mut().remove(requirement);
                    }
                }
            }

            state.in_run_loop.set(false);

            let to_schedule = state.to_schedule.take();
            for command in to_schedule {
                state.schedule_now(command)?;
            }

            let to_cancel = state.to_cancel.take();
            for command in to_cancel {
                state.cancel(&command)?;
            }

            // Add default commands for un-required registered subsystems.
            for (subsystem, command) in state.subsystems.borrow().iter() {
                if let Some(default_command) = command {
                    if !state.requirements.borrow().contains_key(subsystem) {
                        state.schedule_now(default_command.clone())?;
                    }
                }
            }

            Ok(())
        })
    }

    fn requirements_of(command: &dyn Command) -> HashSet<SubsystemRef> {
        command.get_requirements().iter().cloned().collect()
    }

    pub fn cancel_all() -> pros::Result {
        STATE.with(|state| {
            let scheduled_commands = state
                .scheduled_commands
                .borrow()
                .iter()
                .cloned()
                .collect::<Vec<_>>();

            for command in scheduled_commands {
                state.cancel(&command)?;
            }

            Ok(())
        })
    }

    pub fn button_event_loop() -> Rc<RefCell<EventLoop>> {
        STATE.with(|state| state.button_loop.clone())
    }

    pub fn is_scheduled(command: &Rc<RefCell<dyn Command>>) -> bool {
        STATE.with(|state| state.is_scheduled(&CommandRef(command.clone())))
    }
}
