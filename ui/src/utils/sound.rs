use std::{cell::RefCell, rc::Rc};

use gloo::{console::console, utils::{body, document, window}};
use gloo_timers::callback::Interval;
use shared::fuzzy_compare;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::HtmlAudioElement;

#[derive(Debug, Clone, PartialEq)]
pub struct Sound {
    name: &'static str,
    file: HtmlAudioElement,
    playing: bool,
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

            body().append_child(&file).expect("couldnt add audio");

            file
        };

        file.set_autoplay(r#loop);

        Sound { 
            name, 
            file,
            playing: false,
            r#loop
        }
    }

    pub fn has_not_started(&self) -> bool {
        self.file.current_time() == 0.0
    }

    pub fn is_playing(&self) -> bool {
        self.playing && !self.file.paused() && self.file.current_time() > 0.0 && !self.file.ended()
    }

    pub fn play(&mut self) {
        self.playing = true;
        let _ = self.file.play();
    }

    pub fn stop(&mut self, fade_rate: f64) {
        self.playing = false;

        if fade_rate != 0.0 {
            let id = self.name;
            let interval_handle: Rc<RefCell<Option<Interval>>> = Rc::new(RefCell::new(None));

            let interval = Interval::new(150, {
                let interval_handle = interval_handle.clone();
                move || {
                    let file = document().get_element_by_id(id)
                        .unwrap()
                        .dyn_into::<HtmlAudioElement>()
                        .unwrap();
                    
                    let volume = file.volume();
                    if fuzzy_compare!(volume, 0.0, 1e-1) {
                        if let Some(interval) = interval_handle.borrow_mut().take() {
                            console!("paused".to_string());

                            let _ = file.pause();
                            file.set_volume(1.0);
    
                            interval.cancel();
                        }
                    } else {
                        file.set_volume((volume - fade_rate).max(0.0));
                    }
                }
            });
    
            *interval_handle.borrow_mut() = Some(interval);
        } else {
            let _ = self.file.pause();
        }
    }
}