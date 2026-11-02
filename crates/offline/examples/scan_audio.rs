//! Throwaway: transcribes raw 16 kHz mono PCM with timestamps, to choose an excerpt. Not kept.

use kalamu_core::{
    ApiKey, LanguageChoice, PCM_16K_MONO, PieceRef, PreparedInput, ProcessRequest, Provider,
    SetupOptions, StopSignal, Task,
};
use kalamu_offline::{ModelManager, Whisper};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let (models, pcm) = (args.next().ok_or("models")?, args.next().ok_or("pcm")?);
    let whisper = Whisper::new(ModelManager::new(models.into()));
    let input = PreparedInput {
        piece: PieceRef::Segment {
            index: 0,
            start_ms: 0,
            end_ms: 720_000,
        },
        mime: PCM_16K_MONO.into(),
        bytes: std::fs::read(pcm)?,
        file_name: "scan.pcm".into(),
    };
    let options = SetupOptions {
        timestamps: true,
        ..SetupOptions::default()
    };
    let stop = StopSignal::default();
    let languages = LanguageChoice::Auto;
    let out = whisper
        .process(
            &ApiKey::new(""),
            ProcessRequest {
                task: Task::AudioTranscription,
                model: "whisper-large-v3-turbo",
                prompt: "",
                user_prompt: "",
                input: &input,
                languages: &languages,
                options: &options,
                analysis: None,
                stop: &stop,
            },
        )
        .await?;
    for s in &out.segments {
        println!("{:>4}s  {}", s.start_ms / 1000, s.text);
    }
    Ok(())
}
