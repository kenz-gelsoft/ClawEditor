#![cfg_attr(not(test), windows_subsystem = "windows")]

use std::path::Path;

use wx;

mod commands;

mod editor_ctrl;
mod editor_frame;
use editor_frame::EditorFrame;

mod observer;
mod unsaved_changes;

fn main() {
    wx::App::run(|_| {
        let frame = EditorFrame::new();
        let mut file_to_open = None;
        if let Some(file) = wx::App::args().nth(1) {
            if !Path::new(&file).exists() {
                println!("The file {} does not exist.", file);
                frame.close();
                return;
            }
            file_to_open = Some(file);
        }
        frame.show();
        if file_to_open.is_some() {
            frame.open_file(file_to_open.as_deref());
        }
    });
}
