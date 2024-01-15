use alloc::{rc::Rc, vec};
use core::cell::RefCell;

use pros::{
    controller::{Controller, ControllerButton},
    println,
};
use pros_command::{
    command::{button::Trigger, FunctionalCommand},
    robot::ScheduledRobot,
    subsystem::Subsystem,
    CommandScheduler,
};

use crate::{commands::DriveWithJoystickCommand, subsystems::drivetrain::Drivetrain};

pub struct Robot {
    drivetrain: Rc<RefCell<Drivetrain>>,
}

impl Robot {
    pub fn new() -> pros::Result<Self> {
        Ok(Self {
            drivetrain: Drivetrain::new()?.register(),
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
    fn periodic(&mut self) -> pros::Result {
        CommandScheduler::run()?;
        Ok(())
    }
}
