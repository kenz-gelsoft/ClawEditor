use std::os::raw::c_int;
use std::rc::Rc;

use wx;
use wx::methods::*;

use crate::commands::{Command, CommandHandler, EditorCommand};
use crate::editor_ctrl::{Document, DocumentEvent, EditorCtrl};
use crate::observer::Observer;
use crate::unsaved_changes;

const APP_NAME: &str = "カニツメエディタ";
const UNTITLED: &str = "無題";

const CW_USEDEFAULT: c_int = c_int::MIN;

pub struct EditorFrame {
    base: wx::Frame,
    editor: EditorCtrl,
}
impl EditorFrame {
    pub fn new() -> Rc<Self> {
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
        let editor = EditorCtrl::new(&frame);
        let frame = Rc::new(EditorFrame {
            base: frame,
            editor,
        });
        let frame_copy = frame.clone();
        frame.editor.events().borrow_mut().add_observer(frame_copy);
        let frame_copy = frame.clone();
        frame
            .base
            .bind(wx::RustEvent::Menu, move |event: &wx::CommandEvent| {
                let command = Command::from(event.get_id())
                    .map(|command| EditorCommand::Command(command))
                    .unwrap_or(EditorCommand::StandardEvents(event));
                frame_copy.handle_command(&command);
            });
        let frame_copy = frame.clone();
        frame
            .base
            .bind(wx::RustEvent::UpdateUI, move |event: &wx::UpdateUIEvent| {
                frame_copy.on_update_ui(&event);
            });
        let frame_copy = frame.clone();
        frame
            .base
            .bind(wx::RustEvent::CloseWindow, move |event: &wx::CloseEvent| {
                frame_copy.on_close(&event);
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
            .item(wx::ID_ABOUT, "バージョン情報(&A)");
        menu_bar.append(Some(&help_menu), "ヘルプ(&H)");

        self.base.set_menu_bar(Some(&menu_bar));
    }

    pub fn new_file(&self) {
        unsaved_changes::save(&self.editor, &self.base, |editor, saved| {
            if !saved {
                return;
            }
            editor.new_file();
        });
    }

    pub fn open_file(&self, path: Option<&str>) {
        unsaved_changes::save(&self.editor, &self.base, |editor, saved| {
            if !saved {
                return;
            }
            if let Some(path) = path {
                editor.load_from(path);
                return;
            }
            let file_dialog = wx::FileDialog::builder(Some(&self.base)).build();
            if wx::ID_OK == file_dialog.show_modal() {
                let path = file_dialog.get_path();
                editor.load_from(&path);
            }
        });
    }

    pub fn save(&self) -> Result<(), ()> {
        let path = self.editor.file.borrow().to_owned();
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
        if self.editor.save_to(&path) {
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn close(&self) {
        // Rust のイベント処理を引き起こして borrow rule 違反になるため
        // 1 イベント分遅らせて回避。
        let weak_frame = self.base.to_weak_ref();
        self.base.call_after(move |_| {
            if let Some(frame) = weak_frame.get() {
                frame.close(false);
            }
        });
    }

    pub fn on_update_ui(&self, event: &wx::UpdateUIEvent) {
        println!("hello");
    }

    pub fn on_close(&self, event: &wx::CloseEvent) {
        unsaved_changes::save(&self.editor, &self.base, |_, saved| {
            if !saved {
                event.veto(true);
                return;
            }
            event.skip(true);
        });
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
            (wx::OK | wx::CENTRE) as c_int,
            Some(&self.base),
        );
    }

    fn update_title(&self) {
        let mut modified = "";
        let mut file = UNTITLED.to_owned();
        if let Some(path) = self.editor.file.borrow().as_ref() {
            file = path.to_owned();
        }
        if self.editor.is_modified() {
            modified = "*";
        }
        let title = format!("{}{} - {}", modified, file, APP_NAME);
        self.base.set_title(&title);
    }
}
impl<'a> CommandHandler<EditorCommand<'a>> for EditorFrame {
    fn handle_command(&self, editor_command: &EditorCommand<'a>) {
        match editor_command {
            EditorCommand::Command(command) => match &command {
                // ファイル
                Command::FileNew => {
                    self.new_file();
                }
                Command::FileNewWindow => todo!(),
                Command::FileOpen => {
                    self.open_file(None);
                }
                Command::FileSave => {
                    _ = self.save();
                }
                Command::FileSaveAs => {
                    _ = self.save_as();
                }
                Command::FileClose => {
                    self.close();
                }
                // 編集
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
                    self.open_help();
                }
                Command::EditDelete => {
                    self.editor.handle_command(editor_command);
                }
            },
            EditorCommand::StandardEvents(event) => match event.get_id() {
                wx::ID_ABOUT => {
                    self.show_about();
                }
                _ => {
                    self.editor.handle_command(editor_command);
                }
            },
        }
    }
}
impl unsaved_changes::UI for wx::Frame {
    fn confirm_save<CB: FnOnce(Option<bool>)>(&self, on_complete: CB) {
        // TODO: メッセージ調整
        let answer = wx::message_box(
            "変更があります。保存しますか？",
            APP_NAME,
            wx::YES_NO | (wx::CANCEL | wx::CENTRE) as c_int,
            Some(self),
        );
        on_complete(match answer {
            wx::YES => Some(true),
            wx::NO => Some(false),
            _ => None,
        });
    }
    fn get_path_to_save<CB: FnMut(Option<String>)>(&self, mut callback: CB) {
        let file_dialog = wx::FileDialog::builder(Some(self))
            .style(wx::FC_SAVE.into())
            .build();
        callback(if wx::ID_OK == file_dialog.show_modal() {
            Some(file_dialog.get_path())
        } else {
            None
        });
    }
}
impl Observer<DocumentEvent> for EditorFrame {
    fn on_notify(&self, event: DocumentEvent) {
        match event {
            DocumentEvent::TextModified => self.update_title(),
        }
    }
}
