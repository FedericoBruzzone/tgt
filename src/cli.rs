// use clap::ArgGroup;
use clap::Parser;
use clap::Subcommand;

/// The CLI arguments for the application.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// It allows to logout from the tgt
    #[arg(short, long, help = "Logout from the tgt")]
    logout: bool,

    #[arg(short, long, number_of_values = 2, value_names = &["CHAT_NAME", "MESSAGE"], help = "Send a message to a chat", alias = "sm")]
    send_message: Option<Vec<String>>,

    #[command(subcommand)]
    telegram: Option<TelegramSubcommand>,
    // /// Number of times to greet
    // #[arg(short, long, default_value_t = 1)]
    // count: u8,
}

/// The subcommands for the telegram command.
// #[derive(Parser, Debug)]
#[derive(Subcommand, Debug)]
pub enum TelegramSubcommand {
    /// The subcommand to start the telegram bot.
    Test(TelegramStartSubcommand),
}

/// The subcommand to start the telegram bot.
#[derive(Parser, Debug)]
pub struct TelegramStartSubcommand {
    /// The token for the telegram bot.
    #[arg(short, long)]
    token: String,
}
