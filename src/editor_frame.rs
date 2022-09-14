use std::cell::RefCell;
use std::os::raw::{c_int, c_long, c_void};
use std::rc::Rc;

use wx;
use wx::methods::*;

use crate::commands::{self, Command};

const APP_NAME: &str = "カニツメエディタ";
const UNTITLED: &str = "無題";

#[cfg(windows)]
const CW_USEDEFAULT: c_int = c_int::MIN;

#[derive(Clone)]
pub struct EditorFrame {
    base: wx::Frame,
    textbox: wx::TextCtrl,
    // TODO: avoid interior mutability
    file: Rc<RefCell<Option<String>>>,
}
impl EditorFrame {
    pub fn new() -> Self {
        let default_size = if cfg!(windows) {
            // XXX: Windows プログラムとして自然なデフォルトサイズにするため、
            // CW_USEDEFAULT を指定しています。
            // wxMSW が CreateWindow() に size を渡すことに依存しています。
            wx::Size::new_with_int(CW_USEDEFAULT, CW_USEDEFAULT)
        } else {
            wx::Size::default()
        };
        let frame = wx::Frame::builder(wx::Window::none())
            .size(default_size)
            .build();
        let textbox = wx::TextCtrl::builder(Some(&frame))
            .style(wx::TE_MULTILINE.into())
            .build();
        let frame = EditorFrame {
            base: frame,
            textbox,
            file: Rc::new(RefCell::new(None)),
        };
        let frame_copy = frame.clone();
        frame
            .base
            .bind(wx::RustEvent::Menu, move |event: &wx::CommandEvent| {
                if let Some(command) = Command::from(event.get_id()) {
                    commands::handle_command(&frame_copy, &command);
                } else {
                    frame_copy.textbox.process_event(event);
                }
            });

        let frame_copy = frame.clone();
        frame
            .base
            .bind(wx::RustEvent::Text, move |_: &wx::CommandEvent| {
                frame_copy.on_text_modified();
            });
        frame.build_menu();
        frame.update_title();

        frame
    }

    pub fn show(&self) {
        self.base.show(true);
    }

    fn build_menu(&self) {
        let menu_bar = wx::MenuBar::new(0);

        let file_menu = wx::Menu::new()
            .item(Command::FileNew, "新規(&N)\tCtrl-N")
            .item(Command::FileNewWindow, "新しいウィンドウ(&W)\tCtrl-Shift-N")
            .item(Command::FileOpen, "開く(&W)…\tCtrl-O")
            .item(Command::FileSave, "保存(&S)\tCtrl-S")
            .item(Command::FileSaveAs, "名前を付けて保存(&A)…\tCtrl-Shift-S")
            .separator()
            .item(Command::FileClose, "終了(&X)\tCtrl-W");
        menu_bar.append(Some(&file_menu), "ファイル(&F)");

        let edit_menu = wx::Menu::new()
            .item(wx::ID_UNDO, "元に戻す(&U)\tCtrl-Z")
            .separator()
            .item(wx::ID_CUT, "切り取り(&T)\tCtrl-X")
            .item(wx::ID_COPY, "コピー(&C)\tCtrl-C")
            .item(wx::ID_PASTE, "貼り付け(&P)\tCtrl-V")
            .item(Command::EditDelete, "削除(&L)\tDel")
            .separator()
            .item(Command::EditFind, "検索(&F)…\tCtrl-F")
            .item(Command::EditFindNext, "次を検索(&N)\tF3")
            .item(Command::EditFindPrevious, "前を検索(&V)\tShift-F3")
            .item(Command::EditReplace, "置換(&R)…\tCtrl-H")
            .item(Command::EditGo, "行へ移動(&G)…\tCtrl-G")
            .separator()
            .item(wx::ID_SELECTALL, "すべて選択(&A)\tCtrl-A")
            .item(Command::EditDate, "日付と時刻(&D)\tF5");
        menu_bar.append(Some(&edit_menu), "編集(&E)");

        let format_menu = wx::Menu::new()
            .check_item(Command::FormatWordWrap, "右端で折り返す(&W)")
            .item(Command::FormatFont, "フォント(&O)…");
        menu_bar.append(Some(&format_menu), "書式(&O)");

        let view_menu = wx::Menu::new().check_item(Command::ViewStatusBar, "ステータスバー(&S)");
        menu_bar.append(Some(&view_menu), "表示(&V)");

        let help_menu = wx::Menu::new()
            .item(Command::Help, "ヘルプの表示(&H)")
            .separator()
            .item(Command::HelpAbout, "バージョン情報(&A)");
        menu_bar.append(Some(&help_menu), "ヘルプ(&H)");

        self.base.set_menu_bar(Some(&menu_bar));
    }

    pub fn new_file(&self) {
        if self.save_if_modified().is_err() {
            return;
        }
        self.textbox.clear();
        self.set_path(None);
    }

    fn save_if_modified(&self) -> Result<(), ()> {
        if self.textbox.is_modified() {
            self.save()
        } else {
            Ok(())
        }
    }

    fn set_path(&self, path: Option<&str>) {
        *self.file.borrow_mut() = path.map(ToOwned::to_owned);
        self.textbox.set_modified(false);
    }

    pub fn open_file(&self) {
        if self.save_if_modified().is_err() {
            return;
        }
        let file_dialog = wx::FileDialog::builder(Some(&self.base)).build();
        if wx::ID_OK == file_dialog.show_modal() {
            let path = file_dialog.get_path();
            self.textbox.load_file(&path, wx::TEXT_TYPE_ANY);
            self.set_path(Some(&path));
        }
    }

    pub fn save(&self) -> Result<(), ()> {
        // if let 式とまとめると save_to() 内で borrow_mut() するため
        // ランタイムエラーになるため、事前にコピーしている
        let path = self.file.borrow().as_ref().map(ToOwned::to_owned);
        if let Some(path) = path {
            self.save_to(&path)
        } else {
            self.save_as()
        }
    }

    pub fn save_as(&self) -> Result<(), ()> {
        let file_dialog = wx::FileDialog::builder(Some(&self.base))
            .style(wx::FC_SAVE.into())
            .build();
        if wx::ID_OK == file_dialog.show_modal() {
            self.save_to(&file_dialog.get_path())
        } else {
            Err(())
        }
    }

    fn save_to(&self, path: &str) -> Result<(), ()> {
        // TODO: Error Handling
        if self.textbox.save_file(&path, wx::TEXT_TYPE_ANY) {
            self.set_path(Some(path));
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn close(&self) {
        self.base.close(false);
    }

    pub fn delete_selection(&self) {
        let mut from: c_long = 0;
        let mut to: c_long = 0;
        self.textbox.get_selection_long(
            &mut from as *mut c_int as *mut c_void,
            &mut to as *mut c_int as *mut c_void,
        );
        self.textbox.remove(from, to);
    }

    pub fn open_help(&self) {
        let project_home = "https://github.com/kenz-gelsoft/ClawEditor/";
        wx::launch_default_browser(project_home, 0);
    }

    pub fn show_about(&self) {
        wx::message_box(
            &format!(
                "{}\nバージョン {}\n© 2022- KENZ, All Rights Reserved.",
                APP_NAME,
                env!("CARGO_PKG_VERSION")
            ),
            APP_NAME,
            (wx::OK | wx::CENTRE).into(),
            Some(&self.base),
        );
    }

    fn on_text_modified(&self) {
        self.update_title();
    }

    fn update_title(&self) {
        let mut modified = "";
        let mut file = UNTITLED.to_owned();
        if let Some(path) = self.file.borrow().as_ref() {
            file = path.to_owned();
        }
        if self.textbox.is_modified() {
            modified = "*";
        }
        let title = format!("{}{} - {}", modified, file, APP_NAME);
        self.base.set_title(&title);
    }
}
