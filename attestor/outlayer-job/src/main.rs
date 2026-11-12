//! OutLayer WASI entrypoint. Stdin is the public JSON input. `HL_ADDRESS` comes from the environment.
//! This process never prints that address or the raw fills.
use outlayer_job::execute;
use std::io::{self, Read, Write};

#[cfg(all(target_arch = "wasm32", target_os = "wasi"))]
use outlayer_job::{Fetch, HYPERLIQUID_INFO_URL};

#[cfg(not(all(target_arch = "wasm32", target_os = "wasi")))]
use outlayer_job::RejectingFetch;

#[cfg(all(target_arch = "wasm32", target_os = "wasi"))]
struct WasiFetch;

#[cfg(all(target_arch = "wasm32", target_os = "wasi"))]
impl Fetch for WasiFetch {
    fn post_info(&self, body: &str) -> Result<String, String> {
        let response = wasi_http_client::Client::new()
            .post(HYPERLIQUID_INFO_URL)
            .header("Content-Type", "application/json")
            .body(body.as_bytes())
            .connect_timeout(std::time::Duration::from_secs(20))
            .send()
            .map_err(|_| "hyperliquid request failed".to_string())?;
        let status = response.status();
        let bytes = response
            .body()
            .map_err(|_| "hyperliquid response failed".to_string())?;
        if status != 200 {
            return Err(format!("hyperliquid http {status}"));
        }
        String::from_utf8(bytes).map_err(|_| "hyperliquid response is not utf-8".to_string())
    }
}

fn main() {
    let mut input = String::new();
    let _ = io::stdin().read_to_string(&mut input);
    let address = std::env::var("HL_ADDRESS").ok();
    #[cfg(all(target_arch = "wasm32", target_os = "wasi"))]
    let fetch = WasiFetch;
    #[cfg(not(all(target_arch = "wasm32", target_os = "wasi")))]
    let fetch = RejectingFetch;
    match execute(&input, address.as_deref(), &fetch) {
        Ok(output) => {
            if write_stdout(&output).is_err() {
                std::process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

fn write_stdout(output: &str) -> Result<(), ()> {
    let mut stdout = io::stdout();
    stdout.write_all(output.as_bytes()).map_err(|_| ())?;
    stdout.write_all(b"\n").map_err(|_| ())?;
    stdout.flush().map_err(|_| ())
}
