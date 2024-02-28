use std::net::{Ipv4Addr, SocketAddr};

use async_compat::Compat;
use bevy::prelude::*;
use bevy::log::{Level, LogPlugin};
use bevy::tasks::IoTaskPool;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use lightyear::connection::netcode::{ClientId, Key};
use lightyear::shared::log::add_log_layer;
use clap::{Parser, ValueEnum};

mod server;
mod client;
mod game;

pub const CLIENT_PORT: u16 = 0;
pub const SERVER_PORT: u16 = 5000;
pub const PROTOCOL_ID: u64 = 0;

pub const KEY: Key = [0; 32];

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, ValueEnum)]
pub enum Transports {
    Udp,
    WebTransport,
    WebSocket,
}

#[derive(Parser, PartialEq, Debug)]
enum Cli {
    SinglePlayer,
    Server {
        #[arg(long, default_value = "false")]
        headless: bool,
        #[arg(short, long, default_value = "false")]
        inspector: bool,
        #[arg(short, long, default_value_t = SERVER_PORT)]
        port: u16,
        #[arg(short, long, value_enum, default_value_t = Transports::WebTransport)]
        transport: Transports,
    },
    Client {
        #[arg(short, long, default_value = "false")]
        inspector: bool,
        #[arg(short, long, default_value_t = 0)]
        client_id: u64,
        #[arg(long, default_value_t = CLIENT_PORT)]
        client_port: u16,
        #[arg(long, default_value_t=Ipv4Addr::LOCALHOST)]
        server_addr: Ipv4Addr,
        #[arg(short, long, default_value_t = SERVER_PORT)]
        server_port: u16,
        #[arg(short, long, value_enum, default_value_t = Transports::WebTransport)]
        transport: Transports,
    },
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Waiting,
    InGame,
}

pub const FPS: usize = 60;

fn main() {

}

fn setup(app: &mut App, cli: Cli) {
    match cli {
        Cli::SinglePlayer => {
        }
        Cli::Server { .. } => {
            setup_server(app, cli);
        }
        Cli::Client { .. } => {
            setup_client(app, cli);
        }
    }
    let mut app = App::new();

    app.add_plugins((DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
        }),
        game::GamePlugin,
        ))
        .init_state::<AppState>()
        .run();

}

fn setup_server(app: &mut App, cli: Cli) {
    let Cli::Server {
        headless,
        inspector,
        port,
        transport,
    } = cli
    else {
        return;
    };
            if !headless {
                app.add_plugins(DefaultPlugins.build().disable::<LogPlugin>());
            } else {
                app.add_plugins(MinimalPlugins);
            }
            app.add_plugins(LogPlugin {
                level: Level::INFO,
                filter: "wgpu=error,bevy_render=info,bevy_ecs=trace".to_string(),
                update_subscriber: Some(add_log_layer),
            });

            if inspector {
                app.add_plugins(WorldInspectorPlugin::new());
            }

            let server_plugin_group = IoTaskPool::get()
                .scope(|s| {
                    s.spawn(Compat::new(async {
                        ServerPluginGroup::new(port, transport, headless).await
                    }));
                })
            .pop()
                .unwrap();
            app.add_plugins(server_plugin_group.build());

}

fn setup_client(app: &mut App, cli: Cli) {
    let Cli::Client {
        inspector,
        client_id,
        client_port,
        server_addr,
        server_port,
        transport,
    } = cli
    else {
        return;
    };
    app.add_plugins(DefaultPlugins.set(LogPlugin {
        level: Level::INFO,
        filter: "wgpu=error,bevy_render=info,bevy_ecs=trace".to_string(),
        update_subscriber: Some(add_log_layer),
    }));

    if inspector {
        app.add_plugins(WorldInspectorPlugin::new());
    }
    let server_addr = SocketAddr::new(server_addr.into(), server_port);
    let client_plugin_group =
        ClientPluginGroup::new(client_id, client_port, server_addr, transport);
    app.add_plugins(client_plugin_group.build());
}
