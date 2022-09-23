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

// 後で実際の Document を使うようにする
trait DummyDoc {
    // 変更フラグ
    fn is_modified(&self) -> bool;
    fn set_modified(&mut self, modified: bool);

    fn save_to(&mut self, path: String) -> bool;
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
                modified: false,
                save_will_fail: false,
            }
        }
    }
    impl DummyDoc for MockDoc {
        fn is_modified(&self) -> bool {
            self.modified
        }
        fn set_modified(&mut self, modified: bool) {
            self.modified = modified
        }

        fn save_to(&mut self, _: String) -> bool {
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
        let mut doc = MockDoc::new();
        doc.set_modified(true);
        assert!(doc.is_modified());

        let mut save_dlg = MockSaveDialog::new();
        save_dlg.get_path_to_save(move |path| {
            assert_ne!(path, None);
            // When: 保存に成功したら
            assert!(doc.save_to(path.unwrap()));
            // Then: 変更フラグが倒れている
            assert!(!doc.is_modified());
        });
    }

    #[test]
    fn modified_doc_keeps_modified_after_save_cancelled() {
        // Given: ドキュメントの変更フラグが立っている状態から
        let mut doc = MockDoc::new();
        doc.set_modified(true);
        assert!(doc.is_modified());

        let mut save_dlg = MockSaveDialog::new();
        // When: 保存がキャンセルされたら
        save_dlg.will_be_cancelled = true;
        save_dlg.get_path_to_save(move |path| {
            assert_eq!(path, None);
            // Then: 変更フラグは立ったまま
            assert!(doc.is_modified());
        });
    }

    #[test]
    fn modified_doc_keeps_modified_after_save_failed() {
        // Given: ドキュメントの変更フラグが立っている状態から
        let mut doc = MockDoc::new();
        doc.set_modified(true);
        assert!(doc.is_modified());

        let mut save_dlg = MockSaveDialog::new();
        save_dlg.get_path_to_save(move |path| {
            assert_ne!(path, None);
            // When: 保存に失敗したら
            doc.save_will_fail = true;
            assert!(!doc.save_to(path.unwrap()));
            // Then: 変更フラグは立ったまま
            assert!(doc.is_modified());
        });
    }
}

pub trait Document {
    fn events(&self) -> Rc<RefCell<Subject<DocumentEvent>>>;
    fn clear(&self);
    fn is_modified(&self) -> bool;
    fn reset_modified(&self);
    fn load_from(&self, file_path: &str);
    fn save_to(&self, file_path: &str) -> bool;
}

pub struct EditorCtrl {
    ctrl: wx::TextCtrl,
    events: Rc<RefCell<Subject<DocumentEvent>>>,
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
}
impl<'a> CommandHandler<EditorCommand<'a>> for EditorCtrl {
    fn handle_command(&self, editor_command: &EditorCommand<'a>) {
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