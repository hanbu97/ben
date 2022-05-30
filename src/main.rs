mod cli;

use std::io::Write;
use std::{env::current_dir, time::Instant};
use structopt::StructOpt;
use sysinfo::{
    NetworkExt, NetworksExt, ProcessExt, ProcessRefreshKind, RefreshKind, System, SystemExt,
};
use tokio::sync::RwLock;

async fn monitor(
    pid: u32,
    sys: &mut System,
    interval: f32,
    exact: bool,
    mem_vs_time: &mut RwLock<(Vec<u64>, Vec<u64>)>,
    start: &Instant,
) {
    sys.refresh_processes_specifics(
        ProcessRefreshKind::everything()
            .without_cpu()
            .with_disk_usage(),
    );
    if let Some(process) = sys.process(sysinfo::Pid::from(pid as i32)) {
        let mem = process.memory();

        let mut mem_vs_time_lock = mem_vs_time.write().await;
        let t = start.elapsed().as_secs();
        println!("{}  {} KB", t, mem);

        mem_vs_time_lock.0.push(mem);
        mem_vs_time_lock.1.push(t);
    }

    tokio::time::sleep(std::time::Duration::from_secs_f32(interval)).await;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let current_dir = current_dir()?;

    let opt = cli::CmdParams::from_args();
    println!("{:#?}", opt);

    let mut cmd = tokio::process::Command::new("zsh")
        .arg("-c")
        .arg(opt.command)
        .current_dir(current_dir)
        .spawn()?;

    let pid = cmd.id().unwrap_or(0);
    let mut sys = System::new();
    let time_start = std::time::Instant::now();

    let mut mem_vs_time = RwLock::new((Vec::<u64>::new(), Vec::<u64>::new()));

    loop {
        tokio::select! {
            _ = cmd.wait() => {
                println!("job finished");
                let mem_vs_time = mem_vs_time.read().await.clone();
                let time_min = &mem_vs_time.1.iter().min().unwrap().to_owned();

                let time_list: Vec<u64> =  mem_vs_time.1.into_iter().map(|x| x-time_min).collect();
                let mem_list = mem_vs_time.0;

                std::fs::create_dir_all(opt.output.parent().unwrap())?;
                let mut file = std::fs::File::create(opt.output)?;
                for (t,m) in time_list.into_iter().zip(mem_list.into_iter()) {
                    writeln!(file, "{} {}", t, m)?;
                    println!("{} {}", t, m);
                }
                return Ok(())
            },
            _ =monitor(pid, &mut sys, opt.interval, opt.exact,&mut mem_vs_time, &time_start) => {}
        }
    }

    Ok(())
}
