#![no_std]
#![no_main]

extern crate alloc;

use pros::{prelude::*, task};
use robot::Robot;

mod robot;

#[derive(Default)]
struct RobotBase;

impl SyncRobot for RobotBase {
    fn opcontrol(&mut self) -> pros::Result {
        let mut robot = Robot::new();
        robot.configure_button_bindings();
        pros_command::robot::start_robot(robot).unwrap();
        Ok(())
    }
}

sync_robot!(RobotBase);
