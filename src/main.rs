#![feature(pattern)]
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

mod package;
use package::Package;
use std::process::Command;

fn main() {
	for mut package in list_packages().unwrap() {
		println!("Package: {}", package.name);
		println!("         {}", package.description);

		retrieve_dependencies(&mut package).unwrap();
        for dep in &package.dependencies {
            println!("         * {}", dep);
        }
	}
}

fn list_packages() -> Result<Vec<Package>, String> {

	let stdout = Command::new("pkg")
		.args(&["info"])
		.output()
		.map_err(|e| format!("{:?}", e))?
		.stdout;
	let stdout = String::from_utf8_lossy(&stdout);

	let mut packages = Vec::new();
	for line in stdout.split('\n') {
		let mut space_start = None;
		let mut space_end = None;

		let mut i = 0;
		for c in line.chars() {
			if space_start.is_none() {
				if c == ' ' {
					space_start = Some(i);
				}
			}
			else if c != ' ' {
				space_end = Some(i);
				break;
			}
			i += 1;
		}

		if space_start.is_none() || space_start.unwrap() == 0 || space_end.is_none() {
			continue;
		}

		let space_start = space_start.unwrap();
		let space_end = space_end.unwrap();

		assert!(space_end > space_start);

		let (name, version) = split_name_version(&line[0..space_start]).unwrap();
		let desc = &line[space_end..];

		packages.push(Package {
			name: name.to_string(),
            version: version.to_string(),
			description: desc.to_string(),
			dependencies: Vec::new(),
		});
	}

	return Ok(packages);
}

fn split_name_version<'a>(combined: &'a str) -> Option<(&'a str, &'a str)> {
    let last_dash = match combined.rfind('-') {
        None => return None,
        Some(x) => x
    };

    return Some((&combined[0..last_dash], &combined[last_dash + 1..]));
}


fn retrieve_dependencies(package: &mut Package) -> Result<(), String> {
    use serde_json::{Value, Error};
    use std::process::Stdio;

    let cmd = Command::new("pkg")
        .args(&["info", "--raw-format", "json", "--raw", &package.name])
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run pkg info for package {}: {:?}", package.name, e))?;

    let output = cmd.wait_with_output()
        .map_err(|e| format!("Unable to wait on pkg info for package {}: {:?}", package.name, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("pkg info for package {} failed: {}", package.name, stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    let deserialized: Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("JSON deserialization error: {:?}", e))?;

    let deps = &deserialized["deps"].as_object();
    if deps.is_none() {
        return Ok(());
    }

    for dep in deps.unwrap().keys() {
        let dep_str: &str = &format!("{}", dep);
        package.dependencies.push(dep_str.to_string());
    }

    return Ok(());
}
