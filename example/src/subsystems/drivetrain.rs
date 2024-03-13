use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use pros::devices::Controller;
use pros::devices::controller::JoystickAxis;
use pros::prelude::*;
use pros_command::command::Command;
use pros_command::{AnyCommand, AnySubsystem};
use pros_command::subsystem::{Subsystem, SubsystemRefExt};

#[derive(Debug)]
pub struct Drivetrain {
    left_motor: Motor,
    right_motor: Motor,
}

impl Drivetrain {
    pub fn new(left_motor: SmartPort, right_motor: SmartPort) -> Result<Self> {
        Ok(Self {
            left_motor: Motor::new(left_motor, BrakeMode::Brake)?,
            right_motor: Motor::new(right_motor, BrakeMode::Brake)?,
        })
    }

    pub fn drive(self: Rc<RefCell<Self>>, speed: f32) -> impl Command {
        self.run(|| { Ok(()) })
    }

    pub fn drive_with_controller(self: Rc<RefCell<Self>>, controller: Controller) -> impl Command {
        self.run(|| { Ok(()) })
    }
}

impl Subsystem for Drivetrain {
    
}

pub struct DriveCommand {
    pub speed: f32,
    pub drivetrain: Rc<RefCell<Drivetrain>>
}

impl Command for DriveCommand {
    fn get_requirements(&self) -> &[AnySubsystem] {
        &[self.drivetrain.clone().into()]
    }

    fn execute(&mut self) -> Result {
        self.drivetrain.borrow().left_motor.set_output(self.speed)?;
        self.drivetrain.borrow().right_motor.set_output(self.speed)?;
        Ok(())
    }
}


pub struct DriveWithJoystickCommand {
    drivetrain: Rc<RefCell<Drivetrain>>,
    controller: Controller,
    requirements: Vec<AnySubsystem>,
}

impl DriveWithJoystickCommand {
    pub fn new(drivetrain: Rc<RefCell<Drivetrain>>, controller: Controller) -> Self {
        Self {
            requirements: vec![AnySubsystem(drivetrain.clone())],
            drivetrain,
            controller,
        }
    }
}

impl Command for DriveWithJoystickCommand {
    fn get_requirements(&self) -> &[AnySubsystem] {
        &self.requirements
    }

    fn execute(&mut self) -> Result {
        let left_y = self.controller.joystick_axis(JoystickAxis::LeftY)?;
        self.drivetrain.borrow().left_motor.set_output(left_y)?;
        let right_y = self.controller.joystick_axis(JoystickAxis::RightY)?;
        self.drivetrain.borrow().right_motor.set_output(right_y)?;
        Ok(())
    }
}