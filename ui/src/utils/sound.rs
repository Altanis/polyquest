use web_sys::HtmlAudioElement;

#[derive(Debug, Clone, PartialEq)]
pub struct Sound {
    name: &'static str,
    file: HtmlAudioElement,
    r#loop: bool
}

impl Sound {
    pub fn new(name: &'static str, r#loop: bool) -> Sound {
        let file = HtmlAudioElement::new_with_src(&("sounds/".to_owned() + name + ".mp3")).unwrap();
        file.set_autoplay(r#loop);

        Sound { 
            name, 
            file,
            r#loop
        }
    }

    pub fn is_playing(&self) -> bool {
        !self.file.paused() && self.file.current_time() > 0.0 && !self.file.ended()
    }

    pub fn play(&self) {
        let _ = self.file.play();
    }

    pub fn stop(&self) {
        let _ = self.file.pause();
    }
}