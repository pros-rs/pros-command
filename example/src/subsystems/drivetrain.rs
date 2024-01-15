use pros::prelude::*;
use pros_command::subsystem::Subsystem;

#[derive(Debug)]
pub struct Drivetrain {
    left_motor: Motor,
    tick_number: i32,
    speed: f32,
}

impl Drivetrain {
    pub fn new() -> pros::Result<Self> {
        Ok(Self {
            left_motor: Motor::new(1, BrakeMode::Brake)?,
            tick_number: 0,
            speed: 0.0,
        })
    }

    pub fn drive(&mut self, speed: f32) {
        self.speed = speed;
    }
}

impl Subsystem for Drivetrain {
    fn periodic(&mut self) {
        println!("tick {}", self.tick_number);
        self.tick_number += 1;

        _ = self.left_motor.set_output(self.speed);
    }
}
