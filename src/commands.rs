use std::os::raw::c_int;

use crate::editor_frame::EditorFrame;

#[derive(Clone, Copy)]
pub enum Command {
    // ファイル
    FileNew = wx::ID_HIGHEST as isize,
    FileNewWindow,
    FileOpen,
    FileSave,
    FileSaveAs,
    FileClose,
    // 編集
    // wx::ID_UNDO,
    // wx::ID_CUT,
    // wx::ID_COPY,
    // wx::ID_PASTE,
    EditDelete,
    EditFind,
    EditFindNext,
    EditFindPrevious,
    EditReplace,
    EditGo,
    // wx::ID_SELECTALL,
    EditDate,
    // 書式
    FormatWordWrap,
    FormatFont,
    // 表示
    ViewStatusBar,
    // ヘルプ
    Help,
    HelpAbout,
}
impl Command {
    pub fn from(v: c_int) -> Option<Self> {
        use Command::*;
        for e in [
            // ファイル
            FileNew,
            FileNewWindow,
            FileOpen,
            FileSave,
            FileSaveAs,
            FileClose,
            // 編集
            // wx::ID_UNDO,
            // wx::ID_CUT,
            // wx::ID_COPY,
            // wx::ID_PASTE,
            EditDelete,
            EditFind,
            EditFindNext,
            EditFindPrevious,
            EditReplace,
            EditGo,
            // wx::ID_SELECTALL,
            EditDate,
            // 書式
            FormatWordWrap,
            FormatFont,
            // 表示
            ViewStatusBar,
            // ヘルプ
            Help,
            HelpAbout,
        ] {
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

pub fn handle_command(frame: &EditorFrame, command: &Command) {
    match command {
        // ファイル
        Command::FileNew => {
            frame.new_file();
        }
        Command::FileNewWindow => todo!(),
        Command::FileOpen => {
            frame.open_file();
        }
        Command::FileSave => {
            _ = frame.save();
        }
        Command::FileSaveAs => {
            _ = frame.save_as();
        }
        Command::FileClose => {
            frame.close();
        }
        // 編集
        Command::EditDelete => {
            frame.delete_selection();
        }
        Command::EditFind => todo!(),
        Command::EditFindNext => todo!(),
        Command::EditFindPrevious => todo!(),
        Command::EditReplace => todo!(),
        Command::EditGo => todo!(),
        Command::EditDate => todo!(),
        // 書式
        Command::FormatWordWrap => todo!(),
        Command::FormatFont => todo!(),
        // 表示
        Command::ViewStatusBar => todo!(),
        // 書式
        Command::Help => {
            frame.open_help();
        }
        Command::HelpAbout => {
            frame.show_about();
        }
    }
}
