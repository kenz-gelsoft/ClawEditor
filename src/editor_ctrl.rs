use std::cell::RefCell;
use std::os::raw::{c_long, c_void};
use std::rc::Rc;

use wx::methods::*;

use crate::commands::{Command, CommandHandler, EditorCommand};
use crate::observer::Subject;

#[derive(Clone)]
pub enum DocumentEvent {
    TextModified,
}

pub trait UnsavedChangeUI {
    fn confirm_save<CB: FnMut(Option<bool>)>(&self, on_complete: CB);
    fn get_path_to_save<CB: FnMut(Option<String>)>(&self, on_complete: CB);
}

// TODO: future 的なインターフェイス
pub fn save_unsaved_change<D: Document, U: UnsavedChangeUI, CB: Fn(&mut D, bool)>(
    doc: &mut D,
    ui: &U,
    on_complete: CB,
) {
    if !doc.is_modified() {
        on_complete(doc, true);
    } else if let Some(path) = doc.path() {
        doc.save_to(&path);
        on_complete(doc, !doc.is_modified());
    } else {
        ui.get_path_to_save(move |path| {
            if let Some(path) = path {
                // TODO: エラーを返す
                doc.save_to(&path);
            }
            on_complete(doc, !doc.is_modified());
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // TODO: mockall を試す
    struct MockDoc {
        path: Option<String>,
        modified: bool,
        save_will_fail: bool,
    }
    impl MockDoc {
        fn new() -> Self {
            Self {
                path: None,
                modified: true,
                save_will_fail: false,
            }
        }
    }
    impl Document for MockDoc {
        fn events(&self) -> Rc<RefCell<Subject<DocumentEvent>>> {
            todo!()
        }
        fn new_file(&mut self) {
            todo!()
        }

        fn path(&self) -> Option<String> {
            return self.path.clone();
        }

        fn is_modified(&self) -> bool {
            self.modified
        }

        fn reset_modified(&mut self) {
            self.modified = false;
        }

        fn load_from(&mut self, _file_path: &str) {
            todo!()
        }

        fn save_to(&mut self, _file_path: &str) -> bool {
            if self.save_will_fail {
                return false;
            }
            self.modified = false;
            true
        }
    }

    struct MockSaveDialog {
        wont_be_called: bool,
        will_be_cancelled: bool,
    }
    impl MockSaveDialog {
        fn new() -> Self {
            Self {
                wont_be_called: false,
                will_be_cancelled: false,
            }
        }
    }
    impl UnsavedChangeUI for MockSaveDialog {
        fn confirm_save<CB: FnMut(Option<bool>)>(&self, mut on_complete: CB) {
            on_complete(Some(true))
        }
        fn get_path_to_save<CB: FnMut(Option<String>)>(&self, mut on_complete: CB) {
            assert!(!self.wont_be_called);
            if self.will_be_cancelled {
                on_complete(None);
                return;
            }
            on_complete(Some("path/to/save".to_owned()))
        }
    }

    #[test]
    fn do_nothing_if_not_modified() {
        // Given: ドキュメントの変更フラグが立っていない状態から
        let mut doc = MockDoc::new();
        doc.modified = false;
        assert!(!doc.is_modified());

        // When: 保存を判定したら
        let mut save_dlg = MockSaveDialog::new();
        // Then: 保存ダイアログは呼ばれず
        save_dlg.wont_be_called = true;
        save_unsaved_change(&mut doc, &save_dlg, |_doc, saved| {
            // Then: 変更フラグはたっていないまま
            assert!(saved);
        });
    }

    #[test]
    fn save_dlg_wont_be_called_if_has_path() {
        // Given: ドキュメントの変更フラグが立っている状態から
        let mut doc = MockDoc::new();
        doc.path = Some("dummy".to_owned());
        assert!(doc.is_modified());

        let mut save_dlg = MockSaveDialog::new();
        save_dlg.wont_be_called = true;
        // When: 保存に成功したら
        save_unsaved_change(&mut doc, &save_dlg, |_doc, saved| {
            // Then: 変更フラグが倒れている
            assert!(saved);
        });
    }

    #[test]
    fn modified_doc_will_be_unmodified_after_save() {
        // Given: ドキュメントの変更フラグが立っている状態から
        let mut doc = MockDoc::new();
        assert!(doc.is_modified());

        let save_dlg = MockSaveDialog::new();
        // When: 保存に成功したら
        save_unsaved_change(&mut doc, &save_dlg, |_doc, saved| {
            // Then: 変更フラグが倒れている
            assert!(saved);
        });
    }

    #[test]
    fn modified_doc_keeps_modified_after_save_cancelled() {
        // Given: ドキュメントの変更フラグが立っている状態から
        let mut doc = MockDoc::new();
        assert!(doc.is_modified());

        let mut save_dlg = MockSaveDialog::new();
        // When: 保存がキャンセルされたら
        save_dlg.will_be_cancelled = true;
        save_unsaved_change(&mut doc, &save_dlg, |_doc, saved| {
            // Then: 変更フラグは立ったまま
            assert!(!saved);
        });
    }

    #[test]
    fn modified_doc_keeps_modified_after_save_failed() {
        // Given: ドキュメントの変更フラグが立っている状態から
        let mut doc = MockDoc::new();
        assert!(doc.is_modified());

        let save_dlg = MockSaveDialog::new();
        // When: 保存に失敗したら
        doc.save_will_fail = true;
        save_unsaved_change(&mut doc, &save_dlg, |_doc, saved| {
            // Then: 変更フラグは立ったまま
            assert!(!saved);
        });
    }
}

pub trait Document {
    fn events(&self) -> Rc<RefCell<Subject<DocumentEvent>>>;
    fn new_file(&mut self);
    fn path(&self) -> Option<String>;
    fn is_modified(&self) -> bool;
    fn reset_modified(&mut self);
    fn load_from(&mut self, file_path: &str);
    fn save_to(&mut self, file_path: &str) -> bool;
}

pub struct EditorCtrl {
    ctrl: wx::TextCtrl,
    events: Rc<RefCell<Subject<DocumentEvent>>>,
    pub file: Option<String>,
}
impl EditorCtrl {
    pub fn new<W: WindowMethods>(parent: &W) -> Self {
        let textbox = wx::TextCtrl::builder(Some(parent))
            .style(wx::TE_MULTILINE.into())
            .build();
        let events = Rc::new(RefCell::new(Subject::new()));
        let weak_events = Rc::downgrade(&events);
        let textbox_copy = textbox.clone();
        textbox.bind(wx::RustEvent::Text, move |_: &wx::CommandEvent| {
            // テキスト編集にリアルタイムで応答すると、
            // テキスト編集を引き起こしたイベント処理内で borrow_mut() していることがあり、
            // borrow rule に違反して panic する場合がある。
            // テキスト編集に対するイベント処理をを1イベント分遅らせることで回避する。
            let weak_events = weak_events.clone();
            textbox_copy.call_after(move |_| {
                if let Some(events) = weak_events.upgrade() {
                    events.borrow().notify_event(DocumentEvent::TextModified);
                }
            });
        });
        Self {
            ctrl: textbox,
            events,
            file: None,
        }
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

    fn set_path(&mut self, path: Option<&str>) {
        self.file = path.map(ToOwned::to_owned);
        self.reset_modified();
    }
}
impl<'a> CommandHandler<EditorCommand<'a>> for EditorCtrl {
    fn handle_command(&mut self, editor_command: &EditorCommand<'a>) {
        match editor_command {
            EditorCommand::Command(command) => match command {
                Command::EditDelete => {
                    self.delete_selection();
                }
                _ => (),
            },
            EditorCommand::StandardEvents(event) => {
                self.ctrl.process_event(*event);
            }
        }
    }
}
impl Document for EditorCtrl {
    fn events(&self) -> Rc<RefCell<Subject<DocumentEvent>>> {
        self.events.clone()
    }
    fn new_file(&mut self) {
        self.ctrl.clear();
        self.set_path(None);
    }
    fn path(&self) -> Option<String> {
        self.file.clone()
    }
    fn is_modified(&self) -> bool {
        self.ctrl.is_modified()
    }
    fn reset_modified(&mut self) {
        self.ctrl.set_modified(false);
    }
    fn load_from(&mut self, file_path: &str) {
        self.ctrl.load_file(file_path, wx::TEXT_TYPE_ANY);
        self.set_path(Some(&file_path));
    }
    fn save_to(&mut self, file_path: &str) -> bool {
        let result = self.ctrl.save_file(file_path, wx::TEXT_TYPE_ANY);
        self.set_path(Some(file_path));
        result
    }
}
