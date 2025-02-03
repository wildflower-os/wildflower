use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::api::prompt::Prompt;
use crate::api::{fs, io};
use crate::usr::edit::{cols, prompt, rows};

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    if args.len() != 2 {
        help();
        return Err(ExitCode::UsageError);
    }
    if args[1] == "-h" || args[1] == "--help" {
        help();
        return Ok(());
    }

    crate::api::hfs::check_hfs_bounds(args[1])?;

    let pathname = args[1];
    let mut viewer = Viewer::new(pathname);
    viewer.run()
}

pub struct Viewer {
    search_prompt: Prompt,
    search_query: String,
    pathname: String,
    lines: Vec<String>,
    width: usize,
    x: usize,
    y: usize,
}

impl Viewer {
    pub fn new(pathname: &str) -> Self {
        let mut lines = Vec::new();

        let mut width = 0;
        match fs::read_to_string(pathname) {
            Ok(contents) => {
                for line in contents.lines() {
                    lines.push(line.into());
                    width = cmp::max(width, line.chars().count());
                }
                if lines.is_empty() {
                    lines.push(String::new());
                }
            }
            Err(_) => {
                lines.push(String::new());
            }
        };

        let pathname = pathname.into();

        let search_query = String::new();
        let mut search_prompt = Prompt::new();
        search_prompt.eol = false;

        Self {
            search_prompt,
            search_query,
            pathname,
            lines,
            width,
            x: 0,
            y: 0,
        }
    }

    fn print_status(&mut self) {
        let max = 50;
        let mut path = self.pathname.clone();
        if self.pathname.chars().count() > max {
            path.truncate(max - 3);
            path.push_str("...");
        }
        let start = format!("Viewing '{}'", path);

        let x = self.x + 1;
        let y = cmp::min(self.lines.len(), self.y + rows());
        let n = y * 100 / self.lines.len();
        let end = format!("{},{} {:3}%", y, x, n);

        let width = cols() - start.chars().count();
        let status = format!("{}{:>width$}", start, end, width = width);

        let color = Style::color("black").with_background("silver");
        let reset = Style::reset();

        // Move cursor to the bottom of the screen
        print!("\x1b[{};1H", rows() + 1);

        print!("{}{:cols$}{}", color, status, reset, cols = cols());
    }

    fn print_screen(&mut self) {
        let mut lines: Vec<String> = Vec::new();
        let a = self.y;
        let b = self.y + rows();
        for y in a..b {
            lines.push(self.render_line(y));
        }
        println!("\x1b[1;1H{}", lines.join("\n"));
    }

    fn render_line(&self, y: usize) -> String {
        // Render line into a row of the screen, or an empty row when past eof
        let line = if y < self.lines.len() {
            &self.lines[y]
        } else {
            ""
        };

        let s = format!("{:cols$}", line, cols = self.x);
        let mut row: Vec<char> = s.chars().collect();
        let n = self.x + cols();
        let after = if row.len() > n {
            row.truncate(n - 1);
            truncated_line_indicator()
        } else {
            " ".repeat(n - row.len())
        };
        row.extend(after.chars());
        row[self.x..].iter().collect()
    }

    fn scroll_up(&mut self, n: usize) {
        if self.y > 0 {
            self.y = self.y.saturating_sub(n);
            self.print_screen();
        }
    }

    fn scroll_down(&mut self, n: usize) {
        let lines = self.lines.len();
        let bottom = self.y + rows();
        if lines > bottom {
            self.y += cmp::min(n, lines - bottom);
            self.print_screen();
        }
    }

    pub fn run(&mut self) -> Result<(), ExitCode> {
        print!("\x1b[2J\x1b[1;1H"); // Clear screen and move to top
        print!("\x1b[?25l"); // Disable cursor
        self.print_screen();
        self.print_status();

        let mut escape = false;
        let mut csi = false;
        let mut csi_params = String::new();
        loop {
            let c = io::stdin().read_char().unwrap_or('\0');

            match c {
                '\x1B' => {
                    // ESC
                    escape = true;
                    continue;
                }
                '[' if escape => {
                    csi = true;
                    csi_params.clear();
                    continue;
                }
                '\0' => {
                    continue;
                }
                '\x11' | '\x03' => {
                    // Ctrl Q or Ctrl C
                    print!("\x1b[2J\x1b[1;1H"); // Clear screen and move to top
                    print!("\x1b[?25h"); // Enable cursor
                    break;
                }
                '\n' => {
                    // Newline
                    self.scroll_down(1);
                }
                ' ' => {
                    // Space
                    self.scroll_down(rows() - 1);
                }
                '~' if csi && csi_params == "5" => {
                    // Page Up
                    self.scroll_up(rows() - 1);
                }
                '~' if csi && csi_params == "6" => {
                    // Page Down
                    self.scroll_down(rows() - 1);
                }
                'A' if csi => {
                    // Arrow Up
                    self.scroll_up(1);
                }
                'B' if csi => {
                    // Arrow Down
                    self.scroll_down(1);
                }
                'C' if csi => {
                    // Arrow Right
                    if self.x + cols() < self.width {
                        self.x += cols();
                        self.print_screen();
                    }
                }
                'D' if csi => {
                    // Arrow Left
                    if self.x > 0 {
                        self.x = self.x.saturating_sub(cols());
                        self.print_screen();
                    }
                }
                '\x14' => {
                    // Ctrl T -> Go to top of file
                    self.y = 0;
                    self.print_screen();
                }
                '\x02' => {
                    // Ctrl B -> Go to bottom of file
                    self.y = self.lines.len() - rows();
                    self.print_screen();
                }
                '\x06' => {
                    // Ctrl F -> Find
                    self.find();
                    self.print_screen();
                }
                '\x0E' => {
                    // Ctrl N -> Find next
                    self.find_next();
                    self.print_screen();
                }
                c => {
                    if csi {
                        csi_params.push(c);
                        continue;
                    }
                }
            }
            self.print_status();
            escape = false;
            csi = false;
        }
        Ok(())
    }

    pub fn find(&mut self) {
        let res = prompt(&mut self.search_prompt, "Find: ");
        print!("\x1b[?25l"); // Disable cursor

        if let Some(query) = res {
            if !query.is_empty() {
                self.search_prompt.history.add(&query);
                self.search_query = query;
                self.find_next();
            }
        }
    }

    pub fn find_next(&mut self) {
        for (y, line) in self.lines.iter().enumerate() {
            if y <= self.y {
                continue;
            }
            if line.contains(&self.search_query) {
                self.y = y;
                break;
            }
        }
    }
}

fn truncated_line_indicator() -> String {
    let color = Style::color("black").with_background("silver");
    let reset = Style::reset();
    format!("{}>{}", color, reset)
}

fn help() {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} view {}<file>{}",
        csi_title, csi_reset, csi_option, csi_reset
    );
}
