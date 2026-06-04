//! Smoke-test for `frontmost_bundle_id()` on Linux.
//!
//! Polls the focused application once per second and prints its identifier.
//! Switch between windows while it runs to verify detection.
//!
//! # Usage
//!
//! ```text
//! cargo build --example frontmost_app -p openlogi-hook
//! ./target/debug/examples/frontmost_app
//! ```

fn main() {
    #[cfg(not(target_os = "linux"))]
    {
        eprintln!("frontmost_app: Linux only");
        return;
    }
    #[cfg(target_os = "linux")]
    {
        println!("Polling focused app every second — switch windows to test.");
        loop {
            println!("{:?}", openlogi_hook::frontmost_bundle_id());
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}
