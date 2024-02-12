/*
  This wrapper is responsible for creating a Node.js child process,
  spawning Node.js threads and having each worker thread connect to
  sockets on the host process.

  Data is transferred as strings and JSON, this can be made more
  efficient with a custom binary format - but JSON is fine for
  a demo.

  Messages are load balanced between Node workers using round robin.

  The glue code that is run within Node.js is piped in via stdin, 
  but a final implementation would probably ship the JS glue code 
  alongside the binary.

  This can probably be made more efficient using async tasks
*/
use std::collections::HashMap;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::TcpListener;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use serde::de::DeserializeOwned;
use serde::Serialize;

use tokio::sync::oneshot;

use super::spawn::spawn_node_js;
use super::NodeResponse;
use super::NodeWorker;

#[derive(Debug)]
pub struct NodeInstance {
  send_to: Arc<Mutex<usize>>,
  tx_shutdown: Sender<()>,
  workers: Vec<NodeWorker>,
}

impl NodeInstance {
  pub fn new(worker_count: usize) -> NodeInstance {
    // Create socket for Node.js to connect to
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    // Create Node.js child process and pipe JS into
    let mut child = spawn_node_js(&port, &worker_count);

    // Spawn a thread to communicate with each Node.js worker
    let mut workers = Vec::<NodeWorker>::new();
    
    for _ in 0..worker_count {
      // Wait for the Node.js worker thread to connect to the socket
      let Ok((stream, _)) = listener.accept() else {
        panic!("Unable to connect");
      };

      let stream = Arc::new(stream);
      let stream_write = stream.clone();
      let stream_read = stream.clone();

      let (tx_to_child, rx_to_child) = channel::<(String, String, String)>();

      // Messages going to Node.js worker
      // Thread to manage sending messages to the Node.js worker
      thread::spawn(move || {
        while let Ok((msg_ref, action, data)) = rx_to_child.recv() {
          let msg = format!("{}\n{}\n{}\n", msg_ref, action, data);
          if stream_write.as_ref().write(msg.as_bytes()).is_err() {
            break;
          };
        }
      });

      // Messages coming back from Node.js worker
      // This holds messages that are in-flight
      let pending_messages = Arc::new(Mutex::new(HashMap::<String, oneshot::Sender<String>>::new()));
      let pending_messages_thread = pending_messages.clone();
      
      // Thread to manage messages coming back from Node.js worker
      thread::spawn(move || {
        let mut reader = BufReader::new(stream_read.as_ref());
        let mut line = String::new();
        let mut incoming_msg_ref = String::new();

        // Read incoming message until /n character
        while let Ok(value) = reader.read_line(&mut line) {
          if value == 0 {
            break;
          }
          line.pop();
          let value = std::mem::take(&mut line);

          if incoming_msg_ref == "" {
            incoming_msg_ref = value;
            continue;
          }

          let incoming_msg_ref = std::mem::take(&mut incoming_msg_ref);

          let Some(listener) = pending_messages_thread
            .lock()
            .unwrap()
            .remove(&incoming_msg_ref)
          else {
            todo!();
          };

          listener.send(value).unwrap();
        }
      });

      workers.push(NodeWorker {
        tx_to_child,
        pending_messages,
      })
    }

    let (tx_shutdown, rx_shutdown) = channel::<()>();

    // Thread to listen for the shutdown event
    thread::spawn(move || {
      if rx_shutdown.recv().is_err() {
        return;
      }
      drop(listener);
      child.kill().unwrap();
      child.wait().unwrap();
    });

    return NodeInstance {
      send_to: Arc::new(Mutex::new(0)),
      tx_shutdown,
      workers,
    };
  }

  pub fn send<T>(
    &self,
    action: &str,
    data: &T,
  ) -> NodeResponse
  where
    T: ?Sized + Serialize,
  {
    // Pick the worker to send the message to using round robin
    let send_to = {
      let mut send_to = self.send_to.lock().unwrap();

      if *send_to >= self.workers.len() {
        *send_to = 0;
      }

      let send_index = (*send_to).clone();
      *send_to += 1;
      send_index
    };

    self.workers[send_to].send(action, data)
  }

  pub fn send_all<T>(
    &self,
    action: &str,
    data: &T,
  ) -> Result<(), ()>
  where
    T: ?Sized + Serialize,
  {
    let mut responses = vec![];

    for worker in &self.workers {
      let on_response = worker.send(action, data);
      responses.push(on_response);
    }

    for response in &mut responses {
      if response.recv_void().is_err() {
        return Err(());
      }
    }

    return Ok(());
  }

  pub fn send_blocking<T, U>(
    &self,
    action: &str,
    data: &T,
  ) -> Result<U, ()>
  where
    T: ?Sized + Serialize,
    U: DeserializeOwned,
  {
    let mut response = self.send(action, data);
    return response.recv();
  }

  pub fn shutdown(&self) -> Result<(), ()> {
    if self.tx_shutdown.send(()).is_err() {
      return Err(());
    }
    return Ok(());
  }
}

impl Drop for NodeInstance {
    fn drop(&mut self) {
        self.shutdown().ok();
    }
}
