use rodio::{
    mixer::Mixer,
    Sink,
    Source,
};



pub struct AudioPlayer;

impl AudioPlayer {
    pub fn play(
        mixer: &Mixer,
        sound: impl Source + Send + 'static,
    ) {
        let sink = Sink::connect_new(mixer);

        sink.append(sound.amplify(5.0));

        sink.detach();
    }
}