module.exports = function resolve({ from_path, specifier }) {
  return {
    file_path: `${from_path}/${specifier}`
  }
}
