var fs = require('fs');
var path = require('path');

var SITEMAP_PATH = path.join(__dirname, '..', 'dist', 'sitemap.xml');

var EXCLUDE_PAGES = [
    '404.html',
    'subscription-success.html',
    'subscription-error.html',
];

if (!fs.existsSync(SITEMAP_PATH)) {
    console.log('sitemap.xml not found, skipping fix-sitemap');
    process.exit(0);
}

var content = fs.readFileSync(SITEMAP_PATH, 'utf8');
var today = new Date().toISOString().split('T')[0];

// Remove excluded pages
EXCLUDE_PAGES.forEach(function(page) {
    var regex = new RegExp('<url>\\s*<loc>[^<]*' + page.replace('.', '\\.') + '</loc>[\\s\\S]*?</url>', 'g');
    content = content.replace(regex, '');
});

// Strip .html extensions from URLs
content = content.replace(/(<loc>[^<]*)\.html(<\/loc>)/g, '$1$2');

// Convert /index to /
content = content.replace(/(<loc>[^<]*)\/index(<\/loc>)/g, '$1/$2');

// Add lastmod to all url entries that don't have one
content = content.replace(/<\/loc>\s*<\/url>/g, '</loc>\n    <lastmod>' + today + '</lastmod>\n  </url>');

// Clean up empty lines
content = content.replace(/\n\s*\n\s*\n/g, '\n');

fs.writeFileSync(SITEMAP_PATH, content, 'utf8');
console.log('fix-sitemap: done');
