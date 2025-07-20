use std::net::Ipv4Addr;
use std::process::Command;
use std::str;

fn get_gateway() -> Option<Ipv4Addr> {
    let output = Command::new("ip").arg("route").output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.starts_with("default via") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                return parts[2].parse().ok();
            }
        }
    }
    None
}

fn get_mac(ip: Ipv4Addr) -> Option<String> {
    let _ = Command::new("ping")
        .args(["-c", "1", "-W", "1", &ip.to_string()])
        .output()
        .ok()?;

    let output = Command::new("ip").arg("neigh").output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.contains(&ip.to_string()) && line.contains("lladdr") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(pos) = parts.iter().position(|&x| x == "lladdr") {
                return parts.get(pos + 1).map(|s| s.to_string());
            }
        }
    }
    None
}

fn mac_vendor(mac: &str) -> Option<String> {
    let url = format!("https://api.macvendors.com/{}", mac);
    let response = reqwest::blocking::get(&url).ok()?;
    if response.status().is_success() {
        response.text().ok()
    } else {
        None
    }
}

fn hostname(ip: Ipv4Addr) -> Option<String> {
    if let Ok(output) = Command::new("getent")
        .arg("hosts")
        .arg(ip.to_string())
        .output()
    {
        if output.status.success() {
            let line = String::from_utf8_lossy(&output.stdout);
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                return Some(parts[1].to_string());
            }
        }
    }

    if let Ok(output) = Command::new("nmblookup")
        .arg("-A")
        .arg(ip.to_string())
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if line.contains("<00>") && !line.contains("GROUP") {
                let hostname = line.trim().split_whitespace().next()?.to_string();
                return Some(hostname);
            }
        }
    }

    None
}

fn main() {
    let router_ip = get_gateway().expect("Failed to find the default gateway");
    println!("IP router: {}", router_ip);

    let mac = get_mac(router_ip).expect("Failed to get the MAC of the router");
    println!("MAC router: {}", mac);

    if let Some(hostname) = hostname(router_ip) {
        println!("Hostname router: {}", hostname);
    } else {
        println!("Hostname router: Unknown");
    }

    match mac_vendor(&mac) {
        Some(vendor) => println!("Router vendor: {}", vendor),
        None => println!("Unable to determine vendor by MAC"),
    }
}
