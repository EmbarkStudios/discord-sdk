use crate::Error;

pub fn register_app(app: super::Application) -> Result<(), Error> {
    use super::LaunchCommand;

    fn inner(app: super::Application) -> anyhow::Result<()> {
        use anyhow::Context as _;

        let mut desktop_path = app_dirs2::get_data_root(app_dirs2::AppDataType::UserData)
            .context("Unable to get the user data path")?;

        desktop_path.push("applications");

        std::fs::create_dir_all(&desktop_path)
            .with_context(|| format!("unable to create \"{}\"", desktop_path.display()))?;
        {
            let md = std::fs::metadata(&desktop_path)
                .with_context(|| format!("unable to locate \"{}\"", desktop_path.display()))?;

            anyhow::ensure!(
                md.is_dir(),
                "\"{}\" was found, but it's not a directory",
                desktop_path.display()
            );
        }

        desktop_path.push(format!("discord-{}.desktop", app.id));

        let id = app.id;
        let name = app.name.unwrap_or_else(|| id.to_string());

        std::fs::write(
            &desktop_path,
            format!(
                r#"[Desktop Entry]
        Name={name}
        Exec={cmd}
        Type=Application
        NoDisplay=true
        Categories=Discord;Games;
        MimeType=x-scheme-handler/discord-{id};
        "#,
                name = name,
                cmd = match app.command {
                    LaunchCommand::Url(url) => format!("xdg-open {}", url),
                    LaunchCommand::Bin { path, args } => {
                        // So the docs say we can just use normal quoting rules for
                        // the Exec command https://specifications.freedesktop.org/desktop-entry-spec/desktop-entry-spec-latest.html#exec-variables
                        // but...https://askubuntu.com/questions/189822/how-to-escape-spaces-in-desktop-files-exec-line
                        // seems to indicate things are "more complicated" but we'll
                        // just go with what the spec says for now. Also paths with
                        // spaces are wrong, so just don't do that. ;)
                        super::create_command(path, args, "%u")
                    }
                    LaunchCommand::Steam(steam_id) => {
                        format!("xdg-open steam://rungameid/{}", steam_id)
                    }
                },
                id = id,
            ),
        )
        .context("unable to write desktop entry")?;

        // TODO: Would really rather not shell out to a separate program,
        // ideally would implement this in Rust, C code is located in https://gitlab.freedesktop.org/xdg/desktop-file-utils
        match std::process::Command::new("update-desktop-database")
            .arg(format!("{}", desktop_path.parent().unwrap().display()))
            .status()
            .context("failed to run update-desktop-database")?
            .code()
        {
            Some(0) => {}
            Some(e) => anyhow::bail!("failed to run update-desktop-database: {}", e),
            None => anyhow::bail!("failed to run update-desktop-database, interrupted by signal!"),
        }

        // Register the mime type with XDG, we do it manually rather than via
        // xdg-mime shelling out is lame...ignore the above
        //
        // ~/.config/mimeapps.list                      user overrides
        // /etc/xdg/mimeapps.list                       system-wide overrides
        // ~/.local/share/applications/mimeapps.list    (deprecated) user overrides
        // /usr/local/share/applications/mimeapps.list
        // /usr/share/applications/mimeapps.list

        let mut mime_list_path = app_dirs2::data_root(app_dirs2::AppDataType::UserConfig)
            .context("unable to acquire user config directory")?;
        mime_list_path.push("mimeapps.list");

        let discord_scheme = format!(
            "x-scheme-handler/discord-{id}=discord-{id}.desktop\n",
            id = app.id
        );

        let new_list = if mime_list_path.exists() {
            let mut list = std::fs::read_to_string(&mime_list_path)
                .with_context(|| format!("unable to read {}", mime_list_path.display()))?;

            // Only add the scheme if it doesn't already exist
            if !list.contains(&discord_scheme) {
                list.find("[Default Applications]\n").map(|ind| {
                    list.insert_str(ind + 23, &discord_scheme);
                    list
                })
            } else {
                None
            }
        } else {
            Some(format!("[Default Applications]\n{}", discord_scheme))
        };

        if let Some(new_list) = new_list {
            std::fs::write(&mime_list_path, new_list).with_context(|| {
                format!(
                    "unable to add discord scheme to {}",
                    mime_list_path.display()
                )
            })?;
        }

        Ok(())
    }

    inner(app).map_err(Error::AppRegistration)
}
