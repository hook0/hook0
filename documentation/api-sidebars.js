// Wrapper for the generated API sidebar
let apiSidebar = [];
try {
  const generated = require('./api/sidebar.ts');
  apiSidebar = generated.default || generated;
} catch (e) {
  console.log('API sidebar not generated yet');
}

module.exports = {
  apiSidebar: apiSidebar,
};
