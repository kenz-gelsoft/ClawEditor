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

// 保存されていない変更を管理する
trait UnsavedChange {
    fn new_file();
}

trait UnsavedChangeUI {
    fn get_path_to_save<CB: FnMut(Option<String>)>(&mut self, callback: CB);
}

// TODO: future 的なインターフェイス
fn save_unsaved_change<D: Document, U: UnsavedChangeUI, CB: Fn(bool)>(
    mut doc: D,
    mut ui: U,
    callback: CB,
) {
    ui.get_path_to_save(move |path| {
        if let Some(path) = path {
            // TODO: エラーを返す
            doc.save_to(&path);
        }
        callback(!doc.is_modified());
    });
}

#[cfg(test)]
mod test {
    use super::*;

    // TODO: mockall を試す
    struct MockDoc {
        modified: bool,
        save_will_fail: bool,
    }
    impl MockDoc {
        fn new() -> Self {
            Self {
                modified: true,
                save_will_fail: false,
            }
        }
    }
    impl Document for MockDoc {
        fn events(&self) -> Rc<RefCell<Subject<DocumentEvent>>> {
            todo!()
        }
        fn clear(&self) {
            todo!()
        }

        fn is_modified(&self) -> bool {
            self.modified
        }

        fn reset_modified(&mut self) {
            self.modified = false;
        }

        fn load_from(&self, file_path: &str) {
            todo!()
        }

        fn save_to(&mut self, file_path: &str) -> bool {
            if self.save_will_fail {
                return false;
            }
            self.modified = false;
            true
        }
    }

    struct MockSaveDialog {
        will_be_cancelled: bool,
    }
    impl MockSaveDialog {
        fn new() -> Self {
            Self {
                will_be_cancelled: false,
            }
        }
    }
    impl UnsavedChangeUI for MockSaveDialog {
        fn get_path_to_save<CB: FnMut(Option<String>)>(&mut self, mut callback: CB) {
            if self.will_be_cancelled {
                callback(None);
                return;
            }
            callback(Some("path/to/save".to_owned()))
        }
    }

    #[test]
    fn modified_doc_will_be_unmodified_after_save() {
        // Given: ドキュメントの変更フラグが立っている状態から
        let doc = MockDoc::new();
        assert!(doc.is_modified());

        let save_dlg = MockSaveDialog::new();
        // When: 保存に成功したら
        save_unsaved_change(doc, save_dlg, |result| {
            // Then: 変更フラグが倒れている
            assert!(result);
        });
    }

    #[test]
    fn modified_doc_keeps_modified_after_save_cancelled() {
        // Given: ドキュメントの変更フラグが立っている状態から
        let doc = MockDoc::new();
        assert!(doc.is_modified());

        let mut save_dlg = MockSaveDialog::new();
        // When: 保存がキャンセルされたら
        save_dlg.will_be_cancelled = true;
        save_unsaved_change(doc, save_dlg, |result| {
            // Then: 変更フラグは立ったまま
            assert!(!result);
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
        save_unsaved_change(doc, save_dlg, |result| {
            // Then: 変更フラグは立ったまま
            assert!(!result);
        });
    }
}

pub trait Document {
    fn events(&self) -> Rc<RefCell<Subject<DocumentEvent>>>;
    fn clear(&self);
    fn is_modified(&self) -> bool;
    fn reset_modified(&mut self);
    fn load_from(&self, file_path: &str);
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
        textbox.bind(wx::RustEvent::Text, move |_: &wx::CommandEvent| {
            if let Some(events) = weak_events.upgrade() {
                events.borrow().notify_event(DocumentEvent::TextModified);
            }
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

    pub fn set_path(&mut self, path: Option<&str>) {
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
    fn clear(&self) {
        self.ctrl.clear();
    }
    fn is_modified(&self) -> bool {
        self.ctrl.is_modified()
    }
    fn reset_modified(&mut self) {
        self.ctrl.set_modified(false);
    }
    fn load_from(&self, file_path: &str) {
        self.ctrl.load_file(file_path, wx::TEXT_TYPE_ANY);
    }
    fn save_to(&mut self, file_path: &str) -> bool {
        self.ctrl.save_file(file_path, wx::TEXT_TYPE_ANY)
    }
}
