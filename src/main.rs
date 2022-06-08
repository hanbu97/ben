mod cli;

use nvml_wrapper::NVML as Nvml;
use std::env::current_dir;
use std::io::Write;
use structopt::StructOpt;
use sysinfo::{ProcessExt, ProcessRefreshKind, System, SystemExt};
use tokio::sync::RwLock;

async fn monitor<'a>(
    pid: u32,
    sys: &mut System,
    _exact: bool,
    mem_vs_time: &mut RwLock<(Vec<u64>, Vec<u64>)>,
    time_elsapsed: u64,
) {
    sys.refresh_processes_specifics(
        ProcessRefreshKind::everything()
            .without_cpu()
            .with_disk_usage(),
    );
    if let Some(process) = sys.process(sysinfo::Pid::from(pid as i32)) {
        let mem = process.memory();

        let mut mem_vs_time_lock = mem_vs_time.write().await;

        println!("{pid}: {}  {} KB", time_elsapsed, mem);

        mem_vs_time_lock.0.push(mem);
        mem_vs_time_lock.1.push(time_elsapsed);
    }

    // tokio::time::sleep(std::time::Duration::from_secs_f32(interval)).await;
}

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref SUPPORT_MODES: Vec<String> = vec!["mem".into(), "gpu".into()];
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let current_dir = current_dir()?;

    let mut opt = cli::CmdParams::from_args();
    if opt.mode.is_empty() {
        opt.mode = SUPPORT_MODES.to_vec()
    }

    println!("{:#?}", opt);

    // init gpu monitor
    let nvml = Nvml::init()?;
    let device = nvml.device_by_index(0)?;

    // run command
    let mut cmd = tokio::process::Command::new("zsh")
        .arg("-c")
        .arg(opt.command)
        .current_dir(current_dir)
        .spawn()?;

    let pid = cmd.id().unwrap_or(0);
    let mut sys = System::new();
    let time_start = std::time::Instant::now();

    // init data buffer
    let mut mem_vs_time = RwLock::new((Vec::<u64>::new(), Vec::<u64>::new()));
    let gpu_vs_time = RwLock::new((Vec::<u64>::new(), Vec::<u64>::new()));

    // poll monitoring
    loop {
        tokio::select! {
            _ = cmd.wait() => {
                println!("job finished");
                let mem_vs_time = mem_vs_time.read().await.clone();
                let time_min = &mem_vs_time.1.iter().min().unwrap().to_owned();

                let time_list: Vec<u64> =  mem_vs_time.1.into_iter().map(|x| x-time_min).collect();
                let mem_list = mem_vs_time.0;

                std::fs::create_dir_all(opt.output.parent().unwrap())?;
                let mut file = std::fs::File::create(&(opt.output.display().to_string()+"_mem"))?;
                for (t,m) in time_list.iter().zip(mem_list.into_iter()) {
                    writeln!(file, "{} {}", t, m)?;
                    println!("{} {}", t, m);
                }

                let gpu_vs_time = gpu_vs_time.read().await.clone();
                let gpu_list = gpu_vs_time.0;

                let mut file = std::fs::File::create(&(opt.output.display().to_string()+"_gpu"))?;
                for (t,m) in time_list.iter().zip(gpu_list.into_iter()) {
                    writeln!(file, "{} {}", t, m)?;
                    println!("{} {}", t, m);
                }

                return Ok(())
            },
            _ = tokio::time::sleep(std::time::Duration::from_secs_f32(opt.interval)) => {

                let time_elsapsed = time_start.elapsed().as_secs();
                monitor(pid, &mut sys,  opt.exact,&mut mem_vs_time, time_elsapsed).await;

                // gpu process mem usage
                let process = device.running_compute_processes()?;
                let mut used_mem = 0;
                for p in process {
                    let gpid = p.pid;
                    let ggpu = p.used_gpu_memory;

                    if gpid == pid {
                        match ggpu {
                            nvml_wrapper::enums::device::UsedGpuMemory::Used(used) => {used_mem = used/1024},
                            _ => {},
                        }
                    }
                };
                println!("{pid}: {}  {} KB", time_elsapsed, used_mem);
                let mut gpu_vs_time_lock = gpu_vs_time.write().await;
                gpu_vs_time_lock.0.push(used_mem);
                gpu_vs_time_lock.1.push(time_elsapsed);


                // gpu  utilization
                // let util =  device.utilization_rates()?;

                // dbg!(process);
            }
        }
    }
}

// let counter_send = PcieUtilCounter::Send;
// let counter_receive = PcieUtilCounter::Send;

// let pcie = device.pcie_throughput(counter_send);
// dbg!(pcie);

// let mem_info = device.memory_info()?;

// dbg!(mem_info);
