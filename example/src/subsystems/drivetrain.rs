use alloc::rc::Rc;
use core::cell::RefCell;
use pros::prelude::*;
use pros_command::CommandRef;
use pros_command::subsystem::{Subsystem, SubsystemRefExt};

#[derive(Debug)]
pub struct Drivetrain {
    left_motor: Motor,
    right_motor: Motor,
    tick_number: i32,
    speed: f32,
}

impl Drivetrain {
    pub fn new(left_motor: SmartPort, right_motor: SmartPort) -> Result<Self> {
        Ok(Self {
            left_motor: Motor::new(left_motor, BrakeMode::Brake)?,
            right_motor: Motor::new(right_motor, BrakeMode::Brake)?,
            tick_number: 0,
            speed: 0.0,
        })
    }

    pub fn drive(self: Rc<RefCell<Self>>, speed: f32) -> CommandRef {
        self.run(|| { Ok(()) }).into()
    }
}

impl Subsystem for Drivetrain {
    fn periodic(&mut self) {
        println!("tick {}", self.tick_number);
        self.tick_number += 1;

        _ = self.left_motor.set_output(self.speed);
    }
}
