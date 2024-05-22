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

- `setup.js` : Script to delete the stored values from the database from the organizations
- `main.js` : Script to run the tests
- `utils.js` : Utility functions
- `config.js` : Project configuration


## 🎯 Goals

- Create a user and an organization
- Create an application
- Create an application secret token
- Create two event types
- Create two subscriptions (the first will take the two event types, the second will take only one event type)
- Subscribe to the two subscriptions with one event per subscription
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

    const hostname = __ENV.HOSTNAME || DEFAULT_HOSTNAME;
    const targetUrl = __ENV.TARGET_URL || DEFAULT_TARGET_URL;
    const masterApiKey = __ENV.MASTER_API_KEY || DEFAULT_MASTER_API_KEY;

Configurable:
- `VUS` : Number of virtual users
- `ITERATIONS` : Number of iterations per virtual user
- `MAX_DURATION` : Maximum duration of the test execution before it times out
- `HOSTNAME` : Domain name of the API
- `TARGET_URL` : URL that will receive the webhook requests
- `MASTER_API_KEY` : Master API key if you use [this](https://documentation.hook0.com/docs/api-authentication) authentication method