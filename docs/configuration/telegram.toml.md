# telegram.toml

## Default telegram configuration

```toml
# Application identifier for Telegram API access, which can be obtained at https:my.telegram.org
api_id = "94575"
# Application identifier hash for Telegram API access, which can be obtained at https:my.telegram.org
api_hash = "a3406de8d171bb422bb6ddf3bbd800e2"
# The path to the directory for the persistent database; if empty, the current working directory will be used
# If is not overridden, the database will be in the `tgt` directory.
# In Linux and MacOS, the path is:
# $HOME/tgt/.data/tg
# In Windows, the path is:
# C:\Users\YourUsername\tgt\.data\tg
database_dir = ".data/tg"
# Pass true to keep information about downloaded and uploaded files between application restarts
use_file_database = true
# Pass true to keep cache of users, basic groups, supergroups, channels and secret chats between restarts. Implies use_file_database
use_chat_info_database = true
# Pass true to keep cache of chats and messages between restarts. Implies use_chat_info_database
use_message_database = true
# IETF language tag of the user's operating system language; must be non-empty
system_language_code = "en"
# Model of the device the application is being run on; must be non-empty
device_model = "Desktop"
# =========== logging ===========
# New value of the verbosity level for logging.
# Value 0 corresponds to fatal errors,
# value 1 corresponds to errors,
# value 2 corresponds to warnings and debug warnings,
# value 3 corresponds to informational,
# value 4 corresponds to debug,
# value 5 corresponds to verbose debug,
# value greater than 5 and up to 1023 can be used to enable even more logging
verbosity_level = 2
# Path to the file to where the internal TDLib log will be written
# If is not overridden, the log will be in the `tgt` directory.
# In Linux and MacOS, the path is:
# $HOME/tgt/.data/tdlib_rs/tdlib_rs.log
# In Windows, the path is:
# C:\Users\YourUsername\tgt\.data\tdlib_rs\tdlib_rs.log
log_path = ".data/tdlib_rs/tdlib_rs.log"
# Pass true to additionally redirect stderr to the log file. Ignored on Windows
redirect_stderr = false
```

## Custom configuration

### How create a custom configuration file

`tgt` by default reads its **default** configurations from:
- Linux: `/home/<name>/.tgt/config/`
- macOS: `/Users/<name>/.tgt/config/`
- Windows: `C:\Users\<name>\.tgt\config/`

We suggest you to not modify this file, but to create your own **custom** configuration file in the following directories (in order of precedence):

- `$TGT_CONFIG_DIR` (if set)
- `$HOME/.config/tgt/` (for Linux and macOS) and `C:\Users\<name>\AppData\Roaming\tgt\` (for Windows)

Reading configurations from the following directories will override the fields defined in the default configuration files.
It means that the fields that are not present in the custom configuration will be taken from the default configuration, while the fields that are present in the custom configuration will override the default configuration.
Note that after the finding the first configuration file, `tgt` stops looking for more configurations, it is short-circuited.

### Example of a custom telegram configuration

Generally, end-users are expected to supply their own api\_id and api\_hash.
While some open-source clients, including `tgt`, may provide "default" credentials to make the "out-of-the-box" experience smoother, shipping shared credentials carries risks.
The [`telegram.toml.md`](https://github.com/FedericoBruzzone/tgt/tree/main/docs/configuration/telegram.toml.md) file contains more information about this topic.


Example of `telegram.toml`:

```toml
api_id = "<your_api_id>"
api_hash = "<your_api_hash>"
```
