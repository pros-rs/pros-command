#![no_std]

extern crate alloc;

use alloc::{rc::Rc, vec::Vec};
use core::{
    cell::{Cell, RefCell},
    hash::Hash,
    ops::Deref,
};

use command::{Command, InterruptionBehavior};
use event::EventLoop;
use hashbrown::{HashMap, HashSet};
use pros::core::os_task_local;
use pros::devices::competition;
use pros::devices::competition::CompetitionMode;
use pros::prelude::*;
use snafu::{OptionExt, Snafu};
use subsystem::Subsystem;
use crate::SetDefaultCommandError::NotRegistered;

pub mod command;
pub mod event;
pub mod robot;
pub mod subsystem;
pub mod controller;

#[derive(Clone, Debug)]
pub struct AnySubsystem(pub Rc<RefCell<dyn Subsystem>>);

impl PartialEq for AnySubsystem {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for AnySubsystem {}

impl Hash for AnySubsystem {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        (Rc::as_ptr(&self.0) as *const ()).hash(state);
    }
}

impl From<Rc<RefCell<dyn Subsystem>>> for AnySubsystem {
    fn from(subsystem: Rc<RefCell<dyn Subsystem>>) -> Self {
        Self(subsystem)
    }
}

impl<T: Subsystem + 'static> From<T> for AnySubsystem {
    fn from(subsystem: T) -> Self {
        Self(Rc::new(RefCell::new(subsystem)))
    }
}

impl Deref for AnySubsystem {
    type Target = Rc<RefCell<dyn Subsystem>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
pub struct AnyCommand(pub Rc<RefCell<dyn Command>>);

impl PartialEq for AnyCommand {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for AnyCommand {}

impl Hash for AnyCommand {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        (Rc::as_ptr(&self.0) as *const ()).hash(state);
    }
}

impl From<Rc<RefCell<dyn Command>>> for AnyCommand {
    fn from(command: Rc<RefCell<dyn Command>>) -> Self {
        Self(command)
    }
}

impl<T: Command + 'static> From<T> for AnyCommand {
    fn from(subsystem: T) -> Self {
        Self(Rc::new(RefCell::new(subsystem)))
    }
}

impl Deref for AnyCommand {
    type Target = Rc<RefCell<dyn Command>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Snafu)]
pub enum SetDefaultCommandError {
    /// Default commands must require their subsystem
    MustRequireSubsystem,
    /// Cannot set the default command on a subsystem that is not registered.
    NotRegistered,
}

#[derive(Default)]
struct CommandSchedulerState {
    subsystems: RefCell<HashMap<AnySubsystem, Option<AnyCommand>>>,
    in_run_loop: Cell<bool>,
    to_schedule: RefCell<Vec<AnyCommand>>,
    to_cancel: RefCell<Vec<AnyCommand>>,
    scheduled_commands: RefCell<HashSet<AnyCommand>>,
    requirements: RefCell<HashMap<AnySubsystem, AnyCommand>>,
    button_loop: Rc<RefCell<EventLoop>>,
    ending_commands: RefCell<HashSet<AnyCommand>>,
}

impl CommandSchedulerState {
    #[inline]
    fn is_scheduled(&self, command: &AnyCommand) -> bool {
        self.scheduled_commands.borrow().contains(command)
    }

    fn requiring(&self, subsystem: &AnySubsystem) -> Option<AnyCommand> {
        self.requirements.borrow().get(subsystem).cloned()
    }

    fn init_command(
        &self,
        command: AnyCommand,
        requirements: HashSet<AnySubsystem>,
    ) -> Result {
        self.requirements
            .borrow_mut()
            .extend(requirements.into_iter().map(|r| (r, command.clone())));

        let mut scheduled_commands = self.scheduled_commands.borrow_mut();
        let command = scheduled_commands.entry(command).insert();
        (*command.get().0).borrow_mut().initialize()?;
        Ok(())
    }

    fn cancel(&self, command: &AnyCommand) -> Result {
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

    fn schedule_now(&self, command: AnyCommand) -> Result {
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
            let subsystem_ref = AnySubsystem(subsystem.clone());
            state
                .subsystems
                .borrow_mut()
                .insert(subsystem_ref.clone(), None);
            if let Some(default_command) = subsystem.borrow().default_command(subsystem_ref.clone()) {
                CommandScheduler::set_default_command(&subsystem_ref, default_command);
            }
        });
        subsystem
    }

    /// Schedule a command to run.
    pub fn schedule(command: Rc<RefCell<dyn Command>>) -> Result {
        STATE.with(|state| {
            let command = AnyCommand(command);
            if state.in_run_loop.get() {
                state.to_schedule.borrow_mut().push(command);
                return Ok(());
            }

            state.schedule_now(command)
        })
    }

    pub fn cancel(command: Rc<RefCell<dyn Command>>) -> Result {
        STATE.with(|state| state.cancel(&AnyCommand(command)))
    }

    pub fn set_default_command(
        subsystem: &AnySubsystem,
        command: AnyCommand,
    ) -> core::result::Result<(), SetDefaultCommandError> {
        STATE.with(|state| {
            let requirements = CommandScheduler::requirements_of(&*command.borrow());
            if !requirements.contains(subsystem) {
                return MustRequireSubsystemSnafu.fail();
            }

            // if command.get_interruption_behavior() == InterruptionBehavior::CancelIncoming {
            //     weird but ok i guess
            // }

            state
                .subsystems
                .borrow_mut()
                .get_mut(subsystem)
                .context(NotRegisteredSnafu)?
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
                .get_mut(&AnySubsystem(subsystem.clone()))?
                .take();
            command.map(|c| c.0)
        })
    }

    pub fn run() -> Result {
        STATE.with(|state| {
            for subsystem_ctx in state.subsystems.borrow().keys() {
                let mut subsystem = (*subsystem_ctx.0).borrow_mut();
                subsystem.periodic(subsystem_ctx.clone());
                if robot::is_sim() {
                    subsystem.sim_periodic(subsystem_ctx.clone());
                }
            }

            let button_loop = state.button_loop.clone();
            (*button_loop).borrow_mut().poll()?;

            state.in_run_loop.set(true);
            let comp_mode = competition::mode();

            let scheduled_commands = state
                .scheduled_commands
                .borrow()
                .iter()
                .cloned()
                .collect::<Vec<_>>();

            for command in scheduled_commands {
                let mut command_ref = (*command.0).borrow_mut();
                if comp_mode == CompetitionMode::Disabled && !command_ref.runs_when_disabled() {
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

    fn requirements_of(command: &dyn Command) -> HashSet<AnySubsystem> {
        command.get_requirements().iter().cloned().collect()
    }

    pub fn cancel_all() -> Result {
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
        STATE.with(|state| state.is_scheduled(&AnyCommand(command.clone())))
    }
}
