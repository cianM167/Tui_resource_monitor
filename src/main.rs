use std::{io, sync::mpsc, thread, time::Duration};
use sysinfo::System;
use crossterm::{event::{KeyCode, KeyEvent}, terminal};
use ratatui::{DefaultTerminal, Frame, layout::{Constraint, Layout, Rect}, style::{Color, Style, Stylize}, symbols::{block, border}, text::Line, widgets::{Block, Gauge, Widget}};

enum Event {
    Input(crossterm::event::KeyEvent),
    Progress(f64),

}

pub struct App {
    exit: bool,
    progress_bar_colour: Color,
    background_progress: f64,
}

impl App {
    fn run (&mut self, terminal: &mut DefaultTerminal, rx: mpsc::Receiver<Event>) -> io::Result<()>{    
        while !self.exit {
            match rx.recv().unwrap() {
                Event::Input(key_event) => self.handle_key_event(key_event)?,
                Event::Progress(progress) => self.background_progress = progress,
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
                .label(format!("Process 1: {:.2}%", self.background_progress * 100_f64))
                .ratio(self.background_progress);

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

fn handle_input_events(tx: mpsc::Sender<Event>) {
    loop {
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(Key_event) => tx.send(Event::Input(Key_event)).unwrap(),
            _ => ()
        }
    }
}

fn run_background_thread(tx: mpsc::Sender<Event>) {
    let mut progress = 0_f64;
    let increment = 0.01_f64;
    loop {
        thread::sleep(Duration::from_millis(100));
        progress += increment;
        progress = progress.min(1_f64);
        tx.send(Event::Progress(progress)).unwrap();
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
        background_progress: 0_f64,
    };

    let (event_tx, event_rx) = mpsc::channel::<Event>();

    let tx_to_input_events = event_tx.clone();
    thread::spawn(move || {
        handle_input_events(tx_to_input_events);
    });

    let tx_to_progress_events = event_tx;
    thread::spawn(move || {
        run_background_thread(tx_to_progress_events);
    });

    let app_result = app.run(&mut terminal, event_rx);

    ratatui::restore();
    app_result

}
