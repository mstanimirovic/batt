#![allow(dead_code)]

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

const UNUSED_KEYS: [&str; 11] = [
    "DEVTYPE",
    "POWER_SUPPLY_POWER_NOW",
    "POWER_SUPPLY_VOLTAGE_NOW",
    "POWER_SUPPLY_VOLTAGE_MIN_DESIGN",
    "POWER_SUPPLY_PRESENT",
    "POWER_SUPPLY_ENERGY_NOW",
    "POWER_SUPPLY_ENERGY_FULL",
    "POWER_SUPPLY_ENERGY_FULL_DESIGN",
    "POWER_SUPPLY_SERIAL_NUMBER",
    "POWER_SUPPLY_NAME",
    "POWER_SUPPLY_TYPE",
];

#[derive(Debug, Clone)]
struct PowerDevice {
    name: String,
    dtype: String,
    data: BTreeMap<String, String>,
}

impl PowerDevice {
    fn new(path: PathBuf) -> Self {
        let data = parse_uevent(read_file(&path.join("uevent")));
        let name = data.get("POWER_SUPPLY_NAME").unwrap().clone();
        let dtype = data.get("POWER_SUPPLY_TYPE").unwrap().clone();
        PowerDevice { name, dtype, data }
    }

    fn calc_data(&mut self) {
        match self.data.get("POWER_SUPPLY_TYPE") {
            Some(devtype) if devtype != &String::from("Battery") => return,
            _ => {}
        }

        self.calc_health_percentage();
        self.calc_power_consumption();
        self.calc_time_remaining();
    }

    fn calc_health_percentage(&mut self) {
        let energy_full = self
            .data
            .get("POWER_SUPPLY_ENERGY_FULL")
            .unwrap_or(&String::from("0"))
            .clone();

        let energy_full_capacity = self
            .data
            .get("POWER_SUPPLY_ENERGY_FULL_DESIGN")
            .unwrap_or(&String::from("0"))
            .clone();

        let health_percentage = calc_percentage(
            energy_full.clone().parse().unwrap_or(0.0),
            energy_full_capacity.clone().parse().unwrap_or(0.0),
        );

        self.data.insert(
            String::from("POWER_SUPPLY_HEALTH_PERCENTAGE"),
            health_percentage.to_string(),
        );
    }

    fn calc_power_consumption(&mut self) {
        let value = self
            .data
            .get("POWER_SUPPLY_POWER_NOW")
            .unwrap_or(&String::from("0"))
            .parse()
            .unwrap_or(0);

        self.data.insert(
            String::from("POWER_SUPPLY_POWER_CONSUMPTION"),
            format!("{:.2}", value as f32 / 1_000_000.0),
        );
    }

    fn calc_time_remaining(&mut self) {
        let energy_full = self
            .data
            .get("POWER_SUPPLY_ENERGY_FULL")
            .unwrap_or(&String::from("0.0"))
            .parse::<f32>()
            .unwrap_or(0.0)
            / 1_000_000.0;

        let energy_now = self
            .data
            .get("POWER_SUPPLY_ENERGY_NOW")
            .unwrap_or(&String::from("0.0"))
            .parse::<f32>()
            .unwrap_or(0.0)
            / 1_000_000.0;

        let power_now = self
            .data
            .get("POWER_SUPPLY_POWER_NOW")
            .unwrap_or(&String::from("0.0"))
            .parse::<f32>()
            .unwrap_or(0.0)
            / 1_000_000.0;

        let (key, mut value) = match self.data.get("POWER_SUPPLY_STATUS").unwrap().as_str() {
            "Discharging" => ("EMPTY", energy_now / power_now),
            "Charging" => ("FULL", (energy_full - energy_now) / power_now),
            _ => ("N/A", 0.0),
        };

        // convert hours to hours and minutes
        let h = ((value / 1.0) - 0.5).round();
        let r = (value % 1.0) * 60.0 / 100.0;
        value = h + r;

        self.data.insert(
            format!("POWER_SUPPLY_TIME_REMAINING_UNTIL_{}", key),
            format!("{:.2}", value),
        );
    }

    fn format_key(&self, mut key: String) -> String {
        if key.len() > 13 {
            key = key.split_off(13);
        }
        key.replace("_", " ").to_lowercase()
    }

    fn max_key_size(&self) -> usize {
        self.data.keys().map(|k| k.len()).max().unwrap_or(0) - 10
    }
}

impl std::fmt::Display for PowerDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        let max_key_size = self.max_key_size();

        output.push_str(&format!("Device: {} {}\n", self.name, self.dtype));
        for (key, value) in &self.data {
            let formated_key = self.format_key(key.clone());
            if UNUSED_KEYS.contains(&key.as_str()) {
                continue;
            }
            output.push_str(&format!("\t{:<max_key_size$} = {}\n", formated_key, value));
        }
        write!(f, "{}", output)
    }
}

fn calc_percentage(part: f32, total: f32) -> f32 {
    let percentage = (part / total) * 100.0;
    (percentage * 100.0).round() / 100.0
}

fn read_file(path: &PathBuf) -> String {
    match std::fs::read_to_string(path) {
        Ok(content) => content.trim().to_string(),
        Err(error) => panic!("Failed to read file: {}", error),
    }
}

fn parse_uevent(uevent: String) -> BTreeMap<String, String> {
    let mut data = BTreeMap::new();
    for line in uevent.lines() {
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() == 2 {
            data.insert(parts[0].to_string(), parts[1].to_string());
        }
    }

    // return the parsed data
    data
}

fn main() {
    let power_supply = Path::new("/sys/class/power_supply");
    if !power_supply.exists() {
        println!("power_supply directory not found");
        return;
    }

    let mut devices: Vec<PowerDevice> = vec![];
    if let Ok(entries) = fs::read_dir(power_supply) {
        for entry in entries {
            devices.push(PowerDevice::new(entry.unwrap().path()));
        }
    }

    for device in devices.iter_mut() {
        device.calc_data();
        println!("{}", device);
    }
}
