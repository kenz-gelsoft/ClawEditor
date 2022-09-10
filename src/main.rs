#![windows_subsystem = "windows"]

use std::os::raw::c_int;
use wx;
use wx::methods::*;

#[derive(Clone, Copy)]
enum Command {
    FileNew = wx::ID_HIGHEST as isize,
    FileOpen,
    FileSave,
    FileSaveAs,
    FileClose,
}
impl Command {
    fn from(v: c_int) -> Option<Self> {
        use Command::*;
        for e in [FileNew, FileOpen, FileSave, FileSaveAs, FileClose] {
            if v == e.into() {
                return Some(e);
            }
        }
        return None;
    }
}
impl From<Command> for c_int {
    fn from(w: Command) -> Self {
        w as c_int
    }
}

type Frame = wx::FrameIsOwned<false>;

fn main() {
    wx::App::run(|_| {
        let frame = wx::Frame::builder(wx::Window::none())
            .title("カニツメエディタ")
            .build();
        build_menu(&frame);
        let _textbox = wx::TextCtrl::builder(Some(&frame))
            .style(wx::TE_MULTILINE.into())
            .build();
        let weak_frame = frame.to_weak_ref();
        frame.bind(wx::RustEvent::Menu, move |event: &wx::CommandEvent| {
            if let (Some(frame), Some(command)) = (weak_frame.get(), Command::from(event.get_id()))
            {
                handle_command(&frame, &command);
            }
        });
        frame.show(true);
    });
}

fn build_menu(frame: &wx::Frame) {
    let menu_bar = wx::MenuBar::new(0);
    let file_menu = wx::Menu::new()
        .item(Command::FileNew, "新規(&N)\tCtrl-N")
        .item(Command::FileOpen, "開く(&W)\tCtrl-O")
        .item(Command::FileSave, "保存(&S)\tCtrl-S")
        .item(Command::FileSaveAs, "名前を付けて保存(&A)\tCtrl-Shift-S")
        .separator()
        .item(Command::FileClose, "終了(&X)\tCtrl-W");

    menu_bar.append(Some(&file_menu), "ファイル(&F)");
    frame.set_menu_bar(Some(&menu_bar));
}

fn handle_command(frame: &Frame, command: &Command) {
    match command {
        Command::FileNew => todo!(),
        Command::FileOpen => {
            open_file(frame);
        }
        Command::FileSave => todo!(),
        Command::FileSaveAs => {
            save_as(frame);
        }
        Command::FileClose => {
            frame.close(false);
        }
    }
}

fn open_file(frame: &Frame) {
    // TODO: Add Builder for wx::FileDialog
    let file_dialog = wx::FileDialog::new(
        Some(frame),
        "",
        "",
        "",
        "*.*",
        wx::FC_DEFAULT_STYLE.into(),
        &wx::Point::default(),
        &wx::Size::default(),
        "",
    );
    if wx::ID_OK == file_dialog.show_modal() {
        // TODO: open
        let path = file_dialog.get_path();
        println!("open: {}", path);
    }
}

fn save_as(frame: &Frame) {
    // TODO: Add Builder for wx::FileDialog
    let file_dialog = wx::FileDialog::new(
        Some(frame),
        "",
        "",
        "",
        "*.*",
        wx::FC_SAVE.into(),
        &wx::Point::default(),
        &wx::Size::default(),
        "",
    );
    if wx::ID_OK == file_dialog.show_modal() {
        // TODO: open
        let path = file_dialog.get_path();
        println!("save as: {}", path);
    }
}
