#![windows_subsystem = "windows"]

use std::collections::HashMap;
use std::env::{self, current_exe};
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use daemon_slayer::client::config::Level;
use daemon_slayer::client::{self, ServiceManager, State};
use daemon_slayer::core::BoxedError;
use daemon_slayer::tray::event_loop::ControlFlow;
use daemon_slayer::tray::tray_icon::menu::{
    Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem, Submenu,
};
use daemon_slayer::tray::tray_icon::{TrayIcon, TrayIconBuilder, TrayIconEvent};
use daemon_slayer::tray::{MenuHandler, Tray, get_start_stop_text, load_icon};
use futures_util::stream::StreamExt;
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use platuned_client::Channel;
use platuned_client::management::v1::PathMessage;
use platuned_client::management::v1::management_client::ManagementClient;
use platuned_client::player::v1::event_response::EventPayload;
use platuned_client::player::v1::player_client::PlayerClient;
use platuned_client::player::v1::{Event, QueueRequest, SeekMode, SeekRequest, SetVolumeRequest};
use souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig,
    SeekDirection,
};
use tokio::runtime::{self};
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout;

fn main() -> Result<(), BoxedError> {
    let rt = runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(1)
        .build()
        .unwrap();
    let _guard = rt.enter();
    let manager = rt
        .block_on(
            client::builder(
                "com.platune.platuned".parse().unwrap(),
                current_exe()?
                    .parent()
                    .expect("Current exe should have a parent")
                    .join("platuned")
                    .try_into()?,
            )
            .with_description("test service")
            .with_arg(&"run".parse()?)
            .with_service_level(Level::User)
            .build(),
        )
        .unwrap();
    let (player_tx, player_rx) = mpsc::channel(32);
    let (manager_tx, manager_rx) = mpsc::channel(32);

    let hotkeys_manager = GlobalHotKeyManager::new().unwrap();
    #[cfg(not(target_os = "macos"))]
    let hotkey = HotKey::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyJ);
    #[cfg(target_os = "macos")]
    let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyJ);
    let toggle_id = hotkey.id();
    hotkeys_manager.register(hotkey).unwrap();
    let global_hotkey_channel = GlobalHotKeyEvent::receiver();

    tokio::spawn(player_handler(player_rx));
    tokio::spawn(manager_handler(manager, manager_rx));
    thread::spawn({
        let player_tx = player_tx.clone();
        move || {
            while let Ok(event) = global_hotkey_channel.recv() {
                if event.id() == toggle_id && event.state() == HotKeyState::Pressed {
                    player_tx.blocking_send(PlayerCommand::Toggle).unwrap();
                }
            }
        }
    });

    let handler = PlatuneMenuHandler::new(player_tx, manager_tx);
    Tray::with_handler(handler).run();
    Ok(())
}

type MenuFn =
    dyn Fn(State, &mpsc::Sender<PlayerCommand>, &mpsc::Sender<ManagerCommand>) -> ControlFlow;

#[derive(Default)]
struct MenuItemHandler(HashMap<MenuId, Box<MenuFn>>);

impl MenuItemHandler {
    fn add<F>(&mut self, menu: &MenuItem, f: F)
    where
        F: Fn(State, &mpsc::Sender<PlayerCommand>, &mpsc::Sender<ManagerCommand>) -> ControlFlow
            + 'static,
    {
        self.0.insert(menu.id().clone(), Box::new(f));
    }

    fn handle(
        &self,
        event: &MenuEvent,
        state: State,
        player_tx: &mpsc::Sender<PlayerCommand>,
        manager_tx: &mpsc::Sender<ManagerCommand>,
    ) -> ControlFlow {
        self.0.get(event.id()).unwrap()(state, player_tx, manager_tx)
    }
}

pub struct PlatuneMenuHandler {
    icon_path: std::path::PathBuf,
    current_state: State,
    menu: Menu,
    player_tx: mpsc::Sender<PlayerCommand>,
    manager_tx: mpsc::Sender<ManagerCommand>,
    menu_handler: MenuItemHandler,
    start_stop: MenuItem,
}

impl PlatuneMenuHandler {
    fn new(
        player_tx: mpsc::Sender<PlayerCommand>,
        manager_tx: mpsc::Sender<ManagerCommand>,
    ) -> Self {
        let main_menu = Menu::new();
        let service_menu = Submenu::new("Service", true);

        let start_stop_text = get_start_stop_text(&State::NotInstalled);
        let start_stop = MenuItem::new(start_stop_text, true, None);
        let restart = MenuItem::new("Restart", true, None);
        let quit = MenuItem::new("Quit", true, None);
        service_menu
            .append_items(&[&start_stop, &restart, &quit])
            .unwrap();

        let play = MenuItem::new("Play", true, None);
        let pause = MenuItem::new("Pause", true, None);
        let next = MenuItem::new("Next", true, None);
        let previous = MenuItem::new("Previous", true, None);
        let player_stop = MenuItem::new("Stop", true, None);
        let tray_quit = MenuItem::new("Quit", true, None);
        let sep = PredefinedMenuItem::separator();
        main_menu
            .append_items(&[
                &play,
                &pause,
                &next,
                &previous,
                &player_stop,
                &service_menu,
                &sep,
                &tray_quit,
            ])
            .unwrap();

        let icon_path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/icon.png"));

        let mut menu_handler = MenuItemHandler::default();
        menu_handler.add(&start_stop, |current_state, _, manager_tx| {
            if current_state == State::Started {
                manager_tx.blocking_send(ManagerCommand::Stop).unwrap();
            } else {
                manager_tx.blocking_send(ManagerCommand::Start).unwrap()
            }
            ControlFlow::Poll
        });
        menu_handler.add(&restart, |_, _, manager_tx| {
            manager_tx.blocking_send(ManagerCommand::Restart).unwrap();
            ControlFlow::Poll
        });
        menu_handler.add(&quit, |_, _, _| ControlFlow::Exit);

        menu_handler.add(&play, |_, player_tx, _| {
            player_tx.blocking_send(PlayerCommand::Start).unwrap();
            ControlFlow::Poll
        });
        menu_handler.add(&pause, |_, player_tx, _| {
            player_tx.blocking_send(PlayerCommand::Pause).unwrap();
            ControlFlow::Poll
        });
        menu_handler.add(&next, |_, player_tx, _| {
            player_tx.blocking_send(PlayerCommand::Next).unwrap();
            ControlFlow::Poll
        });
        menu_handler.add(&previous, |_, player_tx, _| {
            player_tx.blocking_send(PlayerCommand::Previous).unwrap();
            ControlFlow::Poll
        });
        menu_handler.add(&player_stop, |_, player_tx, _| {
            player_tx.blocking_send(PlayerCommand::Stop).unwrap();
            ControlFlow::Poll
        });
        menu_handler.add(&tray_quit, |_, _, _| ControlFlow::ExitWithCode(0));

        Self {
            player_tx,
            manager_tx,
            current_state: State::NotInstalled,
            icon_path,
            menu: main_menu,
            menu_handler,
            start_stop,
        }
    }
}

enum PlayerCommand {
    Start,
    Stop,
    Pause,
    Toggle,
    SetQueue(String),
    SetVolume(f64),
    Next,
    Previous,
    Seek(Duration, SeekMode),
}

enum ManagerCommand {
    Start,
    Stop,
    Restart,
    State(oneshot::Sender<State>),
}

struct LazyPlayerClient(Option<PlayerClient<Channel>>);

impl LazyPlayerClient {
    async fn get(&mut self) -> &mut PlayerClient<Channel> {
        if self.0.is_some() {
            return self.0.as_mut().unwrap();
        }

        loop {
            if let Ok(client) = PlayerClient::connect_ipc().await {
                self.0 = Some(client);
                return self.0.as_mut().unwrap();
            } else {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

fn normalize_url(url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        "http://".to_string() + url
    }
}

struct LazyMgmtClient(Option<ManagementClient<Channel>>);

impl LazyMgmtClient {
    async fn get(&mut self) -> &mut ManagementClient<Channel> {
        if self.0.is_some() {
            return self.0.as_mut().unwrap();
        }
        let management_urls = env::var("PLATUNE_MANAGEMENT_URL")
            .map(|u| u.split(",").map(normalize_url).collect::<Vec<_>>())
            .unwrap_or_default();

        loop {
            if management_urls.is_empty() {
                if let Ok(client) = ManagementClient::connect_ipc().await {
                    self.0 = Some(client);
                    return self.0.as_mut().unwrap();
                }
            } else {
                for url in &management_urls {
                    if let Ok(client) = ManagementClient::connect_http(url.clone()).await {
                        self.0 = Some(client);
                        return self.0.as_mut().unwrap();
                    }
                }
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}

async fn metadata_updater(mut controls: MediaControls) {
    let mut client = LazyPlayerClient(None);
    let mut stream = client
        .get()
        .await
        .subscribe_events(())
        .await
        .unwrap()
        .into_inner();
    let mut mgmt_client = LazyMgmtClient(None);

    let mut progress = Duration::from_millis(0);
    let mut last_progress = Instant::now();
    let mut status = platuned_client::player::v1::PlayerStatus::Stopped;
    let mut is_init = false;
    let mut current_duration = None;
    loop {
        match timeout(Duration::from_secs(1), stream.next()).await {
            Ok(Some(Ok(message))) => {
                let event_payload = message.event_payload.as_ref().unwrap();
                match event_payload {
                    EventPayload::Progress(position) => {
                        let duration: Duration = position.position.unwrap().try_into().unwrap();
                        let retrieval: Duration =
                            position.retrieval_time.unwrap().try_into().unwrap();
                        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

                        progress = duration + (now - retrieval);
                        last_progress = Instant::now();

                        match status {
                            platuned_client::player::v1::PlayerStatus::Playing => {
                                controls
                                    .set_playback(MediaPlayback::Playing {
                                        progress: Some(MediaPosition(progress)),
                                    })
                                    .unwrap();
                            }
                            platuned_client::player::v1::PlayerStatus::Paused => {
                                controls
                                    .set_playback(MediaPlayback::Paused {
                                        progress: Some(MediaPosition(progress)),
                                    })
                                    .unwrap();
                            }
                            _ => {}
                        }
                    }
                    EventPayload::State(state) => {
                        status = state.status();
                        match message.event() {
                            Event::StartQueue | Event::Ended | Event::Next | Event::Previous => {
                                progress = Duration::default();
                                current_duration =
                                    set_metadata(&mut mgmt_client, state, &mut controls).await;

                                controls
                                    .set_playback(MediaPlayback::Playing {
                                        progress: Some(MediaPosition(progress)),
                                    })
                                    .unwrap();
                                is_init = true;
                                last_progress = Instant::now();
                            }
                            Event::Resume => {
                                if !is_init {
                                    current_duration =
                                        set_metadata(&mut mgmt_client, state, &mut controls).await;
                                    is_init = true;
                                }
                                controls
                                    .set_playback(MediaPlayback::Playing {
                                        progress: Some(MediaPosition(progress)),
                                    })
                                    .unwrap();
                                last_progress = Instant::now();
                            }
                            Event::Pause => {
                                if !is_init {
                                    current_duration =
                                        set_metadata(&mut mgmt_client, state, &mut controls).await;
                                    is_init = true;
                                    // MacOS doesn't register the player if it starts as paused, so
                                    // we have to set it to playing first
                                    controls
                                        .set_playback(MediaPlayback::Playing {
                                            progress: Some(MediaPosition(progress)),
                                        })
                                        .unwrap();
                                }
                                controls
                                    .set_playback(MediaPlayback::Paused {
                                        progress: Some(MediaPosition(progress)),
                                    })
                                    .unwrap();
                            }
                            Event::Stop | Event::QueueEnded => {
                                controls.set_playback(MediaPlayback::Stopped).unwrap();
                            }
                            Event::SetVolume => {
                                #[cfg(target_os = "linux")]
                                controls.set_volume(state.volume as f64).unwrap();
                            }
                            Event::Seek | Event::Position | Event::QueueUpdated => {}
                        }
                    }
                    EventPayload::SeekData(seek) => {
                        progress = Duration::from_millis(seek.seek_millis);
                        last_progress = Instant::now();
                    }
                }
            }
            Ok(Some(Err(err))) => {
                println!("{err:?}");
                if let Ok(new_stream) = client.get().await.subscribe_events(()).await {
                    stream = new_stream.into_inner();
                } else {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
            Ok(None) => {
                if let Ok(new_stream) = client.get().await.subscribe_events(()).await {
                    stream = new_stream.into_inner();
                } else {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
            Err(_) => {
                if status == platuned_client::player::v1::PlayerStatus::Playing {
                    let now = Instant::now();
                    progress = (progress + (now - last_progress))
                        .min(current_duration.unwrap_or(Duration::MAX));
                    last_progress = now;
                    controls
                        .set_playback(MediaPlayback::Playing {
                            progress: Some(MediaPosition(progress)),
                        })
                        .unwrap();
                }
            }
        }
    }
}

async fn set_metadata(
    mgmt_client: &mut LazyMgmtClient,
    state: &platuned_client::player::v1::State,
    controls: &mut MediaControls,
) -> Option<Duration> {
    let pos = state.queue_position as usize;
    if pos < state.queue.len() {
        let path = &state.queue[pos];
        let metadata = mgmt_client
            .get()
            .await
            .get_song_by_path(PathMessage { path: path.clone() })
            .await
            .unwrap()
            .into_inner()
            .song;
        if let Some(metadata) = metadata {
            let duration = metadata.duration.unwrap();
            let duration = Duration::from_secs(duration.seconds as u64)
                + Duration::from_nanos(duration.nanos as u64);
            controls
                .set_metadata(MediaMetadata {
                    title: Some(&metadata.song),
                    album: Some(&metadata.album),
                    artist: Some(&metadata.artist),
                    cover_url: None,
                    duration: Some(duration),
                })
                .unwrap();
            Some(duration)
        } else {
            controls
                .set_metadata(MediaMetadata {
                    title: Some(path),
                    ..Default::default()
                })
                .unwrap();
            None
        }
    } else {
        None
    }
}

async fn player_handler(mut rx: mpsc::Receiver<PlayerCommand>) {
    let mut client = LazyPlayerClient(None);
    while let Some(command) = rx.recv().await {
        match command {
            PlayerCommand::Start => {
                client.get().await.resume(()).await.unwrap();
            }
            PlayerCommand::Stop => {
                client.get().await.stop(()).await.unwrap();
            }
            PlayerCommand::Pause => {
                client.get().await.pause(()).await.unwrap();
            }
            PlayerCommand::Toggle => {
                client.get().await.toggle(()).await.unwrap();
            }

            PlayerCommand::Seek(pos, mode) => {
                client
                    .get()
                    .await
                    .seek(SeekRequest {
                        time: Some(pos.try_into().unwrap()),
                        mode: mode.into(),
                    })
                    .await
                    .unwrap();
            }
            PlayerCommand::SetVolume(volume) => {
                client
                    .get()
                    .await
                    .set_volume(SetVolumeRequest {
                        volume: volume as f32,
                    })
                    .await
                    .unwrap();
            }
            PlayerCommand::Next => {
                client.get().await.next(()).await.unwrap();
            }
            PlayerCommand::Previous => {
                client.get().await.previous(()).await.unwrap();
            }
            PlayerCommand::SetQueue(uri) => {
                client
                    .get()
                    .await
                    .set_queue(QueueRequest { queue: vec![uri] })
                    .await
                    .unwrap();
            }
        }
    }
}

async fn manager_handler(manager: ServiceManager, mut rx: mpsc::Receiver<ManagerCommand>) {
    while let Some(command) = rx.recv().await {
        match command {
            ManagerCommand::Start => {
                manager.start().await.unwrap();
            }
            ManagerCommand::Stop => {
                manager.stop().await.unwrap();
            }
            ManagerCommand::Restart => {
                manager.restart().await.unwrap();
            }
            ManagerCommand::State(res) => {
                res.send(manager.status().await.unwrap().state).unwrap();
            }
        }
    }
}

impl MenuHandler for PlatuneMenuHandler {
    fn refresh_state(&mut self) {
        let (state_tx, state_rx) = oneshot::channel();
        self.manager_tx
            .blocking_send(ManagerCommand::State(state_tx))
            .unwrap();
        self.current_state = state_rx.blocking_recv().unwrap();
    }

    fn build_tray(&mut self) -> TrayIcon {
        let tray = TrayIconBuilder::new().build().unwrap();

        #[cfg(target_os = "windows")]
        let hwnd = Some(tray.window_handle());
        #[cfg(not(target_os = "windows"))]
        let hwnd = None;

        let config = PlatformConfig {
            dbus_name: "platuned",
            display_name: "Platune",
            hwnd,
        };

        let mut controls = MediaControls::new(config).unwrap();

        controls
            .attach({
                let player_tx = self.player_tx.clone();
                move |event: MediaControlEvent| match event {
                    MediaControlEvent::Play => {
                        player_tx.blocking_send(PlayerCommand::Start).unwrap();
                    }
                    MediaControlEvent::Pause => {
                        player_tx.blocking_send(PlayerCommand::Pause).unwrap();
                    }
                    MediaControlEvent::Stop | MediaControlEvent::Quit => {
                        player_tx.blocking_send(PlayerCommand::Stop).unwrap();
                    }
                    MediaControlEvent::OpenUri(uri) => {
                        player_tx
                            .blocking_send(PlayerCommand::SetQueue(uri))
                            .unwrap();
                    }
                    MediaControlEvent::SetVolume(volume) => {
                        player_tx
                            .blocking_send(PlayerCommand::SetVolume(volume))
                            .unwrap();
                    }
                    MediaControlEvent::Next => {
                        player_tx.blocking_send(PlayerCommand::Next).unwrap();
                    }
                    MediaControlEvent::Previous => {
                        player_tx.blocking_send(PlayerCommand::Previous).unwrap();
                    }
                    MediaControlEvent::Toggle => {
                        player_tx.blocking_send(PlayerCommand::Toggle).unwrap();
                    }
                    MediaControlEvent::Seek(direction) => {
                        player_tx
                            .blocking_send(PlayerCommand::Seek(
                                Duration::from_secs(5),
                                match direction {
                                    SeekDirection::Forward => SeekMode::Forward,
                                    SeekDirection::Backward => SeekMode::Backward,
                                },
                            ))
                            .unwrap();
                    }
                    MediaControlEvent::SeekBy(direction, duration) => {
                        player_tx
                            .blocking_send(PlayerCommand::Seek(
                                duration,
                                match direction {
                                    SeekDirection::Forward => SeekMode::Forward,
                                    SeekDirection::Backward => SeekMode::Backward,
                                },
                            ))
                            .unwrap();
                    }
                    MediaControlEvent::SetPosition(MediaPosition(duration)) => {
                        player_tx
                            .blocking_send(PlayerCommand::Seek(duration, SeekMode::Absolute))
                            .unwrap();
                    }
                    MediaControlEvent::Raise => {}
                }
            })
            .unwrap();

        tokio::spawn(metadata_updater(controls));

        tray.set_menu(Some(Box::new(self.menu.clone())));
        tray.set_icon(Some(load_icon(&self.icon_path))).unwrap();
        tray
    }

    fn update_menu(&self) {
        self.start_stop
            .set_text(get_start_stop_text(&self.current_state));
    }

    fn handle_menu_event(&mut self, event: MenuEvent) -> ControlFlow {
        self.menu_handler.handle(
            &event,
            self.current_state,
            &self.player_tx,
            &self.manager_tx,
        )
    }

    fn handle_tray_event(&mut self, _event: TrayIconEvent) -> ControlFlow {
        ControlFlow::Poll
    }
}
