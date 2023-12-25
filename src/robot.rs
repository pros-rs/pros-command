use core::time::Duration;

use pros::task::Interval;

/// Returns true if the code is running on a real robot and not in simulation.
pub const fn is_real() -> bool {
    cfg!(target_os = "vexos")
}

/// Returns true if the code is running in simulation and not on a real robot.
pub const fn is_sim() -> bool {
    !is_real()
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum RobotState {
    Disabled,
    Autonomous,
    Opcontrol,
}

impl RobotState {
    pub fn current() -> Self {
        if pros::competition::is_autonomous() {
            Self::Autonomous
        } else if pros::competition::is_connected() {
            Self::Opcontrol
        } else {
            Self::Disabled
        }
    }
}

pub trait ScheduledRobot {
    fn periodic(&mut self) -> pros::Result {
        Ok(())
    }
    fn sim_periodic(&mut self) -> pros::Result {
        Ok(())
    }
    fn disabled_init(&mut self) -> pros::Result {
        Ok(())
    }
    fn disabled_periodic(&mut self) -> pros::Result {
        Ok(())
    }
    fn autonomous_init(&mut self) -> pros::Result {
        Ok(())
    }
    fn autonomous_periodic(&mut self) -> pros::Result {
        Ok(())
    }
    fn opcontrol_init(&mut self) -> pros::Result {
        Ok(())
    }
    fn opcontrol_periodic(&mut self) -> pros::Result {
        Ok(())
    }
}

pub const ITERATION_PERIOD: Duration = Duration::from_millis(20);

pub fn start_robot(mut robot: impl ScheduledRobot) -> pros::Result {
    let mut phase = None;
    let mut interval = Interval::start();

    loop {
        let new_phase = RobotState::current();
        match new_phase {
            RobotState::Disabled => {
                if Some(new_phase) != phase {
                    robot.disabled_init()?;
                }
                robot.disabled_periodic()?;
            }
            RobotState::Autonomous => {
                if Some(new_phase) != phase {
                    robot.autonomous_init()?;
                }
                robot.autonomous_periodic()?;
            }
            RobotState::Opcontrol => {
                if Some(new_phase) != phase {
                    robot.opcontrol_init()?;
                }
                robot.opcontrol_periodic()?;
            }
        }
        phase = Some(new_phase);

        robot.periodic()?;
        if is_sim() {
            robot.sim_periodic()?;
        }

        interval.delay(ITERATION_PERIOD);
    }
}
