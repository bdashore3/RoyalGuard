pub fn get_time(initial_time: u64, parameter: &str) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let value = match parameter {
        "s" => initial_time,
        "m" => initial_time * 60,
        "h" => initial_time * 3600,
        "d" => initial_time * 86400,
        "w" => initial_time * 604800,
        _ => {
            return Err("Invalid parameter input".into())
        }
    };

    Ok(value)
}