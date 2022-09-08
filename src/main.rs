#![windows_subsystem = "windows"]

use wx;
use wx::methods::*;

fn main() {
    wx::App::run(|_| {
        let frame = wx::Frame::builder(wx::Window::none())
            .title("カニツメエディタ")
            .build();
        let _textbox = wx::TextCtrl::builder(Some(&frame))
            .style(wx::TE_MULTILINE.into())
            .build();
        frame.show(true);
    });
}
