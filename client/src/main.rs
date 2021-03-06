use bevy::prelude::*;
use bevy_networking_turbulence::{NetworkEvent, NetworkResource};
use nrts_core::network::{
    decode_response, encode_request, get_type_registry, restore_world_backup, NetworkRequest,
    NetworkResponse, SERVER_PORT,
};
use std::net::{Ipv4Addr, SocketAddr};

struct Args {
    address: SocketAddr,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(get_type_registry())
        .insert_resource(Args {
            address: SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), SERVER_PORT).into(),
        })
        .add_startup_system(template_setup.system())
        .add_startup_system(startup.system())
        .add_system(template_animation.system())
        .add_system(handle_packets.system())
        .run();
}

fn startup(mut net: ResMut<NetworkResource>, args: Res<Args>) {
    info!("Starting server at {}", args.address);
    net.connect(args.address);
    net.broadcast(encode_request(&NetworkRequest::RequestWorld))
        .unwrap();
}

fn handle_packets(
    mut world: ResMut<World>,
    mut net: ResMut<NetworkResource>,
    mut reader: EventReader<NetworkEvent>,
) {
    for event in reader.iter() {
        match event {
            NetworkEvent::Connected(handle) => {
                info!("Server {:?} connected", handle);
            }
            NetworkEvent::Disconnected(handle) => {
                info!("Server {:?} disconnected", handle);
            }
            NetworkEvent::Packet(handle, packet) => {
                let response: NetworkResponse = decode_response(packet.as_ref());
                info!("Received packet from {:?}: {:?}", handle, response);
                match response {
                    NetworkResponse::ResponseWorld(scene) => {
                        restore_world_backup(&mut *world, scene.as_ref())
                    }
                }
            }
            NetworkEvent::Error(handle, err) => warn!("{:?} error {:?}!", handle, err),
        }
    }
}

fn template_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(Text2dBundle {
        text: Text::with_section(
            "Would you like to play a game?",
            TextStyle {
                font: asset_server.load("fonts/tiny.ttf"),
                font_size: 58.0,
                color: Color::WHITE,
            },
            TextAlignment {
                vertical: VerticalAlign::Center,
                horizontal: HorizontalAlign::Center,
            },
        ),
        ..Default::default()
    });
}

fn template_animation(time: Res<Time>, mut query: Query<&mut Transform, With<Text>>) {
    for mut transform in query.iter_mut() {
        transform.translation.x = 100.0 * time.seconds_since_startup().sin() as f32;
        transform.translation.y = 100.0 * time.seconds_since_startup().cos() as f32;
    }
}
