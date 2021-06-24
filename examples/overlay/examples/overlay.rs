use ds::overlay;
use eframe::egui;
use examples_shared::{self as es, ds, tokio, tracing};

struct App {
    discord: ds::Discord,
    handle: tokio::runtime::Handle,
    overlay_state: tokio::sync::watch::Receiver<ds::wheel::OverlayState>,
    signal: std::sync::Arc<(parking_lot::Mutex<bool>, parking_lot::Condvar)>,
}

impl eframe::epi::App for App {
    fn name(&self) -> &str {
        "discord-overlay"
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut eframe::epi::Frame<'_>) {
        macro_rules! run_async {
            ($code:block) => {
                self.handle.clone().block_on(async { $code })
            };
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.group(|ui| {
                let mut overlay_visible = {
                    let os = self.overlay_state.borrow();

                    ui.set_enabled(os.enabled);

                    os.visible == overlay::Visibility::Visible
                };

                if ui.checkbox(&mut overlay_visible, "Show Overlay").clicked() {
                    run_async!({
                        let mut ow = self.overlay_state.clone();
                        let handle = tokio::task::spawn(async move {
                            ow.changed().await
                        });

                        match self
                            .discord
                            .set_overlay_visibility(if overlay_visible {
                                overlay::Visibility::Visible
                            } else {
                                overlay::Visibility::Hidden
                            })
                            .await
                        {
                            Ok(_) => {
                                tracing::info!(result = ?handle.await, "overlay visibility changed");
                            }
                            Err(e) => {
                                tracing::error!(error = ?e, "failed to change overlay visibility");
                            }
                        }
                    });
                }

                if ui.button("Open Voice Settings").clicked() {
                    run_async!({
                        tracing::info!(result = ?self.discord.open_voice_settings().await, "open voice settings");
                    })
                }
            });
        });
    }

    fn on_exit(&mut self) {
        self.signal.1.notify_one();
    }
}

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .thread_keep_alive(std::time::Duration::from_secs(1_000_000))
        .thread_name("discord")
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");

    let client = rt.block_on(async { es::make_client(ds::Subscriptions::OVERLAY).await });

    let handle = rt.handle().clone();

    let rt_thread =
        std::sync::Arc::new((parking_lot::Mutex::new(false), parking_lot::Condvar::new()));
    let signal = rt_thread.clone();

    let _handle = std::thread::spawn(move || {
        let _guard = rt.enter();

        rt_thread.1.wait(&mut rt_thread.0.lock());
    });

    let overlay_state = client.wheel.overlay().0;

    let app = App {
        discord: client.discord,
        overlay_state,
        signal,
        handle,
    };

    eframe::run_native(
        Box::new(app),
        eframe::NativeOptions {
            transparent: true,
            ..Default::default()
        },
    );
}
