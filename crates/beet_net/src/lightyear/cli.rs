//! This example showcases how to use Lightyear with Bevy, to easily get replication along with prediction/interpolation working.
//!
//! There is a lot of setup code, but it's mostly to have the examples work in all possible configurations of transport.
//! (all transports are supported, as well as running the example in listen-server or host-server mode)
//!
//!
//! Run with
//! - `cargo run -- server`
//! - `cargo run -- client -c 1`
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use crate::prelude::*;
use bevy::log::Level;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::DefaultPlugins;
use clap::Parser;
use clap::ValueEnum;
use lightyear::prelude::client::InterpolationConfig;
use lightyear::prelude::client::InterpolationDelay;
use lightyear::prelude::client::NetConfig;
use lightyear::prelude::TransportConfig;
use lightyear::shared::config::Mode;
use lightyear::shared::log::add_log_layer;
use lightyear::transport::LOCAL_SOCKET;
use serde::Deserialize;
use serde::Serialize;
use std::net::SocketAddr;
use std::str::FromStr;

#[derive(Parser, PartialEq, Debug)]
pub enum Cli {
	/// We have the client and the server running inside the same app.
	/// The server will also act as a client.
	#[cfg(not(target_family = "wasm"))]
	HostServer {
		#[arg(short, long, default_value = None)]
		client_id: Option<u64>,
	},
	#[cfg(not(target_family = "wasm"))]
	/// We will create two apps: a client app and a server app.
	/// Data gets passed between the two via channels.
	ListenServer {
		#[arg(short, long, default_value = None)]
		client_id: Option<u64>,
	},
	#[cfg(not(target_family = "wasm"))]
	/// Dedicated server
	Server,
	/// The program will act as a client
	Client {
		#[arg(short, long, default_value = None)]
		client_id: Option<u64>,
	},
}

/// This is the main function
/// The cli argument is used to determine if we are running as a client or a server (or listen-server)
/// Then we build the app and run it.
///
/// To build a lightyear app you will need to add either the [`client::ClientPlugin`] or [`server::ServerPlugin`]
/// They can be created by providing a [`client::ClientConfig`] or [`server::ServerConfig`] struct, along with a
/// shared [`Protocol`](lightyear::prelude::Protocol) which defines the messages (Messages, Components, Inputs) that
/// can be sent between client and server.
pub fn run(settings: Settings, cli: Cli) {
	match cli {
		// ListenServer using a single app
		#[cfg(not(target_family = "wasm"))]
		Cli::HostServer { client_id } => {
			let client_net_config = NetConfig::Local {
				id: client_id.unwrap_or(settings.client.client_id),
			};
			let mut app = combined_app(settings, vec![], client_net_config);
			app.run();
		}
		#[cfg(not(target_family = "wasm"))]
		Cli::ListenServer { client_id } => {
			// create client app
			let (from_server_send, from_server_recv) =
				crossbeam_channel::unbounded();
			let (to_server_send, to_server_recv) =
				crossbeam_channel::unbounded();
			// we will communicate between the client and server apps via channels
			let transport_config = TransportConfig::LocalChannel {
				recv: from_server_recv,
				send: to_server_send,
			};
			let net_config = build_client_netcode_config(
				client_id.unwrap_or(settings.client.client_id),
				// when communicating via channels, we need to use the address `LOCAL_SOCKET` for the server
				LOCAL_SOCKET,
				settings.client.conditioner.as_ref(),
				&settings.shared,
				transport_config,
			);
			let mut client_app = client_app(settings.clone(), net_config);

			// create server app
			let extra_transport_configs = vec![TransportConfig::Channels {
				// even if we communicate via channels, we need to provide a socket address for the client
				channels: vec![(
					LOCAL_SOCKET,
					to_server_recv,
					from_server_send,
				)],
			}];
			let mut server_app = server_app(settings, extra_transport_configs);

			// run both the client and server apps
			std::thread::spawn(move || server_app.run());
			client_app.run();
		}
		#[cfg(not(target_family = "wasm"))]
		Cli::Server => {
			let mut app = server_app(settings, vec![]);
			app.run();
		}
		Cli::Client { client_id } => {
			let server_addr = SocketAddr::new(
				settings.client.server_addr.into(),
				settings.client.server_port,
			);
			// use the cli-provided client id if it exists, otherwise use the settings client id
			let client_id = client_id.unwrap_or(settings.client.client_id);
			let net_config = get_client_net_config(&settings, client_id);
			let mut app = client_app(settings, net_config);
			app.run();
		}
	}
}

/// Build the client app
fn client_app(settings: Settings, net_config: NetConfig) -> App {
	let mut app = App::new();
	app.add_plugins(DefaultPlugins.build().set(LogPlugin {
		level: Level::INFO,
		filter: "wgpu=error,bevy_render=info,bevy_ecs=warn".to_string(),
		update_subscriber: Some(add_log_layer),
	}));
	let client_config = lightyear::client::config::ClientConfig {
		shared: shared_config(Mode::Separate),
		net: net_config,
		..default()
	};
	let plugin_config =
		lightyear::client::plugin::PluginConfig::new(client_config, protocol());
	app.add_plugins((
		lightyear::client::plugin::ClientPlugin::new(plugin_config),
		ExampleClientPlugin,
		SharedPlugin,
	));
	app
}

/// Build the server app
#[cfg(not(target_family = "wasm"))]
fn server_app(
	settings: Settings,
	extra_transport_configs: Vec<TransportConfig>,
) -> App {
	let mut app = App::new();
	if !settings.server.headless {
		app.add_plugins(DefaultPlugins.build().disable::<LogPlugin>());
	} else {
		app.add_plugins(MinimalPlugins);
	}
	app.add_plugins(LogPlugin {
		level: Level::INFO,
		filter: "wgpu=error,bevy_render=info,bevy_ecs=warn".to_string(),
		update_subscriber: Some(add_log_layer),
	});

	let mut net_configs = get_server_net_configs(&settings);
	let extra_net_configs = extra_transport_configs.into_iter().map(|c| {
		build_server_netcode_config(
			settings.server.conditioner.as_ref(),
			&settings.shared,
			c,
		)
	});
	net_configs.extend(extra_net_configs);
	let server_config = lightyear::server::config::ServerConfig {
		shared: shared_config(Mode::Separate),
		net: net_configs,
		..default()
	};
	app.add_plugins((
		lightyear::server::plugin::ServerPlugin::new(
			lightyear::server::plugin::PluginConfig::new(
				server_config,
				protocol(),
			),
		),
		ExampleServerPlugin,
		SharedPlugin,
	));
	app
}

/// An app that contains both the client and server plugins
#[cfg(not(target_family = "wasm"))]
fn combined_app(
	settings: Settings,
	extra_transport_configs: Vec<TransportConfig>,
	client_net_config: lightyear::prelude::client::NetConfig,
) -> App {
	let mut app = App::new();
	app.add_plugins(DefaultPlugins.build().set(LogPlugin {
		level: Level::INFO,
		filter: "wgpu=error,bevy_render=info,bevy_ecs=warn".to_string(),
		update_subscriber: Some(add_log_layer),
	}));

	// server plugin
	let mut net_configs = get_server_net_configs(&settings);
	let extra_net_configs = extra_transport_configs.into_iter().map(|c| {
		build_server_netcode_config(
			settings.server.conditioner.as_ref(),
			&settings.shared,
			c,
		)
	});
	net_configs.extend(extra_net_configs);
	let server_config = lightyear::server::config::ServerConfig {
		shared: shared_config(Mode::HostServer),
		net: net_configs,
		..default()
	};
	app.add_plugins((
		lightyear::server::plugin::ServerPlugin::new(
			lightyear::server::plugin::PluginConfig::new(
				server_config,
				protocol(),
			),
		),
		ExampleServerPlugin,
	));

	// client plugin
	let client_config = lightyear::client::config::ClientConfig {
		shared: shared_config(Mode::HostServer),
		net: client_net_config,
		interpolation: InterpolationConfig {
			delay: InterpolationDelay::default().with_send_interval_ratio(2.0),
			..default()
		},
		..default()
	};
	let plugin_config =
		lightyear::client::plugin::PluginConfig::new(client_config, protocol());
	app.add_plugins((
		lightyear::client::plugin::ClientPlugin::new(plugin_config),
		ExampleClientPlugin,
	));
	// shared plugin
	app.add_plugins(SharedPlugin);
	app
}
