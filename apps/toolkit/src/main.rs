use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{io, time::Duration};

mod app;
mod event_handler;
mod events;
mod jobs;
mod renderer;
mod update;

use app::App;
use update::update;

// --------------------------------------------------
// MAIN
// --------------------------------------------------

fn main() -> Result<()> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();

    if let Err(error) = execute!(stdout, EnterAlternateScreen) {
        disable_raw_mode()?;
        return Err(error.into());
    }

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

// --------------------------------------------------
// EVENT LOOP
// --------------------------------------------------

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::new();

    while app.running {
        app.process_job_messages();
        terminal.draw(|frame| renderer::render(frame, &app))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if let Some(event) = event_handler::handle_key_event(&app, key) {
                    update(&mut app, event);
                } else if app.screen == app::Screen::DayPilot {
                    if let Some(event) = daypilot::handle_key_event(&app.daypilot, key.code) {
                        update(&mut app, events::AppEvent::DayPilot(event));
                    }
                }
            }
        }
    }

    app.shutdown_jobs();

    Ok(())
}

// --------------------------------------------------
// LEGACY MAIN RENDERING — replaced by renderer.rs
// --------------------------------------------------
#[cfg(any())]
mod legacy_render {
    use super::*;

    // --------------------------------------------------
    // MAIN RENDER
    // --------------------------------------------------

    fn render(frame: &mut Frame, app: &App) {
        let palette = Palette::from(app.theme);

        frame.render_widget(
            Block::default().style(Style::default().bg(palette.background)),
            frame.area(),
        );

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(frame.area());

        render_header(frame, layout[0], &palette);

        match app.screen {
            Screen::Launcher => {
                render_launcher(frame, layout[1], app, &palette);
            }

            Screen::DayPilot => {
                let colors = palette.daypilot_colors();

                daypilot::render(&app.daypilot, frame, layout[1], &colors);
            }

            Screen::Settings => {
                render_settings(frame, layout[1], app, &palette);
            }
        }

        render_footer(frame, layout[2], app, &palette);
    }

    // --------------------------------------------------
    // HEADER
    // --------------------------------------------------

    fn render_header(frame: &mut Frame, area: Rect, palette: &Palette) {
        let header = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    "◆ ",
                    Style::default()
                        .fg(palette.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "PERSONAL TOOLKIT",
                    Style::default()
                        .fg(palette.primary_text)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("                                      "),
                Span::styled("● SYSTEM READY", Style::default().fg(palette.accent)),
            ]),
            Line::from(Span::styled(
                "  YOUR PERSONAL WORKSPACE",
                Style::default().fg(palette.muted_text),
            )),
        ])
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(palette.border)),
        );

        frame.render_widget(header, area);
    }

    // --------------------------------------------------
    // LAUNCHER
    // --------------------------------------------------

    fn render_launcher(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);

        render_menu(frame, columns[0], app, palette);
        render_application_details(frame, columns[1], app, palette);
    }

    fn render_menu(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
        let items: Vec<ListItem> = app
            .menu_items
            .iter()
            .map(|item| {
                let status = if item.available {
                    Span::styled("READY", Style::default().fg(palette.accent))
                } else {
                    Span::styled("SOON", Style::default().fg(palette.muted_text))
                };

                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(
                            format!("{}  ", item.icon),
                            Style::default().fg(palette.secondary_accent),
                        ),
                        Span::styled(
                            item.title,
                            Style::default()
                                .fg(palette.primary_text)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw("   "),
                        status,
                    ]),
                    Line::from(Span::styled(
                        format!("   {}", item.description),
                        Style::default().fg(palette.muted_text),
                    )),
                    Line::from(""),
                ])
            })
            .collect();

        let menu = List::new(items)
            .block(
                Block::default()
                    .title(Span::styled(
                        " APPLICATIONS ",
                        Style::default()
                            .fg(palette.accent)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::RIGHT)
                    .border_style(Style::default().fg(palette.border))
                    .style(Style::default().bg(palette.background)),
            )
            .highlight_style(
                Style::default()
                    .bg(palette.selected_bg)
                    .fg(palette.primary_text)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("  ");

        let mut state = ListState::default();
        state.select(Some(app.selected));

        frame.render_stateful_widget(menu, area, &mut state);
    }

    // --------------------------------------------------
    // APPLICATION DETAILS
    // --------------------------------------------------

    fn render_application_details(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
        let item = &app.menu_items[app.selected];

        let features: Vec<&str> = match item.action {
            ToolkitAction::DayPilot => vec![
                "Organise tasks and projects",
                "Track deadlines and priorities",
                "Manage daily progress",
                "Review unfinished tasks",
            ],

            ToolkitAction::WebScout => vec![
                "Crawl websites",
                "Collect useful information",
                "Organise research results",
            ],

            ToolkitAction::SysPeek => vec![
                "Monitor CPU usage",
                "Track memory consumption",
                "Inspect system resources",
            ],

            ToolkitAction::BackgroundJobs => vec![
                "View running operations",
                "Track job progress",
                "Stop individual background jobs",
            ],

            ToolkitAction::Settings => vec![
                "Change application appearance",
                "Choose your preferred colour theme",
            ],

            ToolkitAction::Exit => vec![
                "Close Personal Toolkit",
                "Stop managed background operations",
            ],
        };

        let mut lines = vec![
            Line::from(Span::styled(
                "APPLICATION OVERVIEW",
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                item.icon,
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                item.title,
                Style::default()
                    .fg(palette.primary_text)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                item.description,
                Style::default().fg(palette.muted_text),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "FEATURES",
                Style::default()
                    .fg(palette.secondary_accent)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];

        for feature in features {
            lines.push(Line::from(vec![
                Span::styled("  ◈  ", Style::default().fg(palette.accent)),
                Span::styled(feature, Style::default().fg(palette.secondary_text)),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(""));

        if item.available {
            lines.push(Line::from(Span::styled(
                match item.action {
                    ToolkitAction::Settings => "ENTER  →  OPEN SETTINGS",
                    ToolkitAction::Exit => "ENTER  →  EXIT TOOLKIT",
                    _ => "ENTER  →  LAUNCH",
                },
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                "THIS APPLICATION IS NOT AVAILABLE YET",
                Style::default().fg(palette.muted_text),
            )));
        }

        let details = Paragraph::new(lines).wrap(Wrap { trim: true }).block(
            Block::default()
                .style(Style::default().bg(palette.panel))
                .padding(ratatui::widgets::Padding::uniform(2)),
        );

        frame.render_widget(details, area);
    }

    // --------------------------------------------------
    // SETTINGS
    // --------------------------------------------------

    fn render_settings(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
        let current_theme = match app.theme {
            Theme::BurntEmber => "Burnt Ember",
            Theme::HighContrast => "High Contrast",
        };

        let theme_description = match app.theme {
            Theme::BurntEmber => {
                "A subtle copper-orange theme with soft text and matte black panels."
            }

            Theme::HighContrast => {
                "A brighter orange theme with stronger contrast and more vivid accents."
            }
        };

        let lines = vec![
            Line::from(Span::styled(
                "PERSONAL TOOLKIT SETTINGS",
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "APPEARANCE",
                Style::default()
                    .fg(palette.secondary_accent)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Select application theme",
                Style::default().fg(palette.primary_text),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    if app.theme == Theme::BurntEmber {
                        "● "
                    } else {
                        "○ "
                    },
                    Style::default().fg(palette.accent),
                ),
                Span::styled("Burnt Ember", Style::default().fg(palette.secondary_text)),
                Span::raw("     "),
                Span::styled(
                    if app.theme == Theme::HighContrast {
                        "● "
                    } else {
                        "○ "
                    },
                    Style::default().fg(palette.accent),
                ),
                Span::styled("High Contrast", Style::default().fg(palette.secondary_text)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                format!("CURRENT THEME: {}", current_theme),
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                theme_description,
                Style::default().fg(palette.muted_text),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Use LEFT / RIGHT or ENTER to change the theme.",
                Style::default().fg(palette.secondary_text),
            )),
        ];

        let settings = Paragraph::new(lines).wrap(Wrap { trim: true }).block(
            Block::default()
                .style(Style::default().bg(palette.panel))
                .padding(ratatui::widgets::Padding::uniform(3)),
        );

        frame.render_widget(settings, area);
    }

    // --------------------------------------------------
    // FOOTER
    // --------------------------------------------------

    fn render_footer(frame: &mut Frame, area: Rect, app: &App, palette: &Palette) {
        let footer_line = match app.screen {
            Screen::Launcher => Line::from(vec![
                Span::styled("↑ ↓ ", Style::default().fg(palette.accent)),
                Span::raw("Navigate   "),
                Span::styled("ENTER ", Style::default().fg(palette.accent)),
                Span::raw("Select   "),
                Span::styled("Q ", Style::default().fg(palette.accent)),
                Span::raw("Exit"),
            ]),

            Screen::DayPilot => Line::from(vec![
                Span::styled("↑ ↓ ", Style::default().fg(palette.accent)),
                Span::raw("Navigate   "),
                Span::styled("SPACE ", Style::default().fg(palette.accent)),
                Span::raw("Toggle completion   "),
                Span::styled("Q ", Style::default().fg(palette.accent)),
                Span::raw("Back to Toolkit"),
            ]),

            Screen::Settings => Line::from(vec![
                Span::styled("← → ", Style::default().fg(palette.accent)),
                Span::raw("Change Theme   "),
                Span::styled("ESC ", Style::default().fg(palette.accent)),
                Span::raw("Back"),
            ]),
        };

        let footer = Paragraph::new(footer_line)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(palette.border)),
            );

        frame.render_widget(footer, area);
    }
}
