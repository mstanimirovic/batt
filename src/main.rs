use std::collections::BTreeMap;

const UNUSED_KEYS: [&'static str; 9] = [
    "DEVTYPE",
    "POWER_NOW",
    "VOLTAGE_NOW",
    "VOLTAGE_MIN_DESIGN",
    "PRESENT",
    "ENERGY_NOW",
    "ENERGY_FULL",
    "ENERGY_FULL_DESIGN",
    "SERIAL_NUMBER",
];

fn calc_percentage(part: f32, total: f32) -> f32 {
    let percentage = (part / total) * 100.0;
    (percentage * 100.0).round() / 100.0
}

fn read_file(filename: &str) -> String {
    match std::fs::read_to_string(filename) {
        Ok(content) => content.trim().to_string(),
        Err(error) => panic!("Failed to read file: {}", error),
    }
}

fn parse_uevent(uevent: &str) -> BTreeMap<String, String> {
    let mut data = BTreeMap::new();
    for line in uevent.lines() {
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() == 2 {
            data.insert(parts[0].to_string(), parts[1].to_string());
        }
    }

    // append percentage sign to capacity
    let capacity = data.get_mut("POWER_SUPPLY_CAPACITY").unwrap();
    capacity.push_str("%");

    // return the parsed data
    data
}

fn calc_health_percentage(data: &mut BTreeMap<String, String>) {
    let energy_full = data
        .get("POWER_SUPPLY_ENERGY_FULL")
        .unwrap_or(&String::from("0"))
        .clone();

    let energy_full_capacity = data
        .get("POWER_SUPPLY_ENERGY_FULL_DESIGN")
        .unwrap_or(&String::from("0"))
        .clone();

    let health_percentage = calc_percentage(
        energy_full.clone().parse().unwrap_or(0.0),
        energy_full_capacity.clone().parse().unwrap_or(0.0),
    );

    data.insert(
        String::from("POWER_SUPPLY_HEALTH_PERCENTAGE"),
        health_percentage.to_string() + "%",
    );
}

fn print_key_value(key: &String, value: &String, width: usize) {
    let key = key.clone().replace("_", " ").to_lowercase();
    print!("{:<width$}", &key);
    println!("{}", value)
}

fn data_console_dump(data: &BTreeMap<String, String>) {
    let longest_key = data
        .keys()
        .max_by_key(|k| k.len())
        .unwrap_or(&String::from(""))
        .clone();

    let max_key_size = if longest_key.len() > 10 {
        longest_key.len() - 10
    } else {
        10
    };

    for (key, value) in data {
        let trimmed_key = key.clone().replace("POWER_SUPPLY_", "");
        if !UNUSED_KEYS.contains(&trimmed_key.as_str()) {
            print_key_value(&trimmed_key, value, max_key_size);
        }
    }
}

fn main() {
    // uevent file contains all the information about the battery
    // KEY=VALUE in this specific format
    let uevent = read_file("/sys/class/power_supply/BAT0/uevent");
    let mut data: BTreeMap<String, String> = parse_uevent(&uevent);

    // (ENERGY_FULL / ENERGY_FULL_DESIGN) * 100 = HEALTH_PERCENTAGE
    calc_health_percentage(&mut data);
    data_console_dump(&data);
}
