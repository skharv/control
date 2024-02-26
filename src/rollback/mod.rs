use bevy::prelude::*;
use bevy_ggrs::prelude::*;
use clap::Parser;
use ggrs::UdpNonBlockingSocket;
use std::net::SocketAddr;
use bytemuck::{Pod, Zeroable};

pub type GameConfig = bevy_ggrs::GgrsConfig<GameInput>;

#[repr(C)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Zeroable, Pod)]
pub struct GameInput(pub u8);

#[derive(Resource)]
struct NetworkStatsTimer(Timer);

#[derive(Resource, Default, Reflect, Hash, Clone, Copy)]
#[reflect(Hash)]
pub struct FrameCount {
    pub frame: u32,
}

#[derive(Parser, Resource)]
struct Options {
    #[clap(short, long)]
    local_port: u16,
    #[clap(short, long, num_args = 1..)]
    players: Vec<String>,
    #[clap(short, long, num_args = 1..)]
    spectators: Vec<SocketAddr>,
}

pub struct RollbackPlugin;

impl Plugin for RollbackPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GgrsPlugin::<GameConfig>::default())
            .insert_resource(FrameCount{ frame: 0 })
            .insert_resource(NetworkStatsTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
            .set_rollback_schedule_fps(crate::FPS)
            .rollback_component_with_clone::<Transform>()
            .rollback_resource_with_copy::<FrameCount>()
            .add_systems(Update, wait_for_players.run_if(in_state(crate::AppState::Waiting)))
            .add_systems(GgrsSchedule, increase_frame_system);
    }
}

fn increase_frame_system(
    mut frame_count: ResMut<FrameCount>,
    ) {
    frame_count.frame += 1;
}

fn wait_for_players(
    mut commands: Commands,
    mut app_state: ResMut<NextState<crate::AppState>>,
    ) {
    let options = Options::parse();

    let num_players = 2;
    if options.players.len() < num_players {
        info!("waiting...");
        return;
    }

    info!("All peers have joined, starting game");

    let mut session_builder = ggrs::SessionBuilder::<GameConfig>::new()
        .with_num_players(num_players)
        .with_desync_detection_mode(ggrs::DesyncDetection::On { interval: 10 })
        .with_max_prediciont_window(12)
        .expect("prediction window can't be 0")
        .with_input_delay(2);

    for (i, player) in options.players.iter().enumerate() {
        if player == "localhost" {
            session_builder = session_builder
                .add_player(PlayerType::Local, i).expect("failed to start game");
        } else {
            let remote_addr: SocketAddr = player.parse().expect("unable to get remote address");
            session_builder = session_builder
                .add_player(PlayerType::Remote(remote_addr), i).expect("failed to start game");
        }
    }

    for (i, spectator_address) in options.spectators.iter().enumerate() {
        session_builder = session_builder
            .add_player(PlayerType::Spectator(*spectator_address), num_players + i).expect("add spectator failed");
    }
    
    if let Ok(socket) = UdpNonBlockingSocket::bind_to_port(options.local_port) {
        let ggrs_session = session_builder
            .start_p2p_session(socket)
            .expect("failed to start session");

        commands.insert_resource(options);
        commands.insert_resource(bevy_ggrs::Session::P2P(ggrs_session));
        app_state.set(crate::AppState::InGame);
    }
}
