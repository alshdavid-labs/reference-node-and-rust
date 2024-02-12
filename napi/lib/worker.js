const native = require('../index.node')

const resolvers = {}

function load_resolver({ specifier }) {
  resolvers[specifier] = require(specifier)
}

globalThis.load_resolver = load_resolver
globalThis.resolvers = resolvers

native.register_worker()
