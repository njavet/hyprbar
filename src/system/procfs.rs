use std::fs;

#[derive(Debug, Clone, Copy)]
pub struct CpuSample {
    pub idle: u64,
    pub total: u64,
}

pub fn cpu_sample() -> Option<CpuSample> {
    let stat = fs::read_to_string("/proc/stat").ok()?;
    let line = stat.lines().next()?;

    let values: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|x| x.parse().ok())
        .collect();

    if values.len() < 4 {
        return None;
    }

    let idle =
        values[3] + values.get(4).copied().unwrap_or(0);

    Some(CpuSample {
        idle,
        total: values.iter().sum(),
    })
}

pub fn cpu_usage(
    previous: &mut Option<CpuSample>,
) -> f64 {
    let Some(current) = cpu_sample() else {
        return 0.0;
    };

    let usage = previous
        .map(|old| {
            let total =
                current.total.saturating_sub(old.total);

            let idle =
                current.idle.saturating_sub(old.idle);

            if total == 0 {
                0.0
            } else {
                1.0 - idle as f64 / total as f64
            }
        })
        .unwrap_or(0.0);

    *previous = Some(current);

    usage
}

pub fn ram_usage() -> f64 {
    let Ok(data) = fs::read_to_string("/proc/meminfo")
    else {
        return 0.0;
    };

    let mut total = None;
    let mut available = None;

    for line in data.lines() {
        if let Some(value) =
            line.strip_prefix("MemTotal:")
        {
            total = value
                .split_whitespace()
                .next()
                .and_then(|v| v.parse::<f64>().ok());
        }

        if let Some(value) =
            line.strip_prefix("MemAvailable:")
        {
            available = value
                .split_whitespace()
                .next()
                .and_then(|v| v.parse::<f64>().ok());
        }
    }

    match (total, available) {
        (Some(t), Some(a)) if t > 0.0 => 1.0 - a / t,
        _ => 0.0,
    }
}

pub fn network_bytes() -> Option<u64> {
    let data = fs::read_to_string("/proc/net/dev").ok()?;

    let mut total = 0;

    for line in data.lines().skip(2) {
        let Some((interface, data)) =
            line.split_once(':')
        else {
            continue;
        };

        if interface.trim() == "lo" {
            continue;
        }

        let values: Vec<&str> =
            data.split_whitespace().collect();

        if values.len() < 9 {
            continue;
        }

        let rx = values[0].parse::<u64>().unwrap_or(0);
        let tx = values[8].parse::<u64>().unwrap_or(0);

        total += rx + tx;
    }

    Some(total)
}
