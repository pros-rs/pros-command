use alloc::{rc::Rc, vec};
use core::cell::RefCell;
use pros::devices::Controller;
use pros::devices::controller::ControllerButton;

use pros_command::{
    command::{FunctionalCommand},
    robot::ScheduledRobot,
    subsystem::Subsystem,
    CommandScheduler,
};
use pros::prelude::*;
use pros_command::controller::Trigger;

use crate::{commands::DriveWithJoystickCommand, subsystems::drivetrain::Drivetrain};

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
            &self.drivetrain,
            DriveWithJoystickCommand::new(self.drivetrain.clone(), Controller::Master),
        )
        .unwrap();

        Trigger::button(Controller::Master, ControllerButton::A)
            .on_true(FunctionalCommand::instant(
                || {
                    println!("Button A pressed");
                    Ok(())
                },
                vec![],
            ))
            .on_false(FunctionalCommand::instant(
                || {
                    println!("Button A released");
                    Ok(())
                },
                vec![],
            ));
    }
}

impl ScheduledRobot for Robot {
    fn periodic(&mut self) -> Result {
        CommandScheduler::run()?;
        Ok(())
    }
}
