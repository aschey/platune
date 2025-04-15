use std::env::current_exe;
use std::ops::Add;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use daemon_slayer::client::config::Level;
use daemon_slayer::client::{self, ServiceManager, State};
use daemon_slayer::core::BoxedError;
use daemon_slayer::tray::event_loop::ControlFlow;
use daemon_slayer::tray::tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, Submenu};
use daemon_slayer::tray::tray_icon::{TrayIcon, TrayIconBuilder, TrayIconEvent};
use daemon_slayer::tray::{MenuHandler, Tray, get_start_stop_text, load_icon};
use futures_util::stream::StreamExt;
use platuned_client::rpc::event_response::EventPayload;
use platuned_client::rpc::{
    Event, PathMessage, QueueRequest, SeekMode, SeekRequest, SetVolumeRequest,
};
use platuned_client::{
    Channel, ManagementClient, PlayerClient, connect_management_ipc, connect_player_ipc,
};
use souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig,
    SeekDirection,
};
use tokio::runtime::{self, Handle};
use tokio::sync::{mpsc, oneshot};

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
    let config = PlatformConfig {
        dbus_name: "platuned",
        display_name: "Platune",
        hwnd: None,
    };
    let (tx, rx) = mpsc::channel(32);

    let mut controls = MediaControls::new(config).unwrap();

    controls
        .attach({
            let tx = tx.clone();
            move |event: MediaControlEvent| match event {
                MediaControlEvent::Play => {
                    tx.blocking_send(Command::Player(PlayerCommand::Start))
                        .unwrap();
                }
                MediaControlEvent::Pause => {
                    tx.blocking_send(Command::Player(PlayerCommand::Pause))
                        .unwrap();
                }
                MediaControlEvent::Stop | MediaControlEvent::Quit => {
                    tx.blocking_send(Command::Player(PlayerCommand::Stop))
                        .unwrap();
                }
                MediaControlEvent::OpenUri(uri) => {
                    tx.blocking_send(Command::Player(PlayerCommand::SetQueue(uri)))
                        .unwrap();
                }
                MediaControlEvent::SetVolume(volume) => {
                    tx.blocking_send(Command::Player(PlayerCommand::SetVolume(volume)))
                        .unwrap();
                }
                MediaControlEvent::Next => {
                    tx.blocking_send(Command::Player(PlayerCommand::Next))
                        .unwrap();
                }
                MediaControlEvent::Previous => {
                    tx.blocking_send(Command::Player(PlayerCommand::Previous))
                        .unwrap();
                }
                MediaControlEvent::Toggle => {
                    tx.blocking_send(Command::Player(PlayerCommand::Toggle))
                        .unwrap();
                }
                MediaControlEvent::Seek(direction) => {
                    tx.blocking_send(Command::Player(PlayerCommand::Seek(
                        Duration::from_secs(5),
                        match direction {
                            SeekDirection::Forward => SeekMode::Forward,
                            SeekDirection::Backward => SeekMode::Backward,
                        },
                    )))
                    .unwrap();
                }
                MediaControlEvent::SeekBy(direction, duration) => {
                    tx.blocking_send(Command::Player(PlayerCommand::Seek(
                        duration,
                        match direction {
                            SeekDirection::Forward => SeekMode::Forward,
                            SeekDirection::Backward => SeekMode::Backward,
                        },
                    )))
                    .unwrap();
                }
                MediaControlEvent::SetPosition(MediaPosition(duration)) => {
                    tx.blocking_send(Command::Player(PlayerCommand::Seek(
                        duration,
                        SeekMode::Absolute,
                    )))
                    .unwrap();
                }
                MediaControlEvent::Raise => {}
            }
        })
        .unwrap();

    tokio::spawn(service_handler(manager, rx));
    tokio::spawn(metadata_updater(controls));

    let handler = PlatuneMenuHandler::new(tx);
    Tray::with_handler(handler).start();
    Ok(())
}

pub struct PlatuneMenuHandler {
    icon_path: std::path::PathBuf,
    current_state: State,
    menu: Menu,
    start_stop_id: MenuId,
    restart_id: MenuId,
    quit_id: MenuId,
    play_id: MenuId,
    pause_id: MenuId,
    next_id: MenuId,
    previous_id: MenuId,
    player_stop_id: MenuId,
    tx: mpsc::Sender<Command>,
}

impl PlatuneMenuHandler {
    fn new(tx: mpsc::Sender<Command>) -> Self {
        let main_menu = Menu::new();
        let service_menu = Submenu::new("Service", true);
        let player_menu = Submenu::new("Player", true);
        main_menu.append(&service_menu).unwrap();
        main_menu.append(&player_menu).unwrap();

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
        player_menu
            .append_items(&[&play, &pause, &next, &previous, &player_stop])
            .unwrap();
        let icon_path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/icon.png"));

        Self {
            tx,
            current_state: State::NotInstalled,
            icon_path,
            menu: main_menu,
            start_stop_id: start_stop.into_id(),
            restart_id: restart.into_id(),
            quit_id: quit.into_id(),
            play_id: play.into_id(),
            pause_id: pause.into_id(),
            next_id: next.into_id(),
            previous_id: previous.into_id(),
            player_stop_id: player_stop.into_id(),
        }
    }
}

enum Command {
    Player(PlayerCommand),
    Manager(ManagerCommand),
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
            self.0.as_mut().unwrap()
        } else {
            self.0 = connect_player_ipc().await.ok();
            self.0.as_mut().unwrap()
        }
    }
}

struct LazyMgmtClient(Option<ManagementClient<Channel>>);

impl LazyMgmtClient {
    async fn get(&mut self) -> &mut ManagementClient<Channel> {
        if self.0.is_some() {
            self.0.as_mut().unwrap()
        } else {
            self.0 = connect_management_ipc().await.ok();
            self.0.as_mut().unwrap()
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
    while let Some(Ok(message)) = stream.next().await {
        match message.event_payload.as_ref().unwrap() {
            EventPayload::Progress(position) => {
                let duration: Duration = position.position.unwrap().try_into().unwrap();
                let retrieval: Duration = position.retrieval_time.unwrap().try_into().unwrap();
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

                progress = duration + (now - retrieval);
                controls
                    .set_playback(MediaPlayback::Playing {
                        progress: Some(MediaPosition(progress)),
                    })
                    .unwrap();
            }
            EventPayload::State(state) => {
                #[cfg(not(target_os = "macos"))]
                controls.set_volume(state.volume as f64).unwrap();
                let metadata = mgmt_client
                    .get()
                    .await
                    .get_song_by_path(PathMessage {
                        path: state.queue[state.queue_position as usize].clone(),
                    })
                    .await
                    .unwrap()
                    .into_inner()
                    .song
                    .unwrap();
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
            }
            EventPayload::SeekData(seek) => {
                progress = Duration::from_millis(seek.seek_millis);
            }
        }
        match message.event() {
            Event::StartQueue
            | Event::QueueUpdated
            | Event::Ended
            | Event::Next
            | Event::Previous => {}
            Event::Resume => {
                controls
                    .set_playback(MediaPlayback::Playing {
                        progress: Some(MediaPosition(progress)),
                    })
                    .unwrap();
            }
            Event::Pause => {
                controls
                    .set_playback(MediaPlayback::Paused {
                        progress: Some(MediaPosition(progress)),
                    })
                    .unwrap();
            }
            Event::Stop | Event::QueueEnded => {
                controls.set_playback(MediaPlayback::Stopped).unwrap();
            }
            Event::SetVolume | Event::Seek | Event::Position => {}
        }
    }
    println!("DONE");
}

async fn service_handler(manager: ServiceManager, mut rx: mpsc::Receiver<Command>) {
    let mut client = LazyPlayerClient(None);
    while let Some(command) = rx.recv().await {
        match command {
            Command::Manager(ManagerCommand::Start) => {
                manager.start().await.unwrap();
            }
            Command::Manager(ManagerCommand::Stop) => {
                manager.stop().await.unwrap();
            }
            Command::Manager(ManagerCommand::Restart) => {
                manager.restart().await.unwrap();
            }
            Command::Manager(ManagerCommand::State(res)) => {
                res.send(manager.status().await.unwrap().state).unwrap();
            }
            Command::Player(PlayerCommand::Start) => {
                client.get().await.resume(()).await.unwrap();
            }
            Command::Player(PlayerCommand::Stop) => {
                client.get().await.stop(()).await.unwrap();
            }
            Command::Player(PlayerCommand::Pause) => {
                client.get().await.pause(()).await.unwrap();
            }
            Command::Player(PlayerCommand::Toggle) => {
                client.get().await.toggle(()).await.unwrap();
            }

            Command::Player(PlayerCommand::Seek(pos, mode)) => {
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
            Command::Player(PlayerCommand::SetVolume(volume)) => {
                client
                    .get()
                    .await
                    .set_volume(SetVolumeRequest {
                        volume: volume as f32,
                    })
                    .await
                    .unwrap();
            }
            Command::Player(PlayerCommand::Next) => {
                client.get().await.next(()).await.unwrap();
            }
            Command::Player(PlayerCommand::Previous) => {
                client.get().await.previous(()).await.unwrap();
            }
            Command::Player(PlayerCommand::SetQueue(uri)) => {
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

impl MenuHandler for PlatuneMenuHandler {
    fn refresh_state(&mut self) {
        let (state_tx, state_rx) = oneshot::channel();
        self.tx
            .blocking_send(Command::Manager(ManagerCommand::State(state_tx)))
            .unwrap();
        self.current_state = state_rx.blocking_recv().unwrap();
    }

    fn get_menu(&mut self) -> Menu {
        self.menu.clone()
    }

    fn build_tray(&mut self, menu: &Menu) -> TrayIcon {
        TrayIconBuilder::new()
            .with_menu(Box::new(menu.clone()))
            .with_icon(load_icon(&self.icon_path))
            .build()
            .unwrap()
    }

    fn update_menu(&self, menu: &Menu) {
        menu.items()[0].as_submenu_unchecked().items()[0]
            .as_menuitem_unchecked()
            .set_text(get_start_stop_text(&self.current_state));
    }

    fn handle_menu_event(&mut self, event: MenuEvent) -> ControlFlow {
        if event.id == self.start_stop_id {
            if self.current_state == State::Started {
                self.tx
                    .blocking_send(Command::Manager(ManagerCommand::Stop))
                    .unwrap();
            } else {
                self.tx
                    .blocking_send(Command::Manager(ManagerCommand::Start))
                    .unwrap()
            }
        } else if event.id == self.restart_id {
            self.tx
                .blocking_send(Command::Manager(ManagerCommand::Restart))
                .unwrap()
        } else if event.id == self.quit_id {
            return ControlFlow::Exit;
        } else if event.id == self.play_id {
            self.tx
                .blocking_send(Command::Player(PlayerCommand::Start))
                .unwrap()
        } else if event.id == self.pause_id {
            self.tx
                .blocking_send(Command::Player(PlayerCommand::Pause))
                .unwrap()
        }

        ControlFlow::Poll
    }

    fn handle_tray_event(&mut self, _event: TrayIconEvent) -> ControlFlow {
        ControlFlow::Poll
    }
}
