#![cfg_attr(not(test), windows_subsystem = "windows")]

use wx;

mod commands;

mod editor_ctrl;
mod editor_frame;
use editor_frame::EditorFrame;

mod observer;

fn main() {
    wx::App::run(|_| {
        let frame = EditorFrame::new();
        frame.borrow().show();
    });
}
