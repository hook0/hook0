/**
 * SearchBar component wrapper for search tracking (Recommendation 1)
 *
 * This component wraps the docusaurus-search-local SearchBar component
 * to track internal site searches in Matomo.
 */
import React, { useEffect, useRef } from 'react';
import SearchBar from '@theme-original/SearchBar';

// Debounce helper to avoid tracking every keystroke
function debounce(func, wait) {
  let timeout;
  return function executedFunction(...args) {
    const later = () => {
      clearTimeout(timeout);
      func(...args);
    };
    clearTimeout(timeout);
    timeout = setTimeout(later, wait);
  };
}

export default function SearchBarWrapper(props) {
  const lastTrackedQuery = useRef('');

  useEffect(() => {
    if (typeof document === 'undefined') return;

    // Track search when user submits or selects a result
    const trackSearch = debounce((query, resultCount) => {
      if (!query || query.length < 2) return;
      if (query === lastTrackedQuery.current) return;

      lastTrackedQuery.current = query;

      if (typeof window !== 'undefined' && window._paq) {
        // Matomo Site Search tracking
        // Parameters: keyword, category (optional), searchCount (optional)
        window._paq.push([
          'trackSiteSearch',
          query,
          'documentation', // category
          resultCount !== undefined ? resultCount : false, // result count or false if unknown
        ]);
      }
    }, 1000);

    // Observer to watch for search results appearing
    const observer = new MutationObserver((mutations) => {
      mutations.forEach((mutation) => {
        mutation.addedNodes.forEach((node) => {
          if (node.nodeType !== 1) return;

          // Look for search results container from docusaurus-search-local
          const searchResults = node.querySelector?.('[class*="searchResultsColumn"]') ||
            node.querySelector?.('[class*="searchResults"]') ||
            (node.classList?.contains('search-result-match') ? node.parentElement : null);

          if (searchResults) {
            // Get the search input value
            const searchInput = document.querySelector('input[type="search"], input[class*="searchInput"]');
            if (searchInput && searchInput.value) {
              // Count results
              const results = searchResults.querySelectorAll('[class*="searchResult"], .search-result-match');
              trackSearch(searchInput.value, results.length);
            }
          }
        });
      });
    });

    // Start observing the document for search results
    observer.observe(document.body, {
      childList: true,
      subtree: true,
    });

    // Also track on Enter key in search input
    const handleKeyDown = (e) => {
      if (e.key === 'Enter' && e.target.matches('input[type="search"], input[class*="searchInput"]')) {
        const query = e.target.value;
        if (query && query.length >= 2) {
          // Small delay to let results render
          setTimeout(() => {
            const resultsContainer = document.querySelector('[class*="searchResultsColumn"], [class*="searchResults"]');
            const resultCount = resultsContainer
              ? resultsContainer.querySelectorAll('[class*="searchResult"], .search-result-match').length
              : 0;
            trackSearch(query, resultCount);
          }, 300);
        }
      }
    };

    document.addEventListener('keydown', handleKeyDown);

    // Track when a search result is clicked
    const handleResultClick = (e) => {
      const resultLink = e.target.closest('[class*="searchResult"] a, .search-result-match a');
      if (resultLink) {
        const searchInput = document.querySelector('input[type="search"], input[class*="searchInput"]');
        if (searchInput && searchInput.value && typeof window !== 'undefined' && window._paq) {
          window._paq.push([
            'trackEvent',
            'Search',
            'ResultClick',
            searchInput.value + ' -> ' + resultLink.getAttribute('href'),
            1,
          ]);
        }
      }
    };

    document.addEventListener('click', handleResultClick);

    return () => {
      observer.disconnect();
      document.removeEventListener('keydown', handleKeyDown);
      document.removeEventListener('click', handleResultClick);
    };
  }, []);

  return <SearchBar {...props} />;
}
