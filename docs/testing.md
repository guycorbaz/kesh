# Tests — patterns et fixtures

Ce document décrit les deux patterns de tests utilisés dans Kesh (Rust intégration et Playwright E2E), la plomberie des fixtures déterministes introduite par la Story 6.4, et les prérequis pour lancer chaque suite en local.

## Vue d'ensemble

| Niveau | Framework | Fixtures | Localisation |
|---|---|---|---|
| Unitaires Rust | `cargo test` | aucunes (logique pure) | `crates/*/src/**/tests` |
| Intégration Rust | `cargo test` + `sqlx::test` | `kesh_db::test_fixtures::seed_accounting_company` | `crates/kesh-api/tests/*_e2e.rs` |
| Unitaires frontend | Vitest | mocks | `frontend/tests/**/*.test.ts` |
| E2E Playwright | `@playwright/test` | endpoint `POST /api/v1/_test/seed` via `seedTestState` | `frontend/tests/e2e/*.spec.ts` |

La Story 6.4 a unifié deux patterns disparates (bypass SQL ad-hoc en Rust, absence totale de reset DB en Playwright) en une seule couche de fixtures partagée par les deux.

## Base de dev jetable — MariaDB en RAM

Depuis la Story 22-5 (issue #251), `docker-compose.dev.yml` monte `/var/lib/mysql` en **tmpfs de 4 Go** et relâche la durabilité (`--innodb_flush_log_at_trx_commit=0 --sync_binlog=0 --innodb-doublewrite=0`). Les bases éphémères de `#[sqlx::test]` ne touchent donc plus le disque.

⚠️ **Rien ne survit au redémarrage du conteneur** — ni la base `kesh`, ni `kesh_e2e`, ni les tables système. C'est délibéré, et cela a trois conséquences qu'il vaut mieux connaître avant de les découvrir :

**1. Les droits et `kesh_e2e` se rejouent tout seuls.** `scripts/mariadb-init/01-dev-grants.sql` est monté dans `/docker-entrypoint-initdb.d/`, que l'entrypoint MariaDB exécute dès que le datadir est vide — donc à chaque démarrage ici. Il rend à l'utilisateur `kesh` les droits globaux dont `#[sqlx::test]` a besoin pour créer ses bases éphémères, et recrée `kesh_e2e`. Sans ce fichier, **toute** la suite d'intégration échouerait au premier `CREATE DATABASE`.

**2. Le seed de la base partagée, lui, se rejoue à la main.** Les 154 tests `kesh-db::repositories::*` travaillent sur la base `kesh` et attendent 1 société, 1 admin, 1 exercice ouvert et ≥ 2 comptes. Après un redémarrage :

```sh
export DATABASE_URL='mysql://kesh:kesh_dev@127.0.0.1:3306/kesh'
sqlx migrate run --source crates/kesh-db/migrations
docker exec -i kesh-mariadb-dev mariadb -uroot -pkesh_dev_root kesh < scripts/seed-dev-db.sql
```

Passer par `sqlx migrate run` et non par une boucle de `mariadb <` sur les fichiers : c'est ce qui remplit `_sqlx_migrations`, sans quoi un `cargo run -p kesh-api` sur cette base tenterait de tout ré-appliquer et refuserait de démarrer.

**L'oubli du seed est bruyant, pas silencieux** : les tests concernés s'ouvrent sur `expect("need at least one company in DB for tests")`. Aucun faux vert possible — c'est ce qui rend la procédure manuelle acceptable.

⚠️ **La base partagée doit être remise à zéro avant CHAQUE gate complet, pas seulement après un gate interrompu** — c'est la **KF-039 ([#310](https://github.com/guycorbaz/kesh/issues/310))**, et le `CLAUDE.md` § *« Un gate laisse la base piégée »* en porte la règle. Le cas du run tué en vol y était déjà décrit ; il en existe un second, plus discret : **un run qui se termine normalement laisse lui aussi la base inutilisable pour le suivant.** Les tests de dépôt y créent des factures liées à des écritures comptables, et le `delete_all_by_company` du montage de `journal_entries::tests` échoue alors sur `fk_invoices_journal_entry` — **34 tests tombent d'un coup**, en 7 ms chacun, sur un module que la branche en cours ne touche pas.

Le geste, avant tout gate complet :

```sh
docker compose -f docker-compose.dev.yml restart mariadb   # le tmpfs efface tout
sqlx migrate run --source crates/kesh-db/migrations
docker exec -i kesh-mariadb-dev mariadb -uroot -pkesh_dev_root kesh < scripts/seed-dev-db.sql
```

Depuis le passage en tmpfs, ce cycle coûte une quinzaine de secondes — c'est ce qui le rend praticable systématiquement, là où il fallait auparavant vider les tables à la main.

Le seed est **idempotent** et se rejoue sans dommage. Il suppose en revanche qu'il possède la base : il cible sa propre société (« CI Seed Company ») par son nom, et **saute la création de l'`admin` si un utilisateur de ce nom existe déjà** ailleurs — `username` porte une contrainte d'unicité globale. Sur une base où vous avez saisi vos propres données, relisez `scripts/seed-dev-db.sql` avant de le lancer.

**3. Les bases éphémères orphelines s'effacent au redémarrage.** Un run interrompu (Ctrl-C, OOM, timeout) laisse derrière lui des `_sqlx_test_database_*` que sqlx n'a pas nettoyées — **et un test ROUGE aussi** : sqlx ne détruit la base que d'un test vert. Sur disque elles s'accumulaient ; en tmpfs, un `docker compose -f docker-compose.dev.yml restart mariadb` les balaie toutes. Pour les compter sans redémarrer :

```sh
docker exec kesh-mariadb-dev mariadb -uroot -pkesh_dev_root \
  -e "SELECT COUNT(*) FROM information_schema.schemata WHERE schema_name LIKE '\_sqlx\_test%';"
```

C'est aussi le geste qui répond à la § *« Un gate laisse la base piégée — et pas seulement quand il est interrompu »* du `CLAUDE.md` : redémarrer le conteneur remet la base de gate à zéro, il ne reste qu'à rejouer migrations et seed.

**4. Un run massivement rouge peut SATURER le tmpfs.** À ~17 Mo la base éphémère et ~3,3 Go libres, il en tient environ **190**. Un squash cassé fait échouer les 1102 tests basculés d'un coup : passé la 190ᵉ base non détruite, MariaDB rend `table is full` sur toutes les suivantes. **Les premiers échecs portent la cause réelle, ceux d'après ne portent que la saturation** — c'est le début du rapport qu'il faut lire, pas la fin. La reprise est un `restart` du conteneur, puis la procédure du point 2.

**5. Le conteneur `kesh` ne survit pas utilement à un redémarrage de MariaDB.** `depends_on` ne gouverne que le démarrage : après un `restart` de `mariadb`, le service `kesh` — toujours vivant sous `restart: unless-stopped` — pointe sur une base repartie **vide**, et il n'applique ses migrations qu'au boot. Chaque requête échoue, et rien ne le rattrape. Redémarrez-le explicitement (`docker compose -f docker-compose.dev.yml restart kesh`) après avoir rejoué migrations et seed. *(En développement quotidien la question ne se pose pas : le backend tourne en `cargo run`, pas dans ce conteneur.)*

**La production n'est pas concernée** : `docker-compose.yml` garde son volume persistant, sa durabilité par défaut, et ne monte pas `scripts/mariadb-init/`. *(`docker-compose.prod.yml`, lui, ne déclare aucun service MariaDB — il pointe une base externe.)*

## Pattern Rust : `seed_accounting_company`

Chaque test intégration backend démarre d'une DB éphémère (fournie par `#[sqlx::test(migrations = "../kesh-db/test-schema")]` — le **squash** du schéma, cf. Story 22-5 / issue #251 ; la graphie est `"./test-schema"` depuis `kesh-db` lui-même) puis seede l'état comptable via le helper :

```rust
use kesh_db::test_fixtures::seed_accounting_company;
use kesh_db::repositories::invoices;

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn my_invoice_flow(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    // seeded : company_id, fiscal_year_id, admin_user_id, changeme_user_id,
    //          accounts: HashMap<"1000"|"1100"|"2000"|"3000"|"4000", i64>

    let (invoice, _) = invoices::create(&pool, seeded.admin_user_id, ...).await.unwrap();
    // Valider via le flow normal (pas d'UPDATE SQL direct — KF-001 closed).
    invoices::validate_invoice(&pool, seeded.company_id, invoice.id, seeded.admin_user_id)
        .await.expect("validate");
}
```

**Ce que le helper crée** :

- 1 `companies` `'CI Test Company'`, org_type `Independant`, langues FR/FR, adresse 2 lignes (`'Test Address 1\n1000 Lausanne'` — QR Bill exige line1/line2).
- 2 `users` Admin actifs : `admin/admin123` et `changeme/changeme` (hashes Argon2id pré-calculés dans `ADMIN_PASSWORD_HASH` / `CHANGEME_PASSWORD_HASH`).
- 1 `fiscal_years` 2020-2030 `Open`.
- 5 `accounts` minimaux : 1000 Caisse (Asset), 1100 Banque (Asset), 2000 Capital (Liability), 3000 Ventes (Revenue), 4000 Charges (Expense).
- 1 `company_invoice_settings` avec `default_receivable_account_id` = 1100, `default_revenue_account_id` = 3000, `default_sales_journal` = `Ventes`.

Les helpers associés (`truncate_all`, `seed_changeme_user_only`, `mark_onboarding_complete`, `seed_contact_and_product`) exposent les briques utilisées par l'endpoint runtime ci-dessous.

Lancer la suite :

```bash
# DB MariaDB de dev démarrée (docker compose -f docker-compose.dev.yml up -d mariadb)
DATABASE_URL="mysql://root:kesh_dev_root@127.0.0.1:3306/kesh" cargo test -p kesh-api --tests
```

## Pattern Playwright : `seedTestState` via endpoint gated

Playwright partage une seule DB MariaDB entre toutes les specs — pas d'équivalent de `sqlx::test` éphémère per-test. La solution Story 6.4 : un endpoint **runtime** `POST /api/v1/_test/seed` qui truncate la DB puis re-seed, exposé **uniquement si `KESH_TEST_MODE=true`** dans l'env du backend.

```ts
// frontend/tests/e2e/homepage-settings.spec.ts
import { test, expect } from '@playwright/test';
import { seedTestState } from './helpers/test-state';

test.beforeAll(async () => {
  await seedTestState('with-company');
});

test('homepage affiche la company seedée', async ({ page }) => {
  await page.goto('/login');
  await page.fill('#username', 'admin');
  await page.fill('#password', 'admin123');
  await page.click('button[type="submit"]');
  await expect(page).toHaveURL('/');
});
```

### Les 4 presets

| Preset | Contenu | Usage typique |
|---|---|---|
| `fresh` | DB vidée, user `changeme/changeme` seul | `onboarding.spec.ts`, `onboarding-path-b.spec.ts` |
| `post-onboarding` | fresh + `admin/admin123` + company + fiscal_year + 5 accounts + company_invoice_settings + `onboarding_state.step_completed = 10` | alias sémantique de `with-company` |
| `with-company` | **identique** à `post-onboarding` (même code path) | toutes les specs admin (`auth`, `accounts`, `contacts`, `products`, `invoices`, `journal-entries`, `users`, `homepage-settings`, `mode-expert`) |
| `with-data` | `with-company` + 1 contact `'CI Contact SA'` + 1 product `'CI Product'` | `invoices_echeancier.spec.ts` (les factures sont créées dynamiquement par le spec — **pas de facture pré-seedée**) |

### Choix `beforeAll` vs `beforeEach`

| Cas | Hook | Raison |
|---|---|---|
| State singleton muté par chaque test (ex: `onboarding_state.step_completed`) | `beforeEach` | Chaque test doit re-partir de zéro, la mutation est irréversible dans le run. |
| Mutations scopées à des rows individuelles (contact, product, etc.) | `beforeAll` | Les tests utilisent des suffixes uniques (`Date.now()`) — pas de collision. |

## Sécurité : le gate `KESH_TEST_MODE`

L'endpoint `/api/v1/_test/*` est une porte ouverte sur la DB complète (truncate + re-seed). Sa sécurité repose sur **trois couches** :

1. **Gate runtime dans `build_router`** : les routes ne sont montées que si `config.test_mode == true`. Une requête POST vers `/api/v1/_test/seed` avec `test_mode=false` retombe sur le fallback `ServeDir` → `404 Not Found` ou `405 Method Not Allowed` (jamais 200). Vérifié par les tests `test_endpoints_e2e::seed_endpoint_not_available_when_test_mode_off`.
2. **Refus de démarrage si bind non-loopback** : `Config::from_env()` retourne `ConfigError::TestModeWithPublicBind` si `KESH_TEST_MODE=true` **et** `KESH_HOST ∉ {127.0.0.1, ::1, localhost}`. Le binaire exit 1 avec un message explicite avant même d'écouter.
3. **`0.0.0.0` explicitement rejeté** : pas accepté comme alias loopback. Raison : en Docker avec `-p 80:80`, un container qui bind `0.0.0.0` en interne expose le port sur le réseau hôte — l'endpoint `/api/v1/_test/*` deviendrait accessible publiquement. La CI et `docker-compose.dev.yml` **doivent** utiliser `KESH_HOST=127.0.0.1` quand `KESH_TEST_MODE=true`.

Le défaut applicatif de `KESH_HOST` est passé de `0.0.0.0` à `127.0.0.1` (Story 6.4 T7.6) — sécurité par défaut, opt-in explicite pour bind public en prod.

## Prérequis Playwright local

⚠️ **`playwright.config.ts` n'a PAS de `webServer`** — rien n'est démarré automatiquement, ni le backend ni le frontend. Playwright tape directement le backend, qui sert la SPA buildée via `KESH_STATIC_DIR` ; `baseURL` vaut `process.env.KESH_BACKEND_URL ?? 'http://127.0.0.1'`.

*(Cette section décrivait jusqu'au 2026-08-12 un `webServer` lançant `npm run preview` sur `:4173`. Il n'existe pas — `grep -c webServer frontend/playwright.config.ts` rend **0**. Une recette fausse coûte plus cher qu'une recette absente : on la suit, elle échoue, et on cherche la panne ailleurs.)*

**Recette minimale** (deux terminaux) :

```bash
# Terminal 1 : DB + frontend buildé + backend qui sert les deux
docker compose -f docker-compose.dev.yml up -d mariadb
cd /path/to/kesh
(cd frontend && npm run build)          # KESH_STATIC_DIR pointe sur le résultat
mkdir -p /tmp/kesh-e2e/inbox /tmp/kesh-e2e/documents
KESH_TEST_MODE=true KESH_HOST=127.0.0.1 KESH_COOKIE_SECURE=false \
  KESH_PORT=3000 KESH_STATIC_DIR="$PWD/frontend/build" \
  KESH_INBOX_DIR=/tmp/kesh-e2e/inbox KESH_DOCUMENTS_DIR=/tmp/kesh-e2e/documents \
  KESH_ADMIN_USERNAME=admin KESH_ADMIN_PASSWORD=e2e-admin-password-12chars \
  KESH_SMTP_HOST=smtp.invalid KESH_SMTP_USER=e2e \
  KESH_SMTP_PASSWORD=e2e KESH_SMTP_FROM=kesh@example.invalid \
  DATABASE_URL="mysql://kesh:kesh_dev@127.0.0.1:3306/kesh_e2e" \
  KESH_JWT_SECRET="dev-secret-at-least-32-bytes-long-for-testing" \
  cargo run -p kesh-api

# Terminal 2 : Playwright
cd frontend
KESH_TEST_MODE=true PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 \
  KESH_BACKEND_URL=http://127.0.0.1:3000 npm run test:e2e
```

⚠️ **Les deux ajouts ci-dessus manquaient à cette recette, et chacun produit des
échecs qui ne ressemblent pas à un défaut de montage.**

- **Les quatre `KESH_SMTP_*` côté backend** : sans elles, `/health.smtpConfigured`
  est `false`, `GET /_test/sent-emails` rend **400**, et les specs d'e-mail
  échouent — puis *entraînent* des cascades de timeouts sur la page de login qui
  n'ont plus rien à voir avec la cause. Mesuré le 2026-08-24 : **44 échecs sans
  ces variables, 16 avec**. Les valeurs sont factices ; en `KESH_TEST_MODE` le
  boot substitue un `MockMailer` capturant (aucun envoi réel).
- **`KESH_TEST_MODE=true` côté RUNNER** (et pas seulement côté backend) :
  `xss-token-protection.spec.ts` lit `process.env.KESH_TEST_MODE` **dans
  Playwright** pour savoir s'il doit exiger `Secure` sur le cookie d'accès. Sans
  la variable, il l'exige alors que `KESH_COOKIE_SECURE=false` — obligatoire en
  HTTP local — l'interdit. **Éprouvé par mutation** : sans elle `1 failed / 2
  passed`, avec elle `3 passed`.

⚠️ **`KESH_INBOX_DIR` et `KESH_DOCUMENTS_DIR` ne sont pas facultatifs non plus, et leur absence
ne ressemble PAS à un problème de configuration.** Leurs défauts sont `/data/inbox` et
`/data/documents` — des chemins de volume Docker, inaccessibles sur un poste de dev. Le backend
démarre quand même, `/health` répond `ok`, et l'échec ne se voit qu'à l'exécution :
`internal: inbox import: racine inbox: Permission denied (os error 13)` dans le log du backend, et
côté Playwright un simple `getByTestId('inbox-import-report')` introuvable. On cherche alors la panne
dans le frontend, qui n'y est pour rien.

Sans ces deux variables, `inbox-import.spec.ts` rend **1 échec et 2 tests skippés** ; avec elles,
**3 passés** — le round-trip complet de l'import de factures fournisseurs. Les deux se corrigent
ensemble : `documents` n'apparaît qu'une fois `inbox` réglé, chaque erreur masquant la suivante.

*(Relevé le 2026-08-20 au gate E2E de la story 23-3. La recette ci-dessus était incomplète depuis
qu'elle existe, et l'écart se lisait comme une régression de branche : 182 passés contre les 183
déclarés à l'implémentation.)*

**Le build du frontend n'est pas facultatif** : sans lui, `KESH_STATIC_DIR` pointe sur un répertoire vide et chaque navigation rend un 404 que rien n'explique.

⚠️ **La suite E2E ne s'exécute QU'EN FRANÇAIS, et c'est un angle mort du dispositif.** Le seed
crée une société en `langues FR/FR` (cf. § *Seed CI* ci-dessus) et rien ne rejoue la suite dans une
autre locale. **Conséquence directe** : un sélecteur figé sur un libellé traduit reste vert tant
que le français ne change pas — et ne casse qu'en production, dans la langue de l'utilisateur.

Le cas s'est présenté, et il est instructif : `has-text("Administration")` (5 occurrences dans
3 fichiers) ciblait le groupe de navigation par son texte. Le libellé est **identique en français,
en allemand et en anglais** ; il ne diffère qu'en **italien** (`Amministrazione`). Ce sélecteur
serait donc resté vert dans trois langues sur quatre, et personne n'aurait rien vu. Il a été trouvé
par un **grep du symptôme**, pas par la suite — celle-ci en était structurellement incapable.

**Règle qui en découle** : un sélecteur E2E ne se fige jamais sur un libellé traduit, `data-testid`
sans exception. Ne pas compter sur la suite pour le rattraper : elle ne le peut pas.

⚠️ **Cette règle est une DISCIPLINE, donc invérifiable en l'état** — c'est la faiblesse que ce
dépôt documente sous « on peut l'affirmer sans l'avoir fait ». **[KF-043 (#326)](https://github.com/guycorbaz/kesh/issues/326)**
tient le sujet ouvert et pose les trois options : ne rien faire, une **garde statique** qui refuse
un sélecteur français dans `tests/e2e/` (même patron que `i18n-libelle-en-dur.test.ts`, qui lit les
sources), ou un run dans une seconde locale. L'arbitrage revient au Project Lead.

*(Écrit le 2026-08-21, passe 3 de revue de la story 23-3b. ⚠️ Une passe antérieure avait neutralisé
ce constat en affirmant que « `CLAUDE.md` le documente comme un angle mort connu » — **il ne le
documentait pas**, et l'angle mort n'était écrit nulle part. Une réfutation qui s'appuie sur une
source inexistante enterre le finding qu'elle prétend traiter.)*

⚠️ **La suite locale n'est pas verte, et ce n'est pas une régression** : des tests échouent aussi sur `main`. **Mais leur nombre était très surestimé, et ça a coûté cher.** Ce paragraphe annonçait « une quarantaine » ; recompté le 2026-08-24 **avec le montage complet ci-dessus**, `main` en rend **15** — et v0.11.0, avant les correctifs de [#107], en rendait **32**. L'écart venait des variables manquantes de la recette : sans elles, on mesure **44**. ⚠️ **Un bruit de fond surestimé n'est pas un détail de documentation : il rend invisibles les défauts qu'il recouvre.** Les six échecs des KF-047 à KF-050 dormaient sous ce chiffre, dont une bannière de mode dégradé qui ne s'affiche peut-être plus quand la base tombe. **Seul le différentiel branche ↔ `main` se lit** — cf. la mémoire projet `comparer-suite-e2e-branche-vs-main` pour le montage à deux ports et deux bases.

> **`KESH_COOKIE_SECURE=false` est obligatoire en local HTTP** : depuis les
> tokens httpOnly (Story 10-5), les cookies d'auth sont `Secure` par défaut.
> Le navigateur Chromium tolère les cookies Secure sur `127.0.0.1` (loopback),
> mais le **request context** Playwright (`authedApiContext`) ne les envoie
> PAS sur `http://` → tous les tests utilisant l'API échouent en 401.

Le `globalSetup` Playwright (`tests/e2e/global-setup.ts`) appelle `seedTestState('with-company')` une seule fois avant tous les workers — si le backend est éteint ou `KESH_TEST_MODE=false`, il throw avec un message listant les 4 prérequis (backend up, `KESH_TEST_MODE`, `KESH_HOST` loopback, `KESH_BACKEND_URL`).

**Variante recovery de mot de passe** (spec `password-recovery.spec.ts`, Story 17-4e) : la spec comporte **5 scénarios E2E Playwright** et exige un backend **feature-on** — sinon ils sont *skipped* (comportement voulu, message explicite) ; et elle **échoue** franchement si le backend est injoignable (pas de faux vert). Recette :

```bash
KESH_TEST_MODE=true KESH_HOST=127.0.0.1 KESH_PORT=8181 \
  KESH_ADMIN_USERNAME=admin KESH_ADMIN_PASSWORD=e2e-admin-password-12chars \
  KESH_FEATURE_FORGOT_PASSWORD=true KESH_PUBLIC_BASE_URL=http://127.0.0.1:8181 \
  KESH_SMTP_HOST=127.0.0.1 KESH_SMTP_PORT=2525 KESH_SMTP_USER=e2e \
  KESH_SMTP_PASSWORD=e2e KESH_SMTP_FROM=kesh@example.invalid \
  DATABASE_URL="mysql://root:kesh_dev_root@127.0.0.1:3306/kesh" \
  KESH_JWT_SECRET="dev-secret-at-least-32-bytes-long-for-testing" \
  cargo run -p kesh-api
# puis : KESH_BACKEND_URL=http://127.0.0.1:8181 npm run test:e2e -- password-recovery
```

Pièges vérifiés sur pièces (17-4e) : la var est `KESH_SMTP_USER` (pas `USERNAME`) ; `KESH_ADMIN_PASSWORD` doit faire ≥ 12 caractères même en test-mode ; choisir un `KESH_PORT` libre (80 demande des privilèges, 8080 est souvent pris). Les valeurs SMTP sont **factices** : le fail-fast boot exige une config complète ; depuis la Story 20-4, en `KESH_TEST_MODE` + SMTP configuré le boot substitue un **MockMailer capturant** (aucun envoi réel, aucune erreur en tâche de fond) — le token de reset est **injecté** via l'endpoint test-mode `POST /api/v1/_test/password-reset-token` `{ "username": … }` → `{ "token": … }`, puis consommé en ouvrant `/reset-password?token=<valeur>` dans le navigateur (ou directement `POST /api/v1/auth/reset-password` `{ "token": …, "newPassword": … }`). Chaque `POST /_test/seed` purge aussi les rate-limiters (login + recovery + **send-email**, Story 20-4) et le **buffer de capture d'e-mails** pour éviter le 429 et les fuites d'état inter-specs. Côté backend Rust, les 14 tests d'intégration `password_recovery_e2e.rs` couvrent les flux complets avec `MockMailer` (aucun SMTP réel en CI).

**E2E envoi de factures par e-mail** (specs `invoice-send-email.spec.ts` + `invoice-send-email-nosmtp.spec.ts`, Story 20-4) : **deux runs séquentiels** avec deux configurations backend opposées.

*Run 1 — round-trip avec capture* (spec principal, 4 tests) : backend `KESH_TEST_MODE=true` **avec vars SMTP factices** → le boot substitue un `MockMailer` au transport réel (log `MockMailer actif`) et expose `GET /api/v1/_test/sent-emails` (non authentifié, comme les autres endpoints `_test`) qui renvoie `{ emails: [{ to, subject, body, fromDisplayName, replyTo, attachmentFilename, attachmentContentType, attachmentSize }] }` :

```bash
KESH_TEST_MODE=true KESH_HOST=127.0.0.1 KESH_PORT=8181 \
  KESH_ADMIN_USERNAME=admin KESH_ADMIN_PASSWORD=e2e-admin-password-12chars \
  KESH_SMTP_HOST=smtp.invalid KESH_SMTP_USER=e2e \
  KESH_SMTP_PASSWORD=e2e KESH_SMTP_FROM=kesh@example.invalid \
  KESH_STATIC_DIR=../frontend/build KESH_COOKIE_SECURE=false \
  DATABASE_URL="mysql://kesh:kesh_dev@127.0.0.1:3306/kesh_e2e" \
  KESH_JWT_SECRET="dev-secret-at-least-32-bytes-long-for-testing" \
  cargo run -p kesh-api
# puis : KESH_BACKEND_URL=http://127.0.0.1:8181 npm run test:e2e -- invoice-send-email.spec
```

*Run 2 — fallback zéro-config* (1 test, gaté `KESH_E2E_NO_SMTP=1` sinon *skipped*) : **même commande sans les 4 vars `KESH_SMTP_*`** (→ `/health.smtpConfigured=false`, bouton grisé + tooltip), puis :

```bash
KESH_BACKEND_URL=http://127.0.0.1:8181 KESH_E2E_NO_SMTP=1 \
  npm run test:e2e -- invoice-send-email-nosmtp.spec
```

Le test de fallback vérifie lui-même `/health` et échoue avec un diagnostic si le backend a été lancé avec SMTP (recette inversée) ; symétriquement, le spec principal échoue franchement sur `GET /_test/sent-emails` si le backend n'est pas en mode capture. Rappels : `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` (Ubuntu 26.04+), DB `kesh_e2e`, et **jamais de pipe sur le runner** (`> log 2>&1; echo EXIT=$?` — un pipe masque l'exit code et tronque la liste des échecs).

Pour surcharger l'URL backend (ex: tests contre un kesh-api distant) :

```bash
KESH_BACKEND_URL=http://localhost:3001 npm run test:e2e
```

## Cleanup entre tests (dette technique acceptée)

- **Pas de reset entre tests individuels d'une même spec** (dette `D-6-4-A`). Si un test pollue (création + archivage incomplet), le test suivant peut être affecté. Mitigation : convention de cleanup explicite dans chaque test, ou adoption progressive de `test.beforeEach(seedTestState(...))` si symptômes apparaissent.
- **Pas de tests d'intégration de l'endpoint depuis le helper TypeScript** (dette `D-6-4-B`). Chicken-and-egg : le helper teste lui-même. La couverture Rust (`test_endpoints_e2e`) + le `globalSetup` Playwright suffisent pour détecter une régression.

## Références

- Story 6.4 : `_bmad-output/implementation-artifacts/6-4-fixtures-e2e-deterministes.md`
- Helper Rust : `crates/kesh-db/src/test_fixtures.rs`
- Endpoint runtime : `crates/kesh-api/src/routes/test_endpoints.rs`
- Helper Playwright : `frontend/tests/e2e/helpers/test-state.ts`
- Tests d'intégration de l'endpoint : `crates/kesh-api/tests/test_endpoints_e2e.rs`
- CI : `.github/workflows/ci.yml` (job `e2e`, step `Smoke test /api/v1/_test/seed`)
