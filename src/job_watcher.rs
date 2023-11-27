use std::collections::HashMap;

use std::path::PathBuf;
use std::{io::BufRead, thread, time::Duration};

use crossbeam::channel::Sender;

use crate::app::AppMessage;
use crate::slurm::SlurmJob;

struct JobWatcher {
    app: Sender<AppMessage>,
    interval: Duration,
    user: String,
    time_limit: usize,
}

#[derive(Debug)]
pub struct JobWatcherHandle {}

impl JobWatcher {
    fn new(app: Sender<AppMessage>, interval: Duration, user: String, time_limit: usize) -> Self {
        Self {
            app,
            interval,
            user,
            time_limit,
        }
    }

    fn run(&mut self) -> Self {
        loop {
            //TODO: how to access the jobs list from outside the app?
            let job_vec = self.refresh_job_list();
            // dbg!(&job_vec);
            self.app.send(AppMessage::JobList(job_vec)).unwrap();
            thread::sleep(self.interval);
        }
    }
    pub fn refresh_job_list(&mut self) -> Vec<SlurmJob> {
        let cmd = format!(
        "sacct -u {} -S $(date -d '{} hours ago' +\"%Y-%m-%dT%H:%M:%S\")  \
        --format=JobID,JobName,Partition,Account,Submit,Start,End,State,WorkDir,Reason,TimeLimit,Elapsed  \
        --parsable2 ", self.user, self.time_limit);
        // let args = vec!["-u", user, "-S", "$(date -d '{} hours ago' +\"%Y-%m-%dT%H:%M:%S\")", "--format=JobID,JobName,Partition,Account,Submit,Start,End,State,WorkDir,Reason,TimeLimit,Elapsed", "--parsable2"];
        let exclude_strings = vec!["batch", "extern", ".0"];
        let status_map = HashMap::from([
            ("PENDING", "PD"),
            ("RUNNING", "R"),
            ("COMPLETED", "CD"),
            ("FAILED", "F"),
            ("CANCELLED", "CA"),
            ("TIMEOUT", "TO"),
            ("PREEMPTED", "PR"),
            ("NODE_FAIL", "NF"),
            ("REVOKED", "RV"),
            ("SUSPENDED", "S"),
        ]);
        let output = std::process::Command::new("bash")
            .arg("-c")
            .arg(cmd)
            // .args(&args)
            .output()
            .expect("failed to execute process");

        // work on the string, parse into a SlurmJob struct
        let output = String::from_utf8_lossy(&output.stdout);
        let mut job_list: Vec<SlurmJob> = Vec::new();
        output.lines().skip(1).for_each(|line| {
            let parts = line.split('|').collect::<Vec<&str>>();
            if let Some(_) = exclude_strings.iter().find(|&&s| parts[0].contains(s)) {
                return;
            }
            if parts[1] == "_interactive" {
                return;
            }
            // TODO: bug - array jobs won't parse into a u32, they are jobid.0 etc
            let job_id = parts[0].to_string();
            let job_name = parts[1].to_string();
            let partition = parts[2].to_string();
            let account = parts[3].to_string();
            let submit = parts[4].to_string(); // parse this to datetime
            let start = parts[5].to_string();
            let end = parts[6].to_string();
            let state = status_map
                .get(parts[7].split_whitespace().nth(0).unwrap())
                .unwrap_or(&parts[7])
                .to_string();
            let work_dir = parts[8].to_string();
            let reason = parts[9].to_string();
            let time_limit = parts[10].to_string();
            let elapsed_time = parts[11].to_string();
            // we don't get stdout from sacct, use best guess for completed jobs, otherwise this
            // will be filled from squeue later.
            let (stdout, stderr) = (None, None);
            job_list.push(SlurmJob::new(
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
            ));
        });

        // now run the squeue command, pick up any jobs that don't show in sacct (e.g. jobs pending
        // without a start time etc)
        let squ_cmd = format! {"squeue -u {} \
        --format=JobID,JobName,Partition,Account,Submit,Start,End,State,WorkDir,Reason,TimeLimit,Elapsed,Stdout,Stderr",
        self.user};

        let output = std::process::Command::new("bash")
            .arg("-c")
            .arg(squ_cmd)
            .output()
            .expect("Failed to spawn squeue command");

        let output = String::from_utf8_lossy(&output.stdout);

        output.lines().skip(1).for_each(|line| {
            let parts = line.split(",").collect::<Vec<&str>>();
            let job_id = parts[0].to_string();
            // check if job_id already exists
            if let Some(job) = job_list.iter_mut().find(|j| j.job_id == job_id) {
                // update the stdout/stderr from the job, only if it exists
                let stdout = parts[12].to_string();
                let stderr = parts[13].to_string();
                if PathBuf::from(&stdout).is_file() {
                    job.stdout = Some(stdout);
                }
                if PathBuf::from(&stderr).is_file() {
                    job.stderr = Some(stderr);
                }
            } else {
                // create the job from scratch, it is pending or in some other state that sacct
                // ignores
                // if job_list.iter().find(|&j| j.job_id == job_id).is_none() {
                // create new SlurmJob
                let job_name = parts[1].to_string();
                let partition = parts[2].to_string();
                let account = parts[3].to_string();
                let submit = parts[4].to_string(); // parse this to datetime
                let start = parts[5].to_string();
                let end = parts[6].to_string();
                let state = status_map
                    .get(parts[7].split_whitespace().nth(0).unwrap())
                    .unwrap_or(&parts[7])
                    .to_string();
                let work_dir = parts[8].to_string();
                let reason = parts[9].to_string();
                let time_limit = parts[10].to_string();
                let elapsed_time = parts[11].to_string();
                let stdout = parts[12].to_string();
                let stderr = parts[13].to_string();
                job_list.push(SlurmJob::new(
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
                    Some(stdout),
                    Some(stderr),
                ));
            };
        });

        job_list.sort_by(|a, b| a.job_id.cmp(&b.job_id));

        job_list
    }
}

impl JobWatcherHandle {
    pub fn new(
        app: Sender<AppMessage>,
        interval: Duration,
        user: String,
        time_limit: usize,
    ) -> Self {
        let mut actor = JobWatcher::new(app, interval, user, time_limit);
        thread::spawn(move || actor.run());

        Self {}
    }
}
