use std::fs;
use std::path::PathBuf;
use ureq;
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters,
};

use crate::subtitles;
use crate::{get_folio_dir, FolioResult};

pub enum TranscriberModel {
    Tiny,
    Base,
    Small,
    Medium,
    Large,
}

pub struct Transcriber {
    ctx: WhisperContext,
}

impl Transcriber {
    pub fn load_model(model: TranscriberModel) -> FolioResult<PathBuf> {
        let link = match model {
            TranscriberModel::Tiny => {
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny-q5_1.bin?download=true"
            }
            TranscriberModel::Base => {
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base-q5_1.bin?download=true"
            }
            TranscriberModel::Small => {
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small-q5_1.bin?download=true"
            }
            TranscriberModel::Medium => {
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium-q5_0.bin?download=true"
            }
            TranscriberModel::Large => {
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-q5_0.bin?download=true"
            }
        };

        // Download the model
        // TOOD: Make this async? perhaps do this in the frontend
        let output = get_folio_dir()?.join(format!("{}.bin", model as u8));

        if !output.exists() {
            println!("Downloading model from {}.", link);
            let tmp = output.with_extension("tmp");
            let mut file = fs::File::create(&tmp)?;

            std::io::copy(
                &mut ureq::get(link).call()?.into_body().into_reader(),
                &mut file,
            )?;

            fs::rename(&tmp, &output)?;
        }

        Ok(output)
    }

    pub fn new(model: TranscriberModel) -> FolioResult<Self> {
        let model_path = Self::load_model(model)?;

        let ctx = WhisperContext::new_with_params(
            model_path.to_string_lossy().as_ref(),
            WhisperContextParameters::default(),
        )?;

        Ok(Self { ctx })
    }

    pub fn transcribe(
        &self,
        audio: &[f32],
    ) -> FolioResult<Box<[subtitles::Segment]>> {
        let mut state = self.ctx.create_state()?;

        let params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        state.full(params, audio)?;

        let result = state
            .as_iter()
            .map(|segment| subtitles::Segment {
                start: segment.start_timestamp(),
                end: segment.end_timestamp(),
                text: segment.to_string(),
            })
            .collect::<Box<[subtitles::Segment]>>();

        Ok(result)
    }
}
