use crate::app::StatefulList;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
        }
    }
}

pub fn refresh_job_list(user: &str, time_period: usize) -> StatefulList<SlurmJob> {
    let cmd = format!("sacct -u {} -S $(date -d '{} hours ago' +\"%Y-%m-%dT%H:%M:%S\") --format=JobID,JobName,Partition,Account,Submit,Start,End,State,WorkDir,Reason --parsable2 ", user, time_period);

    let output = std::process::Command::new("bash")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("failed to execute process");

    // work on the string, parse into a SlurmJob struct
    let output = String::from_utf8_lossy(&output.stdout);
    let mut job_list: Vec<SlurmJob> = Vec::new();
    output.lines().skip(1).for_each(|line| {
        let parts = line.split('|').collect::<Vec<&str>>();
        dbg!(&parts);
        // TODO: bug - array jobs won't parse into a u32, they are jobid.0 etc
        let job_id = parts[0].to_string();
        let job_name = parts[1].to_string();
        let partition = parts[2].to_string();
        let account = parts[3].to_string();
        let submit = parts[4].to_string(); // parse this to datetime
        let start = parts[5].to_string();
        let end = parts[6].to_string();
        let state = parts[7].to_string();
        let work_dir = parts[8].to_string();
        let reason = parts[9].to_string();
        job_list.push(SlurmJob::new(
            job_id, job_name, partition, account, state, start, submit, end, reason, work_dir,
        ));
    });
    StatefulList::with_items(job_list)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std;

    #[test]
    fn test_get_job_list() {
        let user = std::env::var("USER").unwrap();
        let time_period = 24;

        let job_list = get_job_lists(&user, time_period);

        dbg!(&job_list);
    }
}
