## 🚀 Installation

1. Cloner le projet
2. Installer les dépendances
```bash
Installer k6 (https://k6.io/docs/get-started/installation/)
npm install
```

## 🔥 Lancer le projet

```bash
node setup.js # Pour suprimer les valeurs stockés de le base de donnée à partir des organisations
k6 run main.js # Pour lancer les tests
```

## 📝 Description

- `setup.js` : Script pour supprimer les valeurs stockés de le base de donnée à partir des organisations
- `main.js` : Script pour lancer les tests
- `utils.js` : Fonctions utilitaires
- `config.js` : Configuration du projet

## 🎯 Buts

- Créer un utilisateur et une organisation
- Créer une application
- Créer un token secret d'application
- Créer deux events types
- Créer deux subscriptions (la première prendra les deux events types, la deuxième prendra un seul event type)
- S'abonner aux deux subscriptions avec un event par subscription
- Vérifier si les events ont bien été reçus

## 📚 Documentation

- [K6](https://k6.io/docs/)
- [Hook0](https://documentation.hook0.com/)

## ⚙️ Configuration optionnelle

Vous pouvez modifier les valeurs par défaut dans le fichier `config.js`
Ou bien passer par les variables d'enviroennement avec `k6 run main.js -e VAR1=VALUE1 -e VAR2=VALUE2 ...`

const vus = __ENV.VUS || VUS;
const iterations = __ENV.ITERATIONS || ITERATIONS;
const maxDuration = __ENV.MAX_DURATION || MAX_DURATION;

    const hostname = __ENV.HOSTNAME || DEFAULT_HOSTNAME;
    const targetUrl = __ENV.TARGET_URL || DEFAULT_TARGET_URL;
    const authToken = __ENV.AUTH_TOKEN || DEFAULT_AUTH_TOKEN;
    const masterApiKey = __ENV.MASTER_API_KEY || DEFAULT_MASTER_API_KEY;

    const timeBeforeEachRequest = __ENV.TIME_BEFORE_EACH_REQUEST || TIME_BEFORE_EACH_REQUEST;
    const timeBeforeEachVerification = __ENV.TIME_BEFORE_EACH_VERIFICATION || TIME_BEFORE_EACH_VERIFICATION;
    const timeBeforeEachDelete = __ENV.TIME_BEFORE_EACH_DELETE || TIME_BEFORE_EACH_DELETE;

    const retryCount = __ENV.RETRY_COUNT || RETRY_COUNT;

Configurable:
- `VUS` : Nombre d'utilisateurs virtuels
- `ITERATIONS` : Nombre d'itérations par utilisateur virtuel
- `MAX_DURATION` : Durée maximale de l'exécution du test avant qu'il timeout
- `HOSTNAME` : Nom de domaine de l'API
- `TARGET_URL` : URL qui recevera les requêtes des webhooks
- `AUTH_TOKEN` : Token d'authentification
- `MASTER_API_KEY` : Clé master d'api si vous utilisez [cette](https://documentation.hook0.com/docs/api-authentication) méthode d'authentification