// @ts-check
/*
  The entry point for the Node.js process simply spawns
  child processes
*/
const { Worker } = require('node:worker_threads');

const worker_code = `__MACH_WORKER_SCRIPT__`
const worker_count = parseInt('__MACH_WORKER_COUNT__', 10)

for (let i = 0; i < worker_count; i++) {
  new Worker(atob(worker_code), { eval: true })
}
