use crate::editor_ctrl::Document;

pub trait UI {
    fn confirm_save<CB: FnOnce(Option<bool>)>(&self, on_complete: CB);
    fn get_path_to_save<CB: FnMut(Option<String>)>(&self, on_complete: CB);
}

// TODO: future 的なインターフェイス
pub fn save<D: Document, U: UI, CB: Fn(&mut D, bool)>(doc: &mut D, ui: &U, on_complete: CB) {
    if !doc.is_modified() {
        // 変更されていなければ何もしない
        return on_complete(doc, true);
    }
    ui.confirm_save(|result| {
        if let Some(do_save) = result {
            if do_save {
                // 確認ダイアログで「保存する」
                if let Some(path) = doc.path() {
                    doc.save_to(&path);
                    on_complete(doc, !doc.is_modified());
                } else {
                    ui.get_path_to_save(|path| {
                        if let Some(path) = path {
                            // TODO: エラーを返す
                            doc.save_to(&path);
                        }
                        on_complete(doc, !doc.is_modified());
                    });
                }
            } else {
                // 確認ダイアログで「保存しない」
                doc.reset_modified();
                on_complete(doc, !doc.is_modified());
            }
        } else {
            // 確認ダイアログでキャンセル
            on_complete(doc, false);
        }
    });
}

#[cfg(test)]
mod test {
    use super::*;

    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::editor_ctrl::DocumentEvent;
    use crate::observer::Subject;

    // TODO: mockall を試す
    struct MockDoc {
        path: Option<String>,
        modified: bool,
        save_wont_be_called: bool,
        save_will_fail: bool,
    }
    impl MockDoc {
        fn new() -> Self {
            Self {
                path: None,
                modified: true,
                save_wont_be_called: false,
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
            assert!(!self.save_wont_be_called);
            if self.save_will_fail {
                return false;
            }
            self.modified = false;
            true
        }
    }

    struct MockSaveUI {
        confirm_wont_be_called: bool,
        // Yes/No/Cancel
        confirm_result: Option<bool>,
        save_dlg_wont_be_called: bool,
        save_dlg_will_be_cancelled: bool,
    }
    impl MockSaveUI {
        fn new() -> Self {
            Self {
                confirm_wont_be_called: false,
                confirm_result: Some(true),
                save_dlg_wont_be_called: false,
                save_dlg_will_be_cancelled: false,
            }
        }
    }
    impl UI for MockSaveUI {
        fn confirm_save<CB: FnOnce(Option<bool>)>(&self, on_complete: CB) {
            assert!(!self.confirm_wont_be_called);
            on_complete(self.confirm_result)
        }
        fn get_path_to_save<CB: FnMut(Option<String>)>(&self, mut on_complete: CB) {
            assert!(!self.save_dlg_wont_be_called);
            if self.save_dlg_will_be_cancelled {
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
        let mut ui = MockSaveUI::new();
        // Then: 確認ダイアログも
        ui.confirm_wont_be_called = true;
        // Then: 保存ダイアログも呼ばれず
        ui.save_dlg_wont_be_called = true;
        save(&mut doc, &ui, |_doc, saved| {
            // Then: 変更フラグはたっていないまま
            assert!(saved);
        });
    }

    #[test]
    fn do_nothing_if_confirm_cancelled() {
        // Given: ドキュメントの変更フラグが立っている状態から
        let mut doc = MockDoc::new();
        assert!(doc.is_modified());

        let mut ui = MockSaveUI::new();
        // When: 確認ダイアログでキャンセルしたら
        ui.confirm_result = None;
        // Then: 保存ダイアログは呼ばれず
        ui.save_dlg_wont_be_called = true;
        save(&mut doc, &ui, |_doc, saved| {
            // Then: 変更フラグはたったまま
            assert!(!saved);
        });
    }

    #[test]
    fn modified_doc_will_be_unmodified_if_confirm_dont_save() {
        // Given: ドキュメントの変更フラグが立っている状態から
        let mut doc = MockDoc::new();
        assert!(doc.is_modified());

        let mut ui = MockSaveUI::new();
        // When: 確認ダイアログで「保存しない」選択したら
        ui.confirm_result = Some(false);
        // Then: 保存ダイアログは呼ばれず
        ui.save_dlg_wont_be_called = true;
        // Then: 保存も行われないが
        doc.save_wont_be_called = true;
        save(&mut doc, &ui, |_doc, saved| {
            // Then: 変更フラグは倒れる
            assert!(saved);
        });
    }

    #[test]
    fn save_dlg_wont_be_called_if_has_path() {
        // Given: ドキュメントの変更フラグが立っている状態から
        let mut doc = MockDoc::new();
        doc.path = Some("dummy".to_owned());
        assert!(doc.is_modified());

        let mut ui = MockSaveUI::new();
        ui.save_dlg_wont_be_called = true;
        // When: 保存に成功したら
        save(&mut doc, &ui, |_doc, saved| {
            // Then: 変更フラグが倒れている
            assert!(saved);
        });
    }

    #[test]
    fn modified_doc_will_be_unmodified_after_save() {
        // Given: ドキュメントの変更フラグが立っている状態から
        let mut doc = MockDoc::new();
        assert!(doc.is_modified());

        let ui = MockSaveUI::new();
        // When: 保存に成功したら
        save(&mut doc, &ui, |_doc, saved| {
            // Then: 変更フラグが倒れている
            assert!(saved);
        });
    }

    #[test]
    fn modified_doc_keeps_modified_after_save_cancelled() {
        // Given: ドキュメントの変更フラグが立っている状態から
        let mut doc = MockDoc::new();
        assert!(doc.is_modified());

        let mut ui = MockSaveUI::new();
        // When: 保存がキャンセルされたら
        ui.save_dlg_will_be_cancelled = true;
        save(&mut doc, &ui, |_doc, saved| {
            // Then: 変更フラグは立ったまま
            assert!(!saved);
        });
    }

    #[test]
    fn modified_doc_keeps_modified_after_save_failed() {
        // Given: ドキュメントの変更フラグが立っている状態から
        let mut doc = MockDoc::new();
        assert!(doc.is_modified());

        let ui = MockSaveUI::new();
        // When: 保存に失敗したら
        doc.save_will_fail = true;
        save(&mut doc, &ui, |_doc, saved| {
            // Then: 変更フラグは立ったまま
            assert!(!saved);
        });
    }
}
