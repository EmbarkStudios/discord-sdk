use crate::Error;

pub fn register_app(app: super::Application) -> Result<(), Error> {
    use super::LaunchCommand;

    fn inner(app: super::Application) -> anyhow::Result<()> {
        use anyhow::Context as _;
        use winreg::{enums::HKEY_CURRENT_USER, RegKey};

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);

        let mut icon_path = std::env::current_exe()
            .context("unable to retrieve current executable path")?
            .into_os_string();

        let command = match app.command {
            LaunchCommand::Bin { path, args } => {
                icon_path = path.clone().into();

                super::create_command(path, args, "\"%1\"")?
            }
            LaunchCommand::Url(url) => {
                // Unfortunately it doesn't seem like we can just forward one
                // url to another, so we actually have to lookup the command
                // for the registered handler for the scheme and copy it
                let handler = format!(r#"Software\Classes\{}\shell\open\command"#, url.scheme());

                let key = hkcu.open_subkey(&handler).with_context(|| {
                    format!("the '{}' scheme hasn't been registered", url.scheme())
                })?;

                let command: String = key.get_value("").with_context(|| {
                    format!("unable to read value for '{}' scheme", url.scheme())
                })?;

                // The registered scheme handler should be pointing at an executable
                // so we retrieve that to use as the icon path, instead of the default
                // of using the path of the current executable
                let exe_path = match command.strip_prefix('"') {
                    Some(cmd) => {
                        match cmd.find('"') {
                            Some(ind) => cmd[..ind].to_owned(),
                            None => {
                                // If there's not a closing quote just assume something
                                // is wrong and return the whole string
                                command.clone()
                            }
                        }
                    }
                    None => command.split(' ').next().unwrap().to_owned(),
                };

                icon_path = exe_path.into();
                command
            }
            LaunchCommand::Steam(steam_id) => {
                let key = hkcu
                    .open_subkey(r#"Software\Valve\Steam"#)
                    .context("unable to locate Steam registry entry")?;

                let steam_path: String = key
                    .get_value("SteamExe")
                    .context("unable to locate path to steam executable")?;

                // The Discord RPC lib does this, but seems a bit weird that
                // Steam would potentially write it in a way that would break
                // random stuff requiring windows path separators, but who knows!
                let steam_path = steam_path.replace('/', "\\");

                format!(r#""{}" steam://rungameid/{}"#, steam_path, steam_id)
            }
        };

        let id = app.id;
        let discord_handler = format!(r#"Software\Classes\discord-{}"#, id);
        let (disc_key, _disp) = hkcu
            .create_subkey(&discord_handler)
            .context("unable to create discord handler")?;

        let name = app.name.unwrap_or_else(|| id.to_string());

        disc_key.set_value("", &format!("URL:Run {} protocol", name))?;
        disc_key.set_value("URL Protocol", &"")?;
        icon_path.push(",0");
        disc_key.set_value("DefaultIcon", &&*icon_path)?;

        let (open_key, _disp) = disc_key
            .create_subkey(r#"shell\open\command"#)
            .context("unable to create open key")?;
        open_key.set_value("", &command)?;

        Ok(())
    }

    inner(app).map_err(Error::AppRegistration)
}
