use std::fs;
use std::path::PathBuf;

/// Persisted app settings. Kept deliberately tiny; the config file is
/// hand-rolled TOML so we don't pull a TOML crate for two keys.
#[derive(Debug, Clone, Default)]
pub struct AppConfig {
    /// If set, converted files land here instead of next to the input.
    pub output_dir: Option<PathBuf>,
}

impl AppConfig {
    pub fn load() -> Self {
        Self::path()
            .and_then(|p| fs::read_to_string(p).ok())
            .and_then(|contents| Self::parse(&contents))
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let Some(path) = Self::path() else { return };
        if let Some(parent) = path.parent() {
            if let Err(err) = fs::create_dir_all(parent) {
                log::warn!("could not create config dir: {err}");
                return;
            }
        }
        if let Err(err) = fs::write(&path, self.to_toml()) {
            log::warn!("could not save config: {err}");
        }
    }

    fn path() -> Option<PathBuf> {
        let base = directories::BaseDirs::new()?;
        Some(
            base.config_dir()
                .join("renosancse-craft")
                .join("config.toml"),
        )
    }

    fn to_toml(&self) -> String {
        let mut out = String::new();
        if let Some(dir) = &self.output_dir {
            out.push_str(&format!("output_dir = \"{}\"\n", dir.to_string_lossy()));
        }
        out
    }

    fn parse(input: &str) -> Option<Self> {
        let mut config = Self::default();
        for line in input.lines() {
            let line = line.trim();
            if let Some(value) = line.strip_prefix("output_dir = ") {
                let value = value.trim().trim_matches('"');
                config.output_dir = Some(PathBuf::from(value));
            }
        }
        Some(config)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn round_trips_output_dir() {
        let config = AppConfig {
            output_dir: Some(PathBuf::from("/tmp/out")),
        };
        let parsed = AppConfig::parse(&config.to_toml()).unwrap();
        assert_eq!(parsed.output_dir, Some(PathBuf::from("/tmp/out")));
    }

    #[test]
    fn ignores_unknown_lines() {
        let parsed = AppConfig::parse("something = 1\noutput_dir = \"/x\"\n").unwrap();
        assert_eq!(parsed.output_dir.as_deref(), Some(Path::new("/x")));
    }
}
