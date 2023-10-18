use std::{time::Duration, rc::Rc};

use slint::{VecModel, Model, SharedString};
use sysinfo::{CpuExt, System, SystemExt};

slint::include_modules!();

fn vec_to_path(vec: &VecModel<f32>) -> String {
    let mut path = String::new();

    let mut last: Option<f32> = None;
    for (x, y) in vec.iter().enumerate() {
        if let Some(l) = last {
            path.push_str(format!("M {} {} L {} {} ", x-1, 100.0-l, x, 100.0-y).as_str());
        }

        last = Some(y.clone());
    }

    path.push_str("Z");

    path
}

fn main() -> Result<(), slint::PlatformError> {
    let app = AppWindow::new()?;

    let mut sys = System::new_all();

    // Updates thread
    let ui_handle = app.as_weak();
    std::thread::spawn(move || {
        loop {
            sys.refresh_cpu();
            let cpus = sys.cpus();

            let usages: Vec<f32> = cpus.iter().map(|c| c.cpu_usage()).collect();
            let total_cpu: f32 = usages.iter().sum();
            let avg_cpu = total_cpu / cpus.len() as f32;

            ui_handle.upgrade_in_event_loop(move |handle| {
                let model = handle.global::<CpuMonitorPageAdapter>().get_cpu_data();

                let cpus_models = model.as_any()
                    .downcast_ref::<VecModel<CpuData>>()
                    .unwrap();

                for (i, u) in usages.iter().enumerate() {
                    match cpus_models.row_data(i) {
                        Some(mut cpu_data) => {
                            let samples_model = cpu_data.samples.as_any().downcast_ref::<VecModel<f32>>().unwrap();
                            samples_model.push(u.clone());
                            if samples_model.row_count() > 120 {
                                samples_model.remove(0);
                            }

                            cpu_data.path = SharedString::from(vec_to_path(&samples_model));

                            cpus_models.set_row_data(i, cpu_data);
                        }
                        None => {
                            let model: Rc<VecModel<f32>> = Rc::new(VecModel::default());
                            model.set_vec(vec![u.clone()]);
                            cpus_models.push(CpuData{
                                samples: model.into(),
                                path: SharedString::from(""),
                            });
                        }
                    }
                }

                handle.set_cpu_usage(avg_cpu);
            }).unwrap();

            std::thread::sleep(Duration::from_millis(500));
        }
    });

    app.run()
}
