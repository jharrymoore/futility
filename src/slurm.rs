use std::str::FromStr;

use chrono::{NaiveTime, Timelike};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct SlurmJob {
    pub job_id: String,
    pub job_name: String,
    pub partition: String,
    pub account: String,
    pub state: String,
    pub start: String,
    pub submit: String,
    pub end: String,
    pub reason: String,
    pub work_dir: String,
    pub time_limit: String,
    pub elapsed_time: String,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

impl SlurmJob {
    pub fn new(
        job_id: String,
        job_name: String,
        partition: String,
        account: String,
        state: String,
        start: String,
        submit: String,
        end: String,
        reason: String,
        work_dir: String,
        time_limit: String,
        elapsed_time: String,
        stdout: Option<String>,
        stderr: Option<String>,
    ) -> SlurmJob {
        SlurmJob {
            job_id,
            job_name,
            partition,
            account,
            state,
            start,
            submit,
            end,
            reason,
            work_dir,
            time_limit,
            elapsed_time,
            stdout,
            stderr,
        }
    }
}

impl SlurmJob {
    pub fn cancel(&self) {
        let cmd = format!("scancel {}", self.job_id);
        std::process::Command::new("bash")
            .arg("-c")
            .arg(cmd)
            .output()
            .expect("failed to execute process");
    }
    pub fn restart(&self) {
        let cmd = format!("scontrol requeue {}", self.job_id);
        std::process::Command::new("bash")
            .arg("-c")
            .arg(cmd)
            .output()
            .expect("failed to execute process");
    }

    pub fn get_percent_completed(&self) -> u16 {
        let elapsed = NaiveTime::from_str(&self.elapsed_time).unwrap();
        // get elapsed time
        let wall_time = NaiveTime::from_str(&self.time_limit);
        // dbg!(&wall_time);
        match wall_time {
            Ok(wall_time) => {
                let percent_complete = (elapsed.num_seconds_from_midnight() as f32
                    / wall_time.num_seconds_from_midnight() as f32)
                    * 100.;
                percent_complete as u16
                // dbg!(percent_complete);
            }
            Err(_) => 0.0 as u16,
        }
    }
}
