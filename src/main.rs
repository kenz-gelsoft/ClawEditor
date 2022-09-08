#![windows_subsystem = "windows"]

use wx;
use wx::methods::*;

fn main() {
    wx::App::run(|_| {
        let frame = wx::Frame::builder(wx::Window::none())
            .title("カニツメエディタ")
            .build();
        let button = wx::Button::builder(Some(&frame)).label("Greet").build();
        let weak_button = button.to_weak_ref();
        button.bind(wx::RustEvent::Button, move |_: &wx::CommandEvent| {
            if let Some(button) = weak_button.get() {
                button.set_label("clicked");
                println!("s={}", button.get_label())
            }
        });
        frame.show(true);
    });
}
