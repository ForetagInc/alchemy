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

mod lib;
mod meta;
use meta::graphql::{ schema, graphql_meta_route, playground_meta_route };

#[actix_web::main]
async fn main() -> std::io::Result<()>
{
	println!("Starting Alchemy on port 8080");

	// Actix server
	HttpServer::new(|| {
		App::new()
			.app_data(Data::new(schema()))
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
				web::resource("/meta/graphql")
					.route(web::post().to(graphql_meta_route))
					.route(web::get().to(graphql_meta_route))
			)
			.service(web::resource("/meta/playground").route(web::get().to(playground_meta_route)))
	})
	.bind(("0.0.0.0", 8080))?
	.run()
	.await
}
