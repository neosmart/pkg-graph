// extern crate serde_derive;
extern crate serde;
extern crate serde_json;

mod package;
use package::Package;
use std::collections::{HashSet, HashMap};
use std::process::Command;

type DepTree = HashMap<String, Option<Vec<String>>>;

fn get_dep_tree(package_name: &str, mut forward_deps: &mut DepTree) -> Result<(), String> {
    if !forward_deps.contains_key(package_name) {
        let inner_deps = retrieve_dependencies(package_name)?;
        match inner_deps.as_ref() {
            None => {},
            Some(x) => for dep in x {
                get_dep_tree(&dep, &mut forward_deps)?;
            }
        }
        forward_deps.insert(package_name.to_string(), inner_deps);
    }

    return Ok(());
}

fn main() {
    let args: HashSet<String> = std::env::args().skip(1).collect();

    let all_packages = list_packages().unwrap();
    let packages: Vec<&Package> = match args.len() {
        0 => all_packages.iter().collect(),
        _ => all_packages.iter().filter(|ref p| args.contains(&p.name)).collect()
    };

    let mut deptree = DepTree::new();
    for package in &packages {
        get_dep_tree(&package.name, &mut deptree)
            .expect(&format!("Could not get dependencies for package {}", package.name));
    }

    use std::collections::hash_map::Entry::{Occupied, Vacant};
    let mut reverse_tree = DepTree::new();
    for (key, val) in &deptree {
        // we have to explicitly insert all keys upfront in case no one depends on them
        if reverse_tree.get(key).is_none() {
            reverse_tree.insert(key.clone(), None);
        }

        let val: &Vec<String> = match val.as_ref() {
            None => { continue; },
            Some(x) => x
        };

        for dep in val {
            match reverse_tree.entry(dep.to_string()) {
                Vacant(entry) => {
                    let mut v = Vec::new();
                    v.push(key.clone());
                    entry.insert(Some(v));
                },
                Occupied(ref mut entry) => {
                    let is_none = {
                        entry.get().is_none()
                    };
                    if is_none {
                        let mut v = Vec::new();
                        v.push(key.clone());
                        entry.insert(Some(v));
                    }
                    else {
                        let mut v = entry.get_mut().as_mut().unwrap();
                        v.push(key.clone());
                    }
                }
            }
        }
    }

    // begin output
    println!("digraph {{");
    for (package, dependants) in reverse_tree {
        print!("\t");
        let this_deps: &Option<Vec<String>> = &dependants;
        match this_deps {
            None => println!("\"{}\";", package),
            Some(deps) => {
                match deps.len() {
                    1 => println!("\"{}\" -> \"{}\";", package, deps[0]),
                    _ => {
                        print!("\"{}\" -> {{ ", package);
                        for dep in deps {
                            print!("\"{}\" ", dep);
                        }
                        println!("}};");
                    }
                }
            }
        }
    }

    println!("}}");
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
            dependencies: None,
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


fn retrieve_dependencies(package_name: &str) -> Result<Option<Vec<String>>, String> {
    use serde_json::{Value};
    use std::process::Stdio;

    let cmd = Command::new("pkg")
        .args(&["info", "--raw-format", "json", "--raw", package_name])
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run pkg info for package {}: {:?}", package_name, e))?;

    let output = cmd.wait_with_output()
        .map_err(|e| format!("Unable to wait on pkg info for package {}: {:?}", package_name, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("pkg info for package {} failed: {}", package_name, stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    let deserialized: Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("JSON deserialization error: {:?}", e))?;

    let deps = match deserialized["deps"].as_object() {
        None => return Ok(None),
        Some(x) => x
    };

    let mut pdeps = Vec::new();
    for dep in deps.keys() {
        pdeps.push(dep.clone());
    }

    return Ok(Some(pdeps));
}
