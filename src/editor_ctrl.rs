use std::cell::RefCell;
use std::os::raw::{c_long, c_void};
use std::rc::Rc;

use wx::methods::*;

use crate::commands::{Command, CommandHandler, EditorCommand};
use crate::observer::Subject;

pub trait Document {
    fn clear(&self);
    fn is_modified(&self) -> bool;
    fn reset_modified(&self);
    fn load_from(&self, file_path: &str);
    fn save_to(&self, file_path: &str) -> bool;
}

#[derive(Clone)]
pub enum DocumentEvent {
    TextModified,
}

pub struct EditorCtrl {
    ctrl: wx::TextCtrl,
    pub events: Rc<RefCell<Subject<DocumentEvent>>>,
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
