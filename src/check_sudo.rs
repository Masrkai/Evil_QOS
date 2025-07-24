use nix::unistd::Uid;

pub async fn ensure_root() {
    if Uid::effective().is_root() {
        println!("✅ Running as root (sudo)");
    } else {
        println!("❌ Not running as root");
    }
}
