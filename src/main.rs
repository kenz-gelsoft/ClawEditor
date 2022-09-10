#![windows_subsystem = "windows"]

use std::os::raw::c_int;
use wx;
use wx::methods::*;

#[derive(Clone, Copy)]
enum Command {
    // ファイル
    FileNew = wx::ID_HIGHEST as isize,
    FileOpen,
    FileSave,
    FileSaveAs,
    FileClose,
    // 編集
    EditUndo,
    EditCut,
    EditCopy,
    EditPaste,
    EditDelete,
    EditFind,
    EditFindNext,
    EditFindPrevious,
    EditReplace,
    EditGo,
    EditSelectAll,
    EditDate,
    EditFont,
    // 表示
    ViewStatusBar,
    ViewWordWrap,
}
impl Command {
    fn from(v: c_int) -> Option<Self> {
        use Command::*;
        for e in [
            // ファイル
            FileNew,
            FileOpen,
            FileSave,
            FileSaveAs,
            FileClose,
            // 編集
            EditUndo,
            EditCut,
            EditCopy,
            EditPaste,
            EditDelete,
            EditFind,
            EditFindNext,
            EditFindPrevious,
            EditReplace,
            EditGo,
            EditSelectAll,
            EditDate,
            EditFont,
            // 表示
            ViewStatusBar,
            ViewWordWrap,
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

    let edit_menu = wx::Menu::new()
        .item(Command::EditUndo, "元に戻す(&U)\tCtrl-Z")
        .separator()
        .item(Command::EditCut, "切り取り(&T)\tCtrl-X")
        .item(Command::EditCopy, "コピー(&C)\tCtrl-C")
        .item(Command::EditPaste, "貼り付け(&P)\tCtrl-V")
        .item(Command::EditDelete, "削除(&L)\tDel")
        .separator()
        .item(Command::EditFind, "検索する(&F)\tCtrl-F")
        .item(Command::EditFindNext, "次を検索(&N)\tF3")
        .item(Command::EditFindPrevious, "前を検索(&V)\tShift-F3")
        .item(Command::EditReplace, "置換(&R)\tCtrl-H")
        .item(Command::EditGo, "移動(&G)\tCtrl-G")
        .separator()
        .item(Command::EditSelectAll, "すべて選択(&A)\tCtrl-A")
        .item(Command::EditDate, "日付と時刻(&D)\tF5")
        .separator()
        .item(Command::EditSelectAll, "フォント(&O)");
    menu_bar.append(Some(&edit_menu), "編集(&E)");

    let view_menu = wx::Menu::new()
        .check_item(Command::ViewStatusBar, "ステータスバー(&S)")
        .check_item(Command::ViewWordWrap, "右端での折り返し(&W)");
    menu_bar.append(Some(&view_menu), "表示(&V)");

    frame.set_menu_bar(Some(&menu_bar));
}

fn handle_command(frame: &Frame, command: &Command) {
    match command {
        // ファイル
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
        // 編集
        Command::EditUndo => todo!(),
        Command::EditCut => todo!(),
        Command::EditCopy => todo!(),
        Command::EditPaste => todo!(),
        Command::EditDelete => todo!(),
        Command::EditFind => todo!(),
        Command::EditFindNext => todo!(),
        Command::EditFindPrevious => todo!(),
        Command::EditReplace => todo!(),
        Command::EditGo => todo!(),
        Command::EditSelectAll => todo!(),
        Command::EditDate => todo!(),
        Command::EditFont => todo!(),
        // 表示
        Command::ViewStatusBar => todo!(),
        Command::ViewWordWrap => todo!(),
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
