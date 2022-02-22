use crate::lib::schema::get_all_entries;

pub async fn generate_sdl()
{
	let entries = get_all_entries().await;

	let mut sdl = String::new();

	println!("----- SDL GENERATION -----");

	for entry in entries.clone().iter()
	{
		let name = &entry["name"].as_str().unwrap();
		let entry_properties = &entry["schema"].get("properties").unwrap();

		let mut props = String::new();

		for prop in entry_properties.as_object().unwrap().iter()
		{
			let prop_name = &prop.0;
			let prop_type = &prop.1["type"].as_str().unwrap();

			props.push_str(&format!("\t{}: {}\n", prop_name, prop_type));
		}

		println!("Props of {} are: {}", name, props);

		let object = format!(
			r#"
				type {} {{
					test
				}}
			"#,
			format!("{}{}", (&name[..1].to_string()).to_uppercase(), &name[1..])
		);

		sdl.push_str(&object);
		// println!("{:?}", entry);
	}

	println!("----- SDL GENERATED -----");
	println!("SDL: {}", sdl);
}