use alloc::vec;

use pros::{
    controller::{Controller, ControllerButton},
    println,
};
use pros_command::{
    command::{button::Trigger, FunctionalCommand},
    robot::ScheduledRobot,
    CommandScheduler,
};

pub struct Robot {
    pub controller: Controller,
}

impl Robot {
    pub fn new() -> Self {
        Self {
            controller: Controller::Master,
        }
    }

    pub fn configure_button_bindings(&mut self) {
        Trigger::button(self.controller, ControllerButton::A)
            .on_true(FunctionalCommand::instant(
                || {
                    println!("Button A pressed");
                    Ok(())
                },
                vec![],
            ))
            .on_true(FunctionalCommand::instant(
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
