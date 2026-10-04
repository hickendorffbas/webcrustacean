use std::time::Duration;


pub struct CommandLineOptions {
    pub url: Option<String>,
    pub screenshot_path: Option<String>,
    pub screenshot_delay: Duration,
}


pub fn parse_command_line_options(args: &Vec<String>) -> Result<CommandLineOptions, String> {
    let mut options = CommandLineOptions { url: None, screenshot_path: None, screenshot_delay: Duration::from_secs_f32(crate::DEFAULT_SCREENSHOT_DELAY_SECS) };

    let mut args_iter = args.iter().skip(1); //skip the programname itself
    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "--screenshot" => {
                let path = args_iter.next().ok_or("--screenshot needs a file path")?;
                options.screenshot_path = Some(path.clone());
            },
            "--screenshot-delay" => {
                let value = args_iter.next().ok_or("--screenshot-delay needs a number of seconds")?;
                let seconds = value.parse::<f32>().map_err(|_| format!("invalid number of seconds for --screenshot-delay: {}", value))?;
                if seconds < 0.0 {
                    return Err(format!("invalid number of seconds for --screenshot-delay: {}", value));
                }
                options.screenshot_delay = Duration::from_secs_f32(seconds);
            },
            _ => {
                if arg.starts_with("--") {
                    return Err(format!("unknown option: {}", arg));
                }
                if options.url.is_some() {
                    return Err(format!("only one url can be given, got a second one: {}", arg));
                }
                options.url = Some(arg.clone());
            },
        }
    }

    return Ok(options);
}
