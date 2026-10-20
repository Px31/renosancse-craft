use std::fs::File;
use std::io::Read;
use std::path::Path;

/// An audio format we know how to talk about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    Mp3,
    M4a,
    Wav,
    Flac,
    Ogg,
    Aac,
    Aiff,
    Wma,
    Ac3,
}

impl AudioFormat {
    pub fn label(self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "MP3",
            AudioFormat::M4a => "M4A/AAC",
            AudioFormat::Wav => "WAV",
            AudioFormat::Flac => "FLAC",
            AudioFormat::Ogg => "OGG (Vorbis/Opus)",
            AudioFormat::Aac => "AAC (ADTS)",
            AudioFormat::Aiff => "AIFF",
            AudioFormat::Wma => "WMA",
            AudioFormat::Ac3 => "AC-3",
        }
    }

    pub fn is_lossless(self) -> bool {
        matches!(
            self,
            AudioFormat::Wav | AudioFormat::Flac | AudioFormat::Aiff
        )
    }
}

/// Result of sniffing a file.
#[derive(Debug, Clone)]
pub struct Detection {
    pub format: AudioFormat,
    pub by_extension_only: bool,
}

impl Detection {
    pub fn summary(&self) -> String {
        if self.by_extension_only {
            format!("{} (by extension)", self.format.label())
        } else {
            self.format.label().to_string()
        }
    }
}

/// ASF / WMA container GUID.
const ASF_GUID: [u8; 16] = [
    0x30, 0x26, 0xB2, 0x75, 0x8E, 0x66, 0xCF, 0x11, 0xA6, 0xD9, 0x00, 0xAA, 0x00, 0x62, 0xCE, 0x6C,
];

/// Classify a byte signature. Order matters: ADTS before generic MPEG sync,
/// container headers before their payloads.
pub fn classify(bytes: &[u8]) -> Option<AudioFormat> {
    if bytes.starts_with(b"OggS") {
        return Some(AudioFormat::Ogg);
    }
    if bytes.starts_with(b"fLaC") {
        return Some(AudioFormat::Flac);
    }
    if bytes.len() >= 12 {
        if &bytes[0..4] == b"RIFF" && (&bytes[8..12] == b"WAVE" || &bytes[8..12] == b"RF64") {
            return Some(AudioFormat::Wav);
        }
        if &bytes[0..4] == b"FORM" && (&bytes[8..12] == b"AIFF" || &bytes[8..12] == b"AIFC") {
            return Some(AudioFormat::Aiff);
        }
    }
    if bytes.len() >= 16 {
        if &bytes[4..8] == b"ftyp" {
            return Some(AudioFormat::M4a);
        }
        if bytes[0..16] == ASF_GUID {
            return Some(AudioFormat::Wma);
        }
    }
    // AC-3 sync: 'Es' + CRC bytes 0x40 0x08.
    if bytes.len() >= 4 && &bytes[0..2] == b"Es" && bytes[2] == 0x40 && bytes[3] == 0x08 {
        return Some(AudioFormat::Ac3);
    }
    // ADTS AAC frame header (before the generic MPEG sync, which it also matches).
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] & 0xF6 == 0xF0 {
        return Some(AudioFormat::Aac);
    }
    // MPEG-1/2 frame sync.
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] & 0xE0 == 0xE0 {
        return Some(AudioFormat::Mp3);
    }
    if bytes.starts_with(b"ID3") {
        return Some(AudioFormat::Mp3);
    }
    None
}

fn from_mime(mime: &str) -> Option<AudioFormat> {
    Some(match mime {
        "audio/mpeg" | "audio/mp3" => AudioFormat::Mp3,
        "audio/mp4" | "audio/x-m4a" | "audio/aac" => AudioFormat::M4a,
        "audio/wav" | "audio/x-wav" => AudioFormat::Wav,
        "audio/flac" => AudioFormat::Flac,
        "audio/ogg" => AudioFormat::Ogg,
        "audio/x-aiff" | "audio/aiff" => AudioFormat::Aiff,
        "audio/x-ms-wma" => AudioFormat::Wma,
        "audio/ac-3" => AudioFormat::Ac3,
        _ => return None,
    })
}

fn from_extension(path: &Path) -> Option<AudioFormat> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    Some(match ext.as_str() {
        "mp3" | "mp2" | "mpga" | "mpeg" => AudioFormat::Mp3,
        "m4a" | "m4b" | "m4p" => AudioFormat::M4a,
        "wav" | "wave" => AudioFormat::Wav,
        "flac" => AudioFormat::Flac,
        "ogg" | "oga" | "opus" => AudioFormat::Ogg,
        "aac" => AudioFormat::Aac,
        "aif" | "aiff" => AudioFormat::Aiff,
        "wma" => AudioFormat::Wma,
        "ac3" | "eac3" => AudioFormat::Ac3,
        _ => return None,
    })
}

/// Detect what a file actually is: magic bytes first, MIME sniffing second,
/// extension as a last resort (flagged as such).
pub fn detect(path: &Path) -> anyhow::Result<Detection> {
    let mut file = File::open(path)?;
    let mut head = [0u8; 32];
    let n = file.read(&mut head)?;
    let head = &head[..n];

    if let Some(format) = classify(head) {
        return Ok(Detection {
            format,
            by_extension_only: false,
        });
    }
    let mime = infer::get(head).map(|kind| kind.to_string());
    if let Some(mime) = mime {
        if let Some(format) = from_mime(&mime) {
            return Ok(Detection {
                format,
                by_extension_only: false,
            });
        }
    }
    let format = from_extension(path).ok_or_else(|| {
        anyhow::anyhow!(
            "unrecognized audio format (tried magic bytes, MIME sniffing, and the extension)"
        )
    })?;
    Ok(Detection {
        format,
        by_extension_only: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_ogg() {
        let mut b = [0u8; 8];
        b[..4].copy_from_slice(b"OggS");
        assert_eq!(classify(&b), Some(AudioFormat::Ogg));
    }

    #[test]
    fn detects_flac() {
        assert_eq!(classify(b"fLaCxxxxxxxx"), Some(AudioFormat::Flac));
    }

    #[test]
    fn detects_wav() {
        let mut b = [0u8; 12];
        b[..4].copy_from_slice(b"RIFF");
        b[8..].copy_from_slice(b"WAVE");
        assert_eq!(classify(&b), Some(AudioFormat::Wav));
    }

    #[test]
    fn detects_aiff() {
        let mut b = [0u8; 12];
        b[..4].copy_from_slice(b"FORM");
        b[8..].copy_from_slice(b"AIFF");
        assert_eq!(classify(&b), Some(AudioFormat::Aiff));
    }

    #[test]
    fn detects_m4a() {
        let mut b = [0u8; 16];
        b[4..8].copy_from_slice(b"ftyp");
        b[8..12].copy_from_slice(b"M4A ");
        assert_eq!(classify(&b), Some(AudioFormat::M4a));
    }

    #[test]
    fn detects_wma_guid() {
        let b = ASF_GUID;
        assert_eq!(classify(&b), Some(AudioFormat::Wma));
    }

    #[test]
    fn detects_ac3() {
        assert_eq!(classify(&[0x45, 0x73, 0x40, 0x08]), Some(AudioFormat::Ac3));
    }

    #[test]
    fn detects_adts_aac() {
        assert_eq!(classify(&[0xFF, 0xF1, 0x90, 0x00]), Some(AudioFormat::Aac));
    }

    #[test]
    fn detects_mp3_frame_sync() {
        assert_eq!(classify(&[0xFF, 0xFB, 0x90, 0x00]), Some(AudioFormat::Mp3));
    }

    #[test]
    fn detects_mp3_id3_tag() {
        assert_eq!(
            classify(&[0x49, 0x44, 0x33, 0x04, 0x10]),
            Some(AudioFormat::Mp3)
        );
    }

    #[test]
    fn unknown_bytes() {
        assert_eq!(classify(&[1, 2, 3]), None);
    }

    #[test]
    fn extension_fallback_maps_common_suffixes() {
        assert_eq!(
            from_extension(Path::new("song.MP3")),
            Some(AudioFormat::Mp3)
        );
        assert_eq!(
            from_extension(Path::new("song.flac")),
            Some(AudioFormat::Flac)
        );
        assert_eq!(from_extension(Path::new("song.xyz")), None);
    }

    #[test]
    fn lossless_set() {
        assert!(AudioFormat::Wav.is_lossless());
        assert!(AudioFormat::Flac.is_lossless());
        assert!(!AudioFormat::Mp3.is_lossless());
    }
}
