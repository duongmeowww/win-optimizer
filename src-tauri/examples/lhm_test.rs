// Final verification: print max CPU/GPU temps with sensor names
use lhm_sys::{Computer, ComputerOptions};
use std::thread;
use std::time::Duration;

fn main() {
    let Ok(mut computer) = Computer::create() else {
        println!("CREATE_FAILED");
        return;
    };
    computer.set_options(ComputerOptions {
        cpu_enabled: true,
        gpu_enabled: true,
        battery_enabled: false,
        controller_enabled: false,
        memory_enabled: false,
        motherboard_enabled: false,
        network_enabled: false,
        psu_enabled: false,
        storage_enabled: false,
    });

    let mut cpu_val = 0.0f32;
    let mut cpu_name = String::new();
    let mut gpu_val = 0.0f32;
    let mut gpu_name = String::new();

    for round in 0..3 {
        computer.update();
        thread::sleep(Duration::from_millis(1500));
        for hw in computer.hardware() {
            let is_gpu = is_gpu_hw(&hw.name());
            let is_cpu = !is_gpu;
            for s in hw.sensors() {
                let v = s.value();
                if !v.is_finite() || v <= 0.0 || v > 120.0 {
                    continue;
                }
                let sn = s.name().to_lowercase();
                let is_temp = sn.contains("temp")
                    || sn.contains("package")
                    || sn.contains("core max")
                    || sn.contains("core average")
                    || sn.contains("hot spot")
                    || sn.contains("distance to tjmax");
                if !is_temp {
                    continue;
                }
                if is_gpu && v > gpu_val {
                    gpu_val = v;
                    gpu_name = format!("{} ({})", hw.name(), s.name());
                } else if is_cpu && v > cpu_val {
                    cpu_val = v;
                    cpu_name = format!("{} ({})", hw.name(), s.name());
                }
            }
        }
    }

    println!("CPU={:.1} [{}]", cpu_val, cpu_name);
    println!("GPU={:.1} [{}]", gpu_val, gpu_name);
    println!("DONE");
}

fn is_gpu_hw(name: &str) -> bool {
    let n = name.to_lowercase();
    n.contains("nvidia") || n.contains("radeon") || n.contains("gpu")
        || n.contains("rtx") || n.contains("gtx") || n.contains("intel arc")
}
