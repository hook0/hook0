#!/usr/bin/env node

/**
 * Fetch OpenAPI specification from Hook0 API
 * This script runs at build time to fetch the latest OpenAPI spec
 */

const fs = require('fs');
const path = require('path');
const https = require('https');
const http = require('http');

// Configuration
const OPENAPI_URL = process.env.HOOK0_API_URL
  ? `${process.env.HOOK0_API_URL}/api/v1/swagger.json`
  : process.env.NODE_ENV === 'production' || process.env.CI
    ? 'https://app.hook0.com/api/v1/swagger.json'
    : 'http://localhost:8080/api/v1/swagger.json';

const OUTPUT_DIR = path.join(__dirname, '..', 'openapi');
const OUTPUT_FILE = path.join(OUTPUT_DIR, 'hook0-api.json');
const STATIC_DIR = path.join(__dirname, '..', 'static');
const STATIC_FILE = path.join(STATIC_DIR, 'hook0-api.json');

// Ensure output directory exists
if (!fs.existsSync(OUTPUT_DIR)) {
  fs.mkdirSync(OUTPUT_DIR, { recursive: true });
}

// Create a fallback spec function
const createFallbackSpec = () => {
  const fallbackSpec = {
    openapi: '3.0.0',
    info: {
      title: 'Hook0 API',
      version: '1.0.0',
      description: 'Hook0 Webhook Infrastructure API'
    },
    paths: {},
    components: {
      securitySchemes: {
        biscuit: {
          type: 'apiKey',
          in: 'header',
          name: 'Authorization',
          description: 'Biscuit token authentication'
        }
      }
    }
  };

  fs.writeFileSync(OUTPUT_FILE, JSON.stringify(fallbackSpec, null, 2));
  console.log('   Created fallback OpenAPI spec');
};

// Determine protocol
const protocol = OPENAPI_URL.startsWith('https') ? https : http;

console.log(`📥 Fetching OpenAPI spec from: ${OPENAPI_URL}`);

// Fetch the OpenAPI spec
const fetchSpec = () => {
  return new Promise((resolve, reject) => {
    protocol.get(OPENAPI_URL, (res) => {
      if (res.statusCode !== 200) {
        // If we can't fetch from the API, use a fallback or skip
        console.warn(`⚠️  Could not fetch OpenAPI spec (status: ${res.statusCode})`);
        console.warn('   Using fallback or skipping API doc generation');

        // Check if we have a cached version
        if (fs.existsSync(OUTPUT_FILE)) {
          console.log('   Using cached OpenAPI spec');
          resolve();
          return;
        }

        // Create a minimal fallback spec
        createFallbackSpec();
        resolve();
        return;
      }

      let data = '';
      res.on('data', (chunk) => {
        data += chunk;
      });

      res.on('end', () => {
        try {
          const spec = JSON.parse(data);

          // Add production server URL if not present
          if (!spec.servers || spec.servers.length === 0 || !spec.servers.some(s => s.url === 'https://app.hook0.com')) {
            spec.servers.push({
              url: 'https://app.hook0.com',
              description: 'Production API'
            });
          }

          // Enhance the spec with better descriptions if needed
          if (spec.info) {
            spec.info.description = spec.info.description || 'Hook0 is a robust webhook infrastructure that handles event delivery, retries, and monitoring for your applications.';
            spec.info.contact = {
              name: 'Hook0 Support',
              url: 'https://www.hook0.com',
              email: 'support@hook0.com'
            };
            spec.info.license = {
              name: 'Apache 2.0',
              url: 'https://www.apache.org/licenses/LICENSE-2.0.html'
            };
          }

          // Rename biscuit security schemes to API Token for better UX
          if (spec.components && spec.components.securitySchemes) {
            const schemes = spec.components.securitySchemes;
            if (schemes.biscuit) {
              schemes.apiToken = {
                ...schemes.biscuit,
                description: 'API Token authentication. Use the format: `Bearer YOUR_API_TOKEN`'
              };
              delete schemes.biscuit;
            }
            if (schemes.biscuit_user_access) {
              schemes.userAccessToken = {
                ...schemes.biscuit_user_access,
                description: 'User access token for authentication'
              };
              delete schemes.biscuit_user_access;
            }
            if (schemes.biscuit_refresh) {
              schemes.refreshToken = {
                ...schemes.biscuit_refresh,
                description: 'Refresh token for obtaining new access tokens'
              };
              delete schemes.biscuit_refresh;
            }

            // Update security references in paths
            const updateSecurityRefs = (obj) => {
              if (Array.isArray(obj)) {
                return obj.map(item => {
                  if (item.biscuit !== undefined) {
                    return { apiToken: item.biscuit };
                  }
                  if (item.biscuit_user_access !== undefined) {
                    return { userAccessToken: item.biscuit_user_access };
                  }
                  if (item.biscuit_refresh !== undefined) {
                    return { refreshToken: item.biscuit_refresh };
                  }
                  return item;
                });
              }
              return obj;
            };

            // Update global security
            if (spec.security) {
              spec.security = updateSecurityRefs(spec.security);
            }

            // Update path-level security
            if (spec.paths) {
              for (const path in spec.paths) {
                for (const method in spec.paths[path]) {
                  if (spec.paths[path][method].security) {
                    spec.paths[path][method].security = updateSecurityRefs(spec.paths[path][method].security);
                  }
                }
              }
            }
          }

          // Write the enhanced spec
          fs.writeFileSync(OUTPUT_FILE, JSON.stringify(spec, null, 2));
          console.log(`✅ OpenAPI spec saved to: ${OUTPUT_FILE}`);

          // Also copy to static folder for Scalar
          fs.writeFileSync(STATIC_FILE, JSON.stringify(spec, null, 2));
          console.log(`✅ OpenAPI spec copied to: ${STATIC_FILE}`);
          resolve();
        } catch (error) {
          console.warn(`⚠️  Failed to parse OpenAPI spec: ${error.message}`);

          // Check for cached version
          if (fs.existsSync(OUTPUT_FILE)) {
            console.log('   Using cached OpenAPI spec');
            resolve();
          } else {
            console.log('   Creating fallback OpenAPI spec');
            createFallbackSpec();
            resolve();
          }
        }
      });
    }).on('error', (error) => {
      console.warn(`⚠️  Network error fetching OpenAPI spec: ${error.message}`);

      // Check for cached version
      if (fs.existsSync(OUTPUT_FILE)) {
        console.log('   Using cached OpenAPI spec');
        resolve();
      } else {
        // Create fallback spec instead of failing
        console.log('   Creating fallback OpenAPI spec for CI/offline build');
        createFallbackSpec();
        resolve();
      }
    });
  });
};

// Run the fetch
fetchSpec()
  .then(() => {
    console.log('📄 OpenAPI spec ready for documentation generation');
    process.exit(0);
  })
  .catch((error) => {
    // This should rarely happen now as we handle most errors above
    console.error('❌ Unexpected error:', error.message);

    // Even in worst case, try to create fallback
    if (!fs.existsSync(OUTPUT_FILE)) {
      console.log('   Creating emergency fallback OpenAPI spec');
      createFallbackSpec();
    }

    // Exit with success to not break the build
    console.log('📄 Proceeding with available OpenAPI spec');
    process.exit(0);
  });
