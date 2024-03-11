use core::time::Duration;
use pros::core::task::Interval;
use pros::devices::competition;
use pros::devices::competition::CompetitionMode;
use pros::prelude::*;

/// Returns true if the code is running on a real robot and not in simulation.
pub const fn is_real() -> bool {
    cfg!(target_os = "vexos")
}

/// Returns true if the code is running in simulation and not on a real robot.
pub const fn is_sim() -> bool {
    !is_real()
}

pub trait ScheduledRobot {
    fn periodic(&mut self) -> Result {
        Ok(())
    }
    fn sim_periodic(&mut self) -> Result {
        Ok(())
    }
    fn disabled_init(&mut self) -> Result {
        Ok(())
    }
    fn disabled_periodic(&mut self) -> Result {
        Ok(())
    }
    fn autonomous_init(&mut self) -> Result {
        Ok(())
    }
    fn autonomous_periodic(&mut self) -> Result {
        Ok(())
    }
    fn opcontrol_init(&mut self) -> Result {
        Ok(())
    }
    fn opcontrol_periodic(&mut self) -> Result {
        Ok(())
    }
}

pub const ITERATION_PERIOD: Duration = Duration::from_millis(20);

pub fn start_robot(mut robot: impl ScheduledRobot) -> Result {
    let mut previous_mode = None;
    let mut interval = Interval::start();

    loop {
        let current_mode = competition::mode();
        match current_mode {
            CompetitionMode::Disabled => {
                if previous_mode != Some(CompetitionMode::Disabled) {
                    robot.disabled_init()?;
                }
                robot.disabled_periodic()?;
            }
            CompetitionMode::Autonomous => {
                if previous_mode != Some(CompetitionMode::Autonomous) {
                    robot.autonomous_init()?;
                }
                robot.autonomous_periodic()?;
            }
            CompetitionMode::Opcontrol => {
                if previous_mode != Some(CompetitionMode::Opcontrol) {
                    robot.opcontrol_init()?;
                }
                robot.opcontrol_periodic()?;
            }
        }
        previous_mode = Some(current_mode);

        robot.periodic()?;
        if is_sim() {
            robot.sim_periodic()?;
        }

        interval.delay(ITERATION_PERIOD);
    }
}
