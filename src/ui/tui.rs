use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::io::{stdout, Stdout};
use std::ops::Deref;
use crossterm::{event, ExecutableCommand};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::{CompletedFrame, Terminal};
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::layout::{Alignment, Constraint, Margin, Rect};
use ratatui::prelude::{Color, Direction, Layout, Line, Span, Style, Stylize};
use ratatui::widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Tabs, Wrap};
use ratatui::widgets::canvas::{Canvas, Rectangle};
use crate::machine::machine::Machine;
use crate::machine::register::Register;
use crate::machine::runner::Runner;

#[derive(PartialEq)]
pub enum Mode {
    RUN,
    STEP,
    LoadFile,
}

impl Display for Mode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Mode::STEP => "Step",
            Mode::RUN => "Run",
            Mode::LoadFile => "Load",
        };
        f.write_str(string)
    }
}

pub struct TUI {
    mode: Mode,
    runner: Runner,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    selected_tab: u8,
    vertical_scroll: u32,
    selected_file: Option<(String, u8)>,
    load_result: Result<(), String>,
}


impl TUI {
    const TABS: [&'static str; 4] = ["Load file", "Start", "Step", "Quit"];
    pub fn new() -> io::Result<Self> {
        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        Ok(Self {
            mode: Mode::STEP,
            runner: Runner::new(),
            terminal,
            selected_tab: 0,
            vertical_scroll: 0,
            selected_file: None,
            load_result: Ok(()),
        })
    }
    pub fn init(&mut self) -> io::Result<()> {
        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        self.terminal.clear()
    }

    pub fn ui_loop(&mut self) -> io::Result<()> {
        self.runner.start();
        loop {
            let mut step = match self.mode {
                Mode::RUN => Some(self.runner.try_step()),
                _ => None,
            };


            if let Ok(true) = self.handle_events(&mut step) {
                break;
            }

            self.draw(step)?;
        }

        Ok(())
    }

    pub fn stop(&mut self) -> io::Result<()> {
        self.runner.stop();
        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()
    }

    fn handle_events(&mut self, step: &mut Option<Result<(), String>>) -> io::Result<bool> {
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => return Ok(true),
                        KeyCode::Char('s') => self.mode = Mode::STEP,
                        KeyCode::F(8) => {
                            if self.mode == Mode::STEP {
                                *step = Some(self.runner.try_step());
                            } else {
                                self.mode = Mode::STEP;
                            }
                        }
                        KeyCode::F(9) => self.mode = Mode::RUN,
                        KeyCode::Delete => self.runner.stop(),
                        KeyCode::Tab => {
                            self.selected_tab += 1;
                            if self.selected_tab >= Self::TABS.len() as u8 {
                                self.selected_tab = 0;
                            }
                        }
                        KeyCode::BackTab => {
                            if self.selected_tab > 0 {
                                self.selected_tab -= 1;
                            } else {
                                self.selected_tab = (Self::TABS.len() - 1) as u8;
                            }
                        }
                        KeyCode::Enter => {
                            // Open tab
                            match self.selected_tab {
                                0 => {
                                    if self.mode == Mode::LoadFile && self.selected_file.is_some() {
                                        // Load file
                                        let file = File::open(&self.selected_file.as_ref().unwrap().0).unwrap();
                                        self.load_result = self.runner.load_file(&file);
                                        self.mode = Mode::STEP;
                                    } else {
                                        // Open file dialog
                                        self.mode = Mode::LoadFile;
                                    }
                                }
                                1 => {
                                    // Start
                                    self.mode = Mode::RUN;
                                }
                                2 => {
                                    // Step
                                    if self.mode == Mode::STEP {
                                        *step = Some(self.runner.try_step());
                                    } else {
                                        self.mode = Mode::STEP;
                                    }
                                }
                                3 => {
                                    // Quit
                                    return Ok(true);
                                }
                                _ => {}
                            };
                        }
                        KeyCode::Up => {
                            if self.vertical_scroll > 0 {
                                self.vertical_scroll -= 1;
                            }
                        }
                        KeyCode::Down => {
                            self.vertical_scroll += 1;
                        }
                        KeyCode::PageUp | KeyCode::Left => {
                            let left = self.vertical_scroll & 0xF;
                            if left == 0 && self.vertical_scroll >= 16 {
                                self.vertical_scroll -= 16;
                            } else {
                                self.vertical_scroll -= left;
                            }
                        }
                        KeyCode::PageDown | KeyCode::Right => {
                            self.vertical_scroll += 16;
                        }
                        KeyCode::Char('o') => {
                            // Open file
                            self.mode = Mode::LoadFile;
                        }
                        KeyCode::Char('j') => {
                            if let Some((_selected, ix)) = &mut self.selected_file {
                                if *ix < 0xFF {
                                    *ix += 1;
                                }
                            }
                        }
                        KeyCode::Char('k') => {
                            if let Some((_, ix)) = &mut self.selected_file {
                                if *ix > 0 {
                                    *ix -= 1;
                                }
                            }
                        }
                        _ => {}
                    }

                }
            }
        }
        Ok(false)
    }
    fn draw(&mut self, step: Option<Result<(), String>>) -> io::Result<CompletedFrame> {
        self.terminal.draw(|frame| {
            let area = frame.size();

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Length(2),  // title
                    Constraint::Length(4),  // tabs
                    Constraint::Percentage(80),  // main
                    Constraint::Percentage(4), // error
                    Constraint::Percentage(4),  // help
                ])
                .split(area);


            let tabs = Tabs::new(vec!["Load file", "Start", "Step", "Quit"])
                .block(Block::default().title("Tabs").borders(Borders::ALL))
                .style(Style::default().white())
                .highlight_style(Style::default().yellow())
                .select(self.selected_tab as usize)
                .divider("|");

            frame.render_widget(tabs.on_black(), layout[1]);

            frame.render_widget(
                Paragraph::new("Siculator XE")
                    .block(Block::default().borders(Borders::BOTTOM))
                    .on_light_green()
                    .black()
                    .alignment(Alignment::Center),
                layout[0]);

            frame.render_widget(
                Paragraph::new(format!("Help: press q to quit, F8 to step or F9 to run. Current mode: {}.", self.mode).as_str())
                    .wrap(Wrap { trim: true })
                    .block(Block::new().borders(Borders::ALL)),
                layout[4]);

            let main_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![
                    Constraint::Percentage(16),
                    Constraint::Percentage(42),
                    Constraint::Percentage(42),
                ])
                .split(layout[2]);

            let left_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Percentage(80),
                    Constraint::Percentage(20),
                ])
                .split(main_layout[0]);


            // Draw registers
            let mut register_widgets = Vec::new();

            register_widgets.push(
                Paragraph::new("Data Registers")
                    .cyan()
                    .on_black()
            );

            for i in 0..10 {
                let register = Register::from_index(i as u8);

                if let Ok(register) = register {
                    let par = Paragraph::new(format!("{}: {:08X}", register.0, self.runner.machine().get_reg(&register)))
                        .alignment(Alignment::Center)
                        .on_black()
                        .green();
                    register_widgets.push(par);
                } else {
                    // No register at this index
                    register_widgets.push(
                        Paragraph::new("Other Registers")
                            .cyan()
                            .on_black()
                    );
                }
            }

            let register_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(register_widgets
                    .iter()
                    .map(|_| Constraint::Length(1))
                    .collect::<Vec<_>>()
                )
                .split(left_layout[0]);

            for (i, widget) in register_widgets.into_iter().enumerate() {
                frame.render_widget(
                    widget,
                    register_layout[i],
                );
            }


            let vertical_scroll = self.vertical_scroll as usize;

            if self.vertical_scroll > Machine::MAX_ADDRESS {
                self.vertical_scroll = Machine::MAX_ADDRESS;
            }

            let mut memory = Vec::new();
            let mut found_pc = false;
            for addr in vertical_scroll..vertical_scroll + 16 {
                let mut line = Vec::new();
                line.push(Span::raw(format!("{:06X}", addr * 16)).green());
                for i in 0..16 {
                    line.push(Span::raw(" "));

                    let byte_loc = (addr * 16 + i) as u32;
                    let byte = self.runner.machine().read_byte(byte_loc);

                    let mut span = Span::raw(format!("0x{:02X}", byte));
                    if !found_pc {
                        let pc_loc = self.runner.machine().get_reg(&Register::PC);

                        if byte_loc == pc_loc {
                            span = span.on_light_green().black();
                            found_pc = true;
                        }
                    }
                    line.push(span);
                }
                memory.push(Line::from(line));
            }


            let mut scrollbar_state = ScrollbarState::new(memory.len()).position(vertical_scroll);

            let paragraph = Paragraph::new(memory)
                .block(Block::new().borders(Borders::RIGHT)); // to show a background for the scrollbar

            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            frame.render_widget(paragraph, main_layout[1]);
            frame.render_stateful_widget(scrollbar,
                                         main_layout[1].inner(&Margin {
                                             vertical: 1,
                                             horizontal: 0,
                                         }), // using a inner vertical margin of 1 unit makes the scrollbar inside the block
                                         &mut scrollbar_state);


            // Text display (runner_layout[2])
            let display = Canvas::default()
                .block(Block::default().title("Text Display").borders(Borders::ALL))
                .x_bounds([0.0, 200.0])
                .y_bounds([0.0, 25.0])
                .paint(|ctx| {
                    ctx.draw(&Rectangle {
                        x: 10.0,
                        y: 20.0,
                        width: 10.0,
                        height: 10.0,
                        color: Color::Red,
                    });
                });

            frame.render_widget(display, main_layout[2]);


            if let Err(e) = &self.load_result {
                // Print error to screen
                frame.render_widget(
                    Paragraph::new(e.as_str())
                        .on_red()
                        .black()
                        .block(Block::new().borders(Borders::ALL)),
                    layout[3]);
            } else if let Some(Err(e)) = step {
                // Stop running
                self.mode = Mode::STEP;
                self.load_result = Err(e);
            };


            if self.mode == Mode::LoadFile {
                let popup_block = Block::default()
                    .title("Choose file:")
                    .borders(Borders::NONE)
                    .on_dark_gray();

                let area = TUI::popup(area);

                // Draw files in directory into area
                let files = std::fs::read_dir("./").unwrap();
                let mut files = files.map(|f| f.unwrap().path().display().to_string()).collect::<Vec<_>>();
                files.sort();

                if self.selected_file.is_none() {
                    self.selected_file = Some((files[0].clone(), 0));
                } else if let Some((_, ix)) = &self.selected_file {
                    // Highlight selected file
                    let file = files.get(*ix as usize);
                    if let Some(file) = file {
                        self.selected_file = Some((file.clone(), *ix));
                    } else {
                        self.selected_file = Some((files[0].clone(), 0));
                    }
                }

                let files = files.iter().enumerate().map(|(ix, f)| {
                    let mut style = Style::default().bg(Color::DarkGray);
                    if let Some((_, fix)) = &self.selected_file {
                        if *fix == ix as u8 {
                            style = style.fg(Color::Black);
                        }
                    }
                    Line::from(Span::styled(f.as_str(), style))
                }).collect::<Vec<_>>();

                // Render files
                let files = Paragraph::new(files)
                    .block(Block::default().borders(Borders::ALL))
                    .block(popup_block.clone())
                    .alignment(Alignment::Left)
                    .wrap(Wrap { trim: true });

                frame.render_widget(files, area);

                frame.render_widget(popup_block, area);
            }
        })
    }

    /// helper function to create a centered rect using up certain percentage of the available rect `r`
    fn popup(r: Rect) -> Rect {
        let percent_x = 80;
        let percent_y = 80;
        // Cut the given rectangle into three vertical pieces
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        // Then cut the middle vertical piece into three width-wise pieces
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1] // Return the middle chunk
    }
}