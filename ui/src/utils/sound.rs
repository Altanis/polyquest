use std::collections::HashMap;

use gloo::{console::console, utils::{body, document, window}};
use shared::bool;
use wasm_bindgen::JsCast;
use web_sys::HtmlAudioElement;

#[derive(Default)]
pub struct SoundHolder {
    sounds: HashMap<&'static str, Sound>,
    pub can_play: bool
}

impl SoundHolder {
    pub fn new(sounds: Vec<(&'static str, bool)>) -> SoundHolder {
        let mut hm_sounds = HashMap::new();
        for (sound, r#loop) in sounds.iter() {
            hm_sounds.insert(*sound, Sound::new(sound, *r#loop));
        }

        SoundHolder {
            sounds: hm_sounds,
            can_play: false
        }
    }

    pub fn tick(&mut self) {
        let soundtrack_disabled = !bool!(storage_get!("soundtrack").unwrap_or("0".to_string()).as_str());
        let sfx_disabled = !bool!(storage_get!("sfx").unwrap_or("0".to_string()).as_str());

        for (_, sound) in self.sounds.iter_mut() {
            let soundtrack = sound.r#loop;
            if sound.is_playing() {
                sound.disable((soundtrack_disabled && soundtrack) || (sfx_disabled && !soundtrack));
            }
            
            if sound.file.ended() && soundtrack {
                sound.file.set_current_time(0.0);
                let _ = sound.file.play();
            }
        }
    }

    pub fn get_mut_sound(&mut self, sound: &str) -> &mut Sound {
        self.sounds.get_mut(sound).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sound {
    name: &'static str,
    file: HtmlAudioElement,
    r#loop: bool
}

impl Sound {
    pub fn new(name: &'static str, r#loop: bool) -> Sound {
        let file = if let Some(e) = document().get_element_by_id(name) {
            e.dyn_into::<HtmlAudioElement>().unwrap()
        } else {
            let file = document().create_element("audio")
                .unwrap()
                .dyn_into::<HtmlAudioElement>()
                .unwrap();

            file.set_id(name);
            file.set_src(&("sounds/".to_owned() + name + ".mp3"));
            
            let _ = file.dataset().set("playing", "0");
            let _ = file.dataset().set("stopping", "0");

            let soundtrack_disabled = !bool!(storage_get!("soundtrack").unwrap_or("0".to_string()).as_str());
            let sfx_disabled = !bool!(storage_get!("sfx").unwrap_or("0".to_string()).as_str());
            let disabled = (soundtrack_disabled && r#loop) || (sfx_disabled && !r#loop);

            let _ = file.dataset().set("disabled", if disabled { "1" } else { "0" });

            body().append_child(&file).expect("couldnt add audio");
            file
        };

        Sound { 
            name,
            file,
            r#loop
        }
    }

    pub fn has_not_started(&self) -> bool {
        self.file.current_time() == 0.0
    }

    pub fn is_playing(&self) -> bool {
        bool!(self.file.dataset().get("playing").unwrap().as_str())
    }

    pub fn play(&self) {
        let _ = self.file.dataset().set("playing", "1");
        let _ = self.file.play();
    }

    pub fn is_stopping(&self) -> bool {
        !self.file.ended() && bool!(self.file.dataset().get("stopping").unwrap().as_str())
    }

    pub fn stop(&self) {
        let _ = self.file.dataset().set("playing", "0");
        let _ = self.file.dataset().set("stopping", "0");
        let _ = self.file.pause();
    }

    pub fn restart(&self) {
        self.file.set_current_time(0.0);
    }

    pub fn is_disabled(&self) -> bool {
        bool!(self.file.dataset().get("disabled").unwrap().as_str())
    }

    pub fn disable(&self, value: bool) {
        let was_disabled = self.is_disabled();
        let _ = self.file.dataset().set("disabled", &(value as u8).to_string());
        
        match value {
            true => {
                let _ = self.file.pause();
                self.file.set_current_time(0.0);
            },
            false => {
                if was_disabled != value && self.is_playing() {
                    let _ = self.file.play();
                }
            }
        }
    }
}