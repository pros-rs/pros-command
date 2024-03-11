use alloc::{rc::Rc, vec, vec::Vec};
use core::cell::RefCell;
use pros::devices::Controller;
use pros::devices::controller::JoystickAxis;

use pros::prelude::*;
use pros_command::{command::Command, SubsystemRef};

use crate::subsystems::drivetrain::Drivetrain;

pub struct DriveWithJoystickCommand {
    drivetrain: Rc<RefCell<Drivetrain>>,
    controller: Controller,
    requirements: Vec<SubsystemRef>,
}

impl DriveWithJoystickCommand {
    pub fn new(drivetrain: Rc<RefCell<Drivetrain>>, controller: Controller) -> Self {
        Self {
            requirements: vec![SubsystemRef(drivetrain.clone())],
            drivetrain,
            controller,
        }
    }
}

impl Command for DriveWithJoystickCommand {
    fn get_requirements(&self) -> &[SubsystemRef] {
        &self.requirements
    }

    fn execute(&mut self) -> Result {
        let left_y = self.controller.joystick_axis(JoystickAxis::LeftY)?;
        self.drivetrain.borrow_mut().drive(left_y);
        Ok(())
    }
}
