extern crate embed_resource;

use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    if let Ok(wxwin) = env::var("wxwin") {
        let mut wx_rc_dir = PathBuf::from(wxwin);
        wx_rc_dir.push(r"include\wx\msw");
        let mut out_dir = PathBuf::from(env::var("OUT_DIR")?);
        out_dir.push(r"wx\msw");
        copy_dir_all(wx_rc_dir, out_dir).ok();
        embed_resource::compile("wxRust.rc");
    }
    Ok(())
}
