// Wraps @scalar/docusaurus so the API reference script only loads on /api.
//
// The upstream plugin adds `<script src=".../@scalar/api-reference">` to every
// page through injectHtmlTags. That bundle is about 1.2 MB, synchronous, and
// sits in front of the content of every guide, while only the /api route uses
// it. This wrapper keeps the upstream route, options and configuration
// serialization untouched, drops the global tag, and swaps the route component
// for LazyApiReference, which fetches the same script when /api mounts.
//
// If you upgrade @scalar/docusaurus, check that its route component still reads
// `window.Scalar` on mount and that injectHtmlTags still only adds that script.

const path = require("node:path");
const scalarDocusaurus = require("@scalar/docusaurus").default;

const DEFAULT_SCRIPT_SRC = "https://cdn.jsdelivr.net/npm/@scalar/api-reference";

module.exports = function lazyScalarPlugin(context, options) {
  // injectHtmlTags is the global <script>; leaving it out is the whole point.
  const { injectHtmlTags, ...plugin } = scalarDocusaurus(context, options);

  return {
    ...plugin,
    contentLoaded(args) {
      const addRoute = (route) =>
        args.actions.addRoute({
          ...route,
          component: path.resolve(__dirname, "LazyApiReference.js"),
          scriptSrc: options.cdn ?? DEFAULT_SCRIPT_SRC,
        });
      return plugin.contentLoaded({
        ...args,
        actions: { ...args.actions, addRoute },
      });
    },
  };
};
