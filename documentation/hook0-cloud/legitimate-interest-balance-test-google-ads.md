# Balance Test — Intérêt légitime (art. 6.1.f RGPD)

**Traitement** : transmission du gclid à Google Ads pour mesure de conversion publicitaire (server-side)
**Responsable de traitement** : FGRibreau SARL, 3 rue de l'Aubépine, 85110 Chantonnay, France (RCS La Roche-sur-Yon 850 824 350)
**Co-responsable** : Google LLC, dans le cadre des Customer Data Processing Terms (CDPT — art. 26 RGPD)
**Date d'établissement** : 4 mai 2026
**Auteur** : Direction Hook0
**Référence légale principale** : Règlement (UE) 2016/679 (RGPD), art. 6.1.f
**Référentiels appliqués** : WP29/EDPB Guidelines 06/2014 sur la notion d'intérêt légitime (test tripartite), CJUE C-582/14 Breyer (qualification donnée personnelle), Délibération CNIL 2020-091
**Périmètre** : ce document s'applique exclusivement aux campagnes Google Ads opérées par FGRibreau SARL pour le SaaS Hook0 (`www.hook0.com` / `app.hook0.com`). Il ne couvre pas les déploiements self-hosted de Hook0 (open-source) qui n'utilisent pas ce traitement par défaut.

---

## 1. Description du traitement

Le `gclid` (Google Click Identifier) est un identifiant opaque généré par Google côté ad network lors du clic d'un internaute sur une annonce Google Ads. Il est injecté automatiquement dans l'URL de destination par le mécanisme d'auto-tagging de Google Ads (paramètre `?gclid=XXX` ajouté à l'URL de landing). Cet identifiant n'est interprétable que par Google : Hook0 ne peut, à lui seul, ni en déduire l'identité de l'utilisateur, ni le rattacher à un profil publicitaire.

**Trajet fonctionnel des données** :

1. L'utilisateur clique sur une annonce Google Ads et atterrit sur `www.hook0.com/?gclid=XXX`.
2. L'utilisateur clique sur le bouton « Start Free » qui le redirige vers `app.hook0.com/register?gclid=XXX` (le gclid est propagé via le query string).
3. Le frontend Vue lit le gclid depuis l'URL et l'inclut dans le payload du formulaire de signup.
4. Le backend Rust (Axum) valide le formulaire, crée l'utilisateur en base PostgreSQL, et commite la transaction d'inscription.
5. Une fois la transaction d'inscription réussie, un appel `tokio::spawn` fire-and-forget déclenche un upload vers l'API Google Ads `uploadClickConversions` (mode server-side).
6. La réponse de Google n'est ni attendue ni bloquante : l'inscription utilisateur réussit indépendamment de l'issue de l'upload conversion.

**Données effectivement transmises à Google** :

- Le `gclid` lui-même.
- L'identifiant de la `conversionAction` (resource ID statique configuré côté Hook0).
- Le `conversionDateTime` (horodatage de la conversion, au format ISO 8601).

**Données NON transmises à Google** :

- Adresse email de l'utilisateur (en clair ou hashée).
- Adresse IP du client.
- User-Agent du navigateur.
- Prénom, nom, ou tout élément d'état civil.
- Identifiants Hook0 internes (`user_id`, `organization_id`).
- Données techniques relatives à l'usage du SaaS Hook0 (events, webhooks, etc.).

**Persistance côté Hook0** : aucune persistance pérenne du gclid en base. Le gclid n'est conservé que dans la mémoire vive du processus serveur le temps strictement nécessaire à l'upload de la conversion (lifetime de la requête HTTP de signup, soit quelques centaines de millisecondes). Aucune table SQL ne contient de colonne `gclid` après commit. Seuls les logs applicatifs peuvent contenir trace du gclid (voir section 4 sur la rétention).

---

## 2. Qualification juridique du gclid

Le gclid n'identifie pas directement un individu pour FGRibreau SARL : il s'agit d'une chaîne pseudo-aléatoire opaque dont la table de correspondance vers un cookie publicitaire `_gads` est détenue exclusivement par Google.

Cependant, conformément à la jurisprudence **CJUE Breyer (C-582/14, 19 octobre 2016)** et au **considérant 26 du RGPD**, une donnée doit être qualifiée de personnelle dès lors qu'un tiers raisonnablement accessible peut, par des moyens raisonnablement disponibles, la rattacher à une personne physique. Google détient les moyens techniques et juridiques de ré-identifier l'utilisateur derrière un gclid (via son cookie publicitaire et son écosystème ad tech). En conséquence, le gclid constitue **une donnée à caractère personnel au sens de l'art. 4§1 RGPD** dans le chef de FGRibreau SARL, même si celle-ci ne peut pas opérer la ré-identification elle-même.

Cette qualification déclenche l'application du RGPD au traitement décrit. Une base légale est donc requise au titre de l'art. 6 RGPD. FGRibreau SARL retient l'**intérêt légitime (art. 6.1.f)**, dont la validité est étayée par le test tripartite ci-dessous.

---

## 3. Test tripartite WP29/EDPB (Guidelines 06/2014)

### 3.1 Existence d'un intérêt légitime

- **Nature de l'intérêt** : mesurer l'efficacité des campagnes publicitaires Google Ads afin d'optimiser l'allocation du budget marketing (calcul du CPA — coût par acquisition — et du ROI campagne par campagne, mot-clé par mot-clé).
- **Légalité de l'intérêt** : le marketing direct, y compris dans sa dimension d'analyse d'efficacité publicitaire, est expressément reconnu comme un intérêt légitime par les Guidelines 06/2014 du WP29 (exemple 6, p. 25) et par le considérant 47 du RGPD (« Le traitement de données à caractère personnel à des fins de prospection commerciale peut être considéré comme étant réalisé pour répondre à un intérêt légitime »).
- **Réalité de l'intérêt** : FGRibreau SARL opère effectivement des campagnes Google Ads avec un budget mensuel de l'ordre de 500 EUR. L'intérêt n'est donc ni hypothétique ni spéculatif, mais directement lié à une activité économique mesurable.
- **Précision de l'intérêt** : la finalité poursuivie est strictement la mesure agrégée de la performance publicitaire (combien de signups proviennent de chaque campagne / annonce / mot-clé). Il ne s'agit ni de profilage individuel, ni de retargeting, ni d'enrichissement de profil utilisateur.

### 3.2 Nécessité du traitement

L'écosystème Google Ads ne propose **aucun mécanisme alternatif** permettant d'attribuer une conversion à un clic publicitaire en l'absence de remontée du gclid. Sans gclid, Google Ads ne peut pas relier une inscription Hook0 à la campagne qui l'a générée, ce qui rend impossible l'optimisation budgétaire.

**Comparaison avec les alternatives techniquement disponibles** (du plus intrusif au moins intrusif) :

| Alternative | Données transmises à Google | Niveau d'intrusion | Décision |
|-------------|----------------------------|--------------------|----------|
| `gtag.js` client-side classique | gclid + IP + User-Agent + cookies Google + referrer | Élevé | **Rejetée** |
| Enhanced Conversions for Leads (hash email) | gclid + SHA-256(email) | Moyen | **Rejetée** (ré-identifiable chez Google) |
| **Mode A — gclid only server-side** | gclid + conversionAction + timestamp | **Minimal** | **Retenue** |

La solution retenue (Mode A server-side, gclid only) est le **strict minimum techniquement viable** pour atteindre la finalité poursuivie. Elle satisfait au principe de minimisation des données (art. 5.1.c RGPD) et au critère de nécessité du test tripartite.

### 3.3 Balance des intérêts vs droits et libertés des personnes

| En faveur du traitement (FGRibreau SARL) | En faveur de la personne concernée |
|------------------------------------------|------------------------------------|
| La mesure du CPA conditionne l'allocation du budget marketing : sans données fiables, gaspillage budgétaire et campagnes à l'aveugle. | Aucune donnée directement identifiante (email, IP, UA) n'est transmise. Le risque d'identification est limité au gclid seul. |
| L'optimisation des campagnes contribue à la compétitivité d'une PME française (FGRibreau SARL) face à des concurrents internationaux disposant de ressources marketing supérieures. | L'utilisateur effectue un acte volontaire de souscription à un service B2B SaaS (signature d'un contrat). Le contexte est explicitement commercial. |
| Pratique standard et largement documentée du marketing digital B2B (équivalent à un comptage anonymisé des conversions). | L'utilisateur ne s'attend pas nécessairement à ce que le clic publicitaire qui l'a amené sur le site soit retracé jusqu'à son inscription. |
| Aucun impact négatif sur l'expérience utilisateur (pas de profilage, pas de modification du parcours, pas de différenciation tarifaire). | Risque de ré-identification chez Google, qui dispose déjà de données sur l'internaute via son propre écosystème publicitaire. |

**Mitigations effectivement mises en œuvre en faveur de la personne concernée** :

- **Information transparente** : sections 2 et 9b de la politique de confidentialité (`privacy-policy.md`) décrivent le traitement, sa finalité, sa base légale, le co-responsable, et les droits afférents.
- **Information contextuelle au moment du recueil** : mention courte sous le bouton « S'inscrire » sur `app.hook0.com/register`, avec lien vers la politique de confidentialité (conformité art. 13.1 RGPD).
- **Droit d'opposition art. 21.2 RGPD** : effectif via l'adresse `legal@hook0.com`. Toute demande est traitée sous 30 jours.
- **Aucun marketing direct opéré par Hook0** sur le compte créé, à l'exception des newsletters strictement opt-in.
- **Données transmises minimales** : gclid uniquement, conformément au principe de minimisation.
- **Données non persistées en base Hook0** : le gclid disparaît de l'environnement Hook0 dès que l'upload Google Ads est terminé.
- **Co-responsabilité formalisée** : les Customer Data Processing Terms (CDPT) Google Ads sont acceptés et formalisent les obligations respectives au titre de l'art. 26 RGPD.

**Conclusion de la balance** : l'intérêt légitime de FGRibreau SARL à mesurer l'efficacité de ses campagnes publicitaires prévaut sur les droits et libertés des personnes concernées, **sous réserve** que les mitigations énumérées soient dûment mises en œuvre et maintenues opérationnelles.

---

## 4. Mise en œuvre technique des mitigations

| Mesure | Statut | Référence |
|--------|--------|-----------|
| Politique de confidentialité mise à jour (sections 2 + 9b) | Réalisée | `documentation/hook0-cloud/privacy-policy.md`, version du 4 mai 2026 |
| Mention courte sous le bouton « Sign Up » + lien vers la politique de confidentialité | Réalisée | Composant Vue `RegisterForm.vue` |
| Endpoint `legal@hook0.com` opérationnel pour exercice du droit d'opposition art. 21.2 | Réalisé | Documenté dans la politique de confidentialité, section « Vos droits » |
| Procédure interne sur réception d'une demande d'opposition art. 21.2 | Réalisée | Champ `marketing_opt_out_at: timestamptz` ajouté sur `iam.user`. Le code d'upload conversion vérifie `WHERE marketing_opt_out_at IS NULL` avant fire-and-forget. |
| Co-responsabilité Google Ads art. 26 RGPD | Réalisée | Customer Data Processing Terms acceptés dans la console Google Ads |
| Transfert hors UE encadré | Réalisé | Clauses Contractuelles Types (SCC 2021/914) incluses dans les CDPT acceptés |
| Inscription au registre des traitements art. 30 RGPD | Réalisée | Voir document séparé `record-of-processing-activities.md` |

**Logging** : le gclid peut apparaître en logs applicatifs (niveau `info` : préfixe 8 caractères tronqué ; niveau `debug` : valeur complète). La rétention des logs est plafonnée à 30 jours, après quoi les logs sont automatiquement purgés. Aucun partage de logs avec un tiers n'est opéré.

---

## 5. Risques résiduels acceptés

| Risque | Niveau | Mitigation |
|--------|:------:|------------|
| Contestation de la base légale art. 6.1.f par la CNIL en cas de contrôle | Faible | Présent document + CDPT signés + droit d'opposition fonctionnel |
| Bug dans le mécanisme d'opt-out art. 21.2 entraînant un upload malgré une opposition exprimée | Faible | Test unitaire backend à ajouter (vérification du filtre `marketing_opt_out_at IS NULL` avant fire-and-forget) |
| Fuite du gclid via les logs applicatifs | Très faible | Rétention plafonnée à 30 jours, pas de transfert tiers, accès logs restreint à l'équipe technique |
| Évolution jurisprudentielle sur le statut du gclid (CJUE, CNIL) ou modification unilatérale des CDPT par Google | Moyen | Veille juridique CNIL/CJUE annuelle, ré-examen formel du présent document tous les 12 mois |

Ces risques résiduels sont jugés acceptables au regard de l'intérêt légitime poursuivi et de la robustesse des mitigations en place.

---

## 6. Procédure de réexamen

- **Périodicité** : tous les 12 mois, ou immédiatement à chaque évolution majeure (changement de finalité, modification des CDPT Google, jurisprudence CJUE/CNIL pertinente, modification du périmètre des données transmises).
- **Prochain réexamen planifié** : **4 mai 2027**.
- **Responsable du réexamen** : Direction Hook0 (FGRibreau SARL), avec appui du DPO interne désigné le cas échéant.
- **Critères déclenchant un réexamen anticipé** :
  - Évolution des Guidelines EDPB sur l'intérêt légitime ou sur le marketing digital.
  - Décision CNIL ou CJUE remettant en cause la qualification ou le régime du gclid.
  - Modification unilatérale par Google des CDPT, des Terms of Service Google Ads, ou de l'API `uploadClickConversions`.
  - Changement de finalité ou élargissement des données transmises (ex. ajout d'un hash email — déclencherait obligatoirement une nouvelle balance).

---

## 7. Annexes

### Annexe 1 — Références légales et doctrinales

- Règlement (UE) 2016/679 du 27 avril 2016 (RGPD), art. 4§1, art. 5, art. 6.1.f, art. 13, art. 21.2, art. 26, art. 30, art. 44 et s. ; considérants 26, 47.
- Loi Informatique et Libertés modifiée (loi n° 78-17 du 6 janvier 1978).
- WP29/EDPB, *Guidelines 06/2014 on the notion of legitimate interests of the data controller under Article 7 of Directive 95/46/EC* (transposable au RGPD), WP217.
- CJUE, 19 octobre 2016, *Patrick Breyer c. Bundesrepublik Deutschland*, C-582/14 (qualification de donnée personnelle d'un identifiant ré-identifiable par un tiers).
- CJUE, 16 juillet 2020, *Schrems II*, C-311/18 (encadrement des transferts hors UE).
- CNIL, Délibération n° 2020-091 du 17 septembre 2020 portant adoption de lignes directrices relatives aux cookies et autres traceurs.
- CNIL, *Guide pratique de la conformité RGPD pour les TPE/PME*.

### Annexe 2 — Flux de données (description)

```
[Internaute]
   |
   | (1) clic sur annonce Google Ads
   v
[Google Ads]
   |
   | (2) redirection vers www.hook0.com/?gclid=XXX (auto-tagging)
   v
[www.hook0.com — landing]
   |
   | (3) clic sur "Start Free"
   v
[app.hook0.com/register?gclid=XXX]
   |
   | (4) submit form { email, password, gclid, ... }
   v
[Backend Hook0 — Rust/Axum]
   |
   |---(5a) INSERT INTO iam.user (...)  -- transaction commit
   |
   |---(5b) tokio::spawn fire-and-forget
   |        |
   |        v
   |     [Google Ads API]
   |        uploadClickConversions { gclid, conversionAction, conversionDateTime }
   |
   v
[Réponse 200 au frontend, gclid hors mémoire]
```

### Annexe 3 — Référence aux CDPT Google Ads

- Customer Data Processing Terms (Google Ads) : <https://business.safety.google/adscontrollerterms/>
- Acceptation : effectuée dans la console Google Ads par l'administrateur du compte FGRibreau SARL.
- Date d'acceptation : à archiver dans le dossier juridique (capture d'écran horodatée recommandée).

---

*Document interne — non destiné à publication. À conserver dans le registre RGPD de FGRibreau SARL et à présenter sur demande de l'autorité de contrôle (CNIL).*
