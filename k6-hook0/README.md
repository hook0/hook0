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