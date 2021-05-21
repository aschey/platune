use argh::FromArgs;
use crossterm::{
    cursor,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode, KeyEvent},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io::stdout,
    process::Stdio,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Corner, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Gauge, List, ListItem, Paragraph, Wrap},
    Terminal,
};

enum Event<I> {
    Input(I),
    Tick,
}

/// Crossterm demo
#[derive(Debug, FromArgs)]
struct Cli {
    /// time in ms between two ticks.
    #[argh(option, default = "250")]
    tick_rate: u64,
    /// whether unicode symbols are used to improve the overall look of the app
    #[argh(option, default = "true")]
    enhanced_graphics: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli: Cli = argh::from_env();

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;

    // execute!(
    //     std::io::stdout(),
    //     Print(std::str::from_utf8(&output.stdout).unwrap())
    // )
    // .unwrap();

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;

    // Setup input handling
    let (tx, rx) = mpsc::channel();
    let (start_tx, start_rx) = mpsc::channel();

    //thread::sleep(Duration::from_secs(5));

    let tick_rate = Duration::from_millis(cli.tick_rate);

    thread::spawn(move || {
        start_rx.recv().unwrap();
        let mut last_tick = Instant::now();
        loop {
            // // poll for tick rate duration, if no events, sent tick event.
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    tx.send(Event::Input(key)).unwrap();
                }
            }
            if last_tick.elapsed() >= tick_rate {
                // std::process::Command::new("clear").status().unwrap();
                tx.send(Event::Tick).unwrap();
                last_tick = Instant::now();
            }
        }
    });

    //let mut app = App::new("Crossterm Demo", cli.enhanced_graphics);

    terminal.clear()?;

    let mut i = 0.;
    loop {
        terminal.draw(|f| {
            //if i == 0 || i == 10 {
            let conf = viuer::Config {
                // set offset
                x: 2,
                y: 1,
                // set dimensions
                width: Some(12),
                height: Some(6),
                ..Default::default()
            };

            //execute!(std::io::stdout(), cursor::MoveTo(2, 1)).unwrap();
            let mut img = "/home/aschey/test.jpeg";
            if i >= 10. {
                img = "/home/aschey/test2.jpeg"
            }
            // let output = std::process::Command::new("kitty")
            //     .args(&["+kitten", "icat", "/home/aschey/test.jpeg"])
            //     .stdout(Stdio::null())
            //     .stderr(Stdio::null())
            //     .output()
            //     .unwrap();
            viuer::print_from_file(img, &conf).unwrap();
            start_tx.send(()).unwrap();
            // let output = std::process::Command::new("wezterm")
            //     .arg("imgcat")
            //     .arg(img)
            //     .output()
            //     .unwrap();
            // let output = std::process::Command::new("viu")
            // .arg(img);
            //     .arg("+kitten")
            //     .arg("icat")
            //     .arg(img)
            //     .output()
            //     .unwrap();
            // execute!(
            //     std::io::stdout(),
            //     Print(std::str::from_utf8(&output.stdout).unwrap())
            // )
            // .unwrap();
            //}
            let mut bounds = f.size();
            //println!("{:?}", bounds);

            // let top = Rect::new(bounds.x, bounds.y + 1, bounds.width, 7);

            // let middle = Rect::new(bounds.x + 2, bounds.y + 7, bounds.width - 4, 1);
            //let bottom = Rect::new(bounds.x, bounds.y + 110, bounds.width, bounds.height - 110);

            let mut vert_chunks = Layout::default()
                .direction(Direction::Vertical)
                //.margin(1)
                .constraints(
                    [
                        Constraint::Length(7),
                        Constraint::Length(1),
                        //Constraint::Length(1),
                        Constraint::Length(bounds.height - 8),
                    ]
                    .as_ref(),
                )
                .split(f.size());
            // println!("{:?}", vert_chunks);

            let horiz_chunks = Layout::default()
                .direction(Direction::Horizontal)
                //.margin(1)
                .constraints(
                    [
                        Constraint::Length(12),
                        Constraint::Length(bounds.width - 12),
                    ]
                    .as_ref(),
                )
                .split(vert_chunks[0]);
            let mut horiz_chunks2 = Layout::default()
                .direction(Direction::Horizontal)
                //.margin(1)
                .constraints([Constraint::Max(50), Constraint::Max(0)].as_ref())
                .split(vert_chunks[1]);
            let horiz_chunks3 = Layout::default()
                .direction(Direction::Horizontal)
                //.margin(1)
                .constraints([Constraint::Max(50), Constraint::Max(0)].as_ref())
                .split(vert_chunks[2]);

            let gauge = Gauge::default()
                .block(Block::default())
                .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
                .use_unicode(true)
                .ratio(i / 100.);

            let items = [
                ListItem::new(Span::from("\n")),
                ListItem::new(Spans::from(vec![
                    Span::styled("    ", Style::default().fg(Color::Blue)),
                    Span::styled("Snow", Style::default().add_modifier(Modifier::BOLD)),
                ])),
                ListItem::new(Spans::from(vec![
                    Span::styled("    ", Style::default().fg(Color::Red)),
                    Span::styled("Red Hot Chili Peppers", Style::default()),
                ])),
                ListItem::new(Spans::from(vec![
                    Span::styled("    ", Style::default().fg(Color::LightCyan)),
                    Span::styled(
                        "Stadium Arcadium",
                        Style::default().add_modifier(Modifier::ITALIC),
                    ),
                ])),
                ListItem::new(Span::from("   祥[1:34/3:14]")),
                ListItem::new(Span::from("\n")),
                ListItem::new(Spans::from(vec![
                    Span::styled("        ", Style::default().fg(Color::Blue)),
                    Span::styled("  ", Style::default().fg(Color::Green)),
                    Span::styled("  ", Style::default().fg(Color::Yellow)),
                    Span::styled("  ", Style::default().fg(Color::Red)),
                    Span::styled("", Style::default().fg(Color::Blue)),
                ])),
            ];
            let p = List::new(items).style(Style::default().fg(Color::White));

            let controls = [
                // ListItem::new(Span::from("\n")),
                ListItem::new(Spans::from(vec![
                    Span::styled(
                        "                         ",
                        Style::default().fg(Color::Blue),
                    ),
                    Span::styled("  ", Style::default().fg(Color::Green)),
                    Span::styled("  ", Style::default().fg(Color::Yellow)),
                    Span::styled("  ", Style::default().fg(Color::Red)),
                    Span::styled("", Style::default().fg(Color::Blue)),
                    // ▬●
                ])),
                ListItem::new(Span::from("\n")),
            ];
            let p2 = List::new(controls).style(Style::default().fg(Color::White));
            //.start_corner(Corner::TopLeft);
            //.highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            //.highlight_symbol(">>");

            //execute!(std::io::stdout(), cursor::MoveTo(0, 0)).unwrap();
            //println!("{:?}", horiz_chunks2);
            // vert_chunks[1].width = 25;
            // vert_chunks[1].x = 17;

            // horiz_chunks2[0].x = 20;
            // horiz_chunks2[0].width -= 20;
            f.render_widget(p, horiz_chunks[1]);
            f.render_widget(
                Block::default().title("[━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━-----] "),
                horiz_chunks2[0],
            );
            //f.render_widget(p2, horiz_chunks2[0]);
            if i < 99. {
                i += 0.3;
            }
        })?;
        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    )?;
                    terminal.show_cursor()?;
                    break;
                }
                // KeyCode::Char(c) => app.on_key(c),
                // KeyCode::Left => app.on_left(),
                // KeyCode::Up => app.on_up(),
                // KeyCode::Right => app.on_right(),
                // KeyCode::Down => app.on_down(),
                _ => {}
            },
            Event::Tick => {
                //app.on_tick();
            }
        }
        // if app.should_quit {
        //     break;
        // }
    }

    Ok(())
}
