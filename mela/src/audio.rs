use crate::components::Transform;
use crate::debug::{DebugContext, DebugDrawable};
use crate::imgui::im_str;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source, SpatialSink};
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::mem::{replace, swap};
use std::time::Duration;

#[crate::ecs::system(for_each)]
pub fn audio(
    mut audio: &mut Audio,
    transform: &Transform,
    #[resource] output: &OutputStreamHandle,
) {
    if audio.sink.is_none() {
        let file = match File::open(&audio.source) {
            Ok(file) => file,
            Err(_) => return,
        };

        let sink = SpatialSink::try_new(
            &output,
            transform.0.translation.into(),
            [1., 0., 0.],
            [-1., 0., 0.],
        )
        .unwrap();
        sink.append(
            Decoder::new(BufReader::new(file))
                .unwrap()
                .convert_samples::<f32>(),
        );
        audio.sink = Some(sink);
    } else {
        let mut sink = &audio.sink.as_ref().unwrap();
        sink.set_emitter_position(transform.0.translation.into());
        match audio.state {
            AudioState::Play => sink.play(),
            AudioState::Pause => sink.pause(),
        }
    }
}

pub struct Audio {
    pub source: String,
    pub sink: Option<SpatialSink>,
    pub state: AudioState,
}

pub enum AudioState {
    Play,
    Pause,
}

impl Default for Audio {
    fn default() -> Self {
        Audio {
            source: Default::default(),
            sink: None,
            state: AudioState::Play,
        }
    }
}

impl Clone for Audio {
    fn clone(&self) -> Self {
        Audio {
            source: self.source.clone(),
            sink: None,
            state: AudioState::Play,
        }
    }
}

impl Audio {
    pub fn new<S: Into<String>>(string: S) -> Audio {
        Audio {
            source: string.into(),
            sink: None,
            state: AudioState::Play,
        }
    }
}

impl DebugDrawable for Audio {
    fn draw_debug_ui(&mut self, debug_ctx: &DebugContext) {
        let mut file = self.source.clone().into();

        if debug_ctx.ui.input_text(im_str!("File"), &mut file).build() {
            self.source = file.to_string();
            self.sink = None;
        }
        match self.state {
            AudioState::Pause => {
                if debug_ctx.ui.small_button(im_str!("Play")) {
                    self.state = AudioState::Play;
                }
            }
            AudioState::Play => {
                if debug_ctx.ui.small_button(im_str!("Pause")) {
                    self.state = AudioState::Pause;
                }
            }
        }
    }
}
