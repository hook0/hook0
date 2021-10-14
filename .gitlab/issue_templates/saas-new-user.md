# Todo

- [ ] Créer l'orga c-a-d les 3 groupes dans Keycloak
  - [ ] orga_{ORGA_UUID}
    - [ ] /role_editor
    - [ ] /role_viewer
- [ ] Créer l’orga dans la base de données (et la nommer)
- [ ] Créer nouvel user Keycloak (couple email/password)
  - [ ] et lui assigner le rôle orga_{UUID}/role_viewer fraichement créé
- [ ] Créer nouveau client Keycloak "orga_{ORGA_UUID}" :
  - [ ] Access Type confidential
  - [ ] Standard Flow Disabled
  - [ ] Implicit Flow Disabled
  - [ ] Direct Access Grants Enabled
    - [ ] pour faire la requête sur /openid-connect/token avec login/password
  - [ ] Service Accounts Disabled
- [ ] Créer un user Keycloak service account "sa_{UUID}"
  - [ ] L'ajouter dans le groupe
    - [ ] orga_{ORGA_UUID}
      - [ ] /role_editor
