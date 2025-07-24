
**1. Core Networking & Packet Manipulation (Essential):**

*   **`libpnet` (or `pnet`):** This is the primary and most mature choice for low-level packet crafting and manipulation in Rust.
    *   **Why:** It provides direct access to raw sockets (essential for sending/receiving ARP packets and intercepting traffic). It includes comprehensive libraries for parsing and constructing Ethernet, IP, TCP, UDP, and importantly, ARP packets. It handles the serialization/deserialization needed for crafting spoofed packets.
    *   **Alternative:** `socket2` is lower-level and gives you direct bindings to the OS socket API. While powerful, `libpnet` often uses `socket2` internally and provides a higher-level abstraction specifically for packet manipulation, making it generally more convenient for this task. You might use `socket2` *with* `libpnet` or for very specific socket configuration needs.

**2. Asynchronous Runtime (Essential):**

*   **`tokio`:** The dominant asynchronous runtime in the Rust ecosystem.
    *   **Why:** Network operations (sending/receiving packets, potentially scanning) benefit greatly from async/await. Managing the continuous ARP spoofing loop, handling multiple targets, or performing concurrent discovery tasks without blocking the main thread is much easier and more efficient with `tokio`. It integrates well with other async libraries.
    *   **Alternative:** `async-std` is another option, but `tokio` has broader adoption and ecosystem support.

**3. Host Discovery (Built-in or Helper Crates):**

*   **Built-in (`std::net`) + `tokio`:** Implement basic discovery directly.
    *   **ICMP Ping:** Requires raw sockets (`libpnet`). Send ICMP Echo Requests.
    *   **TCP Connect Scan:** Use `tokio::net::TcpStream::connect` with a timeout. If it connects quickly or fails quickly, it indicates a host might be present.
    *   **ARP Ping:** Craft and send ARP requests using `libpnet`.
*   **Helper Crate:**
    *   **`ipnetwork` or `cidr`:** Extremely useful for parsing subnet notations (e.g., `192.168.1.0/24`) and iterating through IP addresses within that range for scanning purposes.

**4. Command-Line Interface (tui) Parsing:**

*   **`clap`:** The standard and most feature-rich crate for building tuis in Rust.
    *   **Why:** Easily define command-line arguments (target IPs, interface, bandwidth limit, mode - drop/limit, etc.), subcommands, and generate help messages. Integrates well with async code.

**5. Logging:**

*   **`log` + `env_logger` (or `tracing`):** Standard choices.
    *   **Why:** Essential for debugging, showing progress (e.g., "Target found", "Spoofing started", "Packets dropped"), and providing user feedback. `log` is the facade, `env_logger` is a simple backend. `tracing` is a newer, more advanced alternative offering structured logging.

**6. Data Structures & Utilities:**

*   **`macaddr` or `eui48`:** For robustly handling and manipulating MAC addresses (formatting, parsing).
*   **`ipnetwork` or `cidr`:** (Already mentioned for discovery) Also useful for general IP/network operations throughout the code.

**7. Privilege Handling:**

*   **Built-in OS Checks + User Guidance:** Rust itself doesn't have a specific crate for privilege escalation, but you can use standard library features (`std::env`) to check the effective user ID (e.g., `unsafe { libc::geteuid() }` on Unix, checking membership in `BUILTIN\Administrators` on Windows via winapi/sys calls) and exit with an error message if insufficient privileges are detected. The program should ideally be run with `sudo` (Linux/macOS) or as Administrator (Windows).

**Selected Core Stack Summary:**

*   **Packet Manipulation:** `libpnet`
*   **Async Runtime:** `tokio`
*   **tui Parsing:** `clap`
*   **Logging:** `log` + `env_logger`
*   **IP/MAC Handling:** `ipnetwork`, `cidr`, `macaddr`
*   **Core Discovery:** `std::net` + `tokio` + `libpnet` (for raw packet scans)

This combination provides the necessary tools to build the core functionality directly in Rust without relying on external executables like `nmap`. `libpnet`.