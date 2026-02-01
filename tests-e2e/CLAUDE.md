# Tests E2E - Notes techniques

## Contrainte CI : Kubernetes Executor

L'intégration continue GitLab utilise un **Kubernetes Executor** qui ne supporte pas Docker-in-Docker.

### Conséquences

- **En local** : Les tests utilisent `docker-compose.yaml` à la racine du projet via la configuration `webServer` de Playwright
- **En CI** : Les tests utilisent les services GitLab CI (postgres, mailpit) et buildent/lancent l'API et le frontend comme processus dans le job

### Exécution locale

```bash
# Démarre docker-compose et lance les tests
cd tests-e2e
npm install
npm test

# Ou manuellement :
docker compose -f ../docker-compose.yaml up -d
npm test
```

### Exécution CI

Le CI :
1. Utilise les services GitLab CI pour postgres et mailpit
2. Build l'API et le frontend depuis les artifacts des jobs précédents
3. Lance l'API et le frontend comme processus background
4. Exécute les tests Playwright contre ces services

### Différences local vs CI

| Aspect | Local | CI |
|--------|-------|-----|
| Infrastructure | docker-compose.yaml | Services GitLab CI |
| API/Frontend | Containers Docker | Binaires/processus |
| Base de données | Container postgres | Service postgres |
| Email | Container mailpit | Service mailpit |
