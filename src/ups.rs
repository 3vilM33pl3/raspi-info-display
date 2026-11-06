//! Read-only NUT telemetry. Neither the collector nor the screen issues UPS commands.
use crate::screens::Screen;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::io;
use std::process::{Command, Stdio};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::System;

const POLL_INTERVAL: Duration = Duration::from_secs(5);
const QUERY_TIMEOUT: Duration = Duration::from_secs(2);
const MAX_AGE: Duration = Duration::from_secs(15);

#[derive(Clone, Debug)]
pub struct UpsReading {
    pub status: String,
    charge: Option<f64>,
    runtime: Option<f64>,
    load: Option<f64>,
    voltage: Option<f64>,
}

impl UpsReading {
    pub fn parse(output: &str) -> Result<Self> {
        let fields: HashMap<&str, &str> = output
            .lines()
            .filter_map(|line| {
                line.split_once(':')
                    .map(|(key, value)| (key.trim(), value.trim()))
            })
            .collect();
        let status = fields
            .get("ups.status")
            .filter(|s| !s.is_empty())
            .ok_or_else(|| anyhow!("Missing UPS status"))?
            .to_string();
        let number = |key, max| {
            fields
                .get(key)
                .and_then(|value| value.parse::<f64>().ok())
                .filter(|value| value.is_finite() && *value >= 0.0 && *value <= max)
        };
        Ok(Self {
            status,
            charge: number("battery.charge", 100.0),
            runtime: number("battery.runtime", 31536000.0),
            load: number("ups.load", 100.0),
            voltage: number("input.voltage", 1000.0),
        })
    }

    fn label(&self) -> (&'static str, bool) {
        let tokens: Vec<_> = self.status.split_whitespace().collect();
        if tokens.contains(&"LB") {
            return ("LOW BATTERY", true);
        }
        if tokens.contains(&"RB") {
            return ("REPLACE BATTERY", true);
        }
        if tokens.contains(&"OVER") {
            return ("OVERLOAD", true);
        }
        if tokens.contains(&"OB") {
            return ("ON BATTERY", true);
        }
        if tokens.contains(&"OL") && tokens.iter().all(|t| matches!(*t, "OL" | "CHRG")) {
            return (
                if tokens.contains(&"CHRG") {
                    "MAINS / CHARGING"
                } else {
                    "MAINS OK"
                },
                false,
            );
        }
        ("CHECK UPS", true)
    }

    fn content(&self) -> String {
        let percent =
            |value: Option<f64>| value.map(|v| format!("{:.0}%", v)).unwrap_or("--".into());
        let runtime = self
            .runtime
            .map(|seconds| format!("~{} min", (seconds / 60.0).floor() as u64))
            .unwrap_or("--".into());
        let voltage = self
            .voltage
            .map(|v| format!("{:.0}V", v))
            .unwrap_or("--".into());
        format!(
            "{}\nBattery: {}\nRuntime: {}\nLoad {} In {}",
            self.label().0,
            percent(self.charge),
            runtime,
            percent(self.load),
            voltage
        )
    }
}

#[derive(Default)]
struct Snapshot(Option<(UpsReading, Instant)>);
impl Snapshot {
    fn fresh(&self) -> Option<&UpsReading> {
        self.0
            .as_ref()
            .filter(|(_, when)| when.elapsed() < MAX_AGE)
            .map(|(reading, _)| reading)
    }
    fn priority(&self) -> bool {
        self.fresh().map(|r| r.label().1).unwrap_or(true)
    }
    fn content(&self) -> String {
        self.fresh()
            .map(UpsReading::content)
            .unwrap_or_else(|| "UPS unavailable\nCheck USB / NUT\nBattery: --\nRuntime: --".into())
    }
}

fn query(program: &str, source: &str, timeout: Duration) -> io::Result<String> {
    let mut child = Command::new(program)
        .arg(source)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let output = child.wait_with_output()?;
                return if status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
                } else {
                    Err(io::Error::other("upsc failed"))
                };
            }
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(20)),
            other => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(match other {
                    Err(error) => error,
                    _ => io::Error::new(io::ErrorKind::TimedOut, "upsc timed out"),
                });
            }
        }
    }
}

pub struct UpsScreen {
    state: Arc<Mutex<Snapshot>>,
    stop: Arc<AtomicBool>,
}
impl UpsScreen {
    pub fn new() -> Self {
        let state = Arc::new(Mutex::new(Snapshot::default()));
        let stop = Arc::new(AtomicBool::new(false));
        let worker_state = Arc::clone(&state);
        let worker_stop = Arc::clone(&stop);
        let source =
            std::env::var("INFO_DISPLAY_UPS_SOURCE").unwrap_or_else(|_| "apc@127.0.0.1".into());
        thread::spawn(move || {
            while !worker_stop.load(Ordering::Relaxed) {
                let started = Instant::now();
                let reading = if source.starts_with('-') || source.is_empty() {
                    None
                } else {
                    query("/usr/bin/upsc", &source, QUERY_TIMEOUT)
                        .ok()
                        .and_then(|text| UpsReading::parse(&text).ok())
                };
                let snapshot = reading.map(|reading| (reading, Instant::now()));
                *worker_state.lock().unwrap_or_else(|e| e.into_inner()) = Snapshot(snapshot);
                while started.elapsed() < POLL_INTERVAL && !worker_stop.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(100));
                }
            }
        });
        Self { state, stop }
    }
}
impl Drop for UpsScreen {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}
impl Screen for UpsScreen {
    fn name(&self) -> &'static str {
        "ups"
    }
    fn title(&self) -> Result<String> {
        let host = hostname::get()?
            .to_string_lossy()
            .split('.')
            .next()
            .unwrap_or("WHITE")
            .to_ascii_uppercase();
        Ok(format!(
            "UPS - {}",
            host.chars().take(12).collect::<String>()
        ))
    }
    fn priority(&self) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .priority()
    }
    fn render(&self, _sys: &System) -> Result<String> {
        Ok(self
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .content())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const ONLINE: &str = "ups.status: OL CHRG\nbattery.charge: 100\nbattery.runtime: 5100\nups.load: 18\ninput.voltage: 238\n";
    #[test]
    fn supported_values_and_frame_fit() {
        let reading = UpsReading::parse(ONLINE).unwrap();
        assert!(!reading.label().1);
        assert!(reading.content().contains("Runtime: ~85 min"));
        assert_eq!(reading.content().lines().count(), 4);
        assert!(
            reading
                .content()
                .lines()
                .all(|line| line.is_ascii() && line.len() <= 21)
        );
    }
    #[test]
    fn alarm_tokens_override_online() {
        for status in [
            "OB DISCHRG",
            "OL LB",
            "OL RB",
            "OL OVER",
            "FSD",
            "OFF",
            "BYPASS",
            "OL UNKNOWN",
            "OL DISCHRG",
        ] {
            assert!(
                UpsReading::parse(&format!("ups.status: {status}"))
                    .unwrap()
                    .label()
                    .1,
                "{status}"
            );
        }
    }
    #[test]
    fn missing_or_invalid_values_are_not_zero() {
        let reading = UpsReading::parse("ups.status: OL\nbattery.charge: NaN\nups.load: -1\ninput.voltage: invalid\nbattery.runtime: inf").unwrap();
        assert_eq!(
            reading.content(),
            "MAINS OK\nBattery: --\nRuntime: --\nLoad -- In --"
        );
        assert!(UpsReading::parse("battery.charge: 100").is_err());
        assert!(UpsReading::parse("ups.status: ").is_err());
    }
    #[test]
    fn stale_cache_is_unavailable_and_priority() {
        let reading = UpsReading::parse(ONLINE).unwrap();
        let mut state = Snapshot(Some((reading.clone(), Instant::now() - MAX_AGE)));
        assert!(state.priority());
        assert!(state.content().starts_with("UPS unavailable"));
        state.0 = Some((reading, Instant::now()));
        assert!(!state.priority());
        state.0 = None;
        assert!(state.priority());
    }
    #[test]
    fn subprocess_failure_and_timeout_are_bounded() {
        assert!(query("/bin/false", "ignored", Duration::from_secs(1)).is_err());
        let started = Instant::now();
        let error = query("/bin/sleep", "5", Duration::from_millis(100)).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert!(started.elapsed() < Duration::from_secs(1));
    }
}
