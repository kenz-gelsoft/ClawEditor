use std::os::raw::c_int;

use wx;

pub trait CommandHandler<C> {
    fn handle_command(&self, command: &C);
}

pub enum EditorCommand<'a> {
    Command(Command),
    StandardEvents(&'a wx::CommandEvent),
}

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
    // wx::ID_ABOUT,
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
            // wx::ID_ABOUT,
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
