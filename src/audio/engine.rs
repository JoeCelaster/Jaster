use rodio::{OutputStream, OutputStreamBuilder};

pub struct AudioEngine {
    stream: OutputStream,
}

impl AudioEngine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let stream = OutputStreamBuilder::open_default_stream()?;

        Ok(Self { stream })
    }

    pub fn mixer(&self) -> &rodio::mixer::Mixer {
        self.stream.mixer()
    }
}