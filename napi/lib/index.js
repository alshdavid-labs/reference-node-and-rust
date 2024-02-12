const path = require('node:path');
const { Worker } = require('node:worker_threads');
const native = require('../index.node')

const WORKERS = process.argv[2] ? parseInt(process.argv[2], 10) : 4;

for (let i = 0; i < WORKERS; i++) {
  new Worker(path.join(__dirname, 'worker.js'))
}

native.register_main(WORKERS)
