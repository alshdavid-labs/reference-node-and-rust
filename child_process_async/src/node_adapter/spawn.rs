use std::process::Stdio;

use tokio::io::AsyncWriteExt;
use tokio::process::Child;
use tokio::process::Command;

use super::js::get_js;

pub async fn spawn_node_js(
  port: &u16,
  worker_count: &usize,
) -> Child {
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
  stdin.write(script.as_bytes()).await.unwrap();
  stdin.flush().await.unwrap();
  drop(stdin);

  return child;
}
