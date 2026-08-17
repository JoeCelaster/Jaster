use colored::*;
use std::io::{self, Read, Write};

pub struct Item {
    pub label: String,
    pub hint: String,
}

pub enum Selection {
    Chosen(usize),
    Cancelled,
    /// stdin or stdout is not a terminal, so nobody can answer the prompt.
    NotInteractive,
}

/// Restores the terminal however the prompt ends.
struct RawMode {
    original: libc::termios,
}

impl RawMode {
    fn enable() -> Option<Self> {
        unsafe {
            if libc::isatty(libc::STDIN_FILENO) != 1 || libc::isatty(libc::STDOUT_FILENO) != 1 {
                return None;
            }

            let mut original: libc::termios = std::mem::zeroed();

            if libc::tcgetattr(libc::STDIN_FILENO, &mut original) != 0 {
                return None;
            }

            let mut raw = original;
            libc::cfmakeraw(&mut raw);

            if libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &raw) != 0 {
                return None;
            }

            Some(Self { original })
        }
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        print!("\x1b[?25h");
        let _ = io::stdout().flush();

        unsafe {
            libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &self.original);
        }
    }
}

fn render(items: &[Item], cursor: usize, first: bool) -> io::Result<()> {
    let mut stdout = io::stdout();

    if !first {
        write!(stdout, "\x1b[{}A", items.len())?;
    }

    let width = items
        .iter()
        .map(|item| item.label.chars().count())
        .max()
        .unwrap_or(0);

    for (index, item) in items.iter().enumerate() {
        let padding = " ".repeat(width - item.label.chars().count());

        let line = if index == cursor {
            format!(
                "\x1b[2K  {} {}{}  {}",
                "❯".cyan().bold(),
                item.label.cyan().bold(),
                padding,
                item.hint.bright_black()
            )
        } else {
            format!(
                "\x1b[2K    {}{}  {}",
                item.label,
                padding,
                item.hint.bright_black()
            )
        };

        write!(stdout, "{line}\r\n")?;
    }

    stdout.flush()
}

/// Arrow-key picker. Returns the index of the chosen item.
pub fn select(prompt: &str, items: &[Item], default: usize) -> io::Result<Selection> {
    if items.is_empty() {
        return Ok(Selection::Cancelled);
    }

    let Some(_raw) = RawMode::enable() else {
        return Ok(Selection::NotInteractive);
    };

    let mut cursor = default.min(items.len() - 1);

    print!("\x1b[?25l");
    print!(
        "  {} {}\r\n\r\n",
        prompt.bold(),
        "(↑/↓ to move, enter to select)".bright_black()
    );
    io::stdout().flush()?;

    render(items, cursor, true)?;

    let mut stdin = io::stdin();
    let mut buffer = [0u8; 8];

    loop {
        let read = stdin.read(&mut buffer)?;

        if read == 0 {
            return Ok(Selection::Cancelled);
        }

        let mut index = 0;

        while index < read {
            let byte = buffer[index];

            match byte {
                b'\r' | b'\n' => {
                    print!("\r\n");
                    io::stdout().flush()?;

                    return Ok(Selection::Chosen(cursor));
                }
                // Ctrl-C, Ctrl-D, q
                3 | 4 | b'q' => {
                    print!("\r\n");
                    io::stdout().flush()?;

                    return Ok(Selection::Cancelled);
                }
                0x1b => {
                    if index + 2 < read && buffer[index + 1] == b'[' {
                        match buffer[index + 2] {
                            b'A' => cursor = if cursor == 0 { items.len() - 1 } else { cursor - 1 },
                            b'B' => cursor = (cursor + 1) % items.len(),
                            _ => {}
                        }

                        index += 2;
                    } else {
                        print!("\r\n");
                        io::stdout().flush()?;

                        return Ok(Selection::Cancelled);
                    }
                }
                b'k' => cursor = if cursor == 0 { items.len() - 1 } else { cursor - 1 },
                b'j' => cursor = (cursor + 1) % items.len(),
                _ => {}
            }

            index += 1;
        }

        render(items, cursor, false)?;
    }
}
