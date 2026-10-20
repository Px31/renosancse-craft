use std::path::PathBuf;
use std::process::Command;

/// Everything needed to run one ffmpeg conversion.
pub struct ConvertJob {
    pub input: PathBuf,
    pub output: PathBuf,
    pub codec_args: Vec<&'static str>,
}

/// Build the ffmpeg argument list. Metadata is always carried over.
pub fn build_arguments(job: &ConvertJob) -> Vec<String> {
    let mut args: Vec<String> = vec![
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
        "-y".into(),
        "-i".into(),
        job.input.to_string_lossy().into_owned(),
    ];
    for arg in &job.codec_args {
        args.push(String::from(*arg));
    }
    args.extend(["-map_metadata".into(), "0".into()]);
    args.push(job.output.to_string_lossy().into_owned());
    args
}

/// Run a conversion synchronously on the calling thread (v0.1; the async
/// pipeline is on the roadmap). Returns the output path on success.
pub fn convert(job: &ConvertJob) -> anyhow::Result<PathBuf> {
    if let Some(parent) = job.output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let args = build_arguments(job);
    let output = Command::new("ffmpeg").args(&args).output()?;
    if !output.status.success() {
        anyhow::bail!(
            "ffmpeg failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(job.output.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job() -> ConvertJob {
        ConvertJob {
            input: PathBuf::from("in.wav"),
            output: PathBuf::from("out.flac"),
            codec_args: vec!["-c:a", "flac"],
        }
    }

    #[test]
    fn command_preserves_metadata() {
        let args = build_arguments(&job());
        assert!(args
            .windows(2)
            .any(|w| w[0] == "-map_metadata" && w[1] == "0"));
    }

    #[test]
    fn command_orders_input_then_codec_then_output() {
        let args = build_arguments(&job());
        let in_pos = args.iter().position(|a| a == "in.wav").unwrap();
        let codec_pos = args.iter().position(|a| a == "flac").unwrap();
        let out_pos = args.iter().position(|a| a == "out.flac").unwrap();
        assert!(in_pos < codec_pos);
        assert!(codec_pos < out_pos);
    }

    #[test]
    fn command_starts_with_input_flag() {
        let args = build_arguments(&job());
        assert!(args.iter().position(|a| a == "-i").is_some());
    }
}
