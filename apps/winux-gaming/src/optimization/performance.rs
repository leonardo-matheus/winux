// System performance tuning for gaming
// CPU governors, GPU settings, and other optimizations

use std::process::Command;
use std::path::PathBuf;
use std::fs;
use anyhow::{Result, Context};

/// Performance profile presets
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PerformanceProfile {
    /// Maximum battery life
    PowerSave,
    /// Balance between performance and power
    Balanced,
    /// Maximum performance
    Performance,
    /// Gaming-optimized (GameMode + performance governor)
    Gaming,
}

impl PerformanceProfile {
    pub fn display_name(&self) -> &str {
        match self {
            Self::PowerSave => "Economia de Energia",
            Self::Balanced => "Balanceado",
            Self::Performance => "Performance",
            Self::Gaming => "Gaming",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::PowerSave => "Maximiza duracao da bateria",
            Self::Balanced => "Equilibrio entre performance e consumo",
            Self::Performance => "Maximo desempenho, maior consumo",
            Self::Gaming => "Otimizado para jogos",
        }
    }

    pub fn cpu_governor(&self) -> &str {
        match self {
            Self::PowerSave => "powersave",
            Self::Balanced => "schedutil",
            Self::Performance | Self::Gaming => "performance",
        }
    }
}

/// Apply a performance profile
pub fn apply_profile(profile: &PerformanceProfile) -> Result<()> {
    // Set CPU governor
    set_cpu_governor(profile.cpu_governor())?;

    // Apply GPU settings based on profile
    match profile {
        PerformanceProfile::PowerSave => {
            set_gpu_power_profile("low")?;
        }
        PerformanceProfile::Performance | PerformanceProfile::Gaming => {
            set_gpu_power_profile("high")?;
        }
        _ => {}
    }

    Ok(())
}

/// Get current CPU governor
pub fn get_cpu_governor() -> Option<String> {
    let path = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor";
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

/// Get available CPU governors
pub fn get_available_governors() -> Vec<String> {
    let path = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors";
    fs::read_to_string(path)
        .ok()
        .map(|s| s.split_whitespace().map(String::from).collect())
        .unwrap_or_default()
}

/// Set CPU governor for all cores
pub fn set_cpu_governor(governor: &str) -> Result<()> {
    // Check if governor is available
    let available = get_available_governors();
    if !available.contains(&governor.to_string()) {
        anyhow::bail!("Governor '{}' is not available", governor);
    }

    // Try using cpupower first (preferred)
    let cpupower_result = Command::new("pkexec")
        .args(["cpupower", "frequency-set", "-g", governor])
        .status();

    if cpupower_result.map(|s| s.success()).unwrap_or(false) {
        return Ok(());
    }

    // Fallback to writing directly to sysfs
    let cpu_count = num_cpus();
    for cpu in 0..cpu_count {
        let path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor", cpu);
        if let Err(e) = fs::write(&path, governor) {
            // Might need root privileges
            Command::new("pkexec")
                .args(["sh", "-c", &format!("echo {} > {}", governor, path)])
                .status()
                .context(format!("Failed to set governor for CPU {}: {}", cpu, e))?;
        }
    }

    Ok(())
}

/// Get number of CPU cores
fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

/// Check if turbo boost is enabled
pub fn is_turbo_enabled() -> Option<bool> {
    // Intel
    if let Ok(content) = fs::read_to_string("/sys/devices/system/cpu/intel_pstate/no_turbo") {
        return Some(content.trim() == "0");
    }

    // AMD
    if let Ok(content) = fs::read_to_string("/sys/devices/system/cpu/cpufreq/boost") {
        return Some(content.trim() == "1");
    }

    None
}

/// Enable or disable turbo boost
pub fn set_turbo(enabled: bool) -> Result<()> {
    // Intel
    let intel_path = "/sys/devices/system/cpu/intel_pstate/no_turbo";
    if std::path::Path::new(intel_path).exists() {
        let value = if enabled { "0" } else { "1" };
        fs::write(intel_path, value)
            .or_else(|_| {
                Command::new("pkexec")
                    .args(["sh", "-c", &format!("echo {} > {}", value, intel_path)])
                    .status()
                    .map(|_| ())
            })
            .context("Failed to set Intel turbo boost")?;
        return Ok(());
    }

    // AMD
    let amd_path = "/sys/devices/system/cpu/cpufreq/boost";
    if std::path::Path::new(amd_path).exists() {
        let value = if enabled { "1" } else { "0" };
        fs::write(amd_path, value)
            .or_else(|_| {
                Command::new("pkexec")
                    .args(["sh", "-c", &format!("echo {} > {}", value, amd_path)])
                    .status()
                    .map(|_| ())
            })
            .context("Failed to set AMD boost")?;
        return Ok(());
    }

    anyhow::bail!("Turbo boost control not available")
}

/// GPU power profile
pub fn set_gpu_power_profile(profile: &str) -> Result<()> {
    // NVIDIA
    if is_nvidia_gpu() {
        let mode = match profile {
            "low" => "0",
            "auto" => "1",
            "high" => "2",
            _ => "1",
        };
        Command::new("nvidia-settings")
            .args(["-a", &format!("[gpu:0]/GpuPowerMizerMode={}", mode)])
            .status()
            .ok();
    }

    // AMD
    let amd_path = "/sys/class/drm/card0/device/power_dpm_state";
    if std::path::Path::new(amd_path).exists() {
        let state = match profile {
            "low" => "battery",
            "high" => "performance",
            _ => "balanced",
        };
        let _ = fs::write(amd_path, state)
            .or_else(|_| {
                Command::new("pkexec")
                    .args(["sh", "-c", &format!("echo {} > {}", state, amd_path)])
                    .status()
                    .map(|_| ())
            });
    }

    Ok(())
}

/// Check for NVIDIA GPU
fn is_nvidia_gpu() -> bool {
    Command::new("which")
        .arg("nvidia-smi")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get current CPU frequency info
#[derive(Debug, Clone)]
pub struct CpuFrequencyInfo {
    pub current_mhz: u64,
    pub min_mhz: u64,
    pub max_mhz: u64,
    pub governor: String,
}

pub fn get_cpu_frequency_info(cpu: usize) -> Option<CpuFrequencyInfo> {
    let base = format!("/sys/devices/system/cpu/cpu{}/cpufreq", cpu);

    let current = fs::read_to_string(format!("{}/scaling_cur_freq", base))
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()? / 1000;

    let min = fs::read_to_string(format!("{}/scaling_min_freq", base))
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()? / 1000;

    let max = fs::read_to_string(format!("{}/scaling_max_freq", base))
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()? / 1000;

    let governor = fs::read_to_string(format!("{}/scaling_governor", base))
        .ok()?
        .trim()
        .to_string();

    Some(CpuFrequencyInfo {
        current_mhz: current,
        min_mhz: min,
        max_mhz: max,
        governor,
    })
}

/// Memory optimization settings
pub fn set_swappiness(value: u8) -> Result<()> {
    let value = value.min(100);

    Command::new("sysctl")
        .args(["-w", &format!("vm.swappiness={}", value)])
        .status()
        .or_else(|_| {
            Command::new("pkexec")
                .args(["sysctl", "-w", &format!("vm.swappiness={}", value)])
                .status()
        })
        .context("Failed to set swappiness")?;

    Ok(())
}

/// Get current swappiness value
pub fn get_swappiness() -> Option<u8> {
    fs::read_to_string("/proc/sys/vm/swappiness")
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

/// Drop caches to free memory before gaming
pub fn drop_caches() -> Result<()> {
    Command::new("pkexec")
        .args(["sh", "-c", "sync && echo 3 > /proc/sys/vm/drop_caches"])
        .status()
        .context("Failed to drop caches")?;
    Ok(())
}

/// Compact memory to reduce fragmentation
pub fn compact_memory() -> Result<()> {
    Command::new("pkexec")
        .args(["sh", "-c", "echo 1 > /proc/sys/vm/compact_memory"])
        .status()
        .context("Failed to compact memory")?;
    Ok(())
}
