use std::process::Stdio;
use std::time::Duration;

use tokio::process::Command;
use tracing::debug;

/// Result of an ICMP (or system-ping) batch.
#[derive(Debug, Clone, PartialEq)]
pub struct IcmpResult {
    pub latency_ms: Option<f32>,
    pub packet_loss_pct: f32,
    pub sent: u32,
    pub received: u32,
}

/// Run system `ping` against `host` for `count` echoes.
pub async fn ping_host(host: &str, count: u32, timeout: Duration) -> Option<IcmpResult> {
    let count = count.clamp(1, 10);
    let output = ping_command(host, count, timeout)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");

    let parsed = parse_ping_output(&combined);
    if parsed.is_none() {
        debug!(host, %combined, "failed to parse ping output");
    }
    parsed
}

fn ping_command(host: &str, count: u32, timeout: Duration) -> Command {
    let mut cmd = Command::new("ping");

    #[cfg(windows)]
    {
        let timeout_ms = timeout.as_millis().clamp(200, 10_000) as u64;
        cmd.args([
            "-n",
            &count.to_string(),
            "-w",
            &timeout_ms.to_string(),
            host,
        ]);
    }

    #[cfg(not(windows))]
    {
        let timeout_secs = timeout.as_secs().clamp(1, 10);
        cmd.args([
            "-c",
            &count.to_string(),
            "-W",
            &timeout_secs.to_string(),
            host,
        ]);
    }

    cmd
}

/// Parse Windows or Unix `ping` stdout into latency + loss.
pub fn parse_ping_output(output: &str) -> Option<IcmpResult> {
    if let Some(result) = parse_windows_ping(output) {
        return Some(result);
    }
    parse_unix_ping(output)
}

fn parse_windows_ping(output: &str) -> Option<IcmpResult> {
    // Packets: Sent = 4, Received = 4, Lost = 0 (0% loss),
    let sent = extract_after_label(output, "Sent = ")?;
    let received = extract_after_label(output, "Received = ")?;
    let loss_pct = extract_percent_loss_windows(output)?;

    let latency_ms = extract_after_label(output, "Average = ")
        .map(|v| v as f32)
        .or_else(|| average_reply_times(output));

    Some(IcmpResult {
        latency_ms,
        packet_loss_pct: loss_pct,
        sent,
        received,
    })
}

fn parse_unix_ping(output: &str) -> Option<IcmpResult> {
    // 4 packets transmitted, 4 received, 0% packet loss
    let transmitted_idx = output.to_ascii_lowercase().find("packets transmitted")?;
    let line_start = output[..transmitted_idx]
        .rfind('\n')
        .map(|i| i + 1)
        .unwrap_or(0);
    let line = output[line_start..]
        .lines()
        .next()
        .unwrap_or("")
        .trim();

    let sent = first_number(line)?;
    let received = line
        .split(',')
        .nth(1)
        .and_then(first_number)?;
    let loss_pct = line
        .split(',')
        .find(|part| part.to_ascii_lowercase().contains("packet loss"))
        .and_then(first_float)?;

    // rtt min/avg/max/mdev = 12.345/14.123/16.789/1.234 ms
    let latency_ms = output
        .lines()
        .find(|l| {
            let lower = l.to_ascii_lowercase();
            lower.contains("rtt") || lower.contains("round-trip")
        })
        .and_then(|l| {
            let after_eq = l.split('=').nth(1)?;
            let avg = after_eq.trim().split('/').nth(1)?;
            avg.trim()
                .split_whitespace()
                .next()
                .and_then(|s| s.parse().ok())
        });

    Some(IcmpResult {
        latency_ms,
        packet_loss_pct: loss_pct,
        sent,
        received,
    })
}

fn extract_after_label(output: &str, label: &str) -> Option<u32> {
    let idx = output.find(label)?;
    first_number(&output[idx + label.len()..])
}

fn extract_percent_loss_windows(output: &str) -> Option<f32> {
    let idx = output.find('%')?;
    let before = &output[..idx];
    let start = before.rfind('(')? + 1;
    before[start..].trim().parse().ok()
}

fn first_number(s: &str) -> Option<u32> {
    let digits: String = s
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

fn first_float(s: &str) -> Option<f32> {
    let mut buf = String::new();
    let mut started = false;
    for c in s.chars() {
        if c.is_ascii_digit() || (c == '.' && started) {
            buf.push(c);
            started = true;
        } else if started {
            break;
        }
    }
    if buf.is_empty() {
        None
    } else {
        buf.parse().ok()
    }
}

fn average_reply_times(output: &str) -> Option<f32> {
    let mut sum = 0.0f32;
    let mut n = 0u32;
    for line in output.lines() {
        let lower = line.to_ascii_lowercase();
        if !lower.contains("time") {
            continue;
        }
        // time=14ms or time<1ms
        if let Some(idx) = lower.find("time=") {
            if let Some(ms) = first_number(&line[idx + 5..]) {
                sum += ms as f32;
                n += 1;
            }
        } else if let Some(idx) = lower.find("time<") {
            if let Some(ms) = first_number(&line[idx + 5..]) {
                sum += ms as f32;
                n += 1;
            }
        }
    }
    if n == 0 {
        None
    } else {
        Some(sum / n as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_windows_ping() {
        let output = r#"
Pinging 8.8.8.8 with 32 bytes of data:
Reply from 8.8.8.8: bytes=32 time=14ms TTL=117
Reply from 8.8.8.8: bytes=32 time=16ms TTL=117

Ping statistics for 8.8.8.8:
    Packets: Sent = 4, Received = 4, Lost = 0 (0% loss),
Approximate round trip times in milli-seconds:
    Minimum = 13ms, Maximum = 16ms, Average = 14ms
"#;
        let result = parse_ping_output(output).expect("parse");
        assert_eq!(result.sent, 4);
        assert_eq!(result.received, 4);
        assert_eq!(result.packet_loss_pct, 0.0);
        assert_eq!(result.latency_ms, Some(14.0));
    }

    #[test]
    fn parses_windows_ping_with_loss() {
        let output = r#"
Packets: Sent = 4, Received = 2, Lost = 2 (50% loss),
Approximate round trip times in milli-seconds:
    Minimum = 20ms, Maximum = 40ms, Average = 30ms
"#;
        let result = parse_ping_output(output).expect("parse");
        assert_eq!(result.packet_loss_pct, 50.0);
        assert_eq!(result.latency_ms, Some(30.0));
    }

    #[test]
    fn parses_linux_ping() {
        let output = r#"
PING 8.8.8.8 (8.8.8.8) 56(84) bytes of data.
--- 8.8.8.8 ping statistics ---
4 packets transmitted, 4 received, 0% packet loss, time 3005ms
rtt min/avg/max/mdev = 12.345/14.123/16.789/1.234 ms
"#;
        let result = parse_ping_output(output).expect("parse");
        assert_eq!(result.sent, 4);
        assert_eq!(result.received, 4);
        assert_eq!(result.packet_loss_pct, 0.0);
        assert!((result.latency_ms.unwrap() - 14.123).abs() < 0.001);
    }

    #[test]
    fn parses_unix_fractional_loss() {
        let output = r#"
4 packets transmitted, 3 received, 25.0% packet loss, time 3003ms
rtt min/avg/max/mdev = 10.0/20.0/30.0/5.0 ms
"#;
        let result = parse_ping_output(output).expect("parse");
        assert_eq!(result.packet_loss_pct, 25.0);
        assert_eq!(result.latency_ms, Some(20.0));
    }
}
