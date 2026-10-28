//! The formats recording pieces are converted to, and the FFmpeg encoder settings for each.

/// The formats pieces are written in, all mono at 16 kHz.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    /// Raw samples for offline engines.
    Pcm16k,
    OggOpus,
    WebmOpus,
    Flac,
    Wav,
}

impl AudioFormat {
    pub fn mime(self) -> &'static str {
        match self {
            Self::Pcm16k => kalamu_core::PCM_16K_MONO,
            Self::OggOpus => "audio/ogg",
            Self::WebmOpus => "audio/webm",
            Self::Flac => "audio/flac",
            Self::Wav => "audio/wav",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Pcm16k => "pcm",
            Self::OggOpus => "ogg",
            Self::WebmOpus => "webm",
            Self::Flac => "flac",
            Self::Wav => "wav",
        }
    }

    pub(crate) fn is_opus(self) -> bool {
        matches!(self, Self::OggOpus | Self::WebmOpus)
    }

    pub(crate) fn encoder_args(self) -> Vec<&'static str> {
        // 32 kbit/s Opus tuned for speech: transparent for recognition, about 0.25 MB a minute.
        const OPUS: [&str; 6] = ["-c:a", "libopus", "-b:a", "32k", "-application", "voip"];
        match self {
            Self::Pcm16k => vec!["-c:a", "pcm_s16le", "-f", "s16le"],
            Self::OggOpus => [OPUS.as_slice(), &["-f", "ogg"]].concat(),
            Self::WebmOpus => [OPUS.as_slice(), &["-f", "webm"]].concat(),
            Self::Flac => vec!["-c:a", "flac", "-f", "flac"],
            Self::Wav => vec!["-c:a", "pcm_s16le", "-f", "wav"],
        }
    }

    /// The same, at the encoder's fastest setting: for listening, where waiting matters more
    /// than the last few per cent of quality.
    pub(crate) fn listening_args(self) -> Vec<&'static str> {
        let mut args = self.encoder_args();
        if self.is_opus() || self == Self::Flac {
            args.extend(["-compression_level", "0"]);
        }
        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn listening_is_the_fastest_setting_and_sending_is_not() {
        let fast = ["-compression_level", "0"];
        let has = |args: &[&str]| args.windows(2).any(|w| w == fast);
        assert!(has(&AudioFormat::OggOpus.listening_args()));
        assert!(has(&AudioFormat::Flac.listening_args()));
        assert!(!has(&AudioFormat::Wav.listening_args()));
        for format in [AudioFormat::OggOpus, AudioFormat::Flac, AudioFormat::Wav] {
            assert!(
                !has(&format.encoder_args()),
                "{format:?} sent to a provider"
            );
        }
    }
}
