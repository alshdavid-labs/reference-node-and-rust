/*
  Runs on each worker

  This runs once per worker and allows a worker to notify
  the main thread that it is ready to receive work.

  It facilitates sending the worker "on ready" event and handles 
  communications with the worker, casting types to/from JavaScript land
*/
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;

use neon::prelude::*;

use once_cell::sync::Lazy;

use crate::worker_farm::PluginRequest;
use crate::worker_farm::PluginResponse;
use crate::worker_farm::RunResolverResponse;

pub type WorkerSender = Sender<(PluginRequest, tokio::sync::oneshot::Sender<PluginResponse>)>;

/*
  This is state that is shared between the Node workers and facilitates
  communicate between Node threads without going through JavaScript

  Global static variables are shared between multiple 
  instances of the same napi module
*/
pub static WORKER_LOADED: Lazy<
  Arc<Mutex<(Sender<WorkerSender>, Option<Receiver<WorkerSender>>)>>,
> = Lazy::new(|| {
  let (tx, rx) = channel::<WorkerSender>();
  Arc::new(Mutex::new((tx, Some(rx))))
});

pub fn register_worker(mut cx: FunctionContext) -> JsResult<JsUndefined> {
  let ctx_load_resolver = cx.global().get_value(&mut cx, "load_resolver").unwrap();
  let ctx_load_resolver: Handle<JsFunction> = ctx_load_resolver.downcast(&mut cx).unwrap();

  let ctx_resolvers = cx.global().get_value(&mut cx, "resolvers").unwrap();
  let ctx_resolvers: Handle<JsObject> = ctx_resolvers.downcast(&mut cx).unwrap();

  let (tx_call, rx_call) =
    channel::<(PluginRequest, tokio::sync::oneshot::Sender<PluginResponse>)>();

  WORKER_LOADED.lock().unwrap().0.send(tx_call).unwrap();

  while let Ok((req, res)) = rx_call.recv() {
    match req {
      PluginRequest::LoadResolver(req) => {
        let js_obj = cx.empty_object();
        let js_specifier = cx.string(req.specifier);
        js_obj.set(&mut cx, "specifier", js_specifier).unwrap();

        ctx_load_resolver
          .call_with(&mut cx)
          .arg(js_obj)
          .apply::<JsUndefined, FunctionContext>(&mut cx)?;

        res.send(PluginResponse::LoadResolver).unwrap();
      }
      PluginRequest::RunResolver(key, req) => {
        let ctx_resolver_fn = ctx_resolvers.get_value(&mut cx, key.as_str()).unwrap();
        let ctx_resolver_fn: Handle<JsFunction> = ctx_resolver_fn.downcast(&mut cx).unwrap();

        let js_obj = cx.empty_object();
        let js_from_path = cx.string(req.from_path.to_str().unwrap());
        let js_specifier = cx.string(req.specifier);

        js_obj.set(&mut cx, "from_path", js_from_path).unwrap();
        js_obj.set(&mut cx, "specifier", js_specifier).unwrap();

        let result = ctx_resolver_fn
          .call_with(&mut cx)
          .arg(js_obj)
          .apply::<JsObject, FunctionContext>(&mut cx)?;

        let file_path: Handle<JsString> = result
          .get_value(&mut cx, "file_path")
          .unwrap()
          .downcast(&mut cx)
          .unwrap();
        let file_path = PathBuf::from(file_path.value(&mut cx));

        res
          .send(PluginResponse::RunResolver(RunResolverResponse {
            file_path,
          }))
          .unwrap();
      }
    }
  }

  return Ok(cx.undefined());
}
