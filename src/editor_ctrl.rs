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

pub trait Document {
    fn events(&self) -> Rc<RefCell<Subject<DocumentEvent>>>;
    fn new_file(&self);
    fn path(&self) -> Option<String>;
    fn is_modified(&self) -> bool;
    fn load_from(&self, file_path: &str);
    fn save_to(&self, file_path: &str) -> bool;
}

pub struct EditorCtrl {
    ctrl: wx::TextCtrl,
    events: Rc<RefCell<Subject<DocumentEvent>>>,
    pub file: Rc<RefCell<Option<String>>>,
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
            file: Rc::new(RefCell::new(None)),
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

    fn select_all(&self) {
        self.ctrl.select_all();
    }

    fn set_path(&self, path: Option<&str>) {
        *self.file.borrow_mut() = path.map(ToOwned::to_owned);
        self.reset_modified();
    }

    fn reset_modified(&self) {
        self.ctrl.set_modified(false);
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
                if event.get_id() == wx::ID_SELECTALL {
                    // GTK+ では wx::TextCtrl が wx::ID_SELECTALL を処理しないため、
                    // 自前で呼び出します。
                    self.select_all();
                    return;
                }
                self.ctrl.process_event(*event);
            }
        }
    }
}
impl Document for EditorCtrl {
    fn events(&self) -> Rc<RefCell<Subject<DocumentEvent>>> {
        self.events.clone()
    }
    fn new_file(&self) {
        self.ctrl.clear();
        self.set_path(None);
    }
    fn path(&self) -> Option<String> {
        self.file.borrow().clone()
    }
    fn is_modified(&self) -> bool {
        self.ctrl.is_modified()
    }
    fn load_from(&self, file_path: &str) {
        self.ctrl.load_file(file_path, wx::TEXT_TYPE_ANY);
        self.set_path(Some(&file_path));
    }
    fn save_to(&self, file_path: &str) -> bool {
        let result = self.ctrl.save_file(file_path, wx::TEXT_TYPE_ANY);
        self.set_path(Some(file_path));
        result
    }
}
