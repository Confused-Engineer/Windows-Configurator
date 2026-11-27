//! System / network snapshot shown on the "System Information" page.
//! Collected on demand (only while that page is open), all tools in parallel.

use crate::platform;

#[derive(Clone, Default)]
pub struct SysInfo {
    pub systeminfo: String,
    pub ipconfig: String,
    pub netstat: String,
    pub arp: String,
    pub routes: String,
}

fn run(program: &str, args: &[&str]) -> String {
    match platform::capture(program, args) {
        Ok((_, out)) if !out.trim().is_empty() => out.trim_end().replace("\r\n", "\n"),
        Ok((status, _)) => format!("{program} produced no output ({status})"),
        Err(e) => format!("could not run {program}: {e}"),
    }
}

pub fn collect() -> SysInfo {
    std::thread::scope(|s| {
        let systeminfo = s.spawn(|| run("systeminfo", &[]));
        let ipconfig = s.spawn(|| run("ipconfig", &["/all"]));
        let netstat = s.spawn(|| run("netstat", &["-n"]));
        let arp = s.spawn(|| run("arp", &["-a"]));
        let routes = s.spawn(|| run("route", &["print"]));
        let join = |h: std::thread::ScopedJoinHandle<'_, String>| h.join().unwrap_or_default();
        SysInfo {
            systeminfo: join(systeminfo),
            ipconfig: join(ipconfig),
            netstat: join(netstat),
            arp: join(arp),
            routes: join(routes),
        }
    })
}
