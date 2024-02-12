use base64::engine::general_purpose;
use base64::Engine as _;

const SCRIPT_MAIN: &str = include_str!("./assets/main.js");
const SCRIPT_WORKER: &str = include_str!("./assets/worker.js");

pub fn get_js(
  port: &u16,
  worker_count: &usize,
) -> String {
  let script_worker = SCRIPT_WORKER.replace("__MACH__PORT__", port.to_string().as_str());
  let script_worker_b64 = general_purpose::STANDARD.encode(&script_worker);
  let script = SCRIPT_MAIN
    .replace("__MACH_WORKER_SCRIPT_B64__", &script_worker_b64)
    .replace("__MACH_WORKER_SCRIPT__", &script_worker)
    .replace("__MACH_WORKER_COUNT__", &(worker_count - 1).to_string());
  return script;
}
