use std::{io, sync::mpsc, thread, time::Duration};
use sysinfo::System;
use color_eyre::{Result, owo_colors::OwoColorize};
use crossterm::{event::{KeyCode, KeyEvent}, terminal};
use ratatui::{DefaultTerminal, Frame, layout::{Constraint, Direction, Layout, Rect}, style::{Color, Style, Stylize}, symbols::{bar, block, border}, text::Line, widgets::{Bar, BarChart, BarGroup, Block, Gauge, Widget}};

enum Event {
    Input(crossterm::event::KeyEvent),
    Progress(Vec<f64>),
}

pub struct App {
    exit: bool,
    progress_bar_colour: Color,
    background_progress: Vec<f64>,
    cpu_brand: String,
}

impl App {
    fn new() -> Self {
        let mut sys = System::new_all();
        Self { 
            exit: false, 
            progress_bar_colour: Color::Cyan, 
            background_progress: vec![],
            cpu_brand: sys.cpus()[0].brand().to_string(),
        }
    }

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
        let [title, vertical] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .spacing(1)
        .areas(frame.area());

        frame.render_widget("Monitor".bold().into_centered_line(), title);
        frame.render_widget(vertical_barchart(&self.background_progress, self.cpu_brand.clone()), vertical);
    }

    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> io::Result<()> {
        if key_event.kind == crossterm::event::KeyEventKind::Press && key_event.code == KeyCode::Char('q') {
            self.exit = true;
        }
        Ok(())
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

fn vertical_barchart(temperatures: &[f64], cpu_brand: String) -> BarChart {
    //make vec of bars
    let bars: Vec<Bar> = temperatures
        .iter()
        .enumerate()
        .map(|(cpu, value)| vertical_bar(cpu, value))
        .collect();

    let block = Block::bordered()
        .title(Line::from(format!(" {}: ", cpu_brand)))
        .border_set(border::THICK);

    BarChart::default()
        .data(BarGroup::default().bars(&bars))
        .block(block)
        .bar_width(5)
}

fn vertical_bar(cpu: usize, usage: &f64) -> Bar {
    Bar::default()
        .style(Style::default().fg(Color::Cyan))
        .value(u64::from(*usage as u64))
        .label(Line::from(format!("Cpu{}:", cpu+1)))
}

fn run_background_thread(tx: mpsc::Sender<Event>) {
    let mut sys = System::new_all();
    let alpha = 0.2;
    //let cores = sys.cpus();
    //println!("{:?}", cores.len());
    let mut cores_usages: Vec<f64> = vec![0_f64; sys.cpus().len()];

    loop {
        sys.refresh_cpu_usage(); // Refreshing CPU usage.
        for (i, cpu) in sys.cpus().iter().enumerate() {

            cores_usages[i] = (alpha * cpu.cpu_usage() as f64) + (1.0 - alpha) * cores_usages[i];
            //println!("cpu_{}:{}% ", i , cpu.cpu_usage());
        }
        // Sleeping to let time for the system to run for long
        // enough to have useful information.
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        //println!();
        tx.send(Event::Progress(cores_usages.clone())).unwrap();
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
    let (event_tx, event_rx) = mpsc::channel::<Event>();

    let tx_to_input_events = event_tx.clone();
    thread::spawn(move || {
        handle_input_events(tx_to_input_events);
    });

    let tx_to_progress_events = event_tx;
    thread::spawn(move || {
        run_background_thread(tx_to_progress_events);
    });

    let mut terminal = ratatui::init();
    let app_result = App::new().run(&mut terminal, event_rx);

    ratatui::restore();
    app_result

}

fn temperature_style(value: u8) -> Style {
    let green = (255.0 * (1.0 - f64::from(value - 50) / 40.0)) as u8;
    let color = Color::Rgb(255, green, 0);
    Style::new().fg(color)
}
