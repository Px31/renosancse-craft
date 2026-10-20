use crate::audio::detect::AudioFormat;
use crate::audio::probe::ProbeInfo;

/// A target format decision with the exact codec arguments to realize it.
#[derive(Debug, Clone)]
pub struct Recommendation {
    pub output_ext: &'static str,
    pub codec_args: Vec<&'static str>,
    pub reason: String,
}

/// Plain-Rust rules engine:
/// - lossless sources → keep lossless, archive as FLAC
/// - lossy sources → MP3 320 kbps for universal playback
/// - M4A/AAC keeps the source bit rate in the reason text for context
pub fn recommend(format: AudioFormat, probed: Option<&ProbeInfo>) -> Recommendation {
    if format.is_lossless() {
        return Recommendation {
            output_ext: "flac",
            codec_args: vec!["-c:a", "flac"],
            reason: "Lossless source — archive as FLAC".to_string(),
        };
    }

    let reason = match format {
        AudioFormat::M4a | AudioFormat::Aac => match probed.and_then(|p| p.bit_rate_kbps) {
            Some(kbps) => format!(
                "M4A/AAC source at {kbps} kbps — re-encode to MP3 320 kbps for universal playback"
            ),
            None => "M4A/AAC source — re-encode to MP3 320 kbps for universal playback".to_string(),
        },
        _ => "Lossy source — re-encode to MP3 320 kbps for universal playback".to_string(),
    };

    Recommendation {
        output_ext: "mp3",
        codec_args: vec!["-c:a", "libmp3lame", "-b:a", "320k"],
        reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lossless_becomes_flac() {
        let rec = recommend(AudioFormat::Wav, None);
        assert_eq!(rec.output_ext, "flac");
        assert_eq!(rec.codec_args, vec!["-c:a", "flac"]);
    }

    #[test]
    fn flac_stays_flac() {
        let rec = recommend(AudioFormat::Flac, None);
        assert_eq!(rec.output_ext, "flac");
    }

    #[test]
    fn lossy_becomes_mp3_320() {
        let rec = recommend(AudioFormat::Ogg, None);
        assert_eq!(rec.output_ext, "mp3");
        assert!(rec.codec_args.contains(&"libmp3lame"));
        assert!(rec.codec_args.contains(&"320k"));
    }

    #[test]
    fn m4a_reason_mentions_source_bitrate() {
        let probed = ProbeInfo {
            container: "mp4".to_string(),
            duration_secs: None,
            bit_rate_kbps: Some(256),
        };
        let rec = recommend(AudioFormat::M4a, Some(&probed));
        assert!(rec.reason.contains("256 kbps"));
    }
}
