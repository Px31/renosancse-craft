use std::path::Path;
use std::process::Command;

use serde::Deserialize;

/// What `ffprobe` told us about a file.
#[derive(Debug, Clone)]
pub struct ProbeInfo {
    pub container: String,
    pub duration_secs: Option<f64>,
    pub bit_rate_kbps: Option<u64>,
}

impl ProbeInfo {
    /// One-line human summary for the queue rows.
    pub fn summary(&self) -> String {
        let mut out = self.container.clone();
        if let Some(secs) = self.duration_secs {
            out.push_str(&format!(" · {secs:.1}s"));
        }
        if let Some(kbps) = self.bit_rate_kbps {
            out.push_str(&format!(" · {kbps} kbps"));
        }
        out
    }
}

#[derive(Debug, Deserialize)]
struct RawProbe {
    #[serde(default)]
    format: RawFormat,
}

#[derive(Debug, Default, Deserialize)]
struct RawFormat {
    #[serde(default)]
    format_name: String,
    #[serde(default, deserialize_with = "opt_number")]
    duration: Option<f64>,
    #[serde(default, deserialize_with = "opt_number")]
    bit_rate: Option<f64>,
}

/// ffprobe emits numeric fields like `duration` and `bit_rate` as JSON
/// strings, so accept either representation.
fn opt_number<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<f64>, D::Error> {
    #[derive(Debug, serde::Deserialize)]
    #[serde(untagged)]
    enum N {
        N(f64),
        S(String),
    }

    Ok(Option::<N>::deserialize(d)?.map(|n| match n {
        N::N(v) => v,
        N::S(s) => s.trim().parse().unwrap_or(0.0),
    }))
}

/// Run `ffprobe` on a file. Fails (not panics) when ffprobe is missing or
/// chokes — callers degrade gracefully to magic-byte detection.
pub fn probe(path: &Path) -> anyhow::Result<ProbeInfo> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=format_name,duration,bit_rate",
            "-of",
            "json",
        ])
        .arg(path)
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "ffprobe failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let raw: RawProbe = serde_json::from_slice(&output.stdout)?;
    Ok(ProbeInfo {
        container: raw.format.format_name,
        duration_secs: raw.format.duration,
        bit_rate_kbps: raw.format.bit_rate.map(|b| (b / 1_000.0) as u64),
    })
}

/// First line of `ffmpeg -version`, for the About dialog and `--about`.
pub fn ffmpeg_version() -> Option<String> {
    let output = Command::new("ffmpeg").arg("-version").output().ok()?;
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .map(|line| line.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_probe_json() {
        let json = r#"{
            "format": {
                "format_name": "mp3",
                "duration": "12.5",
                "bit_rate": "320000"
            }
        }"#;
        let raw: RawProbe = serde_json::from_str(json).unwrap();
        let info = ProbeInfo {
            container: raw.format.format_name,
            duration_secs: raw.format.duration,
            bit_rate_kbps: raw.format.bit_rate.map(|b| (b / 1_000.0) as u64),
        };
        assert_eq!(info.container, "mp3");
        assert_eq!(info.duration_secs, Some(12.5));
        assert_eq!(info.bit_rate_kbps, Some(320));
    }
}
