use alloc::{rc::Rc, vec};
use core::cell::RefCell;
use pros::devices::Controller;
use pros::devices::controller::ControllerButton;

use pros_command::{command::{FunctionalCommand}, robot::ScheduledRobot, subsystem::Subsystem, CommandScheduler, run_once};
use pros::prelude::*;
use pros_command::controller::Trigger;

use crate::subsystems::drivetrain::{DriveWithJoystickCommand, Drivetrain};

pub struct Robot {
    drivetrain: Rc<RefCell<Drivetrain>>,
}

impl Robot {
    pub fn new(peripherals: Peripherals) -> Result<Self> {
        Ok(Self {
            drivetrain: Drivetrain::new(peripherals.port_1, peripherals.port_2)?.register(),
        })
    }

    pub fn configure_button_bindings(&mut self) {
        CommandScheduler::set_default_command(
            &self.drivetrain.clone().into(),
            DriveWithJoystickCommand::new(self.drivetrain.clone(), Controller::Master).into()
        )
        .unwrap();

        Trigger::button(Controller::Master, ControllerButton::A)
            .on_true(run_once!({
                    println!("Button A pressed");
                }))
            .on_false(run_once!({
                    println!("Button A released");
                }));
    }
}

impl ScheduledRobot for Robot {
    fn periodic(&mut self) -> Result {
        CommandScheduler::run()?;
        Ok(())
    }
}
