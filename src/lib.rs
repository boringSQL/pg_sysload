use pgrx::prelude::*;
use std::fs;
use std::io::Read;
use std::str::FromStr;
use std::sync::Once;

static INIT: Once = Once::new();

pgrx::pg_module_magic!();

#[pg_guard]
fn _PG_init() {
    INIT.call_once(|| {
        let loadavg_available = fs::metadata("/proc/loadavg").is_ok();
        if !loadavg_available {
            pgrx::error!("/proc/loadavg not found. Extension cannot load.");
        }
    });
}

#[pg_extern]
fn sys_loadavg() -> Option<Vec<f64>> {
    // read the contents of the /proc/loadavg
    let mut file = match fs::File::open("/proc/loadavg") {
        Ok(file) => file,
        Err(err) => {
            pgrx::error!("Error reading /proc/loadavg: {}", err);
        }
    };
    let mut contents = String::new();
    if let Err(err) = file.read_to_string(&mut contents) {
        pgrx::error!("Error reading /proc/loadavg: {}", err);
    }

    // extract the load average fields
    let fields = contents.split_whitespace().collect::<Vec<_>>();
    if fields.len() >= 3 {
        Some(
            fields[..3]
                .iter()
                .filter_map(|s| f64::from_str(s).ok())
                .collect(),
        )
    } else {
        pgrx::error!("Invalid format in /proc/loadavg");
    }
}
