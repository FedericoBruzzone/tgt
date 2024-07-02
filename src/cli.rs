use clap::Parser;
// use clap::Subcommand;

/// The CLI arguments for the application.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[command(flatten)]
    telegram_cli: TelegramCli,
    // #[command(subcommand)]
    // telegram: Option<Telegram>,
}

impl CliArgs {
    /// Get the Telegram CLI arguments.
    pub fn telegram_cli(&self) -> &TelegramCli {
        &self.telegram_cli
    }
}

#[derive(Parser, Debug)]
/// The Telegram commands.
pub struct TelegramCli {
    #[arg(
        short,
        long,
        visible_alias = "lo",
        help = "Logout from the tgt",
        default_value_t = false
    )]
    logout: bool,

    #[arg(
        short,
        long,
        visible_alias = "sm",
        number_of_values = 2,
        value_names = &["CHAT_NAME", "MESSAGE"],
        help = "Send a message to a chat"
    )]
    send_message: Option<Vec<String>>,
}

impl TelegramCli {
    /// Get the logout flag.
    pub fn logout(&self) -> bool {
        self.logout
    }
    /// Get the send message arguments.
    pub fn send_message(&self) -> Option<&Vec<String>> {
        self.send_message.as_ref()
    }
}

// #[derive(Parser, Debug)]
// // #[derive(Subcommand, Debug)]
// pub enum Telegram {
//     Test(TelegramStartSubcommand),
//     Add { name: Option<String> }
// }
