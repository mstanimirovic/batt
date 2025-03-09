## Batt - Battery Information CLI Tool

![Workflow](https://github.com/mstanimirovic/batt/actions/workflows/rust.yml/badge.svg)

Batt is a lightweight command-line tool written in Rust that provides detailed information about your system's battery status. It is designed to be fast, efficient, and easy to use, making it a great utility for developers and system administrators who need quick access to battery metrics. Application runs only on Linux for now.

### Todo
- Implement a more user-friendly output format
- Improve error handling and reporting
- Add time remaining until full charge
- Add time remaining until empty

### Output example
```
Device: ADP0 Mains
	online    = 0

Device: BAT0 Battery
	capacity                      = 24
	capacity level                = Normal
	cycle count                   = 501
	health percentage             = 82.58
	manufacturer                  = Celxpert
	model name                    = L20C2PF0
	power consumption             = 11.14
	status                        = Discharging
	technology                    = Li-poly
	time remaining until empty    = 0.41
```
