# Hook0 CLI — Rapport de corrections de bugs

> Branche : `feat/cli`
> Date : 2026-03-23
> Validé par : 7 runs QA indépendants (214 tests automatisés + ~100 tests manuels par run)

---

## Bug 1 (Critique) : Le keyring utilise MockCredential — les credentials ne sont jamais persistées

**Symptôme :** `hook0 login` affiche "Credentials saved" mais chaque commande suivante échoue avec "No matching entry found in secure storage". Le crate keyring v3 utilise `MockCredential` par défaut quand aucune feature platform n'est activée.

**Cause racine :** `cli/Cargo.toml` déclarait `keyring = "3.6"` sans features spécifiques à la plateforme.

**Correction :** Dépendances spécifiques par target avec les backends keyring natifs.

```diff
# cli/Cargo.toml

- keyring = "3.6"
+ keyring = { version = "3.6" }
+
+ [target.'cfg(target_os = "macos")'.dependencies]
+ keyring = { version = "3.6", features = ["apple-native"] }
+
+ [target.'cfg(target_os = "windows")'.dependencies]
+ keyring = { version = "3.6", features = ["windows-native"] }
+
+ [target.'cfg(target_os = "linux")'.dependencies]
+ keyring = { version = "3.6", features = ["linux-native"] }
```

**Validé sur :** macOS (`MacCredential`) et Linux Docker (`KeyutilsCredential`).

---

## Bug 2 (Haute) : `event send` — mauvais endpoint API

**Symptôme :** `event send` retourne 405 Method Not Allowed.

**Cause racine :** Le client envoie `POST /events` mais l'API attend `POST /event/` (singulier, avec slash final).

**Correction :**

```diff
# cli/src/api/client.rs

-    .post(self.url("/events"))
+    .post(self.url("/event/"))
```

---

## Bug 3 (Haute) : `event send` — payload encodé en base64 à tort

**Symptôme :** L'API rejette l'événement avec "Event payload is not encoded in valid JSON format". Le CLI encode le payload en base64 mais l'API attend du JSON brut.

**Correction :**

```diff
# cli/src/api/models.rs — EventPost::new_json

-    payload: base64_encode(&payload.to_string()),
+    payload: payload.to_string(),
```

---

## Bug 4 (Haute) : `event-type delete` — mauvais endpoint API

**Symptôme :** Retourne 405 Method Not Allowed. Le client envoie `DELETE /event_types?query_params` mais l'API attend `DELETE /event_types/{name}`.

**Correction :**

```diff
# cli/src/api/client.rs

-    .delete(self.url("/event_types"))
-    .query(&[
-        ("application_id", ...),
-        ("service", ...),
-        ("resource_type", ...),
-        ("verb", ...),
-    ])
+    let event_type_name = format!("{}.{}.{}", service, resource_type, verb);
+    .delete(self.url(&format!("/event_types/{}", event_type_name)))
+    .query(&[("application_id", application_id.to_string())])
```

---

## Bug 5 (Haute) : `event get` — paramètre `application_id` manquant

**Symptôme :** Retourne 400 "missing field application_id".

**Correction :**

```diff
# cli/src/api/client.rs

- pub async fn get_event(&self, event_id: &Uuid) -> Result<Event, ApiError> {
+ pub async fn get_event(&self, event_id: &Uuid, application_id: &Uuid) -> Result<Event, ApiError> {
      ...
      .get(self.url(&format!("/events/{}", event_id)))
+     .query(&[("application_id", application_id.to_string())])
```

---

## Bug 6 (Haute) : `subscription update/enable/disable` — `application_id` manquant dans le payload

**Symptôme :** Retourne 400 "missing field application_id".

**Correction :** Ajout du champ `application_id` au struct `SubscriptionPut` et à tous les sites d'appel.

```diff
# cli/src/api/models.rs

 pub struct SubscriptionPut {
+    pub application_id: Uuid,
     pub event_types: Vec<String>,
     ...
```

---

## Bug 7 (Haute) : `subscription delete` — paramètre `application_id` manquant

**Symptôme :** Retourne 400 "missing field application_id".

**Correction :**

```diff
# cli/src/api/client.rs

- pub async fn delete_subscription(&self, subscription_id: &Uuid) -> ...
+ pub async fn delete_subscription(&self, subscription_id: &Uuid, application_id: &Uuid) -> ...
      ...
+     .query(&[("application_id", application_id.to_string())])
```

---

## Bug 8 (Haute) : Le mode override (`--secret` + `--api-url`) utilise `Uuid::nil()` comme application_id

**Symptôme :** Tous les appels API échouent avec 401 car le profil a `00000000-0000-0000-0000-000000000000` comme application_id.

**Correction :** Ajout du flag global `--application-id` (env : `HOOK0_APPLICATION_ID`) et obligation de le fournir en mode override.

```diff
# cli/src/lib.rs

+    /// Override application ID
+    #[arg(long, env = "HOOK0_APPLICATION_ID", global = true)]
+    pub application_id: Option<uuid::Uuid>,

# cli/src/commands/mod.rs

     if let (Some(secret), Some(api_url)) = (&cli.secret, &cli.api_url) {
-        let profile = Profile::new(api_url.clone(), uuid::Uuid::nil());
+        let app_id = cli.application_id.ok_or_else(|| {
+            anyhow!("--application-id (or HOOK0_APPLICATION_ID) is required when using --secret and --api-url overrides")
+        })?;
+        let profile = Profile::new(api_url.clone(), app_id);
```

---

## Bug 9 (Haute) : `replay` — `application_id` et `Content-Type` manquants

**Symptôme :** `POST /events/{id}/replay` retourne 400 "Content-Type header should be set to application/json" et "missing field application_id".

**Correction :**

```diff
# cli/src/api/client.rs

- pub async fn replay_event(&self, event_id: &Uuid) -> ...
+ pub async fn replay_event(&self, event_id: &Uuid, application_id: &Uuid) -> ...
      ...
-     .send()
+     .json(&serde_json::json!({ "application_id": application_id }))
+     .send()
+     // Le replay peut retourner 204 No Content
+     if response.status() == reqwest::StatusCode::NO_CONTENT {
+         return Ok(vec![]);
+     }
```

---

## Bug 10 (Moyenne) : `RequestAttemptStatus` — mauvais discriminant serde

**Symptôme :** `event get --attempts` échoue avec "missing field status". L'API utilise `"type"` comme tag de l'enum, pas `"status"`.

**Correction :**

```diff
# cli/src/api/models.rs

-#[serde(tag = "status", rename_all = "snake_case")]
+#[serde(tag = "type", rename_all = "snake_case")]
 pub enum RequestAttemptStatus {
```

---

## Bug 11 (Moyenne) : `subscription enable/disable` — `dedicated_workers` vide rejeté

**Symptôme :** L'API retourne 422 "dedicated_workers: length min=1". Le CLI envoie `[]` que l'API rejette.

**Correction :** Envoyer `None` au lieu de `Some(vec![])` quand le vecteur est vide.

```diff
# cli/src/api/client.rs (enable_subscription & disable_subscription)

-    dedicated_workers: Some(sub.dedicated_workers),
+    dedicated_workers: if sub.dedicated_workers.is_empty() { None } else { Some(sub.dedicated_workers) },
```

---

## Bug 12 (Moyenne) : Le flag `--content-type` est ignoré sur `event send`

**Symptôme :** Envoyer avec `--content-type text/plain` stocke quand même `application/json` car `new_text()` hardcode le content type.

**Correction :** Nouvelle méthode `EventPost::new_with_content_type()` qui accepte le content type en paramètre.

```diff
# cli/src/commands/event.rs

-    EventPost::new_text(profile.application_id, ...)
+    EventPost::new_with_content_type(profile.application_id, ..., args.content_type.clone(), ...)
```

---

## Bug 13 (Moyenne) : `whoami` échoue en mode override sans fichier de config

**Symptôme :** `--secret X --api-url Y --application-id Z whoami` retourne "Configuration file not found" car whoami appelle toujours `Config::load()`.

**Correction :** Support du mode override dans whoami, validation des credentials contre l'API.

```diff
# cli/src/commands/auth.rs — whoami

-    let config = Config::load()?;
-    let (name, profile) = config.get_profile(profile_name)?;
-    let has_secret = Config::has_secret(&name, &profile.application_id);
+    let (name, profile, has_secret) =
+        if let (Some(secret), Some(api_url), Some(app_id)) = (&cli.secret, &cli.api_url, &cli.application_id) {
+            let client = ApiClient::new(api_url, secret);
+            let valid = client.get_application(app_id).await.is_ok();
+            ("override".to_string(), Profile::new(api_url.clone(), *app_id), valid)
+        } else {
+            // ... chargement depuis la config comme avant
+        };
```

---

## Bug 14 (Moyenne) : `login -o json` mélange du texte de progression avec le JSON

**Symptôme :** `login --output json` affiche "Hook0 CLI Login", "Validating credentials..." avant l'objet JSON, rendant la sortie impossible à parser par `jq`.

**Correction :** Suppression des messages d'info quand le format de sortie est JSON.

```diff
# cli/src/commands/auth.rs

-    output_info("Hook0 CLI Login");
-    output_info("===============");
+    if cli.output != OutputFormat::Json {
+        output_info("Hook0 CLI Login");
+        output_info("===============");
+    }
     ...
-    output_info("Validating credentials...");
+    if cli.output != OutputFormat::Json {
+        output_info("Validating credentials...");
+    }
```

---

## Bug 15 (Moyenne) : `replay -o json` mélange du texte de progression avec le JSON

**Symptôme :** Même problème que le bug 14 — la ligne "Replaying N event(s)..." apparaît avant la sortie JSON.

**Correction :**

```diff
# cli/src/commands/replay.rs

-    println!("Replaying {} event(s)...", events.len());
+    if cli.output != OutputFormat::Json {
+        println!("Replaying {} event(s)...", events.len());
+    }
```

---

## Bug 16 (Moyenne) : `logout -p <profil>` ignore le flag `-p`

**Symptôme :** `logout -p beta` supprime le profil par défaut au lieu de "beta". Le flag `-p` est le global `--profile`, mais logout ne vérifiait que `--profile-name`/`-n`.

**Correction :** Logout respecte maintenant le global `--profile`/`-p` comme fallback.

```diff
# cli/src/commands/auth.rs

     let profile_name = args.profile_name.clone()
+        .or_else(|| cli.profile.clone())
         .or_else(|| config.default_profile.clone())
         .unwrap_or_else(|| "default".to_string());
```

---

## Bug 17 (Basse) : `--enable` et `--disable` acceptés simultanément sur subscription update

**Symptôme :** `subscription update --enable --disable` réussit silencieusement. Les flags devraient être mutuellement exclusifs.

**Correction :**

```diff
# cli/src/commands/subscription.rs

-    #[arg(long)]
+    #[arg(long, conflicts_with = "disable")]
     pub enable: bool,
-    #[arg(long)]
+    #[arg(long, conflicts_with = "enable")]
     pub disable: bool,
```

---

## Bug 18 (Basse) : `--label` non marqué comme requis dans le help

**Symptôme :** `event send` et `subscription create` affichent `--label` comme optionnel dans le help, mais l'API exige au moins un label.

**Correction :**

```diff
# cli/src/commands/event.rs + subscription.rs

-    #[arg(long, short = 'l', value_parser = parse_label)]
+    #[arg(long, short = 'l', required = true, value_parser = parse_label)]
```

---

## Bug 19 (Basse) : `event-type delete -o json` et `subscription delete -o json` sortent du texte brut

**Symptôme :** Les commandes de suppression affichent toujours du texte lisible, ignorant `--output json`.

**Correction :**

```diff
# cli/src/commands/event_type.rs + subscription.rs

-    output_success(&format!("... deleted successfully!"));
+    if cli.output == OutputFormat::Json {
+        println!("{}", serde_json::json!({"deleted": true, ...}));
+    } else {
+        output_success(&format!("... deleted successfully!"));
+    }
```

---

## Bug 20 (Basse) : Les messages d'erreur dupliquent le JSON brut dans les hints NotFound et ValidationError

**Symptôme :** `Error: Resource not found: {JSON brut}` suivi de `Hint: The {JSON brut} was not found.`

**Correction :**

```diff
# cli/src/main.rs

-    hook0_cli::ApiError::NotFound(resource) => {
-        eprintln!("Error: {e}");
-        eprintln!("\nHint: The {resource} was not found. ...");
+    hook0_cli::ApiError::NotFound(_) => {
+        eprintln!("Error: Resource not found. Check the ID and try again.");

-    hook0_cli::ApiError::ValidationError(msg) => {
-        eprintln!("Error: {e}");
-        eprintln!("\nHint: {msg}");
+    hook0_cli::ApiError::ValidationError(_) => {
+        eprintln!("Error: {e}");
```

---

## Bug 21 (Basse) : Le message d'erreur du login dit `--application-secret` au lieu de `--secret`

**Symptôme :** En lançant login sans secret en mode non-interactif, l'erreur dit "Provide via --application-secret" mais le vrai flag est `--secret`.

**Correction :**

```diff
# cli/src/commands/auth.rs

-    prompt_message.to_lowercase().replace(' ', "-"),
+    env_var_name.to_lowercase().trim_start_matches("hook0_"),
```

---

## Bug 22 (Basse) : Un payload JSON invalide donne un message d'erreur cryptique

**Symptôme :** `event send -d "not json"` affiche "expected ident at line 1 column 2" — une erreur serde brute.

**Correction :**

```diff
# cli/src/commands/event.rs

-    serde_json::from_str(&payload_str)?
+    serde_json::from_str(&payload_str)
+        .map_err(|e| anyhow!("Invalid JSON payload: {}", e))?
```

---

## Infrastructure : variable d'environnement `HOOK0_CONFIG_DIR`

Ajoutée pour supporter l'isolation des répertoires de config (critique pour l'exécution parallèle des tests et la CI).

```diff
# cli/src/config/mod.rs

 pub fn config_dir() -> PathBuf {
+    if let Ok(dir) = std::env::var("HOOK0_CONFIG_DIR") {
+        return PathBuf::from(dir);
+    }
     dirs::config_dir().expect("...").join("hook0")
 }
```

---

## Infrastructure : modèle `Event` — champs optionnels

L'API retourne des formes différentes de Event selon l'endpoint (send retourne un objet minimal, list omet certains champs, get retourne l'objet complet). Les champs ont été rendus optionnels pour gérer toutes les variantes.

```diff
# cli/src/api/models.rs

 pub struct Event {
     pub event_id: Uuid,
-    pub application_id: Uuid,
-    pub event_type_name: String,
-    pub payload: String,
-    pub payload_content_type: String,
+    pub application_id: Option<Uuid>,
+    pub event_type_name: Option<String>,
+    pub payload: Option<String>,
+    pub payload_content_type: Option<String>,
     ...
-    pub occurred_at: DateTime<Utc>,
+    pub occurred_at: Option<DateTime<Utc>>,
 }
```

---

## Couverture de tests ajoutée

Nouveau fichier `cli/tests/e2e_api.rs` — 27 tests d'intégration couvrant toutes les commandes API :

- Event-type CRUD (création par nom + composants, liste avec filtres, suppression avec vérification)
- Event send (payload, fichier, labels, ID custom, content-type, validation)
- Event list (tous formats, filtres) + get (détail, JSON, --attempts)
- Subscription cycle de vie complet (create, get, disable, enable, update tous flags, delete)
- Application (get, current, list 403, switch)
- Replay (dry-run, filtres, replay réel)
- Isolation du répertoire de config, multi-profil, logout, mode override
- Gestion d'erreurs (mauvaise URL, mauvais secret)
- Vérification des formats de sortie (structure JSON)
