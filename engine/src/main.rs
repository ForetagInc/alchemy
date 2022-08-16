#![feature(trait_alias)]
#![feature(derive_default_enum)]
#![feature(const_try)]

extern crate juniper;

#[macro_use]
extern crate derivative;
#[macro_use]
extern crate juniper_codegen;

use actix_cors::Cors;
use actix_web::dev::Server;
use actix_web::{
	http::header,
	middleware,
	web::{self, Data},
	App, HttpServer,
};
use arangodb_events_rs::api::DocumentOperation;
use arangodb_events_rs::{
	AsyncHandlerOutput, Handler, HandlerContextFactory, HandlerEvent, Trigger,
	TriggerAuthentication,
};

use std::sync::{Arc, Mutex};

mod api;
mod lib;
mod meta;

use lib::database::generate_sdl;
use lib::CONFIG;

#[tokio::main]
async fn main() {
	pluralizer::initialize();

	let app_port = CONFIG.app_port.parse::<u16>().unwrap_or(8080);

	println!("Starting Alchemy on port {:?}", app_port);

	let map = generate_sdl().await;
	let api_schema = Data::new(Mutex::new(api::schema::schema(map.clone())));

	let (http, _) = tokio::join!(
		get_http_server(
			app_port,
			api_schema.clone(),
			Data::new(meta::graphql::schema())
		),
		run_arangodb_listener(api_schema)
	);

	http.expect("Error running HTTP Server");
}

fn get_http_server(
	port: u16,
	api_schema: Data<Mutex<api::schema::Schema>>,
	meta_schema: Data<meta::graphql::Schema>,
) -> Server {
	HttpServer::new(move || {
		App::new()
			.app_data(meta_schema.clone())
			.app_data(api_schema.clone())
			.wrap(
				Cors::default()
					.allow_any_origin()
					.allowed_methods(vec!["POST", "GET"])
					.allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
					.allowed_header(header::CONTENT_TYPE)
					.supports_credentials()
					.max_age(3600),
			)
			.wrap(middleware::Compress::default())
			.wrap(middleware::Logger::default())
			.service(
				web::resource("/api/graphql")
					.route(web::post().to(api::server::graphql_api_route))
					.route(web::get().to(api::server::graphql_api_route)),
			)
			.service(
				web::resource("/api/playground")
					.route(web::get().to(api::server::playground_api_route)),
			)
			.service(
				web::resource("/meta/graphql")
					.route(web::post().to(meta::graphql::server::graphql_meta_route))
					.route(web::get().to(meta::graphql::server::graphql_meta_route)),
			)
			.service(
				web::resource("/meta/playground")
					.route(web::get().to(meta::graphql::server::playground_meta_route)),
			)
	})
	.bind(("0.0.0.0", port))
	.expect("Error binding HTTP server address")
	.run()
}

pub struct ArangoDBListener;

impl Handler for ArangoDBListener {
	type Context = Arc<Mutex<api::schema::Schema>>;

	fn call<'a>(ctx: &'a Self::Context, _: &'a DocumentOperation) -> AsyncHandlerOutput<'a> {
		Box::pin(async move {
			println!("Schema update requested");

			let mut schema = ctx.lock().unwrap();

			let map = generate_sdl().await;
			let api_schema = api::schema::schema(map.clone());

			*schema = api_schema;
		})
	}
}

async fn run_arangodb_listener(schema: Data<Mutex<api::schema::Schema>>) {
	let mut trigger = Trigger::new_auth(
		CONFIG.db_host.as_str(),
		CONFIG.db_name.as_str(),
		TriggerAuthentication::new(CONFIG.db_user.as_str(), CONFIG.db_pass.as_str()),
	);

	let context_data = HandlerContextFactory::from(schema.into_inner());

	trigger.subscribe_to::<ArangoDBListener>(
		HandlerEvent::InsertOrReplace,
		"alchemy_collections",
		context_data.clone(),
	);

	trigger.subscribe_to::<ArangoDBListener>(
		HandlerEvent::Remove,
		"alchemy_collections",
		context_data,
	);

	trigger.init().await.unwrap();

	loop {
		trigger.listen().await.unwrap()
	}
}
