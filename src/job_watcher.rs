use std::collections::HashMap;
use std::path::PathBuf;
use std::{io::BufRead, process::Command, thread, time::Duration};

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
            ));
        });
        job_list
    }

    // fn resolve_path(
    //     path: &str,
    //     array_master: &str,
    //     array_id: &str,
    //     id: &str,
    //     host: &str,
    //     user: &str,
    //     name: &str,
    //     working_dir: &str,
    // ) -> Option<PathBuf> {
    //     // see https://slurm.schedmd.com/sbatch.html#SECTION_%3CB%3Efilename-pattern%3C/B%3E
    //     lazy_static::lazy_static! {
    //         static ref RE: Regex = Regex::new(r"%(%|A|a|J|j|N|n|s|t|u|x)").unwrap();
    //     }
    //
    //     let mut path = path.to_owned();
    //     let slurm_no_val = "4294967294";
    //     let array_id = if array_id == "N/A" {
    //         slurm_no_val
    //     } else {
    //         array_id
    //     };
    //
    //     if path.is_empty() {
    //         // never happens right now, because `squeue -O stdout` seems to always return something
    //         path = if array_id == slurm_no_val {
    //             PathBuf::from(working_dir).join("slurm-%J.out")
    //         } else {
    //             PathBuf::from(working_dir).join("slurm-%A_%a.out")
    //         }
    //         .to_str()
    //         .unwrap()
    //         .to_owned()
    //     };
    //
    //     for cap in RE
    //         .captures_iter(&path.clone())
    //         .collect::<Vec<_>>() // TODO: this is stupid, there has to be a better way to reverse the captures...
    //         .iter()
    //         .rev()
    //     {
    //         let m = cap.get(0).unwrap();
    //         let replacement = match m.as_str() {
    //             "%%" => "%",
    //             "%A" => array_master,
    //             "%a" => array_id,
    //             "%J" => id,
    //             "%j" => id,
    //             "%N" => host.split(',').next().unwrap_or(host),
    //             "%n" => "0",
    //             "%s" => "batch",
    //             "%t" => "0",
    //             "%u" => user,
    //             "%x" => name,
    //             _ => unreachable!(),
    //         };
    //
    //         path.replace_range(m.range(), replacement);
    //     }
    //
    //     Some(PathBuf::from(path))
    // }
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
