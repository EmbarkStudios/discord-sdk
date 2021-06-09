use super::LaunchCommand;
use anyhow::{self, ensure, Context as _};
use std::path::PathBuf;

pub fn register_app(app: super::Application) -> anyhow::Result<()> {
    fn inner(app: super::Application) -> anyhow::Result<(), anyhow::Error> {
        match app.command {
            LaunchCommand::Url(url) => {
                create_shim(app.id, url.into())?;
            }
            LaunchCommand::Steam(steam_id) => {
                create_shim(app.id, format!("steam://rungameid/{}", steam_id))?;
            }
            LaunchCommand::Bin { path, args } => {
                let script = make_script(path, args)?;

                let app_path = PathBuf::from(format!("/Applications/discord-{}.app", app.id));

                let script_hash = {
                    // simple djb2 hash https://theartincode.stanis.me/008-djb2/
                    let mut hash = 5381u32;
                    for byte in script.as_bytes() {
                        hash = hash
                            .overflowing_shl(5)
                            .0
                            .overflowing_add(hash)
                            .0
                            .overflowing_add(*byte as u32)
                            .0;
                    }
                    hash
                };

                // Check to see if we've already got an app that is alread up to
                // date, or if we need to create/overwrite it
                if let Some(plist_path) = needs_overwrite(script_hash, app.id, &app_path) {
                    if app_path.exists() {
                        std::fs::remove_dir_all(&app_path).with_context(|| {
                            format!("unable to remove '{}'", app_path.display())
                        })?;
                    }

                    // osacompile doesn't seem to support stdin input (at least, from the man page
                    // I found on the internet) so we need to write the script to temp
                    // file in order to compile it
                    let mut script_path = std::env::temp_dir();
                    script_path.push(format!("{}.applescript", script_hash));

                    std::fs::write(&script_path, &script).context("Couldn't write script file")?;

                    // compile the AppleScript to a .app application
                    let output = std::process::Command::new("osacompile")
                        .arg("-o")
                        .arg(&app_path)
                        .arg(&script_path)
                        .output()
                        .context("Couldn't compile script to app")?;

                    ensure!(
                        output.status.success(),
                        "osacompile failed with status {}: {}",
                        output.status,
                        std::str::from_utf8(&output.stderr)
                            .context("Couldn't convert osacompile error output")?
                    );
                    ensure!(
                        app_path.exists(),
                        "osacompile appeared to succeed but didn't actually write an app"
                    );

                    // overwrite the .app Info.plist file with our own that contains
                    // the correct name of our application as well as the URL scheme
                    std::fs::write(
                        &plist_path,
                        format!(
                            r#"
    <?xml version="1.0" encoding="UTF-8"?>
    <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
    <plist version="1.0">
    <dict>
        <key>CFBundleExecutable</key>
        <string>applet</string>
        <key>CFBundleIconFile</key>
        <string>applet</string>
        <key>CFBundleIdentifier</key>
        <string>com.{hash}.AppleScript.discord-{id}</string>
        <key>CFBundleInfoDictionaryVersion</key>
        <string>6.0</string>
        <key>CFBundleName</key>
        <string>discord-{id}</string>
        <key>CFBundlePackageType</key>
        <string>APPL</string>
        <key>CFBundleSignature</key>
        <string>aplt</string>
        <key>CFBundleURLTypes</key>
        <array>
            <dict>
                <key>CFBundleURLName</key>
                <string>discord-{id}</string>
                <key>CFBundleURLSchemes</key>
                <array>
                    <string>discord-{id}</string>
                </array>
            </dict>
        </array>
        <key>LSRequiresCarbon</key>
        <true/>
    </dict>
    </plist>"#,
                            id = app.id,
                            hash = script_hash
                        ),
                    ).context("failed to write .plist")?;
                }
            }
        }

        Ok(())
    }

    inner(app).map_err(Error::AppRegistration)
}

/// Usually I would leave a salty comment about macs here but I'm just too tired,
/// so here is a copy of the discord RPC's reason for this hack
///
/// There does not appear to be a way to register arbitrary commands on OSX, so
/// instead we'll save the command to a file in the Discord config path, and
/// when it is needed, Discord can try to load the file there, and open the
/// command therein (will pass to js's window.open, so requires a url-like thing)
fn create_shim(id: i64, url: String) -> anyhow::Result<()> {
    let home = std::env::var("HOME").context("no $HOME detected, are we running sandboxed?")?;
    ensure!(!home.is_empty(), "$HOME is empty");

    let mut path = PathBuf::from(home);
    path.push("Library");
    path.push("Application Support");
    path.push("discord");

    ensure!(path.exists(), "Discord does not seem to be installed");

    path.push("games");
    std::fs::create_dir_all(&path)
        .context("unable to create 'games' in Discord config directory")?;

    path.set_file_name(format!("{}.json", id));

    std::fs::write(&path, &format!(r#"{{"command": "{}"}}"#, url))?;

    Ok(())
}

/// Create a small Apple Script file that supports launching the executable as
/// well as launching it with a specific URL
fn make_script(path: PathBuf, args: Vec<super::BinArg>) -> anyhow::Result<String> {
    use std::fmt::Write;
    let mut sargs = String::new();

    for arg in args {
        match arg {
            super::BinArg::Url => write!(&mut sargs, " '\" & this_URL & \"'")?,
            super::BinArg::Arg(a) => {
                if a.contains(' ') {
                    write!(&mut sargs, " '{}'", a)
                } else {
                    write!(&mut sargs, " {}", a)
                }?
            }
        }
    }

    // the magic "> /dev/null 2>&1 &" in the end is simply to launch the executable
    // in the background so that the script itself will actually exit
    Ok(format!(
        r#"
on run
	do shell script "{exe} > /dev/null 2>&1 &"
end run

on open location this_URL
	do shell script "{exe}{args} > /dev/null 2>&1 &"
end open location
    "#,
        exe = path.display(),
        args = sargs
    ))
}

fn needs_overwrite(script_hash: u64, app_id: i64, app_path: &std::path::Path) -> Option<PathBuf> {
    let plist_path = app_path.join("Contents/Info.plist");

    if !app_path.exists() {
        return Some(plist_path);
    }

    let plist = match std::fs::read_to_string(&plist_path) {
        Ok(pl) => pl,
        Err(_) => return Some(plist_path),
    };

    let bundle_id = format!(
        "<string>com.{hash}.AppleScript.discord-{id}</string>",
        hash = script_hash,
        id = app_id
    );

    if plist.contains(&bundle_id) {
        None
    } else {
        Some(plist_path)
    }
}
