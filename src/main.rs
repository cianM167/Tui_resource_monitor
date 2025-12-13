use std::io;
use sysinfo::System;
use crossterm::{event::{KeyCode, KeyEvent}, terminal};
use ratatui::{DefaultTerminal, Frame, layout::{Constraint, Layout, Rect}, style::{Color, Style, Stylize}, symbols::{block, border}, text::Line, widgets::{Block, Gauge, Widget}};

pub struct App {
    exit: bool,
    progress_bar_colour: Color,
}

impl App {
    fn run (&mut self, terminal: &mut DefaultTerminal) -> io::Result<()>{    
        while !self.exit {
            match crossterm::event::read()? {
                crossterm::event::Event::Key(KeyEvent) => self.handle_key_event(KeyEvent)?,
                _ => ()
            }
            terminal.draw(|frame| self.draw(frame))?;
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> io::Result<()> {
        if key_event.kind == crossterm::event::KeyEventKind::Press && key_event.code == KeyCode::Char('q') {
            self.exit = true;
        }
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
        where
            Self: Sized 
        {
            let veritcal_layout = 
                Layout::vertical([Constraint::Percentage(20), Constraint::Percentage(80)]);
            let [title_area, gauge_area] = veritcal_layout.areas(area);
            
            //render title 
            Line::from("process overview")
                .bold()
                .centered()
                .render(title_area, buf);

            let instructions = Line::from(vec![
                " Change colour ".into(),
                "<C>".blue().bold().into(),
                " Quit".into(),
                "<Q>".red().bold(),
            ]).centered();

            let block = Block::bordered()
                .title(Line::from(" Background processes "))
                .title_bottom(instructions)
                .border_set(border::THICK);

            let progress_bar = Gauge::default()
                .gauge_style(Style::default().fg(self.progress_bar_colour))
                .block(block)
                .label(format!("Process 1: 50%"))
                .ratio(0.5);

            progress_bar.render(Rect {
                x: gauge_area.left(),
                y: gauge_area.top(),
                width: gauge_area.width,
                height: 3,
                }, 
                buf
                );

        }
}

fn cpu_data() {
    let mut sys = System::new();

    loop {
        sys.refresh_cpu_usage(); // Refreshing CPU usage.
        for (i, cpu) in sys.cpus().iter().enumerate() {
            println!("cpu_{}:{}% ", i , cpu.cpu_usage());
        }
        // Sleeping to let time for the system to run for long
        // enough to have useful information.
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    }
}


fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();

    let mut app = App { 
        exit: false,
        progress_bar_colour: Color::Cyan,
    };

    let app_result = app.run(&mut terminal);

    ratatui::restore();
    app_result

}
