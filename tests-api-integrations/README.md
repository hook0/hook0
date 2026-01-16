## 🚀 Installation

1. Clone the project
2. Install dependencies

```bash
# Install k6 (https://k6.io/docs/get-started/installation/)
npm install
```

## 🔥 Launch the project

```bash
node setup.js # To delete the stored values from the database from the organizations
k6 run main.js # To run the tests
```

## 📝 Description

- `main.js` : Script to run the tests
- `config.js` : Project configuration

## 🎯 Goals

- Create an application
- Create two event types
- Create two subscriptions (the first will take the two event types, the second will take only one event type)
- Subscribe to the two subscriptions with one event per subscription (so in total three events)
- Check if the events have been received

## 📚 Documentation

- [K6](https://k6.io/docs/)
- [Hook0](https://documentation.hook0.com/)

## ⚙️ Optional configuration

You can modify the default values in the `config.js` file
Or use environment variables with `k6 run main.js -e VAR1=VALUE1 -e VAR2=VALUE2 ...`

    const vus = __ENV.VUS || VUS;
    const iterations = __ENV.ITERATIONS || ITERATIONS;
    const maxDuration = __ENV.MAX_DURATION || MAX_DURATION;

    const apiOrigin = __ENV.API_ORIGIN || DEFAULT_API_ORIGIN;
    const targetUrl = __ENV.TARGET_URL || DEFAULT_TARGET_URL;
    const serviceToken = __ENV.SERVICE_TOKEN || DEFAULT_SERVICE_TOKEN;
    const organizationId = __ENV.ORGANIZATION_ID || DEFAULT_ORGANIZATION_ID;

Configurable:

- `VUS` : Number of virtual users
- `ITERATIONS` : Number of iterations per virtual user
- `MAX_DURATION` : Maximum duration of the test execution before it times out
- `API_ORIGIN` : Origin of the API (exemple: http://localhost:8081)
- `TARGET_URL` : URL that will receive the webhook requests
- `SERVICE_TOKEN` : Service Token is used for authenticated in your organization [this](https://documentation.hook0.com/docs/api-authentication) authentication method
- `ORGANIZATION_ID` : Organization ID is used to identify the organization that the user belongs to [this](https://documentation.hook0.com/docs/api-authentication) authentication method

### Account Deletion Test (Optional)

To run the account deletion test (Scenario 4), you need to provide test user credentials:

- `TEST_USER_EMAIL` : Email address of a test user account
- `TEST_USER_PASSWORD` : Password of the test user account

If these variables are not set, the account deletion test will be skipped.

Example:
```bash
k6 run main.js -e TEST_USER_EMAIL=test@example.com -e TEST_USER_PASSWORD=testpassword123
```

### Account Isolation Test (Optional)

To run the account isolation security test (Scenario 5), you need to provide credentials for **two different users**:

- `TEST_USER_EMAIL` : Email address of User A
- `TEST_USER_PASSWORD` : Password of User A
- `TEST_USER_EMAIL_2` : Email address of User B
- `TEST_USER_PASSWORD_2` : Password of User B

If any of these variables are not set, the account isolation test will be skipped.

Example:
```bash
k6 run main.js \
  -e TEST_USER_EMAIL=usera@example.com \
  -e TEST_USER_PASSWORD=passwordA \
  -e TEST_USER_EMAIL_2=userb@example.com \
  -e TEST_USER_PASSWORD_2=passwordB
```
