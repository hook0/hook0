# Hook0 Project Guidelines

## Documentation

When adding new documentation files (`.md`) to the `documentation/` directory, always update `documentation/sidebars.js` to include the new pages in the navigation sidebar.

### Documentation Structure

- `documentation/concepts/` - Core concepts documentation
- `documentation/tutorials/` - Step-by-step tutorials
- `documentation/how-to-guides/` - Practical guides
- `documentation/reference/` - Technical reference
- `documentation/explanation/` - In-depth explanations
- `documentation/self-hosting/` - Self-hosting guides
- `documentation/hook0-cloud/` - Hook0 Cloud policies and procedures
- `documentation/resources/` - Additional resources

### Legal Documents

Legal documents (Privacy Policy, Terms, DPA, Subprocessors) are in `website/src/` and static assets like DPA PDFs are in `website/static/legal/`.


## Testing

E2E tests are in `e2e-tests/` and use k6 for load testing. Run with:
```bash
cd e2e-tests && k6 run src/main.js
```

All tests must be black-box tests - never use mocks.
