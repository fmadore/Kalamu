// The projects named in Settings → About, where the app ships their code or models. Addresses,
// not text, so they stay out of the translation files; the names shown are in `creditNames`.

export const CREDIT_URLS: Record<string, string> = {
	ffmpeg: 'https://ffmpeg.org/',
	// LGPL: FFmpeg's corresponding source is attached to every release.
	source: 'https://github.com/fmadore/Kalamu/releases',
	opus: 'https://opus-codec.org/',
	whisper: 'https://github.com/ggml-org/whisper.cpp',
	models: 'https://github.com/openai/whisper'
};
