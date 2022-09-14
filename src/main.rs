#![windows_subsystem = "windows"]

use wx;

mod commands;

mod editor_frame;
use editor_frame::EditorFrame;

fn main() {
    wx::App::run(|_| {
        let frame = EditorFrame::new();
        frame.show();
    });
}
