// Docusaurus client module that updates Mermaid SVG colors on theme change

var DARK_COLORS = {
  external:   { fill: '#3b82f6', stroke: '#bfdbfe', color: '#ffffff' },
  hook0:      { fill: '#22c55e', stroke: '#bbf7d0', color: '#ffffff' },
  customer:   { fill: '#f97316', stroke: '#fed7aa', color: '#ffffff' },
  processing: { fill: '#a855f7', stroke: '#e9d5ff', color: '#ffffff' },
  danger:     { fill: '#ef4444', stroke: '#fecaca', color: '#ffffff' },
};

var LIGHT_COLORS = {
  external:   { fill: '#dbeafe', stroke: '#60a5fa', color: '#1e3a5f' },
  hook0:      { fill: '#dcfce7', stroke: '#4ade80', color: '#14532d' },
  customer:   { fill: '#ffedd5', stroke: '#fb923c', color: '#7c2d12' },
  processing: { fill: '#ede9fe', stroke: '#a78bfa', color: '#3b0764' },
  danger:     { fill: '#fee2e2', stroke: '#f87171', color: '#7f1d1d' },
};

function applyMermaidTheme(isDark) {
  var colors = isDark ? DARK_COLORS : LIGHT_COLORS;

  document.querySelectorAll('.mermaid').forEach(function(svg) {
    Object.keys(colors).forEach(function(className) {
      var palette = colors[className];
      // Target node groups with the class
      svg.querySelectorAll('.node.' + className + ' > rect, .node.' + className + ' > polygon, .node.' + className + ' > circle, .node.' + className + ' > path').forEach(function(el) {
        el.style.fill = palette.fill;
        el.style.stroke = palette.stroke;
      });
      svg.querySelectorAll('.node.' + className + ' .nodeLabel').forEach(function(el) {
        el.style.color = palette.color;
      });
    });
  });
}

function isDarkMode() {
  return document.documentElement.getAttribute('data-theme') === 'dark';
}

// Use a standard export that Docusaurus client modules support
export function onRouteDidUpdate() {
  // Delay to let Mermaid render first
  setTimeout(function() { applyMermaidTheme(isDarkMode()); }, 100);
  setTimeout(function() { applyMermaidTheme(isDarkMode()); }, 500);
}

// Watch for theme changes
if (typeof window !== 'undefined') {
  var observer = new MutationObserver(function(mutations) {
    mutations.forEach(function(mutation) {
      if (mutation.attributeName === 'data-theme') {
        setTimeout(function() { applyMermaidTheme(isDarkMode()); }, 50);
      }
    });
  });

  // Start observing when DOM is ready
  if (document.documentElement) {
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme'] });
  }
}
