## Known Issues

### Register with existing email shows wrong error
- When registering with an existing email, the API returns "Insufficient rights, You don't have the right to access or edit this resource."
- This is a backend issue — the API should return a proper "Email already registered" error
- Frontend displays whatever the API returns
