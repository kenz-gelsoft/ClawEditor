use std::cell::RefCell;
use std::os::raw::{c_int, c_long, c_void};
use std::rc::Rc;

use wx;
use wx::methods::*;

use crate::commands::{Command, CommandHandler};
use crate::observer::{Observer, Subject};

const APP_NAME: &str = "カニツメエディタ";
const UNTITLED: &str = "無題";

const CW_USEDEFAULT: c_int = c_int::MIN;

#[derive(Clone)]
enum DocumentEvent {
    TextModified,
}

trait Document {
    fn forward_event(&self, event: &wx::CommandEvent);
    fn clear(&self);
    fn delete_selection(&self);
    fn is_modified(&self) -> bool;
    fn reset_modified(&self);
    fn load_from(&self, file_path: &str);
    fn save_to(&self, file_path: &str) -> bool;
}

struct EditorCtrl {
    ctrl: wx::TextCtrl,
    events: Rc<RefCell<Subject<DocumentEvent>>>,
}
impl EditorCtrl {
    fn new<W: WindowMethods>(parent: &W) -> Self {
        let textbox = wx::TextCtrl::builder(Some(parent))
            .style(wx::TE_MULTILINE.into())
            .build();
        let events = Rc::new(RefCell::new(Subject::new()));
        let weak_events = Rc::downgrade(&events);
        textbox.bind(wx::RustEvent::Text, move |_: &wx::CommandEvent| {
            if let Some(events) = weak_events.upgrade() {
                events.borrow().notify_event(DocumentEvent::TextModified);
            }
        });
        Self {
            ctrl: textbox,
            events,
        }
    }
}
impl Document for EditorCtrl {
    fn forward_event(&self, event: &wx::CommandEvent) {
        self.ctrl.process_event(event);
    }
    fn clear(&self) {
        self.ctrl.clear();
    }
    fn delete_selection(&self) {
        let mut from: c_long = 0;
        let mut to: c_long = 0;
        self.ctrl.get_selection_long(
            &mut from as *mut c_long as *mut c_void,
            &mut to as *mut c_long as *mut c_void,
        );
        self.ctrl.remove(from, to);
    }
    fn is_modified(&self) -> bool {
        self.ctrl.is_modified()
    }
    fn reset_modified(&self) {
        self.ctrl.set_modified(false);
    }
    fn load_from(&self, file_path: &str) {
        self.ctrl.load_file(file_path, wx::TEXT_TYPE_ANY);
    }
    fn save_to(&self, file_path: &str) -> bool {
        self.ctrl.save_file(file_path, wx::TEXT_TYPE_ANY)
    }
}

pub struct EditorFrame {
    base: wx::Frame,
    editor: EditorCtrl,
    // TODO: avoid interior mutability
    file: Rc<RefCell<Option<String>>>,
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
            file: Rc::new(RefCell::new(None)),
        });
        frame.editor.events.borrow_mut().add_observer(frame.clone());
        let frame_copy = frame.clone();
        frame
            .base
            .bind(wx::RustEvent::Menu, move |event: &wx::CommandEvent| {
                if let Some(command) = Command::from(event.get_id()) {
                    frame_copy.handle_command(&command);
                } else {
                    match event.get_id() {
                        wx::ID_ABOUT => {
                            frame_copy.show_about();
                        }
                        _ => {
                            frame_copy.editor.forward_event(event);
                        }
                    }
                }
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
        if self.save_if_modified().is_err() {
            return;
        }
        self.editor.clear();
        self.set_path(None);
    }

    fn save_if_modified(&self) -> Result<(), ()> {
        if self.editor.is_modified() {
            self.save()
        } else {
            Ok(())
        }
    }

    fn set_path(&self, path: Option<&str>) {
        *self.file.borrow_mut() = path.map(ToOwned::to_owned);
        self.editor.reset_modified();
    }

    pub fn open_file(&self) {
        if self.save_if_modified().is_err() {
            return;
        }
        let file_dialog = wx::FileDialog::builder(Some(&self.base)).build();
        if wx::ID_OK == file_dialog.show_modal() {
            let path = file_dialog.get_path();
            self.editor.load_from(&path);
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
        if self.editor.save_to(&path) {
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
        self.editor.delete_selection();
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
        if let Some(path) = self.file.borrow().as_ref() {
            file = path.to_owned();
        }
        if self.editor.is_modified() {
            modified = "*";
        }
        let title = format!("{}{} - {}", modified, file, APP_NAME);
        self.base.set_title(&title);
    }
}
impl CommandHandler<Command> for EditorFrame {
    fn handle_command(&self, command: &Command) {
        match command {
            // ファイル
            Command::FileNew => {
                self.new_file();
            }
            Command::FileNewWindow => todo!(),
            Command::FileOpen => {
                self.open_file();
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
            Command::EditDelete => {
                self.delete_selection();
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
                self.open_help();
            }
        }
    }
}
impl Observer<DocumentEvent> for EditorFrame {
    fn on_notify(&self, event: DocumentEvent) {
        match event {
            DocumentEvent::TextModified => self.update_title(),
        }
    }
}
