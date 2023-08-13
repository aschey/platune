use clap::builder::styling;
use daemon_slayer::build_info::cli::BuildInfoCliProvider;
use daemon_slayer::build_info::vergen_pretty::{self, vergen_pretty_env, PrettyBuilder};
use daemon_slayer::build_info::{self};
use daemon_slayer::core::Label;

pub const MAIN_SERVER_PORT: usize = 50051;

pub fn clap_base_command() -> clap::Command {
    clap::Command::default().styles(
        styling::Styles::styled()
            .header(
                styling::Style::default()
                    .bold()
                    .fg_color(Some(styling::Color::Ansi(styling::AnsiColor::Blue))),
            )
            .placeholder(styling::Style::default().dimmed()),
    )
}

pub fn build_info() -> BuildInfoCliProvider {
    let config = PrettyBuilder::default()
        .env(vergen_pretty_env!())
        .key_style(
            vergen_pretty::Style::default()
                .fg(build_info::Color::Cyan)
                .bold(),
        )
        .value_style(vergen_pretty::Style::default())
        .category(false)
        .build()
        .unwrap();

    BuildInfoCliProvider::new(config)
}

pub fn service_label() -> Label {
    "com.platune.platuned"
        .parse()
        .expect("Label failed to parse")
}
