use std::io;
use crate::ui::tui::TUI;

mod errors;
mod device;
mod machine;
mod opcode;
mod ui;

fn main() -> io::Result<()> {
    let mut tui = TUI::new()?;
    tui.init()?;

    tui.ui_loop()?;

    tui.stop()
}
