# WEBSITE STATUS CHECKER PAOLA QUINTERO
# website_status_checker_rust

A concurrent website monitoring tool written in **Rust**, built for the CSCI-3334-01 class final project assignment. This tool checks multiple websites in parallel, reports status and response times, and outputs results in both terminal and JSON formats.

## Project Overview

This command-line utility checks the availability of websites using a fixed pool of threads. It supports:

- Input from a file or command-line arguments
- Configurable number of worker threads
- Timeout and retry support
- Live human-readable output
- Final output to a `status.json` file
- Uses **only** the standard library and **reqwest** (with `blocking` feature)

## Functional Requirements

### Website Status Structure

The tool captures for each URL:

- `url`: Original URL string
- `status`: `u16` status code on success (e.g. 200)
- `error`: If failed, a string describing the error
- `response_time_ms`: Time taken for the request in milliseconds
- `timestamp`: When the response completed (UTC)

These fields are printed live and saved to `status.json`.

## Build instructions 

Ensure Rust (v1.78 or later) is installed. Then, build with:
```bash
cargo build --release 
```


## Usage


website_checker [--file sites.txt] [URL ...]
                [--workers N] [--timeout S] [--retries N]
### Required:
You must provide either:

- --file <file>: a list of URLs (one per line), OR
- one or more URLs as positional arguments.

### Optional Flags:

| Flag                   | Description                   |
|------------------------|-------------------------------|
| --file                 | Path to file containing URLs  |
| --workers N            | Number of threads in the pool |
| --timeout S            | Request timeout in seconds    |
| --retries N            | Number of tries to retry failed requests |

## Example Usage 
### Manual Input (success and failure)
cargo run -- https://www.google.com https://thidoesnotexist.com
### Output
https://www.google.com - 200 OK - 103ms

https://thisdoesnotexist.com - error: DNS resolution failed - 502ms

### File Input
sites.txt

https://www.google.com

https://thisdoesnotexist.com

cargo run -- --file sites.txt --workers 4 --timeout 3 --retries 1

## Output Details 
### Live Terminal Output:

Each result is printed immediately:

https://www.google.com - 200 OK - 103ms

https://thisdoesnotexist.com - error: DNS resolution failed - 502ms

status.json Output:

A JSON file is created after all URLs are processed:

[
  {

    "url": "https://www.google.com",

    "status": 200,

    "response_time_ms": 103,

    "timestamp": "2025-05-14T20:01:12Z"
  },
  {

    "url": "https://thisdoesnotexist.com",

     "error": "DNS resolution failed",
    
    "response_time_ms": 502,

    "timestamp": "2025-05-14T20:01:12Z"
  }
]

Field names:

"url": original URL string

"status": HTTP status code if successful

"error": Error message (if failed)

"response_time_ms": Duration of the request

"timestamp": System UTC time of request completion

## How It Works

- A fixed worker thread pool is created.
- A channel sends URLs to threads.
- Each thread:
  - Makes a blocking HTTP request using reqwest
  - Tracks response time and timestamp
  - Retries if specified
  - Sends results back to the main thread
- Results are:
  - Printed immediately to terminal
  - Appended to a shared list and saved to JSON at the end




