use std::env;
use std::time::{SystemTime, Duration};

fn sys_time() -> i64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or(Duration::ZERO).as_secs() as i64
}

fn timestamp_date(timestamp: i64) -> String {
    let format = time::format_description::parse_borrowed::<3>("[year]-[month]-[day]").expect("invalid date format");
    time::OffsetDateTime::from_unix_timestamp(timestamp)
        .expect("invalid timestamp")
        .format(&format)
        .expect("error formatting date")
}

fn main() {
    // use SOURCE_DATE_EPOCH if present to support reproducible builds
    let timestamp = match env::var("SOURCE_DATE_EPOCH") {
        Ok(val) => { val.parse::<i64>().expect("invalid value in env var SOURCE_DATE_EPOCH") }
        Err(_) => { sys_time() }
    };
    let build_date = timestamp_date(timestamp);
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);
}
