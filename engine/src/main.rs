#![feature(derive_default_enum)]

extern crate juniper;
#[macro_use] extern crate juniper_codegen;

use actix_cors::Cors;
use actix_web::{
	http::header,
	middleware,
	web::{ 
		self, 
		Data
	},
	App,
	HttpServer, 
};

mod api;
mod lib;
mod meta;

use lib::CONFIG;

#[actix_web::main]
async fn main() -> std::io::Result<()>
{
	let app_port = CONFIG.app_port.parse::<u16>().unwrap();

	println!("Starting Alchemy on port {:?}", app_port);

	// Actix server
	HttpServer::new(|| {
		App::new()
			.app_data(Data::new(meta::graphql::schema()))
			.app_data(Data::new(api::graphql::schema()))
			.wrap(
				Cors::default()
					.allow_any_origin()
					.allowed_methods(vec!["POST", "GET"])
					.allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
					.allowed_header(header::CONTENT_TYPE)
					.supports_credentials()
					.max_age(3600)
			)
			.wrap(middleware::Compress::default())
			.wrap(middleware::Logger::default())
			.service(
				web::resource("/api/graphql")
					.route(web::post().to(api::graphql::server::graphql_api_route))
					.route(web::get().to(api::graphql::server::graphql_api_route))
			)
			.service(web::resource("/api/playground").route(web::get().to(api::graphql::server::playground_api_route)))
			.service(
				web::resource("/meta/graphql")
					.route(web::post().to(meta::graphql::server::graphql_meta_route))
					.route(web::get().to(meta::graphql::server::graphql_meta_route))
			)
			.service(web::resource("/meta/playground").route(web::get().to(meta::graphql::server::playground_meta_route)))
	})
	.bind(("0.0.0.0", app_port))?
	.run()
	.await
}
