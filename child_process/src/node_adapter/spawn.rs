use std::process::Stdio;
use std::process::Command;
use std::process::Child;
use std::io::Write;

use super::js::get_js;

pub fn spawn_node_js(port: &u16, worker_count: &usize) -> Child {
  let mut command = Command::new("node");
  command.arg("--title");
  command.arg("child_process_node");

  command.stderr(Stdio::inherit());
  command.stdout(Stdio::inherit());
  command.stdin(Stdio::piped());

  let mut child = command.spawn().unwrap();

  // Execute the glue code within Node.js
  let mut stdin = child.stdin.take().unwrap();
  let script = get_js(port, worker_count);
  stdin.write(script.as_bytes()).unwrap();
  drop(stdin);

  return child;
}