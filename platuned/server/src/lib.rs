use std::env;
use std::num::ParseIntError;

use clap::builder::styling;
use daemon_slayer::build_info::cli::BuildInfoCliProvider;
use daemon_slayer::build_info::vergen_pretty::{self, PrettyBuilder, vergen_pretty_env};
use daemon_slayer::core::Label;

const DEFAULT_MAIN_SERVER_PORT: usize = 50051;
const DEFAULT_FILE_SERVER_PORT: usize = 50050;

pub fn main_server_port() -> Result<usize, ParseIntError> {
    Ok(match env::var("PLATUNE_SERVER_PORT") {
        Ok(port) => port.parse()?,
        Err(_) => DEFAULT_MAIN_SERVER_PORT,
    })
}

pub fn file_server_port() -> Result<usize, ParseIntError> {
    Ok(match env::var("PLATUNE_FILE_SERVER_PORT") {
        Ok(port) => port.parse()?,
        Err(_) => DEFAULT_FILE_SERVER_PORT,
    })
}

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
                .fg(console::Color::Cyan)
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
