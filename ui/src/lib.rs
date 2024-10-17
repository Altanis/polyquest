use canvas2d::Canvas2d;
pub trait UiElement {
    fn setup(&mut self);
    
    fn on_hover(&self);
    fn on_click(&self);

    fn render(&self, context: &mut Canvas2d);
}

macro_rules! impl_builder {
    ($Struct:ident { $($field:ident : $type:ty),* $(,)? }) => {
        impl $Struct {
            pub fn new() -> $Struct {
                $Struct::default()
            }

            $(
                pub fn $field(mut self, value: $type) -> Self {
                    self.$field = value;
                    self
                }
            )*
        }
    };
}

pub mod color;
pub mod canvas2d;
pub mod label;