#![windows_subsystem = "windows"]

use std::os::raw::c_int;
use wx;
use wx::methods::*;

enum File {
    New = 100,
    Open,
    Close,
    Save,
}
impl From<File> for c_int {
    fn from(w: File) -> Self {
        w as c_int
    }
}

fn main() {
    wx::App::run(|_| {
        let frame = wx::Frame::builder(wx::Window::none())
            .title("カニツメエディタ")
            .build();
        let menu_bar = wx::MenuBar::new(0);
        let file_menu = wx::Menu::new()
            .item(File::New, "新規\tCtrl-N")
            .item(File::Open, "開く\tCtrl-O")
            .separator()
            .item(File::Close, "閉じる\tCtrl-W")
            .item(File::Save, "保存\tCtrl-S");

        menu_bar.append(Some(&file_menu), "ファイル");
        frame.set_menu_bar(Some(&menu_bar));
        let _textbox = wx::TextCtrl::builder(Some(&frame))
            .style(wx::TE_MULTILINE.into())
            .build();
        frame.show(true);
    });
}
