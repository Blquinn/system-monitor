use std::time::Duration;

use slint::Timer;
use sysinfo::{CpuExt, System, SystemExt};

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    let mut sys = System::new_all();
    let timer = Timer::default();

    let ui_handle = ui.as_weak();
    timer.start(slint::TimerMode::Repeated, Duration::from_millis(500), move || {
        sys.refresh_cpu();
        let cpus = sys.cpus();
        let total_cpu = cpus.iter().fold(0.0, |acc, cpu| acc + cpu.cpu_usage());
        let avg_cpu = total_cpu / cpus.len() as f32;
        ui_handle.unwrap().set_cpu_usage(avg_cpu);
    });

    ui.run()
}
