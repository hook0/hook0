# Registre des activités de traitement — Art. 30 RGPD

**Document public — FGRibreau SARL**
Ce document est établi conformément à l'article 30 du Règlement (UE) 2016/679 (RGPD). Ce registre est publié dans le dépôt open-source Hook0 (`https://gitlab.com/hook0/hook0/`) par souci de transparence vis-à-vis des utilisateurs et de la communauté. Sa publication ne dispense pas FGRibreau SARL de le tenir à jour conformément à l'art. 30 RGPD ni de le présenter à la CNIL en cas de contrôle.

---

| Métadonnée | Valeur |
|---|---|
| **Version** | 1.1 |
| **Date d'établissement** | 4 mai 2026 |
| **Prochain réexamen** | 4 mai 2027 |
| **Responsable de la tenue** | Direction FGRibreau SARL |
| **Contact RGPD** | legal@hook0.com |
| **DPO désigné** | Non (structure non soumise à l'obligation — art. 37 RGPD) |
| **Référence légale** | Règlement (UE) 2016/679, art. 30 ; Loi n° 78-17 du 6 janvier 1978 modifiée |

---

## Sommaire

1. [Gestion des comptes utilisateurs et authentification](#traitement-1)
2. [Gestion des webhooks — cœur de métier](#traitement-2)
3. [Facturation et paiement](#traitement-3)
4. [Communications transactionnelles par email](#traitement-4)
5. [Support client (chat et email)](#traitement-5)
6. [Sécurité et supervision du service](#traitement-6)
7. [Analytique du site web (Matomo auto-hébergé)](#traitement-7)
8. [Mesure des conversions publicitaires Google Ads (server-side)](#traitement-8)
9. [Communications commerciales (newsletters et release notes)](#traitement-9)
10. [Journaux HTTP — tracking par requête utilisateur](#traitement-10)
11. [Outil gratuit play.hook0.com — webhook playground](#traitement-11)
12. [Feedback produit in-app via Formbricks](#traitement-12)

**Annexes**

- [Annexe 1 — Liste consolidée des sous-traitants](#annexe-1)
- [Annexe 2 — Procédure de mise à jour du registre](#annexe-2)
- [Annexe 3 — Procédure en cas de violation de données personnelles](#annexe-3)

---

## Traitement n°1 — Gestion des comptes utilisateurs et authentification {#traitement-1}

### 1. Identité du responsable de traitement

**FGRibreau SARL**
3 rue de l'Aubépine — 85110 Chantonnay, France
RCS La Roche-sur-Yon 850 824 350 — TVA FR27850824350
Contact RGPD : legal@hook0.com

### 2. Finalité(s) du traitement

**Finalité principale :** Créer et gérer les comptes utilisateurs de la plateforme SaaS Hook0, permettre l'authentification et contrôler les autorisations d'accès aux ressources (organisations, applications, API).

**Finalités secondaires :**
- Envoi des emails de vérification d'adresse et de réinitialisation de mot de passe.
- Journalisation des connexions à des fins de sécurité et de détection des accès non autorisés.
- Révocation des API keys en cas d'incident de sécurité.

### 3. Base légale (art. 6 RGPD)

**Art. 6.1.b — Exécution du contrat :** le traitement est nécessaire à l'exécution du contrat SaaS auquel la personne concernée (représentant de l'entreprise cliente) est partie. Sans compte utilisateur, le service ne peut pas être fourni.

Référence : considérant 44 du RGPD (« le traitement devrait être licite lorsqu'il est nécessaire dans le cadre d'un contrat »).

### 4. Catégories de personnes concernées

Utilisateurs professionnels (personnes physiques représentant une entreprise cliente ou y travaillant) ayant créé un compte sur app.hook0.com. Périmètre exclusivement B2B.

### 5. Catégories de données personnelles

- Adresse email professionnelle
- Prénom et nom de famille
- Mot de passe (haché Argon2id — jamais stocké en clair)
- Clés API (générées par Hook0, pseudonymes)
- Adresse IP lors des connexions
- Horodatages de connexion (created_at, last_login_at)
- Identifiants de session et tokens JWT
- Préférences de compte (langue, notifications)

### 6. Catégories de destinataires

**Interne :** équipe technique FGRibreau SARL (accès restreint en lecture, en base de besoin).

**Sous-traitants :**
- **Clever Cloud SAS** (France) — hébergement de la base de données PostgreSQL et de l'application backend Rust/Axum.
- **Brevo (Sendinblue)** (France) — envoi des emails transactionnels de vérification et de réinitialisation.

### 7. Transferts hors UE

Aucun. Clever Cloud SAS et Brevo sont établis en France.

### 8. Durée de conservation

| Données | Durée | Justification |
|---|---|---|
| Données de compte actif | Durée du contrat | Nécessité contractuelle |
| Données après suppression du compte | 30 jours | Permettre l'export des données avant suppression définitive |
| Journaux de connexion | 30 jours | Sécurité et détection des incidents |
| Preuves de consentement (opt-ins) | 5 ans | Art. 7.1 RGPD — démonstration de la conformité |

### 9. Mesures de sécurité (description générale)

- Chiffrement en transit : TLS 1.2 minimum sur tous les endpoints.
- Chiffrement des mots de passe : Argon2id (résistant aux attaques par GPU), paramètres conformes aux recommandations OWASP.
- Chiffrement au repos : chiffrement du volume de base de données par Clever Cloud.
- Contrôle d'accès : MFA obligatoire pour les accès infrastructure ; accès base de données restreint à l'équipe technique.
- Révocation automatique des sessions et des tokens en cas de changement de mot de passe.
- Référence détaillée : `secure-engineering-policy.md` et `access-control-policy.md` (ISMS FGRibreau SARL).

### 10. Source des données

Collectées directement auprès de la personne concernée lors de l'inscription sur app.hook0.com/register.

---

## Traitement n°2 — Gestion des webhooks — cœur de métier {#traitement-2}

### 1. Identité du responsable de traitement

**FGRibreau SARL** — cf. traitement n°1.

Nota bene : FGRibreau SARL agit en qualité de **sous-traitant au sens de l'art. 28 RGPD** pour les données personnelles éventuellement contenues dans les payloads webhook transmis par ses clients. Les clients de Hook0 (les développeurs et entreprises qui utilisent l'API) sont, à l'égard de leurs propres utilisateurs finaux, les responsables de traitement. FGRibreau SARL s'engage contractuellement à traiter ces données uniquement selon leurs instructions (cf. DPA disponible sur www.hook0.com/data-processing-addendum).

### 2. Finalité(s) du traitement

**Finalité principale :** Recevoir, stocker temporairement, délivrer et relivrer (retry) les événements webhook des clients de Hook0 aux endpoints de subscription qu'ils ont configurés.

**Finalités secondaires :**
- Journalisation des tentatives de livraison (succès, échec, code HTTP retourné, latence) à des fins de monitoring et de debug par le client.
- Permettre la consultation des logs de livraison dans l'interface app.hook0.com.

### 3. Base légale (art. 6 RGPD)

**Art. 6.1.b — Exécution du contrat :** la réception et la livraison des webhooks constituent la prestation contractuelle principale de Hook0.

Concernant les données personnelles éventuellement présentes dans les payloads : Hook0 agit en tant que sous-traitant (art. 28 RGPD) sur instruction du responsable de traitement (le client). La base légale applicable est celle retenue par le client pour son propre traitement, que Hook0 exécute sans en modifier la finalité.

### 4. Catégories de personnes concernées

- Utilisateurs professionnels de Hook0 (pour les métadonnées de configuration : URLs de subscription, headers).
- Utilisateurs finaux des clients de Hook0 (dont des données personnelles peuvent figurer dans les payloads webhook, selon la configuration du client).

### 5. Catégories de données personnelles

**Données propres à FGRibreau SARL (en tant que responsable de traitement) :**
- URLs de subscription (peuvent contenir des sous-domaines ou identifiants clients).
- Headers HTTP de subscription (peuvent contenir des tokens d'authentification).
- Logs de livraison : horodatage, statut HTTP, latence, identifiant de l'événement.

**Données en tant que sous-traitant (contenu des payloads webhook) :**
- Nature variable, déterminée exclusivement par le client. Peut inclure toute catégorie de données personnelles selon l'application du client.
- Hook0 ne lit pas, n'analyse pas et ne modifie pas le contenu des payloads.

### 6. Catégories de destinataires

**Interne :** équipe technique (accès restreint aux logs, en base de besoin).

**Sous-traitants :**
- **Clever Cloud SAS** (France) — workers de livraison et base de données des événements et logs.
- **Scaleway SAS** (France) — workers dédiés privés pour les offres sélectionnées ; Scaleway Object Storage S3-compatible pour le stockage des payloads webhook volumineux (offload depuis PostgreSQL).

### 7. Transferts hors UE

Aucun. Les workers et la base de données sont hébergés en France (Clever Cloud, Paris ; Scaleway, Paris). Le stockage objet Scaleway est configuré par l'env var `OBJECT_STORAGE_HOST` — en production : datacenter Scaleway Paris (fr-par). Les self-hosters peuvent substituer tout provider S3-compatible de leur choix ; dans ce cas, les garanties de transfert hors UE relèvent de leur responsabilité.

### 8. Durée de conservation

| Données | Durée | Justification |
|---|---|---|
| Payloads et logs de livraison (PostgreSQL) | 7 à 30 jours selon le plan d'abonnement souscrit | Nécessité du service (consultation des logs par le client) ; configurable par plan |
| Payloads volumineux (Scaleway Object Storage) | 7 à 30 jours selon le plan ; cleanup automatique via job périodique | Cohérent avec la rétention PostgreSQL ; suppression automatique des objets orphelins |
| Métadonnées de configuration (URLs, headers) | Durée du contrat | Nécessité contractuelle |

### 9. Mesures de sécurité (description générale)

- Chiffrement en transit : TLS 1.2+ entre le client, Hook0 et les endpoints de destination.
- Isolation des données par organisation (multi-tenant) : chaque client ne peut accéder qu'à ses propres données.
- Chiffrement au repos des données de la base de données (Clever Cloud).
- Les payloads ne sont jamais indexés en vue d'une analyse de contenu par Hook0.
- Référence : `secure-engineering-policy.md` (ISMS FGRibreau SARL).

### 10. Source des données

- Métadonnées de configuration : collectées directement auprès du client via l'API ou l'interface.
- Payloads webhook : reçus des systèmes du client via l'API Hook0 (donc indirectement, via le client responsable de traitement).

---

## Traitement n°3 — Facturation et paiement {#traitement-3}

### 1. Identité du responsable de traitement

**FGRibreau SARL** — cf. traitement n°1.

### 2. Finalité(s) du traitement

**Finalité principale :** Gérer les abonnements SaaS, encaisser les paiements, émettre les factures et conserver les pièces comptables.

**Finalités secondaires :**
- Gestion des changements de plan (upgrade, downgrade, résiliation).
- Émission des avoirs en cas de remboursement.
- Conservation des pièces comptables pour satisfaire aux obligations fiscales et comptables françaises.

### 3. Base légale (art. 6 RGPD)

**Art. 6.1.b — Exécution du contrat :** la facturation et l'encaissement sont nécessaires à l'exécution du contrat SaaS à titre onéreux.

**Art. 6.1.c — Obligation légale :** la conservation des pièces comptables pendant 10 ans est imposée par l'article L102 B du Livre des procédures fiscales (Code général des impôts). Cette obligation est opposable à FGRibreau SARL en tant qu'assujettie à la TVA et soumise aux règles de la comptabilité commerciale.

### 4. Catégories de personnes concernées

Représentants des entreprises clientes (nom, email professionnel, adresse de facturation) ayant souscrit un abonnement payant Hook0.

### 5. Catégories de données personnelles

- Nom et prénom (du titulaire de compte)
- Adresse email professionnelle
- Adresse de facturation (entreprise, rue, code postal, ville, pays)
- Données d'instrument de paiement : 4 derniers chiffres de la carte, date d'expiration, marque (Stripe stocke les données complètes de carte — FGRibreau SARL n'y a jamais accès)
- Historique des abonnements et des transactions
- Numéro de TVA intracommunautaire (pour les clients UE)

### 6. Catégories de destinataires

**Interne :** comptabilité et direction FGRibreau SARL.

**Sous-traitants / partenaires :**
- **Stripe, Inc.** (USA) — traitement des paiements par carte, gestion des abonnements récurrents, stockage sécurisé des données de carte (certifié PCI-DSS niveau 1).

**Co-responsables potentiels :**
- Expert-comptable et commissaire aux comptes de FGRibreau SARL, dans le cadre de leurs obligations légales (accès aux pièces comptables).

### 7. Transferts hors UE

**Oui — vers les États-Unis (Stripe, Inc.).**
Base de transfert : Clauses Contractuelles Types (CCT), Décision de la Commission européenne 2021/914 du 4 juin 2021 — modules responsable de traitement vers sous-traitant.
Lien : https://www.stripeprivacy.com/dpa

Stripe, Inc. est également certifié EU-U.S. Data Privacy Framework (DPF), offrant une garantie complémentaire de niveau adéquat.

### 8. Durée de conservation

| Données | Durée | Justification |
|---|---|---|
| Pièces comptables et factures | 10 ans à compter de la date de transaction | Obligation légale — art. L102 B du Livre des procédures fiscales |
| Données d'abonnement actif | Durée du contrat | Nécessité contractuelle |
| Données de carte (chez Stripe) | Jusqu'à révocation ou expiration | Stripe Privacy Policy et PCI-DSS |

### 9. Mesures de sécurité (description générale)

- Stripe est certifié PCI-DSS niveau 1 : FGRibreau SARL ne stocke jamais de données de carte en clair.
- Accès aux données de facturation restreint à la direction et à la comptabilité (principe du moindre privilège).
- Chiffrement en transit TLS 1.2+ entre l'application et l'API Stripe.
- Authentification forte (MFA) sur le compte Stripe de FGRibreau SARL.

### 10. Source des données

Collectées directement auprès du représentant de l'entreprise cliente lors de la souscription de l'abonnement.

---

## Traitement n°4 — Communications transactionnelles par email {#traitement-4}

### 1. Identité du responsable de traitement

**FGRibreau SARL** — cf. traitement n°1.

### 2. Finalité(s) du traitement

**Finalité principale :** Envoyer les emails nécessaires au fonctionnement du service : confirmation d'inscription, vérification d'adresse email, réinitialisation de mot de passe, notifications de facturation, alertes de sécurité sur le compte.

### 3. Base légale (art. 6 RGPD)

**Art. 6.1.b — Exécution du contrat :** l'envoi des emails transactionnels est indissociable de la fourniture du service (activation du compte, sécurisation des accès, suivi de facturation).

### 4. Catégories de personnes concernées

Utilisateurs professionnels titulaires d'un compte Hook0.

### 5. Catégories de données personnelles

- Adresse email professionnelle
- Prénom (personnalisation de la salutation)
- Nom (pour certains emails de facturation)
- Contenu fonctionnel de l'email (lien de vérification, lien de réinitialisation, récapitulatif de facture)

### 6. Catégories de destinataires

**Sous-traitants :**
- **Brevo SAS** (France) — routage principal des emails transactionnels.
- **Postmark (ActiveCampaign, Inc.)** (USA) — routage des emails transactionnels en cas de bascule ou en complément (deliverability).

### 7. Transferts hors UE

**Oui — vers les États-Unis (Postmark / ActiveCampaign, Inc.) lorsque Postmark est utilisé.**
Base de transfert : Clauses Contractuelles Types (CCT), Décision 2021/914.
Lien : https://postmarkapp.com/eu-privacy

Brevo SAS est établie en France (UE) : aucun transfert hors UE pour les envois routés via Brevo.

### 8. Durée de conservation

| Données | Durée | Justification |
|---|---|---|
| Logs d'envoi (adresse, statut, horodatage) | Durée du contrat | Traçabilité des notifications et preuve de livraison |
| Contenu des emails | Durée du contrat | Gestion des litiges |

### 9. Mesures de sécurité (description générale)

- Authentification DKIM et SPF configurés pour le domaine hook0.com.
- Accès à l'API Brevo / Postmark restreint via clés API à portée limitée.
- Aucun contenu sensible (mot de passe, données de carte) n'est inclus dans les emails.

### 10. Source des données

Collectées directement auprès de la personne concernée lors de la création du compte.

---

## Traitement n°5 — Support client (chat et email) {#traitement-5}

### 1. Identité du responsable de traitement

**FGRibreau SARL** — cf. traitement n°1.

### 2. Finalité(s) du traitement

**Finalité principale :** Répondre aux demandes d'assistance technique et commerciale des utilisateurs de Hook0, par chat en direct (Crisp) et par email (Gmail/Google).

**Finalités secondaires :**
- Conservation des historiques pour assurer la continuité du support et gérer les litiges éventuels.
- Amélioration du service à partir des retours utilisateurs.

### 3. Base légale (art. 6 RGPD)

**Support par email — Art. 6.1.f — Intérêt légitime :** répondre aux demandes des clients constitue un intérêt légitime évident de FGRibreau SARL dans le cadre d'une relation contractuelle B2B. La balance d'intérêts est favorable : la personne concernée s'attend raisonnablement à ce que ses demandes soient traitées et conservées.

**Support par chat (Crisp) — Art. 6.1.a — Consentement :** le widget Crisp n'est chargé qu'après recueil du consentement explicite via le bandeau cookies de www.hook0.com (conformément à la délibération CNIL n° 2020-091 du 17 septembre 2020).

### 4. Catégories de personnes concernées

Utilisateurs professionnels de Hook0 ayant contacté le support.

### 5. Catégories de données personnelles

- Nom et prénom
- Adresse email professionnelle
- Contenu des échanges (questions, réponses, pièces jointes éventuelles)
- Métadonnées de navigation : URL de la page depuis laquelle le chat est initié, type de navigateur (collectées par Crisp)
- Horodatages des échanges

### 6. Catégories de destinataires

**Interne :** équipe support FGRibreau SARL.

**Sous-traitants :**
- **Crisp IM SAS** (France) — plateforme de chat en direct et de ticketing.
- **Google LLC (Gmail)** (USA) — messagerie email de l'équipe support (legal@hook0.com, support@hook0.com).

### 7. Transferts hors UE

**Oui — vers les États-Unis (Google LLC / Gmail).**
Base de transfert : Clauses Contractuelles Types (CCT), Décision 2021/914.
Google LLC est également certifié EU-U.S. Data Privacy Framework (DPF).
Lien DPA Google Workspace : https://workspace.google.com/terms/dpa_terms.html

Crisp IM SAS est établie en France (UE) : aucun transfert hors UE pour le chat.

### 8. Durée de conservation

| Données | Durée | Justification |
|---|---|---|
| Historiques de chat (Crisp) | 3 ans après le dernier échange | Prescription quinquennale des actions contractuelles (art. 2224 Code civil) ; durée réduite appliquée par principe de minimisation |
| Emails de support (Gmail) | 3 ans après le dernier échange | Même justification |

### 9. Mesures de sécurité (description générale)

- Accès à Crisp et à la messagerie Gmail restreint aux membres de l'équipe support (authentification MFA).
- Crisp : chiffrement en transit TLS, données hébergées en France.
- Gmail : chiffrement en transit TLS, MFA obligatoire sur les comptes Google Workspace de FGRibreau SARL.

### 10. Source des données

Collectées directement auprès de la personne concernée lors de la prise de contact avec le support.

---

## Traitement n°6 — Sécurité et supervision du service {#traitement-6}

### 1. Identité du responsable de traitement

**FGRibreau SARL** — cf. traitement n°1.

### 2. Finalité(s) du traitement

**Finalité principale :** Assurer la disponibilité, l'intégrité et la sécurité de la plateforme Hook0 par la détection des erreurs applicatives, la supervision de la disponibilité et la protection contre les attaques (DDoS, abus).

**Finalités secondaires :**
- Investigation post-incident et correction des bugs.
- Détection des tentatives d'intrusion ou d'abus de l'API.
- Suivi de la disponibilité (uptime) et notification des incidents aux clients.

### 3. Base légale (art. 6 RGPD)

**Art. 6.1.f — Intérêt légitime :** la sécurisation et la supervision d'un service SaaS constituent un intérêt légitime prépondérant, reconnu par l'EDPB (Guidelines 06/2014, exemple sur la sécurité des systèmes d'information). La balance d'intérêts est favorable : les personnes concernées bénéficient directement de la sécurisation du service qui traite leurs données, et les données collectées (IP, métadonnées techniques) sont strictement limitées au minimum nécessaire.

### 4. Catégories de personnes concernées

Utilisateurs professionnels de Hook0 (lors de l'utilisation de l'API ou de l'interface) et visiteurs de www.hook0.com dont les requêtes transitent par Cloudflare.

### 5. Catégories de données personnelles

- Adresses IP (des clients API et des visiteurs)
- Traces d'erreurs applicatives (stack traces — peuvent inclure des identifiants utilisateur internes)
- Métadonnées des requêtes HTTP : méthode, URL, code de statut, latence, User-Agent
- Résultats des sondes de disponibilité (BetterUptime)

### 6. Catégories de destinataires

**Interne :** équipe technique FGRibreau SARL.

**Sous-traitants :**
- **Sentry, Inc.** (USA) — agrégation et alerting des erreurs applicatives.
- **BetterStack (BetterUptime)** (République tchèque, UE) — monitoring de disponibilité et page de statut publique.
- **Cloudflare, Inc.** (USA) — proxy réseau, protection DDoS, résolution DNS.
- **Cloudflare, Inc. — Turnstile** (USA) — CAPTCHA anti-bot invisible, vérifié à l'inscription sur app.hook0.com/register. Les données traitées lors de la vérification Turnstile incluent l'adresse IP, le User-Agent et des métadonnées comportementales du navigateur (évaluation bot/humain). Cloudflare agit à la fois comme sous-traitant de FGRibreau SARL et, pour ses propres finalités de sécurité anti-fraude globale, comme responsable autonome.

### 7. Transferts hors UE

**Oui — vers les États-Unis (Sentry, Inc. et Cloudflare, Inc.).**

- **Sentry** : Clauses Contractuelles Types (CCT), Décision 2021/914. Lien DPA : https://sentry.io/legal/dpa/
- **Cloudflare (proxy DDoS/DNS et Turnstile)** : Clauses Contractuelles Types (CCT), Décision 2021/914. Le DPA Cloudflare couvre l'ensemble des services Cloudflare contractualisés. À vérifier : confirmer que le DPA Cloudflare existant de FGRibreau SARL liste explicitement Turnstile ou couvre les services anti-bot. Lien DPA : https://www.cloudflare.com/cloudflare-customer-dpa/

BetterStack est établi dans l'UE (République tchèque) : aucun transfert hors UE.

### 8. Durée de conservation

| Données | Durée | Justification |
|---|---|---|
| Logs d'erreurs (Sentry) | 30 jours | Minimisation ; délai suffisant pour investigation post-incident |
| Données BetterUptime | 90 jours | Analyse des tendances de disponibilité |
| Logs Cloudflare (réseau) | Selon politique Cloudflare (≤ 30 jours) | Fonction de sécurité réseau |

### 9. Mesures de sécurité (description générale)

- Accès à Sentry et BetterUptime restreint à l'équipe technique (MFA).
- Les données personnelles dans les erreurs Sentry sont masquées ou scrubées via la configuration du SDK (PII scrubbing activé).
- Cloudflare : aucune donnée de contenu des webhooks ne transite via Cloudflare (les workers de livraison sont sur Clever Cloud en direct).
- Référence : `logging-policy.md` et `secure-engineering-policy.md` (ISMS FGRibreau SARL).

### 10. Source des données

Collectées automatiquement lors de l'utilisation de la plateforme (logs générés par le système).

---

## Traitement n°7 — Analytique du site web (Matomo auto-hébergé) {#traitement-7}

### 1. Identité du responsable de traitement

**FGRibreau SARL** — cf. traitement n°1.

### 2. Finalité(s) du traitement

**Finalité principale :** Mesurer l'audience et le comportement des visiteurs sur www.hook0.com afin d'optimiser le contenu éditorial et les parcours utilisateurs.

**Finalités secondaires :**
- Identifier les pages à fort trafic et les sources de trafic (moteurs de recherche, campagnes, direct).
- Évaluer l'efficacité des pages de destination.

### 3. Base légale (art. 6 RGPD)

**Art. 6.1.a — Consentement :** la mesure d'audience via traceurs nécessite le consentement préalable des visiteurs, conformément à l'article 82 de la loi n° 78-17 du 6 janvier 1978 modifiée (transposition de l'art. 5.3 de la Directive 2002/58/CE dite e-Privacy) et à la délibération CNIL n° 2020-091 du 17 septembre 2020.

Le consentement est recueilli via le bandeau cookies de www.hook0.com. Il est valide 13 mois maximum et peut être retiré à tout moment.

Nota bene : si Matomo était configuré en mode exempté de consentement (IP anonymisée à 2 octets, pas de cookie cross-session, pas de fingerprinting), la base légale pourrait être l'intérêt légitime. La configuration actuelle utilise le consentement.

### 4. Catégories de personnes concernées

Visiteurs du site www.hook0.com ayant accordé leur consentement au tracking analytique.

### 5. Catégories de données personnelles

- Adresse IP anonymisée (les 2 derniers octets sont masqués avant tout stockage)
- Pages visitées et durée des visites
- URL de référence (referrer)
- Type d'appareil, navigateur et système d'exploitation (sans fingerprinting individuel)
- Durée de session
- Identifiant de visite Matomo (pseudonyme, cookie `_pk_id`)

### 6. Catégories de destinataires

**Interne :** équipe marketing et direction FGRibreau SARL.

**Infrastructure :**
- **FGRibreau SARL elle-même** via matomo.hook0.com, hébergé sur **Clever Cloud SAS** (France). Aucun sous-traitant analytique tiers.

### 7. Transferts hors UE

Aucun. Matomo est auto-hébergé sur l'infrastructure Clever Cloud en France.

### 8. Durée de conservation

| Données | Durée | Justification |
|---|---|---|
| Données analytiques agrégées et individuelles | 25 mois | Recommandation CNIL pour les données d'analyse d'audience (délibération 2020-091) |
| Préférences de consentement (localStorage) | 13 mois | Durée maximale recommandée par la CNIL pour le stockage des préférences cookies |

### 9. Mesures de sécurité (description générale)

- Matomo est auto-hébergé : aucune donnée n'est transmise à un tiers.
- Anonymisation de l'IP avant stockage (2 derniers octets masqués).
- Interface d'administration Matomo protégée par authentification (MFA recommandé).
- Chiffrement en transit TLS 1.2+ entre le navigateur du visiteur et matomo.hook0.com.

### 10. Source des données

Collectées automatiquement lors de la navigation sur www.hook0.com, après consentement de la personne concernée.

---

## Traitement n°8 — Mesure des conversions publicitaires Google Ads (server-side) {#traitement-8}

### 1. Identité du responsable de traitement

**FGRibreau SARL** — cf. traitement n°1.

**Co-responsable de traitement (art. 26 RGPD) :** Google LLC, 1600 Amphitheatre Parkway, Mountain View, CA 94043, USA — dans le cadre des Customer Data Processing Terms (CDPT) Google Ads acceptés par FGRibreau SARL.

### 2. Finalité(s) du traitement

**Finalité unique :** Mesurer le coût par acquisition (CPA) et le retour sur investissement (ROI) des campagnes publicitaires Google Ads opérées par FGRibreau SARL pour le SaaS Hook0, en transmettant côté serveur l'identifiant de clic (gclid) à Google Ads lors d'une inscription validée.

Aucune finalité de profilage, de retargeting ou d'enrichissement de profil utilisateur.

### 3. Base légale (art. 6 RGPD)

**Art. 6.1.f — Intérêt légitime :** la mesure de l'efficacité des campagnes publicitaires constitue un intérêt légitime au sens du considérant 47 du RGPD. Une balance test complète (test tripartite WP29/EDPB Guidelines 06/2014) a été réalisée et documentée dans le fichier `legitimate-interest-balance-test-google-ads.md` (ISMS FGRibreau SARL, version du 4 mai 2026).

**Droit d'opposition (art. 21.2 RGPD) :** exercice via email à legal@hook0.com. Traitement de l'opposition sous 30 jours. L'opposition est persistée en base (`iam.user.marketing_opt_out_at`) pour bloquer tout upload futur.

### 4. Catégories de personnes concernées

Utilisateurs professionnels ayant créé un compte Hook0 après avoir cliqué sur une annonce Google Ads.

### 5. Catégories de données personnelles

- **gclid** (Google Click Identifier) : identifiant pseudonyme et opaque généré par Google lors du clic sur l'annonce. Qualifié de donnée personnelle conformément à la jurisprudence CJUE Breyer (C-582/14, 19 octobre 2016) et au considérant 26 du RGPD.

**Données NON traitées dans ce traitement :** adresse email, adresse IP, User-Agent, identifiants internes Hook0.

**Données transmises à Google :** gclid + identifiant de l'action de conversion (resource ID statique) + horodatage de conversion (ISO 8601).

**Persistance côté Hook0 :** le gclid n'est pas stocké en base de données. Il est traité exclusivement en mémoire vive le temps de l'upload à l'API Google Ads (`uploadClickConversions`). Les logs applicatifs peuvent contenir le gclid tronqué (8 premiers caractères, niveau info) pendant 30 jours.

### 6. Catégories de destinataires

**Interne :** direction marketing FGRibreau SARL (accès aux rapports agrégés Google Ads).

**Co-responsable :**
- **Google LLC** (USA) — dans le cadre des CDPT Google Ads.

### 7. Transferts hors UE

**Oui — vers les États-Unis (Google LLC).**
Base de transfert : Clauses Contractuelles Types (CCT) incluses dans les CDPT Google Ads, Décision 2021/914.
Lien CDPT : https://business.safety.google/adscontrollerterms/

### 8. Durée de conservation

| Données | Durée | Justification |
|---|---|---|
| gclid en mémoire vive | Quelques centaines de millisecondes (lifetime de la requête HTTP d'inscription) | Minimisation — le gclid disparaît dès l'upload terminé |
| Logs applicatifs contenant le gclid tronqué | 30 jours | Cohérent avec la politique de rétention des logs (logging-policy.md) |
| Rows orphelines `iam.signup_attribution` (utilisateurs non vérifiés) | 30 jours (cleanup automatique) | Minimisation ; nettoyage automatique des données non consolidées |

### 9. Mesures de sécurité (description générale)

- Aucun stockage persistant du gclid en base de données.
- Vérification du flag `marketing_opt_out_at IS NULL` avant tout upload (opposabilité du droit d'opposition).
- Accès à l'API Google Ads via OAuth2 service account, credentials stockés en variable d'environnement chiffrée.
- Référence : `legitimate-interest-balance-test-google-ads.md`, section 4 (mesures de sécurité).

### 10. Source des données

Le gclid est généré par Google Ads et injecté dans l'URL de destination par le mécanisme d'auto-tagging de Google (paramètre `?gclid=XXX`). Il est propagé par le navigateur de la personne concernée jusqu'au backend Hook0 lors de la soumission du formulaire d'inscription.

---

## Traitement n°9 — Communications commerciales (newsletters et release notes) {#traitement-9}

### 1. Identité du responsable de traitement

**FGRibreau SARL** — cf. traitement n°1.

### 2. Finalité(s) du traitement

**Finalité principale :** Informer les utilisateurs ayant consenti des nouveautés produit de Hook0 (nouvelles fonctionnalités, mises à jour majeures, release notes, communications commerciales).

### 3. Base légale (art. 6 RGPD)

**Art. 6.1.a — Consentement :** l'envoi de communications commerciales par email est subordonné au recueil d'un consentement explicite et distinct, séparé de l'acceptation des conditions générales lors de l'inscription. Ce consentement peut être retiré à tout moment via le lien de désabonnement inclus dans chaque email (conformément à l'art. L34-5 du Code des postes et des communications électroniques).

### 4. Catégories de personnes concernées

Utilisateurs professionnels de Hook0 ayant activement opté pour la réception de communications commerciales (opt-in).

### 5. Catégories de données personnelles

- Adresse email professionnelle
- Prénom (personnalisation des communications)

### 6. Catégories de destinataires

**Interne :** équipe marketing FGRibreau SARL.

**Sous-traitants :**
- **Brevo SAS** (France) — routage et gestion des listes de diffusion.

### 7. Transferts hors UE

Aucun. Brevo SAS est établie en France.

### 8. Durée de conservation

| Données | Durée | Justification |
|---|---|---|
| Email et prénom dans la liste de diffusion | Jusqu'au retrait du consentement | Art. 6.1.a RGPD — le traitement cesse dès retrait du consentement (art. 7.3) |
| Preuve de consentement (opt-in) | 5 ans à compter de la date du consentement | Art. 7.1 RGPD — démonstration de la conformité |

### 9. Mesures de sécurité (description générale)

- Lien de désabonnement (unsubscribe) inclus dans chaque email, fonctionnel en un clic.
- Double opt-in recommandé (envoi d'un email de confirmation avant ajout définitif à la liste).
- Accès à la liste Brevo restreint à l'équipe marketing (authentification MFA).

### 10. Source des données

Collectées directement auprès de la personne concernée lors de l'inscription (case opt-in séparée) ou via un formulaire de subscription dédié sur www.hook0.com.

---

## Traitement n°10 — Journaux HTTP — tracking par requête utilisateur {#traitement-10}

### 1. Identité du responsable de traitement

**FGRibreau SARL** — cf. traitement n°1.

### 2. Finalité(s) du traitement

**Finalité principale :** Conserver les journaux des requêtes HTTP à des fins de sécurité, de diagnostic technique et de prévention des abus (rate limiting, détection d'utilisation frauduleuse de l'API).

**Finalités secondaires :**
- Investigation post-incident (analyse forensique en cas de violation de données ou d'incident de sécurité).
- Détection et blocage des tentatives d'abus (scraping, brute-force, DDoS applicatif).

### 3. Base légale (art. 6 RGPD)

**Art. 6.1.f — Intérêt légitime :** la conservation des journaux d'accès constitue un intérêt légitime de sécurité informatique reconnu (cf. EDPB, Guidelines on Art. 6.1.f). La durée de conservation limitée à 30 jours et la stricte limitation d'accès garantissent la proportionnalité du traitement.

### 4. Catégories de personnes concernées

Utilisateurs professionnels de Hook0 utilisant l'API ou l'interface, et visiteurs de www.hook0.com dont les requêtes transitent par l'infrastructure Hook0 ou Cloudflare.

### 5. Catégories de données personnelles

- Adresse IP (source des requêtes)
- User-Agent du navigateur ou du client API
- URL et méthode HTTP de la requête
- Code de statut HTTP de la réponse
- Horodatage de la requête
- Latence (temps de réponse)

### 6. Catégories de destinataires

**Interne :** équipe technique FGRibreau SARL.

**Sous-traitants :**
- **Cloudflare, Inc.** (USA) — proxy réseau traitant les requêtes entrantes, journalisation des requêtes bloquées ou suspectes.
- **Sentry, Inc.** (USA) — journalisation des erreurs applicatives incluant les métadonnées de requête.

### 7. Transferts hors UE

**Oui — vers les États-Unis (Cloudflare, Inc. et Sentry, Inc.).**
Base de transfert : Clauses Contractuelles Types (CCT), Décision 2021/914.
- Cloudflare DPA : https://www.cloudflare.com/cloudflare-customer-dpa/
- Sentry DPA : https://sentry.io/legal/dpa/

### 8. Durée de conservation

| Données | Durée | Justification |
|---|---|---|
| Logs applicatifs (serveur Rust/Axum) | 30 jours | Minimisation — délai suffisant pour investigation post-incident |
| Logs Cloudflare | ≤ 30 jours | Politique Cloudflare, configurée au minimum |
| Logs Sentry (métadonnées requête) | 30 jours | Cohérent avec la politique de rétention Sentry de FGRibreau SARL |

### 9. Mesures de sécurité (description générale)

- Accès aux logs restreint à l'équipe technique (MFA).
- PII scrubbing activé dans le SDK Sentry : les données personnelles identifiables sont masquées avant transmission.
- Cloudflare : configuration firewall rules pour limiter la surface de collecte.
- Référence : `logging-policy.md` (ISMS FGRibreau SARL).

### 10. Source des données

Collectées automatiquement par les serveurs Hook0 et l'infrastructure Cloudflare lors de chaque requête HTTP.

---

## Traitement n°11 — Outil gratuit play.hook0.com — webhook playground {#traitement-11}

### 1. Identité du responsable de traitement

**FGRibreau SARL** — cf. traitement n°1.

Nota bene : pour les payloads que les utilisateurs de play.hook0.com envoient vers leurs endpoints de test, FGRibreau SARL agit en qualité de **sous-traitant au sens de l'art. 28 RGPD** si ces payloads contiennent des données personnelles appartenant à des tiers. L'utilisateur de l'outil reste responsable de ne pas y transmettre des données personnelles réelles de ses propres utilisateurs sans encadrement contractuel adéquat.

### 2. Finalité(s) du traitement

**Finalité principale :** Permettre à tout développeur de tester des webhooks entrants gratuitement et sans authentification, via un endpoint temporaire généré aléatoirement sur play.hook0.com.

**Finalités secondaires :**
- Faire connaître la plateforme Hook0 et ses capacités (outil de découverte/marketing).
- Permettre le débogage de l'intégration webhook d'un système tiers sans nécessiter un compte payant.

### 3. Base légale (art. 6 RGPD)

**Art. 6.1.f — Intérêt légitime :** la mise à disposition d'un outil gratuit de test webhook constitue un intérêt légitime de FGRibreau SARL (acquisition et fidélisation de développeurs). La balance d'intérêts est favorable : les données collectées sont minimales, éphémères et ne concernent que les métadonnées techniques de requêtes HTTP envoyées volontairement par l'utilisateur.

### 4. Catégories de personnes concernées

- Développeurs et techniciens utilisant l'outil de test (usage anonyme, pas d'inscription requise).
- Tiers dont les données personnelles pourraient figurer dans les payloads de test envoyés par les utilisateurs (responsabilité de l'utilisateur).

### 5. Catégories de données personnelles

**Données collectées par FGRibreau SARL en tant que responsable de traitement :**
- Adresse IP de l'utilisateur (logs HTTP standards).
- URL d'endpoint temporaire (pseudo-aléatoire, générée par le service).
- Headers HTTP des requêtes entrantes envoyées à l'endpoint de test.
- Horodatages des requêtes.

**Données en tant que sous-traitant potentiel (contenu des payloads) :**
- Nature variable, déterminée exclusivement par l'utilisateur. Peut contenir des données personnelles si l'utilisateur teste avec des données réelles.
- Hook0 ne lit pas, n'analyse pas et ne modifie pas le contenu des payloads.

### 6. Catégories de destinataires

**Interne :** équipe technique FGRibreau SARL (accès restreint aux logs d'infrastructure).

**Sous-traitants :**
- **France-Nuage** (France) — hébergement Kubernetes de l'application play.hook0.com (namespace `hosted-hook0-play-server-prod`). À vérifier : confirmer la signature d'un DPA avec France-Nuage.
- **Redis in-cluster** — stockage éphémère des payloads reçus (TTL court, géré par France-Nuage dans le cluster Kubernetes).

### 7. Transferts hors UE

Aucun. France-Nuage est un hébergeur français, datacenter en France (UE).

### 8. Durée de conservation

| Données | Durée | Justification |
|---|---|---|
| Payloads reçus sur l'endpoint de test | Quelques heures (TTL Redis configurable) | Minimisation — l'outil est conçu pour des tests temporaires |
| Logs HTTP (IP, headers, horodatage) | 30 jours maximum | Cohérent avec la politique de rétention des logs de FGRibreau SARL |

### 9. Mesures de sécurité (description générale)

- Pas d'authentification requise — les endpoints sont pseudo-aléatoires (sécurité par obscurité, non par authentification).
- Payloads stockés uniquement en mémoire Redis in-cluster, non persistés en base de données durable.
- Chiffrement en transit TLS 1.2+ entre l'utilisateur et play.hook0.com.
- Avertissement à intégrer dans les CGU et l'interface de play.hook0.com : recommander aux utilisateurs de ne pas envoyer de données personnelles réelles de leurs propres clients.

### 10. Source des données

Collectées automatiquement lors de l'envoi de requêtes HTTP vers les endpoints de test par les utilisateurs de l'outil.

---

## Traitement n°12 — Feedback produit in-app via Formbricks {#traitement-12}

### 1. Identité du responsable de traitement

**FGRibreau SARL** — cf. traitement n°1.

### 2. Finalité(s) du traitement

**Finalité principale :** Collecter le feedback des utilisateurs authentifiés de app.hook0.com via des sondages in-app (NPS, satisfaction produit, enquêtes sur les fonctionnalités) afin d'améliorer la plateforme Hook0.

**Finalités secondaires :**
- Mesurer le Net Promoter Score (NPS) de Hook0.
- Identifier les fonctionnalités à prioriser dans la roadmap produit.
- Détecter les utilisateurs en difficulté ou à risque de churn.

### 3. Base légale (art. 6 RGPD)

**Art. 6.1.f — Intérêt légitime :** l'amélioration continue d'un service SaaS à partir du feedback de ses utilisateurs constitue un intérêt légitime prépondérant de FGRibreau SARL. La balance d'intérêts est favorable : les utilisateurs professionnels B2B peuvent raisonnablement s'attendre à être sollicités pour améliorer un outil qu'ils utilisent dans le cadre de leur activité professionnelle ; ils peuvent refuser les sondages ou contacter legal@hook0.com pour exercer leur droit d'opposition.

Alternative à envisager : si Formbricks affiche des sondages pouvant être assimilés à des communications marketing, basculer vers le consentement (art. 6.1.a).

### 4. Catégories de personnes concernées

Utilisateurs professionnels authentifiés sur app.hook0.com ayant été exposés à un sondage Formbricks.

### 5. Catégories de données personnelles

- Identifiant utilisateur Hook0 (transmis à Formbricks via `formbricks.setUserId(state.userId)`).
- Adresse email professionnelle (transmise comme attribut utilisateur Formbricks).
- Prénom et nom (transmis comme attributs utilisateur Formbricks).
- Page courante dans l'application au moment du sondage.
- Réponses aux sondages (NPS, verbatim, choix multiples selon le sondage).

### 6. Catégories de destinataires

**Interne :** équipe produit et direction FGRibreau SARL.

**Sous-traitants :**
- **Formbricks GmbH** (Allemagne) — plateforme de sondages in-app. L'API host est configurable via l'env var `FORMBRICKS_API_HOST` (par défaut : `https://app.formbricks.com`). Si l'instance est self-hosted par FGRibreau SARL, Formbricks GmbH ne serait pas destinataire des données.

### 7. Transferts hors UE

**À vérifier.** Formbricks GmbH est une entité allemande (UE). Cependant, son offre SaaS Cloud (`app.formbricks.com`) peut s'appuyer sur des infrastructures cloud dont certaines ressources sont localisées hors UE (notamment AWS ou Azure US). FGRibreau SARL doit :
1. Consulter le DPA Formbricks et la liste de ses sous-traitants (sous-processors).
2. Vérifier que les CCT 2021/914 couvrent les éventuels transferts vers des sous-processors hors UE.
3. Documenter le résultat dans ce registre lors du prochain réexamen.

Si Formbricks Cloud transfère des données aux USA, la base de transfert applicable est les CCT 2021/914 incluses dans le DPA Formbricks. Lien DPA Formbricks : https://formbricks.com/privacy (à vérifier).

### 8. Durée de conservation

| Données | Durée | Justification |
|---|---|---|
| Réponses aux sondages | Durée du contrat + 3 ans | Analyse des tendances produit et gestion des litiges |
| Attributs utilisateur (email, prénom, nom) | Jusqu'à suppression du compte Hook0 | Cohérence avec le traitement n°1 |

### 9. Mesures de sécurité (description générale)

- Accès à la plateforme Formbricks restreint à l'équipe produit (authentification MFA recommandée).
- Chiffrement en transit TLS 1.2+ entre l'application et l'API Formbricks.
- L'env var `FORMBRICKS_API_HOST` permet de basculer vers une instance Formbricks self-hosted si FGRibreau SARL souhaite éliminer tout transfert de données vers un tiers.

### 10. Source des données

Collectées directement auprès de la personne concernée lors de sa participation à un sondage in-app sur app.hook0.com, et indirectement via les attributs utilisateur transmis par l'API Hook0 à Formbricks lors de l'initialisation de la session.

---

## Annexe 1 — Liste consolidée des sous-traitants {#annexe-1}

| Sous-traitant | Rôle | Pays | DPA en vigueur | Base de transfert hors UE |
|---|---|---|---|---|
| **Clever Cloud SAS** | Hébergement infrastructure (BDD, API, workers) | France (UE) | DPA Clever Cloud | N/A |
| **Scaleway SAS** | Workers dédiés (offres sélectionnées) + Object Storage S3-compatible (payloads webhook volumineux) | France (UE) | DPA Scaleway (à vérifier : couvre Object Storage en plus des workers ?) | N/A |
| **Stripe, Inc.** | Paiement et gestion des abonnements | USA | DPA Stripe | CCT 2021/914 + DPF |
| **Brevo SAS (Sendinblue)** | Emails transactionnels et newsletters | France (UE) | DPA Brevo | N/A |
| **Postmark (ActiveCampaign, Inc.)** | Emails transactionnels (complément / bascule) | USA | DPA Postmark | CCT 2021/914 |
| **Crisp IM SAS** | Chat support client (conditionné au consentement) | France (UE) | DPA Crisp | N/A |
| **Google LLC (Gmail / Google Workspace)** | Messagerie email du support | USA | DPA Google Workspace | CCT 2021/914 + DPF |
| **Google LLC (Google Ads)** | Mesure de conversion server-side (co-responsable art. 26) | USA | CDPT Google Ads | CCT 2021/914 + DPF |
| **Sentry, Inc.** | Suivi des erreurs applicatives | USA | DPA Sentry | CCT 2021/914 |
| **BetterStack (BetterUptime)** | Monitoring de disponibilité | République tchèque (UE) | DPA BetterStack | N/A |
| **Cloudflare, Inc.** | Proxy réseau, DDoS, DNS | USA | DPA Cloudflare | CCT 2021/914 + DPF |
| **Cloudflare, Inc. — Turnstile** | CAPTCHA anti-bot à l'inscription | USA | DPA Cloudflare (à vérifier : Turnstile explicitement couvert ?) | CCT 2021/914 + DPF |
| **France-Nuage** | Hébergement Kubernetes play.hook0.com | France (UE) | DPA à signer (vérification requise) | N/A |
| **Formbricks GmbH** | In-app surveys / feedback utilisateur (app.hook0.com) | Allemagne (UE) — Cloud peut utiliser infra hors UE | DPA Formbricks à vérifier et signer | À vérifier (sous-processors Formbricks Cloud) |

**Légende :**
- CCT 2021/914 : Clauses Contractuelles Types, Décision de la Commission européenne du 4 juin 2021.
- DPF : EU-U.S. Data Privacy Framework (Décision d'adéquation de la Commission du 10 juillet 2023, C(2023) 4745).
- N/A : sous-traitant établi dans l'UE, aucun transfert hors UE.

**Liens de référence :**
- Stripe DPA : https://stripe.com/privacy
- Postmark DPA : https://postmarkapp.com/eu-privacy
- Google Workspace DPA : https://workspace.google.com/terms/dpa_terms.html
- CDPT Google Ads : https://business.safety.google/adscontrollerterms/
- Sentry DPA : https://sentry.io/legal/dpa/
- Cloudflare DPA : https://www.cloudflare.com/cloudflare-customer-dpa/
- Formbricks DPA : https://formbricks.com/privacy (à vérifier et signer)
- France-Nuage : DPA à solliciter auprès de France-Nuage

---

## Annexe 2 — Procédure de mise à jour du registre {#annexe-2}

### Qui peut modifier le registre

Le registre peut être modifié par :
1. Le représentant légal de FGRibreau SARL (David Sferruzza).
2. Tout membre de l'équipe désigné par le représentant légal pour la conformité RGPD.

Toute modification doit être approuvée par le représentant légal avant d'être considérée comme officielle.

### Quand mettre à jour le registre

Le registre doit être mis à jour dans les situations suivantes :

**Révision annuelle obligatoire :**
- Prochain réexamen planifié : **4 mai 2027**.

**Mise à jour immédiate requise (dans les 30 jours) :**
- Ajout d'un nouveau sous-traitant ou d'un nouveau traitement de données.
- Modification d'une finalité, d'une base légale ou d'une durée de conservation.
- Changement de sous-traitant existant (remplacement, cessation de service).
- Évolution légale ou réglementaire affectant les bases légales retenues (nouvelle décision CJUE, délibération CNIL, etc.).
- Incident de sécurité ayant conduit à une violation de données personnelles.
- Évolution du périmètre du service Hook0 (nouvelles fonctionnalités, nouveau territoire).

### Comment versionner

1. Mettre à jour le numéro de version (format `MAJEUR.MINEUR` : MAJEUR pour un nouveau traitement ou une modification substantielle, MINEUR pour une correction ou clarification).
2. Mettre à jour la date d'établissement dans l'en-tête.
3. Créer un commit git dédié avec le message : `docs(rgpd): mise à jour registre des traitements vX.Y — [description courte]`.
4. Conserver l'historique git comme journal d'audit des modifications.

### Procédure d'ajout d'un nouveau traitement

1. Identifier la nouvelle activité de traitement (nouvelle fonctionnalité, nouveau sous-traitant, nouvelle finalité).
2. Compléter une fiche de traitement selon le modèle des 10 sections ci-dessus.
3. Évaluer si une Analyse d'Impact relative à la Protection des Données (AIPD) est requise (art. 35 RGPD), notamment si le traitement implique : des données sensibles, du profilage à grande échelle, ou un traitement systématique de données à grande échelle.
4. Mettre à jour l'Annexe 1 (liste des sous-traitants) si un nouveau sous-traitant est impliqué et vérifier la signature d'un DPA.
5. Mettre à jour la politique de confidentialité publique (`privacy-policy.ejs`) en cohérence.
6. Soumettre à approbation du représentant légal.

---

## Annexe 3 — Procédure en cas de violation de données personnelles {#annexe-3}

### Définition (art. 4.12 RGPD)

Une violation de données personnelles est une violation de la sécurité entraînant, de manière accidentelle ou illicite, la destruction, la perte, l'altération, la divulgation non autorisée de données personnelles transmises, conservées ou traitées d'une autre manière, ou l'accès non autorisé à de telles données.

### Obligation de notification à la CNIL — art. 33 RGPD

**Délai :** 72 heures à compter de la prise de connaissance de la violation.

**Exception :** si la violation est peu susceptible d'engendrer un risque pour les droits et libertés des personnes physiques, la notification n'est pas obligatoire (art. 33.1 in fine). Cette appréciation doit être documentée.

**Destinataire :** Commission Nationale de l'Informatique et des Libertés (CNIL) — notification en ligne via : https://notifications.cnil.fr/notifications/index

**Contenu obligatoire de la notification (art. 33.3 RGPD) :**
1. Description de la nature de la violation (catégories et nombre approximatif de personnes concernées, catégories et nombre approximatif d'enregistrements).
2. Coordonnées du point de contact RGPD (legal@hook0.com).
3. Description des conséquences probables de la violation.
4. Description des mesures prises ou envisagées pour y remédier.

La notification peut être effectuée en plusieurs étapes si toutes les informations ne sont pas immédiatement disponibles (art. 33.4 RGPD).

### Obligation de communication aux personnes concernées — art. 34 RGPD

Si la violation est **susceptible d'engendrer un risque élevé** pour les droits et libertés des personnes physiques, FGRibreau SARL est tenue d'en informer les personnes concernées **dans les meilleurs délais**.

**Communication non requise** si l'une des conditions suivantes est remplie (art. 34.3 RGPD) :
- Les données étaient chiffrées et la clé n'a pas été compromise.
- Des mesures ultérieures rendent le risque élevé improbable.
- La communication exigerait des efforts disproportionnés (communication publique possible à la place).

### Procédure interne de FGRibreau SARL

1. **Détection :** tout membre de l'équipe ayant connaissance d'un incident de sécurité potentiel le signale immédiatement à legal@hook0.com.
2. **Qualification (H+0 à H+4) :** l'équipe technique détermine si l'incident constitue une violation de données personnelles au sens de l'art. 4.12 RGPD.
3. **Évaluation du risque (H+4 à H+24) :** évaluation de la probabilité et de la gravité du risque pour les personnes concernées (grille CNIL : https://www.cnil.fr/fr/les-violations-de-donnees-personnelles).
4. **Décision de notification (H+24) :** le représentant légal décide de notifier ou non la CNIL, et documente la décision.
5. **Notification CNIL (avant H+72) :** notification via le portail CNIL, avec le contenu requis par l'art. 33.3 RGPD.
6. **Communication aux personnes (si risque élevé) :** email individuel aux personnes concernées dans les meilleurs délais.
7. **Documentation interne (art. 33.5 RGPD) :** consignation de la violation, de son impact, des mesures prises et de toutes les décisions (notification ou non). Cette documentation est conservée 5 ans.

**Référence complémentaire :** `business-continuity-disaster-recovery.md` (ISMS FGRibreau SARL) — section sur la procédure de notification des incidents.

---

*Document public FGRibreau SARL — Version 1.1 — Établi le 4 mai 2026 — Prochain réexamen : 4 mai 2027.*
*Publié dans le dépôt open-source Hook0 par souci de transparence. Disponible sur demande de la CNIL conformément à l'article 30.4 du Règlement (UE) 2016/679.*
