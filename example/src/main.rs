#![no_std]
#![no_main]

extern crate alloc;

use pros::core::task;
use pros::prelude::*;
use robot::Robot;

pub mod commands;
pub mod robot;
pub mod subsystems;

struct RobotBase;

impl Default for RobotBase {
    fn default() -> Self {
        task::spawn(|| {
            let mut robot = Robot::new().unwrap();
            robot.configure_button_bindings();
            pros_command::robot::start_robot(robot).unwrap();
        });
        Self
    }
}

impl SyncRobot for RobotBase {}

sync_robot!(RobotBase);
