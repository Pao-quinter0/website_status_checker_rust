use std::{
    env,
    fs::File,
    io::{self, BufRead},
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant, SystemTime},
};

use reqwest::blocking::Client;

#[derive(Debug)]
struct WebsiteStatus {
    url: String,
    action_status: Result<u16, String>,
    response_time: Duration,
    timestamp: SystemTime,
}

fn check_website(
    client: &Client,
    url: &str,
    timeout: Duration,
    retries: usize,
) -> WebsiteStatus {
    let mut attempt = 0;
    let start_time = Instant::now();

    loop {
        let result = client.get(url).timeout(timeout).send();

        let status = match result {
            Ok(resp) => Ok(resp.status().as_u16()),
            Err(e) => Err(e.to_string()),
        };

        if status.is_ok() || attempt >= retries {
            return WebsiteStatus {
                url: url.to_string(),
                action_status: status,
                response_time: start_time.elapsed(),
                timestamp: SystemTime::now(),
            };
        }

        attempt += 1;
        thread::sleep(Duration::from_millis(100));
    }
}

fn read_urls_from_file(path: &str) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    Ok(reader
        .lines()
        .filter_map(Result::ok)
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        })
        .collect())
}

fn print_usage_and_exit() -> ! {
    eprintln!(
        "Usage: website_checker [--file sites.txt] [URL ...] [--workers N] [--timeout S] [--retries N]"
    );
    std::process::exit(2);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut urls: Vec<String> = Vec::new();
    let mut file_path = None;
    let mut workers = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let mut timeout = Duration::from_secs(5);
    let mut retries = 0;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--file" => {
                i += 1;
                if i >= args.len() {
                    print_usage_and_exit();
                }
                file_path = Some(args[i].clone());
            }
            "--workers" => {
                i += 1;
                if i >= args.len() {
                    print_usage_and_exit();
                }
                workers = args[i].parse().unwrap_or(workers);
            }
            "--timeout" => {
                i += 1;
                if i >= args.len() {
                    print_usage_and_exit();
                }
                timeout = Duration::from_secs(args[i].parse().unwrap_or(5));
            }
            "--retries" => {
                i += 1;
                if i >= args.len() {
                    print_usage_and_exit();
                }
                retries = args[i].parse().unwrap_or(0);
            }
            _ if args[i].starts_with("--") => {
                print_usage_and_exit();
            }
            other => {
                urls.push(other.to_string());
            }
        }
        i += 1;
    }

    if let Some(path) = file_path {
        match read_urls_from_file(&path) {
            Ok(mut file_urls) => urls.append(&mut file_urls),
            Err(_) => {
                eprintln!("Failed to read file: {}", path);
                std::process::exit(1);
            }
        }
    }

    if urls.is_empty() {
        print_usage_and_exit();
    }

    let (tx, rx) = mpsc::channel::<String>();
    let rx = Arc::new(Mutex::new(rx));

    for url in urls {
        tx.send(url).unwrap();
    }
    drop(tx); 

    let (result_tx, result_rx) = mpsc::channel::<WebsiteStatus>();
    let client = Client::builder().build().unwrap();

    let mut handles = vec![];

    for _ in 0..workers {
        let rx = Arc::clone(&rx);
        let tx = result_tx.clone();
        let client = client.clone();
        let timeout = timeout.clone();

        let handle = thread::spawn(move || {
            while let Ok(url) = rx.lock().unwrap().recv() {
                let status = check_website(&client, &url, timeout, retries);
                println!(
                    "{} - {} - {:?}",
                    status.url,
                    status
                        .action_status
                        .as_ref()
                        .map(|code| code.to_string())
                        .unwrap_or_else(|e| e.to_string()),
                    status.response_time
                );
                tx.send(status).unwrap();
            }
        });

        handles.push(handle);
    }

    drop(result_tx); 

    let mut results = vec![];
    for result in result_rx {
        results.push(result);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let mut json = String::from("[\n");
    for (i, status) in results.iter().enumerate() {
        let json_obj = format!(
            "  {{ \"url\": \"{}\", \"status\": {}, \"time_ms\": {}, \"timestamp\": {:?} }}",
            status.url,
            match &status.action_status {
                Ok(code) => format!("{}", code),
                Err(err) => format!("\"{}\"", err),
            },
            status.response_time.as_millis(),
            status.timestamp.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
        );
        json.push_str(&json_obj);
        if i != results.len() - 1 {
            json.push_str(",\n");
        }
    }
    json.push_str("\n]\n");

    std::fs::write("status.json", json).expect("Failed to write status.json");
}
