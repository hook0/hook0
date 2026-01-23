// Hook0 Service Worker
// Provides offline caching for static assets to improve mobile performance

const CACHE_NAME = 'hook0-v1';
const STATIC_ASSETS = [
  '/',
  '/style.css',
  '/mediakit/logo/110x110-white.png',
  '/mediakit/logo/256x256-white.png',
  '/mediakit/logo/512x512-white.png'
];

// Install event - cache static assets
self.addEventListener('install', function(event) {
  event.waitUntil(
    caches.open(CACHE_NAME)
      .then(function(cache) {
        return cache.addAll(STATIC_ASSETS);
      })
      .then(function() {
        return self.skipWaiting();
      })
  );
});

// Activate event - clean up old caches
self.addEventListener('activate', function(event) {
  event.waitUntil(
    caches.keys()
      .then(function(cacheNames) {
        return Promise.all(
          cacheNames
            .filter(function(cacheName) {
              return cacheName !== CACHE_NAME;
            })
            .map(function(cacheName) {
              return caches.delete(cacheName);
            })
        );
      })
      .then(function() {
        return self.clients.claim();
      })
  );
});

// Fetch event - stale-while-revalidate strategy
self.addEventListener('fetch', function(event) {
  // Only handle GET requests
  if (event.request.method !== 'GET') {
    return;
  }

  // Skip cross-origin requests and API calls
  var url = new URL(event.request.url);
  if (url.origin !== self.location.origin) {
    return;
  }

  // Skip tracking and analytics
  if (url.pathname.includes('piwik') || url.pathname.includes('matomo')) {
    return;
  }

  event.respondWith(
    caches.open(CACHE_NAME)
      .then(function(cache) {
        return cache.match(event.request)
          .then(function(cachedResponse) {
            var fetchPromise = fetch(event.request)
              .then(function(networkResponse) {
                // Only cache successful responses
                if (networkResponse && networkResponse.status === 200) {
                  cache.put(event.request, networkResponse.clone());
                }
                return networkResponse;
              })
              .catch(function() {
                // Network failed, return cached version if available
                return cachedResponse;
              });

            // Return cached response immediately, update in background
            return cachedResponse || fetchPromise;
          });
      })
  );
});
