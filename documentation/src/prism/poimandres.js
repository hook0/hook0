// Hook0 Prism theme - based on poimandres structure with Hook0 design system colors
// Colors from src/css/custom.css dark mode variables

const theme = {
  plain: {
    color: "#fafafa", // --h0-text-primary
    backgroundColor: "#0d1117", // --h0-bg-code
  },
  styles: [
    {
      types: ["comment", "prolog", "doctype", "cdata"],
      style: {
        color: "#737373", // --h0-text-muted
      },
    },
    {
      types: ["punctuation"],
      style: {
        color: "#a3a3a3", // --h0-text-secondary
      },
    },
    {
      types: ["namespace"],
      style: {
        opacity: 0.7,
      },
    },
    {
      types: ["property", "tag", "constant", "symbol", "deleted"],
      style: {
        color: "#fafafa", // --h0-text-primary
      },
    },
    {
      types: ["boolean", "number"],
      style: {
        color: "#4ade80", // --h0-green
      },
    },
    {
      types: ["selector", "attr-value", "string", "char", "builtin", "inserted"],
      style: {
        color: "#4ade80", // --h0-green
      },
    },
    {
      types: ["attr-name", "operator", "entity", "url", "variable"],
      style: {
        color: "#fafafa", // white
      },
    },
    {
      types: ["atrule", "function", "class-name"],
      style: {
        color: "#fafafa", // white
      },
    },
    {
      types: ["keyword"],
      style: {
        color: "#818cf8", // --h0-indigo-light
      },
    },
    {
      types: ["regex", "important"],
      style: {
        color: "#f59e0b", // warning color
      },
    },
    {
      types: ["important", "bold"],
      style: {
        fontWeight: "bold",
      },
    },
    {
      types: ["italic"],
      style: {
        fontStyle: "italic",
      },
    },
  ],
};

module.exports = theme;
