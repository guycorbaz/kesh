# Story v0172-correctifs : la 0.17.2 — la session (#876), l'import simultané (#840), la liste des factures fournisseur (#870), le menu (#851), le taux de TVA désactivé (#882), les dates (#880), six LOW (#883), la date d'une pièce importée (#885)

Status: ready-for-dev

*Fiche rédigée le 2026-11-04, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-0172b`, branche
`story/v0172-correctifs` partie de `d7ec2ea2` (le merge de la v0.17.1 sur `main`). Release **0.17.2**, **sans migration**
(§ *Contrainte*). Liste acceptée par le Project Lead le 2026-11-04. Registre : `v0172-choix-autonomes.md`, choix
`C-v0172b-1` à `C-v0172b-22`. Clé courte : `v0172b` (port 3173, bases `kesh_v0172b` / `kesh_e2e_v0172b`). Issues : **#876**,
**#840** (bugs, P2), **#870**, **#851**, **#882** (bugs, P3), **#880**, **#883** (bugs, P4), **#885** (demande), en `refs`
seulement dans les commits ; la fermeture se porte sur la PR de release. Les numéros de ligne sont ceux de `d7ec2ea2`.
Une autre story de la 0.17.2 (`v0172-annulees`, #868) travaille en parallèle sur une autre branche : CHANGELOG et registre
se fusionnent **par union** à l'intégration.*

## Découpage — trois sous-stories dans cette fiche (règle de splitting préventif)

Le périmètre touche plus de cinq modules (`kesh-api/routes/auth`, `kesh-api/routes/bank_imports`, `kesh-api/qr_decode`,
`frontend/lib/shared/utils` + `lib/app/stores`, `routes/(app)/+layout`, `lib/components/invoices`,
`lib/features/supplier-invoices`, `lib/features/fiscal-years`, et une douzaine d'écrans pour les dates). Il est découpé en
**trois sous-stories**, posées dans cette même fiche (C-v0172b-1), développées **dans l'ordre A → B → C** sur la même
branche, chacune close par ses propres commits, son gate et sa revue de code avant la suivante :

| Sous-story | Issues | Modules | Nature |
|---|---|---|---|
| **A — la session et l'import simultané** | #876 (P2), #840 (P2) | `routes/auth.rs`, `routes/setup.rs`, `routes/bank_imports.rs`, `lib/shared/utils/api-client.ts`, `lib/app/stores/auth.svelte.ts` (+ un module neuf `session-renewal.ts`) | comportement, concurrence |
| **B — les écrans de saisie et de liste** | #870 (P3), #851 (P3), #882 (P3), #885 (CR) | `routes/(app)/supplier-invoices/+page.svelte`, `lib/features/fiscal-years/FiscalYearChecks.svelte`, `routes/(app)/+layout.svelte`, `lib/components/invoices/InvoiceForm.svelte`, `lib/features/supplier-invoices/SupplierInvoiceForm.svelte` | écrans |
| **C — la forme** | #880 (P4), #883 (P4) | `reports.api.ts` (helper), onze écrans comptables et la page des clés API, `qr_decode.rs`, `inbox_import_e2e.rs` | déploiement mécanique d'un motif + tests |

Chaque sous-story touche au plus cinq modules ; C est un *rollout* mécanique (un helper, puis son application site par
site), revu fichier par fichier. **Ordre** : A d'abord (les deux P2), C en dernier (#883.6 change `formatSwissDate`, dont
dépend #880 ; aucun des sites de B n'en dépend).

## Story

En tant que Comptable qui saisit une facture pendant vingt minutes, je ne veux pas être renvoyé à la connexion au moment
d'enregistrer ; en tant que Comptable qui importe un relevé, je ne veux pas qu'un double clic l'enregistre deux fois ; en
tant que Comptable qui clôture, je veux atteindre chaque brouillon de facture fournisseur que le contrôle de clôture me
nomme ; en tant qu'Administrateur, je veux que le menu garde ses entrées ; en tant que Comptable, je veux pouvoir corriger
un brouillon de facture dont un taux de TVA a été désactivé, lire les dates des écrans comptables en forme suisse, et partir
de la date du jour — signalée comme telle — sur une facture fournisseur créée depuis un document.

## Contrainte — production et migrations

- **Kesh 0.17.1 tient des livres réels sur le NAS du Project Lead** (`kesh-gate-logs/prompts/v017-donnees-a-garder.md`) :
  aucun essai de cette story ne se joue sur le NAS ni sur des données réelles ; fixtures synthétiques, bases `#[sqlx::test]`
  jetables, base de démonstration jetable `kesh_v0172b` pour les captures et la recette de session.
- **Aucune migration** (C-v0172b-21) : `ls crates/kesh-db/migrations/*.sql | wc -l` vaut **86** avant et après. #840 se
  corrige par un **verrou nommé**, non par une contrainte (§ A.2, C-v0172b-6) ; #876 ajoute un **champ JSON** aux réponses
  d'authentification, sans schéma neuf. Aucune donnée existante n'est lue autrement, aucune n'est réécrite.
  `kesh_version_min_required` reste à 0.17.0 ; **le retour à la 0.17.1 reste possible sans restauration** (le binaire 0.17.1
  ignore le champ JSON neuf, et aucune ligne de `_sqlx_migrations` n'est ajoutée). Pas de P5, P6, P7.
- **Hors périmètre** (release, faits par l'orchestrateur) : bump de version Cargo, macros des manuels (`kesh-style.sty`),
  tag. Les PDF des manuels sont régénérés par le développeur pour les seuls textes modifiés ici.

---

# Sous-story A — la session (#876) et l'import simultané (#840)

## A.1 — #876 : la session expire pendant une activité continue

### Ce que le code dit (établi au code, d7ec2ea2)

La prémisse de l'issue (« à vérifier ») est **confirmée, et le défaut est plus net que l'issue ne le dit** : avec les
réglages par défaut, **le renouvellement sur 401 ne peut jamais réussir**, si bien qu'une session dure ≈ 15 minutes après
la connexion, **quelle que soit l'activité**.

1. **Deux durées de 15 minutes, comptées du même instant.** À la connexion, le jeton d'accès vaut
   `state.config.jwt_expiry` (`routes/auth.rs:244-250`, défaut 15 min, `config.rs:1633`) et le jeton de renouvellement
   reçoit `expires_at = now + refresh_inactivity` (`routes/auth.rs:254`, défaut 15 min, `config.rs:755`, `:1635`). Même
   règle au renouvellement (`:422`), au changement de mot de passe (`:524`) et à l'assistant (`routes/setup.rs:288`).
   `.env.example:129, :160` et les deux `docker-compose*.yml` posent les mêmes 15/15.
2. **Le cookie d'accès disparaît à 15 minutes.** `build_auth_cookies` (`routes/auth.rs:44-`) pose `kesh_access_token` avec
   `max_age = jwt_expiry` (`:54`) : le navigateur le supprime à l'échéance ; la requête suivante part sans jeton et reçoit
   401 (le `leeway = 60` de `auth/jwt.rs:91` n'y change rien, le cookie n'est plus envoyé — et pour un en-tête `Bearer`, il
   ne fait que repousser le 401 d'une minute).
3. **Le frontend ne renouvelle qu'après ce 401.** `request` (`lib/shared/utils/api-client.ts:452-460`) et `requestRaw`
   (`:551-`) appellent `refreshTokens()` sur 401 ; **aucun** renouvellement préalable n'existe : `git grep -nE
   '\.expiresIn\b' -- frontend/src ':!*.test.ts'` ne rend que des affectations (`auth.svelte.ts:117, :259`,
   `api-client.ts:115-170`, `setup.api.ts:53`, `login/+page.svelte:48`) — `_expiresIn` est stocké et jamais lu pour
   planifier quoi que ce soit (le commentaire « refresh proactif » de `auth.svelte.ts:129` et `api-client.ts:146` décrit un
   mécanisme qui n'existe pas).
4. **Le renouvellement arrive trop tard.** `refresh` refuse un jeton dont `expires_at < now` (`routes/auth.rs:399`). Or le
   401 survient à `t₀ + jwt_expiry` (cookie supprimé), et le jeton de renouvellement a expiré à `t₀ + refresh_inactivity`
   (+ quelques millisecondes) : avec 15 = 15, le `POST /auth/refresh` arrive **après** son expiration ⇒ `InvalidRefreshToken
   ("token expired")` ⇒ `doRefresh` fait `clearSession()` puis `window.location.replace('/login?reason=session_expired')`
   (`api-client.ts:176-177`) — **la saisie en cours est perdue**. Toute configuration où `refresh_inactivity ≤ jwt_expiry`
   produit la même chose ; seule `refresh_inactivity > jwt_expiry` laisse une fenêtre.
5. **Le manuel promet le contraire** : `docs/manual/fr/user-manual.tex:95` — « La session reste active 15 minutes, prolongée
   automatiquement à chaque action grâce au système de \emph{refresh token}. Après 15 minutes sans activité, vous devrez
   vous reconnecter ». Le manuel d'administration (`admin-manual.tex:779`) décrit `KESH_REFRESH_INACTIVITY_MINUTES` comme
   une inactivité « sliding » : elle ne glisse qu'à un renouvellement, qui n'a jamais lieu.
6. **Ce qui sonde le serveur sans l'utilisateur** : `BackupScheduleBanner.svelte:34` (`setInterval`, Administrateur) et
   `api-health.svelte.ts:102` (sonde `/health` en mode dégradé). Une règle « toute requête prolonge la session » rendrait
   éternelle une session laissée ouverte sur un écran d'administration : **les requêtes ne sont pas une activité**
   (C-v0172b-3).
7. **Plusieurs onglets partagent les cookies**, et la rotation détecte le rejeu : un jeton de renouvellement déjà tourné
   (`revoked_reason = "rotation"`) présenté de nouveau révoque **toutes** les sessions de l'utilisateur
   (`routes/auth.rs:377-393`). Deux onglets qui renouvellent au même instant avec le même cookie déconnectent tout. Ce
   risque existe déjà sur le chemin 401 — il ne se manifestait pas, faute de renouvellement qui réussisse ; **le correctif
   le rend réel** et doit le fermer (C-v0172b-4).

### Conception retenue (C-v0172b-2 à 5)

- **Serveur** — un champ additif `sessionExpiresIn` (secondes avant l'expiration du jeton de **renouvellement**) dans
  `LoginResponse` (`routes/auth.rs:135`, rendu par `login` et par l'assistant `routes/setup.rs:308`), `RefreshResponse`
  (`:343`, rendu par `refresh` et `change_password`) — valeur `refresh_inactivity.num_seconds()` — et `MeResponse`
  (`:571`) — valeur calculée depuis la ligne `refresh_tokens` du cookie `kesh_refresh_token` (le cookie a pour chemin
  `/api/v1/auth`, `:63` : il accompagne `GET /api/v1/auth/me`), `max(0, expires_at − now)`, et **`null`** si le cookie
  manque, est inconnu, révoqué ou expiré (`find_active_by_token`, `repositories/refresh_tokens.rs:79`). Aucune écriture,
  aucun schéma.
- **Frontend** — un module `lib/shared/utils/session-renewal.ts` (une seule définition) :
  - **échéances** : à chaque `login` / `hydrate` / renouvellement, le module retient `accessDeadline = now + expiresIn` et
    `sessionDeadline = now + sessionExpiresIn` (absent ⇒ `accessDeadline`, ce qui rend le comportement prudent face à un
    serveur 0.17.1) ;
  - **activité** : des écouteurs passifs `keydown`, `pointerdown`, `wheel`, `touchstart` sur `window` notent
    `lastActivityAt` (C-v0172b-3) ; **les requêtes HTTP ne comptent pas** ;
  - **minuterie** : une seule par onglet, armée à `min(accessDeadline, sessionDeadline) − MARGE` (`MARGE` = 60 s, bornée à
    la moitié de la plus courte des deux durées) ; à son échéance, **si** `lastActivityAt > lastRenewalAt`, elle renouvelle
    (`refreshTokens()`), sinon elle passe en **attente d'activité** : la prochaine entrée de l'utilisateur **avant**
    `sessionDeadline` renouvelle aussitôt ; après `sessionDeadline`, rien (la session est morte, la prochaine requête rend
    401 et mène à la connexion, comme aujourd'hui) ;
  - **entre onglets** : `refreshTokens()` passe sous `navigator.locks.request('kesh:auth:refresh', …)` ; une fois le verrou
    obtenu, si `localStorage['kesh:auth:renewedAt']` est postérieur au moment où l'onglet a décidé de renouveler, un autre
    onglet l'a fait : **pas de `POST /auth/refresh`**, un `GET /auth/me` recale les échéances ; après un renouvellement
    réussi, `renewedAt` est écrit et le canal `authBroadcast` (`auth.svelte.ts`, existant) diffuse
    `{ type: 'auth-renewed' }`, sur lequel les autres onglets recalent leurs échéances par `/me` **sans** renouveler. Le même
    chemin verrouillé sert le renouvellement sur 401. Sans `navigator.locks` (jsdom, navigateur ancien), repli sans verrou,
    sur la seule comparaison de `renewedAt` (C-v0172b-4).
- **Écartées** (C-v0172b-2) : (a) côté serveur seul, porter `expires_at` à `now + jwt_expiry + leeway + refresh_inactivity`
  — le 401 réussirait, mais l'inactivité tolérée deviendrait « entre 15 et 31 minutes » selon le moment de la dernière
  requête, et une saisie de plus de 31 minutes sans requête serait perdue quand même ; (b) prolonger à chaque requête — les
  sondes de fond (§ 6) rendraient la session éternelle ; (c) ne corriger que le manuel — la perte de saisie est le défaut.

### Acceptance Criteria (A.1)

**AC-A1 — Le champ `sessionExpiresIn`.** `POST /api/v1/auth/login`, `POST /api/v1/auth/refresh`,
`PUT /api/v1/auth/password` et la fin de l'assistant (`routes/setup.rs`) rendent `sessionExpiresIn ==
refresh_inactivity.num_seconds()` (900 par défaut) ; `GET /api/v1/auth/me` rend la durée restante du jeton de
renouvellement du cookie (entre 0 et 900 juste après la connexion, **décroissante** : un `/me` lu après avoir reculé
`expires_at` de 300 s en base rend ≤ 600), et `null` sans cookie de renouvellement (en-tête `Bearer` seul), avec un cookie
révoqué ou expiré. `expiresIn` est inchangé partout. Tests : `crates/kesh-api/tests/` (fichier des E2E d'authentification
existant — le développeur le nomme au Change Log après `git grep -ln 'auth/me' crates/kesh-api/tests`).

**AC-A2 — Le défaut reste visible au serveur, figé par un test.** Un test d'intégration documente la sémantique : un jeton
de renouvellement dont `expires_at` est passé est refusé (`401`, `INVALID_REFRESH_TOKEN`) — il **reste vert** : c'est ce
qui oblige le frontend à renouveler **avant** l'échéance. (Sans lui, un correctif futur pourrait « réparer » #876 en
relâchant l'expiration, ce que C-v0172b-2 écarte.)

**AC-A3 — Le renouvellement anticipé pendant l'activité.** `session-renewal.test.ts` (Vitest, minuteries simulées) :
(a) connexion avec `expiresIn = 900`, `sessionExpiresIn = 900` ; une entrée clavier à t = 5 min ; à t = 14 min (900 − 60 s),
`POST /api/v1/auth/refresh` **est appelé**, **avant** toute requête rejetée ; les nouvelles échéances sont relues (900 de
plus) ; une seconde activité à t = 20 min déclenche le renouvellement suivant à t = 28 min ; **rouge avant correctif** ;
(b) **sans aucune entrée** : à t = 14 min, aucun `POST /auth/refresh` ; une entrée à t = 14 min 30 s (avant 15 min) renouvelle
aussitôt ; une entrée à t = 15 min 30 s ne renouvelle **pas** ; (c) une requête de fond (mock `listBackupSchedule` ou
`fetch('/health')`) à chaque minute **ne** déclenche **aucun** renouvellement ; (d) `sessionExpiresIn = 300` (inactivité
réglée à 5 minutes) avec `expiresIn = 900` : la minuterie s'arme à 4 min, non à 14 ; (e) `sessionExpiresIn` absent : la
minuterie s'arme sur `expiresIn` seul.

**AC-A4 — Un seul renouvellement pour plusieurs onglets.** (a) Deux instances du module partageant un `localStorage` et un
`navigator.locks` simulés, échéances identiques, activité dans les deux : **un seul** `POST /auth/refresh` ; l'autre
instance fait `GET /auth/me` et recale ses échéances ; (b) le message `auth-renewed` reçu recale les échéances sans
`POST` ; (c) deux 401 simultanés **dans deux onglets** (deux instances) : un seul `POST /auth/refresh`, les deux requêtes
rejouées une fois ; (d) sans `navigator.locks` : (a) tient par la comparaison de `renewedAt` quand les deux décisions sont
séquentielles (la fenêtre de course résiduelle est déclarée au Dev Agent Record, C-v0172b-4). Les tests existants du
mutex d'onglet (`api-client.test.ts:270-316`) restent verts.

**AC-A5 — Recette sur une installation de dev.** Sur `kesh_v0172b`, backend lancé avec `KESH_JWT_EXPIRY_MINUTES=2` et
`KESH_REFRESH_INACTIVITY_MINUTES=2` : (a) en tapant dans une facture en brouillon (sans enregistrer) pendant **6 minutes**,
puis en enregistrant : l'enregistrement réussit, aucune redirection vers `/login` ; (b) sans rien toucher pendant 3
minutes, puis un clic sur une entrée du menu : retour à la connexion (`reason=session_expired`) ; (c) deux onglets ouverts,
actifs : au bout de 6 minutes, aucun des deux n'est déconnecté, et aucune ligne `token replay detected` au journal du
backend. Le déroulé (heures, résultat) va au Dev Agent Record. Ce n'est **pas** un test automatisé : la suite E2E partage un
backend aux réglages par défaut (C-v0172b-5).

**AC-A6 — Le manuel dit vrai.** `user-manual.tex:95` est récrit (§ *Manuel*), et le manuel d'administration
(`admin-manual.tex:779`) dit ce que mesure l'inactivité : le temps **sans entrée au clavier ni à la souris** dans Kesh, au
plus `KESH_REFRESH_INACTIVITY_MINUTES` ; une saisie en cours prolonge la session.

## A.2 — #840 : deux envois simultanés du même fichier enregistrent deux fois le relevé

### Ce que le code dit (d7ec2ea2)

8. **Le contrôle « fichier déjà importé » et l'`INSERT` ne sont pas exclusifs.** `create` (`routes/bank_imports.rs:1157-`)
   ouvre une transaction (`:1229-1234`), lit `find_by_company_and_hash` (`:1240`), puis plus loin
   `find_in_dedup_window` (`:1302`) et `create_import_or_roll_back` (`:1378`), commit (`:1395`). Même schéma pour le CSV :
   `create_csv` (`:2060-`), transaction `:2068`, contrôle `:2084`, fenêtre `:2232`, `INSERT` `:2304`, commit `:2321`.
   L'unicité `(company_id, file_hash)` a été retirée par `20260507000001_bank_imports_relax_hash_unique.sql` (le
   réimport **confirmé** — `confirmDuplicateFile` — est un usage légitime), et le compte n'est pris qu'en `LOCK IN SHARE
   MODE` (deux lecteurs compatibles). Sous REPEATABLE READ, deux requêtes simultanées voient chacune une base sans
   l'autre : deux relevés identiques, **sans confirmation**. Le commentaire `:1212-1223` le reconnaît (« limitations v0.1
   L11 race acceptée »).
9. **Le remède existe dans le dépôt** — la saisie manuelle (v016-2a, C-v016-2-24) :
   `routes/bank_statements_manual.rs:211-283` prend `with_account_lock` (`kesh_reconciliation::mutex`,
   `GET_LOCK('reconcile:<base>:<société>:<compte>')`, `MANUAL_STATEMENT_LOCK_TIMEOUT_SECS = 5`, `:287`) sur une connexion
   **dédiée** (`lock_tx`), crée dans sa **propre** transaction (`create_under_lock`), commite, puis rend le verrou ; ses
   tests : `bank_statements_manual_e2e.rs:1431` (`create_waits_for_the_account_lock_and_sees_the_committed_twin`,
   entrelacement forcé par `take_account_lock`, `:1408`, et `attendre_une_requete_en_cours`) et `:1519`
   (`two_identical_simultaneous_creations_give_one_statement`). La table d'appariement `(statuts 409)` y est déjà écrite.

### Conception retenue (C-v0172b-6)

Les deux chemins de **fichier** (`create`, CAMT, et `create_csv`) prennent **le même verrou nommé du compte**, sur le motif
exact de `bank_statements_manual::create` : la section qui va de l'ouverture de la transaction à son commit (contrôle du
fichier identique, analyse, contrôles de solde, fenêtre de doublons, `INSERT`, trace) passe **telle quelle** dans une
fonction `…_under_lock(pool, …)` appelée dans la fermeture de `with_account_lock` ; le verrou est rendu **après** le
commit. L'ordre des refus est **inchangé** (aucun contrôle ne change de place), l'empreinte d'aperçu reste vérifiée avant
(`:1200-1209`). Délai : `MANUAL_STATEMENT_LOCK_TIMEOUT_SECS` réutilisée (pas de constante neuve) ; un verrou tenu au-delà
rend `409 RECONCILIATION_ACCOUNT_LOCKED`, sans écriture. Le chemin manuel et les chemins de fichier se sérialisent donc aussi
entre eux sur un même compte — c'est voulu (fenêtre de doublons commune). **Aucune migration** : une contrainte
d'unicité interdirait le réimport confirmé, et `#819` (clé de doublon des relevés déjà importés) est hors périmètre.

### Acceptance Criteria (A.2)

**AC-A7 — Entrelacement forcé, CAMT et CSV.** Dans `crates/kesh-api/tests/bank_imports_e2e.rs`, sur le modèle de
`bank_statements_manual_e2e.rs:1431` (helper `take_account_lock` recopié ou partagé via `tests/common`) : une connexion de
test tient le verrou du compte ; le `POST /api/v1/bank-imports` (multipart CAMT) est **vu en attente** sur `GET_LOCK`
(`attendre_une_requete_en_cours`) ; la connexion enregistre alors un relevé de **même `file_hash`**, commite, rend le
verrou ; le `POST` rend `422 BANK_IMPORT_DUPLICATE_FILE` ; **un seul** relevé en base. Même test pour un fichier CSV (profil
de test existant). **Rouges avant correctif** (le `POST` n'attend rien et rend `201`, deux relevés).

**AC-A8 — Deux envois lancés ensemble.** `tokio::join!` de deux `POST` identiques (CAMT), puis CSV : statuts `[201, 422]`,
le refusé porte `BANK_IMPORT_DUPLICATE_FILE`, **un** relevé et ses transactions une seule fois. Vert après correctif ;
avant, il est rouge **ou** vert selon l'ordonnancement — il n'est donc pas la preuve, AC-A7 l'est.

**AC-A9 — Le réimport confirmé et le délai.** `confirmDuplicateFile=true` sur un fichier déjà importé enregistre toujours
un second relevé (`201`, modificateur `duplicate_file` à la trace) — comportement 8-3 inchangé ; un verrou tenu 6 s par une
connexion de test rend `409 RECONCILIATION_ACCOUNT_LOCKED` et n'écrit rien. Les tests existants de `bank_imports_e2e.rs`
restent verts sans changement de leurs assertions.

**AC-A10 — L'écran dit le 409.** `BankImportUpload.svelte` affiche le message du 409 (chemin d'erreur générique existant,
`isApiError` → `err.message`) : le développeur le vérifie par un test Vitest (mock 409 `RECONCILIATION_ACCOUNT_LOCKED`) ; si
le message est déjà affiché, le test le fige ; sinon, correction.

## Tests et mutations (A)

| # | Test | Attendu |
|---|---|---|
| TA1 | `sessionExpiresIn` aux cinq réponses (AC-A1) | rouge avant correctif |
| TA2 | refus d'un jeton de renouvellement expiré (AC-A2) | vert avant et après |
| TA3 | Vitest renouvellement anticipé a–e (AC-A3) | (a), (b) 2ᵉ partie, (d) rouges avant ; (b) 1ʳᵉ partie, (c) verts avant (rien n'est appelé) |
| TA4 | Vitest plusieurs onglets a–d (AC-A4) | (a), (b), (c) rouges avant |
| TA5 | entrelacement forcé CAMT et CSV (AC-A7) | **rouges avant correctif** |
| TA6 | deux envois ensemble (AC-A8) | vert après |
| TA7 | réimport confirmé, délai (AC-A9) | vert avant et après (délai : rouge avant, aucun verrou n'est attendu) |
| TA8 | Vitest 409 de l'import (AC-A10) | selon le code |

**Mutations nommées (P2)** — sous le verrou de gate ; après une restauration par `cp`/`mv`, **toucher le fichier** :
- **MA1** — la minuterie n'appelle plus `refreshTokens()` : TA3 (a) rougit.
- **MA2** — la condition d'activité retirée (la minuterie renouvelle toujours) : TA3 (b) et (c) rougissent.
- **MA3** — les requêtes comptées comme activité (l'enveloppe `request` note `lastActivityAt`) : TA3 (c) rougit.
- **MA4** — la comparaison `renewedAt` retirée sous le verrou : TA4 (a) et (c) rougissent.
- **MA5** — `/me` rend `refresh_inactivity` au lieu de la durée restante : TA1 (le `/me` décroissant) rougit.
- **MA6** — `create` (CAMT) appelle `…_under_lock` **sans** `with_account_lock` : TA5 CAMT rougit seul.
- **MA7** — idem pour `create_csv` : TA5 CSV rougit seul.
- **MA8** — le verrou rendu **avant** le commit (le commit sorti de la fermeture) : TA5 rougit (le `POST` voit un relevé non
  commité ou n'en voit aucun).

---

# Sous-story B — les écrans

## B.1 — #870 : la liste des factures fournisseur

### Ce que le code dit

10. `routes/(app)/supplier-invoices/+page.svelte:71-73` : `listSupplierInvoices(showCancelled ? { limit: 100 } : { limit:
    100, status: WITHOUT_CANCELLED })` ; aucune lecture de `page.url.searchParams`, aucun `offset`, aucun bouton de page.
    **Le serveur pagine déjà** : `ListSupplierInvoicesQuery { limit, offset, status }` (`routes/supplier_invoices.rs:640-`),
    écrêtage `LIMITE_DEFAUT = 50`, `LIMITE_MAX = 200` (`services/factures_fournisseur.rs:28, :30, :116`), `total` **filtré
    par les statuts** (`repositories/supplier_invoices.rs:218-` : `SELECT COUNT(*) … WHERE si.company_id = ?{filter}`), et
    `status` accepte une liste (`statuts_du_filtre`, `services/factures_fournisseur.rs:65`). **Aucune modification serveur.**
11. Le contrôle de clôture K8 (`lib/features/fiscal-years/FiscalYearChecks.svelte:280-311`) nomme vingt brouillons, dit
    « … et N autre(s), dans la liste des factures fournisseurs » et lie `href="/supplier-invoices"` (`:308`), sans filtre.
    Au-delà de cent factures (plus récentes d'abord), un brouillon ancien n'est atteignable nulle part — le manuel affirme
    pourtant « Pour les brouillons, le lien ouvre la liste complète » (`user-manual.tex:833`), ce qui est faux.
12. **Le motif de pagination** à reprendre : `routes/(app)/invoices/+page.svelte` — `initFromUrl` (`:71-107`, lecture et
    validation de `status`, `limit`, `offset`), `syncUrl` (`:118-153`, `goto(…, { replaceState: true, keepFocus: true,
    noScroll: true })`), boutons *Précédent* / *Suivant* et total (`:451-471`).

### Conception (C-v0172b-7)

La liste lit `?status=` (**une** valeur parmi `draft`, `open`, `paid`, `cancelled`), `?offset=` et `?limit=` (défaut 50,
borné à [1, 200] comme le serveur), selon le motif de `invoices/+page.svelte`, et les réécrit par `goto(…, { replaceState:
true })`. Sans `status`, comportement 0.17.1 : sans les annulées, la case *Afficher les factures annulées* les rend. Avec
`status`, la liste ne montre que ce statut, la case est **masquée** et un bandeau dit le filtre (« Seules les factures
« Brouillon » sont affichées. » + lien *Afficher toutes les factures* → `/supplier-invoices`). Une valeur inconnue est
ignorée (liste par défaut), jamais envoyée au serveur. Pied : « N facture(s) », *Précédent*, *Suivant*. Le lien de K8 devient
`/supplier-invoices?status=draft` ; son libellé « Ouvrir les brouillons de factures fournisseurs ». Hors périmètre : la limite
de 200 factures **validées** proposées à un lot de paiement (`routes/(app)/payment-batches/+page.svelte:54`), défaut de même
famille, non mécanique (un sélecteur paginé) — **issue demandée** (C-v0172b-19).

### Acceptance Criteria (B.1)

**AC-B1** — `/supplier-invoices?status=draft` appelle `listSupplierInvoices({ limit: 50, offset: 0, status: ['draft'] })`,
affiche le bandeau et masque la case ; `?status=bogus` appelle la liste par défaut (`status: ['draft','open','paid']`).
**AC-B2** — avec `total = 120` et `limit = 50` : page 1 *Précédent* désactivé ; *Suivant* appelle `offset: 50` et réécrit
l'URL `?offset=50` (statut conservé) ; page 3 (`offset=100`) *Suivant* désactivé ; « 120 factures ». Bascule de la case ou
retrait du filtre ⇒ `offset` revient à 0. Le jeton `requestSeq` (`:65`) reste en place.
**AC-B3** — K8 : `fiscal-year-checks-link-DRAFT_SUPPLIER_INVOICES` (`FiscalYearChecks.svelte:280`, `check.code === 'DRAFT_SUPPLIER_INVOICES'`) a
`href="/supplier-invoices?status=draft"`. **AC-B4** — E2E `supplier-invoices.spec.ts` : 55 brouillons créés par l'API
(fixture), la liste filtrée par l'URL montre 50 lignes puis 5 après *Suivant*.
Tests : `supplier-invoices-list-page.test.ts` (existant), `FiscalYearChecks.test.ts` (existant ou créé). **Rouges avant
correctif** : AC-B1, AC-B2, AC-B3.

## B.2 — #851 : des entrées du menu disparaissent

### Ce que le code dit

13. **L'hypothèse de l'issue (rôle momentanément nul) est réfutée au code.** `isAdmin` (`routes/(app)/+layout.svelte:45`)
    lit `authState.currentUser?.role` ; `login()` remplace `_currentUser` d'un seul tenant (`auth.svelte.ts:111-127`), sans
    passer par `null` ; `hydrate()` n'est appelé qu'au démarrage (`hooks.client.ts:22`) et sur un message inter-onglets
    (`auth.svelte.ts:330-336`) ; et la garde de `/admin/mcp` (`routes/(app)/admin/mcp/+page.ts:12-14`) **redirigerait vers
    `/`** si le rôle manquait au chargement. Surtout, un rôle nul ferait disparaître **tout** le bloc `adminOnly` (neuf
    entrées), pas « une partie des entrées ».
14. **La cause lue au code : le menu déborde sans défiler.** La barre latérale est
    `<nav class="sticky top-14 flex h-[calc(100vh-3.5rem)] w-56 flex-col …">` (`:374-376`) : **hauteur fixe**, **aucun**
    `overflow-y`. Le groupe Administration porte 8 + 2 + 9 = **19 entrées** pour un Administrateur (`:119-190`), Quotidien
    11, Mensuel 4 ; en mode guidé chaque lien a `min-height: 44px` (`app.css:57`) : ≈ 34 × 46 px ≈ 1 560 px de liens pour
    une fenêtre de 720 px. Ce qui dépasse la hauteur de la barre se dessine **sous** elle ; collée (`sticky`) au haut de
    l'écran, elle défile avec la page : sur une page **courte** (dont `/admin/mcp`), le bas du groupe — justement les
    entrées « Serveur MCP », « Envois », « Prompts » et voisines — reste hors d'atteinte ou se cache sous le pied de page ;
    sur une page longue, il réapparaît en bas de défilement. D'où « en ouvrant Serveur MCP (et d'autres) ». Ouvrir une
    entrée d'Administration déplie ce groupe (`$effect`, `:258-266`), ce qui allonge le menu.

### Conception (C-v0172b-8)

**Reproduire d'abord** (AC-B5, rouge attendu), puis corriger par la seule classe `overflow-y-auto` (et `min-h-0`) sur la
`<nav>` : la barre garde sa hauteur et défile **seule**. Si la reproduction **ne** rougit **pas**, ne rien corriger : relever
les entrées absentes par capture, le dire au Dev Agent Record et au rapport (l'orchestrateur commente l'issue) — l'hypothèse
du rôle (§ 13) reste écartée sauf preuve. Hors périmètre : le dépliage exclusif des groupes (issue voisine citée par #851).

### Acceptance Criteria (B.2)

**AC-B5** — `tests/e2e/sidebar-navigation.spec.ts`, Administrateur, fenêtre 1280 × 720, mode guidé, les trois groupes
dépliés : sur `/admin/mcp` puis sur `/` et `/settings`, **chacune** des 19 entrées d'Administration (sélecteurs
`nav-link-*`, liste écrite en dur dans le test, recomptée depuis `navGroups`) est **atteignable** : après
`locator.scrollIntoViewIfNeeded()`, `expect(locator).toBeInViewport()` **et** `document.elementFromPoint` au centre de sa
boîte rend le lien lui-même (ni le pied de page, ni le contenu). **Rouge avant correctif** (au moins une entrée).
**AC-B6** — le nombre d'entrées `nav-link-*` du groupe Administration est 19 sur `/admin/mcp`, `/admin/mcp/sends`,
`/admin/mcp/prompts`, `/users` et `/` (garde de l'hypothèse du rôle, vert avant et après). Captures : menu déplié sur
`/admin/mcp`, avant (le bas manquant) et après (barre défilée jusqu'en bas).

## B.3 — #882 : une ligne au taux de TVA désactivé

### Ce que le code dit

15. `InvoiceForm.svelte:54-74` : `vatOptions = rates.map((r) => r.rate)` depuis `getVatRates()` — **taux actifs seulement**
    (`repositories/vat_rates.rs:55`, `active = TRUE`). Le `<select bind:value={line.vatRate}>` (`:810-820`, `bind:value` à `:811`) n'a d'option que
    pour eux : une ligne de brouillon (`initLines`, `:119` ; `reloadInvoice`, `:616`) ou d'article (`onProductSelect`,
    `:430`) qui porte un taux désactivé depuis s'affiche **sans option choisie**, et `validate` refuse « Ligne N : taux TVA
    invalide » (`:496-497`) sans dire pourquoi. Le repli `:83-92` ne couvre que `8.10`.
16. **Le serveur refuse aussi** : `create_invoice` / `update_invoice` appellent `verify_vat_rates_against_db`
    (`routes/invoices.rs:661, :749` → `routes/vat.rs`, `… WHERE company_id = ? AND active = TRUE AND rate IN (…)`) ⇒ `400`
    « Taux TVA non autorisé pour cette entreprise. » (`VAT_REJECTED_MSG`, `vat.rs:49`). Proposer le taux à l'écran ne
    suffirait donc pas à enregistrer.
17. **Le symétrique fournisseur existe** : `SupplierInvoiceForm.svelte:187-196` (`extraRates`, revue P1 de la v017-4b,
    E-7) propose les taux des lignes absents des taux actifs ; le serveur des factures fournisseur ne vérifie pas les taux
    (aucun appel `verify_vat_rates_against_db` dans `routes/supplier_invoices.rs`).

### Conception (C-v0172b-9)

Côté client seulement, **sans changer la règle du serveur** (accepter un taux désactivé sur une facture cliente touche à la
TVA, dont les dix P1 du jalon « Vague 2 » sont reportés) : (1) le `<select>` d'une ligne dont le taux n'est pas actif
propose ce taux, **libellé** « 7.70 % (désactivé) », afin que la ligne ne s'affiche jamais vide ; l'ensemble des taux
proposés est **dérivé des lignes** (`$derived`), non calculé une fois, pour couvrir `onProductSelect` et `reloadInvoice` ;
(2) `validate` rend, pour une telle ligne, « Ligne N : le taux de TVA 7.70 % a été désactivé. Choisissez un taux actif,
ou réactivez-le dans Paramètres › TVA. » (clé i18n neuve, quatre locales), à la place de « taux TVA invalide » — qui reste
pour une valeur inconnue qui n'est **ni** active **ni** portée par la ligne d'origine (cas qui ne se produit plus) ;
(3) choisir un taux actif retire le libellé et le refus. Le chemin de l'écran des réglages TVA est relu au code par le
développeur (route exacte) avant d'écrire le texte.

### Acceptance Criteria (B.3)

**AC-B7** — `InvoiceForm.test.ts` : un brouillon dont une ligne porte `7.70` (taux absent de `getVatRates()` mocké à
`['8.10','2.60','0.00']`) s'ouvre avec le `<select>` sur `7.70`, option libellée `7.70% (désactivé)` (texte exact relu à la
clé i18n) ; *Enregistrer* n'appelle pas l'API et affiche le message de C-v0172b-9 ; choisir `8.10` puis *Enregistrer* envoie
`vatRate: "8.10"`. Même chose après `reloadInvoice` (conflit de version) et pour une ligne ajoutée depuis un article dont
`vatRate = '7.70'`. **Rouge avant correctif** (le `<select>` a la valeur `''` ou la première option ; le message est
« taux TVA invalide »). **AC-B8** — une ligne à taux actif : aucune option « (désactivé) » n'apparaît (test négatif). Le
serveur est inchangé (`git diff d7ec2ea2 -- crates/kesh-api/src/routes/vat.rs crates/kesh-api/src/routes/invoices.rs` vide).
Captures : brouillon rouvert avec la ligne au taux désactivé et son message.

## B.4 — #885 : la date d'une facture fournisseur créée depuis un document

### Ce que le code dit

18. `SupplierInvoiceForm.svelte:144-185` (`startState`) — à la création **avec** document : `invoiceDate: ''`, `dueDate:
    ''` (`:159-160`) ; sans document : la date du jour, échéance vide. Le commentaire (`:146-151`) explique #853 : « la date
    du jour n'est presque jamais celle d'une facture reçue ». À l'édition, `initial` prime (`:170-172`).
19. **La date du jour est la date UTC** : `new Date().toISOString().slice(0, 10)` (`:153`) — entre minuit et 1 h (2 h en
    été) en Suisse, c'est la veille. Le dépôt a l'helper local `todayLocal()` (`lib/features/open-items/open-items.ts:26`).
    Neuf autres sites emploient la forme UTC (`git grep -lE "new Date\(\)\.toISOString\(\)\.slice\(0, ?10\)" -- frontend/src
    ':!*.test.ts'`) : hors périmètre, **issue demandée** (C-v0172b-20) ; seul celui de ce formulaire passe à `todayLocal()`.
20. **Prémisse de l'issue à corriger** : « l'échéance aussi [vide] (sauf conditions de paiement du fournisseur) » — le
    formulaire fournisseur **n'emploie jamais** les conditions de paiement du fournisseur : `git grep -nE
    'defaultPaymentTermsDays' -- frontend/src ':!*.test.ts'` ne le trouve que dans `InvoiceForm.svelte:406-407` (facture
    **cliente**, #245 : `addDaysIso(date, c.defaultPaymentTermsDays)`, `lib/features/invoices/invoice-helpers.ts`) et dans
    l'écran des contacts. Le serveur ne la calcule pas non plus (`routes/supplier_invoices.rs` : aucun
    `default_payment_terms_days`). L'échéance fournisseur démarre **toujours** vide.
21. **Le contrôle de #853** : `dueBeforeInvoiceDate` (`supplier-invoice-form.ts:180`), affiché sous le champ et bloquant
    (`SupplierInvoiceForm.svelte:207-208, :337, :591-597`) ; le refus serveur `SUPPLIER_INVOICE_DUE_BEFORE_INVOICE_DATE`
    (`routes/supplier_invoices.rs:550-556`). Inchangés.
22. **Les tests qui figent l'état actuel** : `SupplierInvoiceForm.test.ts:574-586` (« avec un document, la date démarre
    VIDE … ») et `:150-156` — à mettre à jour, avec la raison au Change Log.

### Conception (C-v0172b-15, 16)

- **Date** — à la création avec document, `invoiceDate = todayLocal()` et un état `dateToVerify = true` ; sous le champ, à la
  place de l'aide actuelle, un avis **visible** (`data-testid="supplier-invoice-date-to-verify"`, fond `warning/10`,
  `aria-live="polite"`) : « Date du jour, proposée par défaut — ce n'est pas celle du document. Vérifiez-la. » avec un
  bouton *C'est la bonne date* (`supplier-invoice-date-confirm`). L'avis disparaît quand l'utilisateur **modifie** la date
  (`oninput`) ou **confirme**. Le champ porte `aria-describedby` vers l'avis. **L'enregistrement n'est pas bloqué** par une
  date non confirmée (C-v0172b-15 : l'avis est visible, ce qui distingue l'état de #853, où la date du jour restait sans le
  dire). Sans document et à l'édition : aucun avis (comportement inchangé). Si l'utilisateur vide la date, le message
  « Saisissez la date de la facture, telle qu'imprimée sur le document. » reste.
- **Échéance** — à la création (avec ou sans document) : un état `dueFollows = true` ; tant qu'il l'est, `fDueDate =
  invoiceDate + defaultPaymentTermsDays du fournisseur choisi` (`addDaysIso`, `invoice-helpers.ts`), ou `invoiceDate` si
  le fournisseur n'en a pas ou si aucun fournisseur n'est choisi ; elle **se recalcule** quand la date ou le fournisseur
  change. Une saisie dans le champ échéance (`oninput`) met `dueFollows = false` ; elle n'est plus jamais recalculée. Une date
  vide ⇒ échéance vide (tant que `dueFollows`). À l'édition (`initial`), `dueFollows = false`. Le contrôle de #853 reste :
  une échéance saisie antérieure à la date est signalée et bloque.
- **Écartée** : bloquer l'envoi tant que la date n'est pas confirmée — c'est rouvrir le champ vide de #853 avec un clic de
  plus ; réversible si le Project Lead le souhaite.

### Acceptance Criteria (B.4)

**AC-B9** — `SupplierInvoiceForm.test.ts`, horloge fixée (`vi.setSystemTime('2026-11-04T23:30:00Z')`, soit le 5 novembre
à 00:30 en Suisse) : avec document, la date vaut `2026-11-05` (date **locale**, non UTC — la moitié qui rougit sous
`toISOString`) et l'avis « à vérifier » est visible ; sans document, `2026-11-05` et aucun avis ; à l'édition, la date de
`initial` et aucun avis.
**AC-B10** — l'avis disparaît après une saisie dans la date, ou après *C'est la bonne date* ; il ne revient pas.
**AC-B11** — les trois cas de l'échéance (issue) : fournisseur à 30 jours ⇒ échéance `2026-12-05`, et `2026-12-10` après
avoir porté la date au `2026-11-10` ; fournisseur sans délai ⇒ échéance = date, et elle suit la date ; échéance saisie à la
main (`2026-11-20`) ⇒ changer la date ou le fournisseur ne la touche plus. Changer de fournisseur (30 → 10 jours) recalcule
tant que l'échéance suit.
**AC-B12** — #853 inchangé : échéance saisie `2026-11-01` pour une date `2026-11-05` ⇒ message sous le champ, rien ne part ;
la requête envoyée sans toucher à rien porte `invoiceDate: '2026-11-05'` et `dueDate` calculée. Les tests `:574-586` et
`:150-156` sont mis à jour (la date ne démarre plus vide). E2E : `supplier-invoice-draft-cycle.spec.ts` ou l'E2E de la
complétion d'un document (relu par le développeur) — si une assertion lit la date vide, la mettre à jour.
Captures : complétion d'un document, avis visible ; après *C'est la bonne date* ; échéance calculée d'un fournisseur à délai.

## Tests et mutations (B)

| # | Test | Attendu |
|---|---|---|
| TB1 | liste filtrée par l'URL, pagination (AC-B1, B2) | rouges avant |
| TB2 | lien de K8 (AC-B3) | rouge avant |
| TB3 | E2E 55 brouillons (AC-B4) | rouge avant |
| TB4 | E2E menu atteignable (AC-B5) | rouge avant (attendu) |
| TB5 | E2E nombre d'entrées (AC-B6) | vert avant et après |
| TB6 | Vitest taux désactivé (AC-B7, B8) | B7 rouge avant ; B8 vert |
| TB7 | Vitest date et avis (AC-B9, B10) | rouges avant |
| TB8 | Vitest échéance (AC-B11) | rouges avant |
| TB9 | Vitest #853 (AC-B12) | vert avant (contrôle) ; tests mis à jour |

Mutations : **MB1** `offset` non relu de l'URL ⇒ TB1 rougit ; **MB2** lien K8 sans `?status=draft` ⇒ TB2 ; **MB3**
`overflow-y-auto` retiré ⇒ TB4 ; **MB4** l'ensemble des taux proposés calculé une fois (non dérivé) ⇒ le cas
`onProductSelect` de TB6 rougit ; **MB5** `toISOString` rétabli ⇒ TB7 (date locale) ; **MB6** `dueFollows` jamais remis à
`false` ⇒ le cas « saisie à la main » de TB8 ; **MB7** l'avis non retiré à la saisie ⇒ TB7 (AC-B10).

---

# Sous-story C — la forme

## C.1 — #880 : les dates ISO des écrans comptables et des clés API

### Ce que le code dit — inventaire clos

23. **Méthode.** Le périmètre est l'ensemble des `.svelte` (hors tests) de ces neuf dossiers — **32 fichiers** (`git ls-files
    'src/routes/(app)/journal-entries/*.svelte' 'src/lib/features/journal-entries/*.svelte'
    'src/routes/(app)/settings/fiscal-years/*.svelte' 'src/lib/features/fiscal-years/*.svelte'
    'src/routes/(app)/bank-import/*.svelte' 'src/lib/features/bank-import/*.svelte' 'src/routes/(app)/reconciliation/*.svelte'
    'src/lib/features/reconciliation/*.svelte' 'src/routes/(app)/settings/api-keys/*.svelte' | grep -v '\.test\.'`, depuis
    `frontend/`, rend 32 lignes). Sur eux, deux motifs, relus ligne à ligne :
    `M₁ = '\{[^}]*(\.(date|[a-zA-Z]*Date|[a-zA-Z]*At|periodFrom|periodTo|startDate|endDate|[a-zA-Z]*On)\b|slice\(0, ?10\))'`
    (interpolations) et `M₂ = '(\b(date|from|to|start|end|period|since|until)[A-Za-z]*: [a-zA-Z])|toLocale(Date)?String|formatDate|new Date\('`
    (paramètres de messages i18n et formats maison), en excluant `bind:value|value=\{|onchange|oninput`.
24. **Les sites à corriger** (affichage d'une date ISO brute) — **24 sites** :

    | # | Site | Valeur | Forme cible |
    |---|---|---|---|
    | 1 | `routes/(app)/journal-entries/+page.svelte:551` | `entry.entryDate` (liste) | `formatSwissDate` |
    | 2 | `journal-entries/+page.svelte:366` | `booksLockedThrough` (bandeau « Livres verrouillés jusqu'au ») | idem |
    | 3 | `journal-entries/[id]/+page.svelte:590` | `entry.entryDate` (fiche) | idem |
    | 4–5 | `settings/fiscal-years/+page.svelte:459, :460` | `fy.startDate`, `fy.endDate` (liste) | idem |
    | 6–7 | `settings/fiscal-years/+page.svelte:636, :642` | `renameTarget.startDate`, `.endDate` (dialogue) | idem |
    | 8 | `lib/features/bank-import/BankImportList.svelte:38` | `imp.importedAt` (horodatage) | `formatSwissDateTime` |
    | 9 | `BankImportList.svelte:40` | `imp.periodFrom → imp.periodTo` (deux dates, une ligne) | `formatSwissDate` ×2 |
    | 10 | `BankImportUpload.svelte:513` | `preview.selectedStatement.periodFrom → …periodTo` | idem ×2 |
    | 11 | `BankImportUpload.svelte:544` | `preview.warnings.duplicateFile.existingImportedAt` | `formatSwissDateTime` |
    | 12 | `BankImportUpload.svelte:799` | `tx.bookingDate` (aperçu) | `formatSwissDate` |
    | 13 | `ManualStatementForm.svelte:629` | `previousPeriodTo` (message `{ $date }`) | idem |
    | 14 | `ManualStatementForm.svelte:648` | `` `${o.periodFrom} → ${o.periodTo}` `` (message `{ $periods }`) | idem ×2 |
    | 15–16 | `DeleteBankImportAction.svelte:150-151, :161-162` | `from`/`to` des deux messages de suppression | idem |
    | 17 | `routes/(app)/bank-import/[id]/+page.svelte:49` | `from`/`to` du message de succès de suppression | idem |
    | 18 | `bank-import/[id]/+page.svelte:139, :144` | `detail.importedAt` (« Saisi le », « Importé le ») | `formatSwissDateTime` |
    | 19 | `bank-import/[id]/+page.svelte:149` | `detail.periodFrom → detail.periodTo` | `formatSwissDate` ×2 |
    | 20 | `bank-import/[id]/+page.svelte:179` | `tx.bookingDate` (transactions) | idem |
    | 21 | `lib/features/reconciliation/ReconciliationProposals.svelte:480` (cellule) et `:461, :471` (`aria-label` « Sélectionner la transaction du { $date } ») | `p.transaction.bookingDate` | idem (C-v0172b-11) |
    | 22 | `ManualMatchModal.svelte:173` ; `TransactionSplitModal.svelte:234` | `proposal.transaction.bookingDate` | idem |
    | 23 | `LinkSettlementModal.svelte:205, :290, :350` | `valueDate ?? bookingDate`, `c.settledOn`, `e.entryDate` | idem |
    | 24 | `routes/(app)/settings/api-keys/+page.svelte:74-77` (`formatDate`, `value.slice(0, 10)` — sert `:94, :99, :104, :371`) et `:80-83` (`formatDateTime`, `value.slice(0, 16).replace('T', ' ')` — sert `:372`) | `createdAt`, `revokedAt`, `expiresAt`, `lastUsedAt` | les deux fonctions locales **délèguent** aux helpers communs (DRY) |

    Les sites multiples d'une même ligne du tableau comptent pour une ligne ; le décompte des **lignes de code** touchées va au
    Dev Agent Record, recompté.
25. **Conformes ou hors classe, justifiés** : `FiscalYearChecks.svelte:73, :82, :215, :248, :294` et
    `journal-entries/[id]/+page.svelte:673` (déjà `formatSwissDate`) ; `FiscalYearChecks.svelte:116` (URL de lien,
    `dateFrom=${c.startDate}` — valeur envoyée, non affichée) ; `journal-entries/+page.svelte:79-80`,
    `journal-entries/[id]/+page.svelte:290, :319`, `JournalEntryForm.svelte:53, :85`, `settings/fiscal-years/+page.svelte:202-203`,
    `api-keys/+page.svelte:51, :56` (valeurs d'état, de calcul ou envoyées à l'API) ; `BankProfileForm.svelte:83, :91`
    (`date: dateIdx`, index de colonne CSV) ; `UnrejectTransactionAction.svelte:39` (`Intl.DateTimeFormat(i18nLocale(),
    { dateStyle: 'medium' })` — forme localisée, non ISO, C-v0172b-12) ; les `<input type="date">` (le navigateur les affiche dans
    sa langue).
26. **Tests qui figent l'ISO** (à mettre à jour, raison au Change Log) : `DeleteBankImportAction.test.ts:103, :268`,
    `LinkSettlementModal.test.ts:287, :289, :292`, `ReconciliationProposals.test.ts:398, :428, :542`. Restent ISO, à bon droit :
    `ManualMatchModal.test.ts:125, :147` (valeur d'un `<input type="date">`), `reconciliation.api.test.ts:133` (corps de requête),
    `JournalEntryForm.edit.test.ts:191-192`, `form-helpers.test.ts:157-158`, `query-helpers.test.ts:30-31` (bornes, paramètres
    d'URL). Le développeur greppe aussi `frontend/tests/e2e/*.ts` (assertions de texte portant une date ISO sur ces écrans) et
    joint la sortie au Change Log.

### Conception (C-v0172b-10)

`formatSwissDate` (`lib/features/reports/reports.api.ts`) **tolère un horodatage** : si l'entrée commence par
`AAAA-MM-JJ` suivi de `T`, d'une espace ou de rien, elle rend `JJ.MM.AAAA` ; toute autre entrée est rendue telle quelle
(comportement actuel pour une valeur illisible) — c'est #883.6. Un helper **voisin** `formatSwissDateTime(iso)` rend
`JJ.MM.AAAA HH:mm` pour un horodatage naïf (`2026-01-29T10:15:00` → `29.01.2026 10:15`), la date seule pour une date pure,
l'entrée telle quelle sinon. Aucun fuseau n'est appliqué : les horodatages du serveur sont naïfs et affichés comme tels
aujourd'hui (`api-keys`, `:82`). `api-keys/+page.svelte` garde `formatDate`/`formatDateTime` comme noms locaux (ils portent
le repli `—` et « Jamais utilisée ») mais délègue la mise en forme aux helpers.

### Acceptance Criteria (C.1)

**AC-C1** — `reports.api.test.ts` : `formatSwissDate('2026-01-29') = '29.01.2026'`, `('2026-11-03T17:40:45') =
'03.11.2026'`, `('2026-11-03T17:40:45.123') = '03.11.2026'`, `('2026-11-03 17:40') = '03.11.2026'`, `('') = ''`,
`('29.01.2026') = '29.01.2026'`, `('n/a') = 'n/a'` ; `formatSwissDateTime('2026-01-29T10:15:00') = '29.01.2026 10:15'`,
`('2026-01-29') = '29.01.2026'`, `('bogus') = 'bogus'`. **Rouge avant** pour l'horodatage (`'03T17:40:45.11.2026'`).
**AC-C2** — chaque site du § 24 affiche la forme suisse : un test Vitest **par écran** (liste et fiche du journal ; liste et
dialogue de renommage des exercices ; liste, aperçu et fiche des relevés ; dialogue de suppression et message de succès ;
avertissements de la saisie manuelle ; propositions de rapprochement, y compris l'`aria-label` ; les trois modales ; clés
API, quatre dates et la dernière utilisation) : `JJ.MM.AAAA` présent, `not.toContain` de l'ISO. Un écran qui porte deux
sites (p. ex. `bank-import/[id]` : `importedAt` et `bookingDate`) les lit **tous deux** dans son test.
**AC-C3** — **Contrôle décidable** : les motifs `M₁` et `M₂` du § 23, rejoués sur **les 32 fichiers** après correction et
filtrés par `| /usr/bin/grep -vE 'formatSwissDate|formatSwissDateTime|formatDate\(|formatDateTime\('`, ne rendent **que** les
lignes du § 25 (listées nominativement au Dev Agent Record, chacune relue) et la **première ligne d'une expression écrite
sur plusieurs lignes** dont la suite passe par un helper (listée et relue) ; aucune autre. **Sonde positive** : les mêmes
motifs sur `d7ec2ea2` rendent les sites du § 24 (décompte de lignes joint, recompté depuis la sortie).

## C.2 — #883 : les six LOW de la revue de la 0.17.1

27. **#883.1** — `crates/kesh-api/src/qr_decode.rs:8` : la ligne fait 121 colonnes (`awk '{print length}'`) ; refluer le
    doc-comment `:7-9` sous 100 colonnes, sans changer le texte.
28. **#883.2** — « des lignes Svelte dépassent probablement la largeur Prettier » : **sans objet**. Le dépôt n'a **ni
    Prettier ni configuration de largeur** (`grep -nE 'prettier|eslint' frontend/package.json` ne rend rien ;
    `git ls-files | grep -iE 'prettier|editorconfig'` ne rend rien) : aucune règle n'est enfreinte, et refluer à la main
    sans règle produirait du bruit de diff. Rien n'est changé ; le rapport demande à l'orchestrateur de le dire à l'issue
    (C-v0172b-13).
29. **#883.3** — `crates/kesh-api/tests/inbox_import_e2e.rs:667` : `assert_ne!(entry["qr"], "none", …)` suit
    `assert_eq!(entry["qr"], "present", …)` (`:666`) : retirer la ligne redondante.
30. **#883.4** — `est_payload_spc` (`qr_decode.rs:39-41`) : ses bornes figées par un test `v0172_est_payload_spc_bornes` —
    `"SPC\r\n0200\n…"` (en-têtes mêlés) **retenu** ; `""` **refusé** ; `"\u{FEFF}SPC\n0200\n…"` (BOM en tête) **refusé** — c'est
    le comportement actuel, figé tel quel, et la norme SIX n'admet pas de BOM (C-v0172b-14) ; `"SPC\n0200"` sans suite
    **retenu** par le préfiltre — et `first_spc` sur ce payload rend `Err(InvalidSpcPayload)` (le parseur le refuse, moins de
    31 lignes).
31. **#883.5** — `v0171_t4_crlf_inexploitable` (`qr_decode.rs:1234-1244`) emploie `Faux::SpcCrlfInvalide` = `qr("SPC\r\n0200\r\n1\r\nPAS-UN-IBAN")`
    (`:1056`) : 4 lignes, refusé au compte de lignes avant tout en-tête. Remplacer la fixture par un payload CR+LF **de 31 lignes
    au moins** (le payload de `build_payload(&donnees_spc_test())` réécrit en CR+LF, l'IBAN remplacé par une valeur invalide)
    et attendre `Err(DecodeError::InvalidIban(_))` : il rougit alors **aussi** sous la mutation qui retire
    `trim_end_matches('\r')` du parseur (`kesh-qrbill/src/parser.rs:95`), qui rend `InvalidSpcPayload` dès l'en-tête `SPC\r`.
    Le jumeau LF existant reste.
32. **#883.6** — traité par C.1 (`formatSwissDate` tolère l'horodatage, AC-C1).

**AC-C4** — #883.1, .3 : `git diff` montre le reflux et la ligne retirée, rien d'autre ; #883.4 : le test des bornes, vert ;
#883.5 : le test rougit sous **MC3** ci-dessous ; #883.2 : aucun fichier Svelte modifié pour la largeur.

## Tests et mutations (C)

| # | Test | Attendu |
|---|---|---|
| TC1 | `formatSwissDate` / `formatSwissDateTime` (AC-C1) | horodatage rouge avant |
| TC2 | Vitest par écran (AC-C2) | rouges avant |
| TC3 | bornes de `est_payload_spc` (#883.4) | vert avant et après |
| TC4 | `v0171_t4_crlf_inexploitable` réécrit (#883.5) | vert avant et après ; rouge sous MC3 |

Mutations : **MC1** `formatSwissDate` rendu à sa forme actuelle (`split('-')`) ⇒ TC1 ; **MC2** retirer le helper d'un site
(une mutation par écran) ⇒ son test seul ; **MC3** retirer `.map(|l| l.trim_end_matches('\r'))` de `parser.rs:95` ⇒ TC4
rougit (il ne rougissait pas avec la fixture de 4 lignes) ; **MC4** `est_payload_spc` accepte tout préfixe `SPC` ⇒ TC3.

---

## Tâches

Par sous-story, dans l'ordre A, B, C ; chacune : tests rouges d'abord (constatés et notés), correctif, mutations, gates,
captures, commit `refs #N`, revue de code.

1. **A** — TA1, TA3, TA4, TA5 rouges sur `d7ec2ea2` ; `sessionExpiresIn` (serveur) ; `session-renewal.ts`, son câblage
   (`auth.svelte.ts` `login`/`hydrate`, `api-client.ts` `doRefresh`/`refreshTokens`, écouteurs posés une fois au montage du
   layout `(app)` et retirés au démontage) ; verrou nommé des deux chemins d'import ; AC-A5 (recette) ; manuels.
2. **B** — TB1–TB4, TB6–TB8 rouges ; liste paginée et filtrée ; lien K8 ; `overflow-y-auto` après reproduction ; taux
   désactivé ; date et échéance ; i18n quatre locales (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`) ;
   `npm run lint-i18n-ownership` ; manuel.
3. **C** — TC1, TC2 rouges ; helpers ; les 24 sites ; tests ISO mis à jour ; #883.1, .3, .4, .5 ; AC-C3.
4. **Gates** : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `scripts/test-fast.sh`
   (sous `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock` et `mem-guard`), `cd frontend && npm run check && npm run
   lint-i18n-ownership && npm run test:unit && npm run build`, E2E complet au **dernier commit de code** de **chaque**
   sous-story (7 à 10 échecs attendus, `docs/testing.md`) ; base remise à zéro avant chaque gate complet. **A touche
   `routes/auth.rs` et `bank_imports.rs`** (aucun repository, aucune migration) : gate ciblé permis en boucle, complet au push.
5. **Captures** (thème clair, base de démonstration jetable `kesh_v0172b`, dans
   `/home/gcorbaz/devel/kesh-gate-logs/v0172b/captures/`) : B — liste filtrée « Brouillon » avec son bandeau, pied de pagination,
   lien K8 ; menu déplié sur `/admin/mcp` avant/après ; brouillon au taux désactivé ; complétion d'un document avec l'avis, après
   confirmation, échéance calculée. C — un écran par ligne du tableau du § 24 (journal liste et fiche, exercices, relevés liste,
   aperçu, fiche, dialogue de suppression, saisie manuelle avec avertissement, rapprochement et ses trois modales, clés API).
   A n'a pas de contrôle visible neuf (AC-A10 : capture du message 409 si le texte change).

## Manuel

`docs/manual/fr/user-manual.tex` (le PDF régénéré par `make fr` dans `docs/manual/`, contrôlé aplati : `pdftotext
docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, puis `grep -F` de chaque phrase neuve) :
- **#876** — `:95` récrit : la session se prolonge **tant que vous travaillez dans Kesh** (une frappe, un clic) ; elle prend fin
  après 15 minutes **sans aucune entrée au clavier ni à la souris** (durée réglable par l'administrateur, de 1 minute à
  24 heures) ; une saisie longue, même sans enregistrer, la garde ouverte ; plusieurs onglets partagent la même session.
  `admin-manual.tex:779` : la description de `KESH_REFRESH_INACTIVITY_MINUTES` dit la même chose ; si une autre phrase du manuel
  d'administration décrit le renouvellement « sur 401 » ou le jeton de 15 minutes, la corriger (`grep -nE 'refresh|jeton|JWT'`).
- **#870** — `:823` et `:833` : le lien de K8 ouvre la liste **filtrée sur les brouillons** (« Pour les brouillons, le lien ouvre
  la liste complète » devient vrai) ; § des factures fournisseur (`:1888-1892`, « Les factures annulées dans la liste ») : la liste
  est paginée (50 par page, *Précédent* / *Suivant*).
- **#885** — `:1607-1612` (« Les dates ») et `:2049-2053` (« Créer la facture d'un document ») : à côté d'un document, la date est
  **proposée au jour même et marquée « à vérifier »** jusqu'à ce que vous la modifiiez ou la confirmiez ; l'échéance suit la date
  (plus le délai de paiement du fournisseur, s'il en a un) tant que vous ne l'avez pas saisie ; une échéance antérieure reste
  signalée. Le rappel historique « depuis la version 0.17.0 : la date du jour n'est presque jamais celle d'une facture reçue »
  devient « depuis la version 0.17.2, elle est proposée et signalée ».
- **#882** — § des lignes de facture (TVA par ligne) : une ligne dont le taux a été désactivé l'affiche « (désactivé) » ; la facture
  ne s'enregistre qu'avec un taux actif.
- **#851, #880, #883** : rien à décrire (le manuel ne décrit ni la forme des dates de ces écrans — `grep -nE 'AAAA-MM-JJ|jj\.mm'
  docs/manual/fr/user-manual.tex` à joindre au Change Log — ni la hauteur du menu).
- `docs/manual/{de,en,it}/` : `ls` ne rend que `README.md` (aucun `.tex`) — rien à traduire.

## CHANGELOG

Section commune `## [0.17.2] — Non publié`, au-dessus de la 0.17.1 (l'autre story de la 0.17.2 y écrit aussi : **fusion par union**,
sans réécrire ses puces). Intro, si elle n'existe pas encore : « La 0.17.2 est une version corrective : **aucune migration**, aucune
donnée réécrite, **le retour à la 0.17.1 est possible sans restauration**. » (à vérifier contre la story voisine à l'intégration.)

`### Corrigé`
- **La session ne se ferme plus pendant que vous travaillez** ([#876](https://github.com/guycorbaz/kesh/issues/876)) : elle n'était
  jamais prolongée — avec les réglages par défaut, elle prenait fin quinze minutes après la connexion, même en pleine saisie, et la
  saisie en cours était perdue. Elle se prolonge désormais tant que vous tapez ou cliquez, et se ferme après 15 minutes sans aucune
  entrée.
- **Un relevé importé deux fois en même temps n'est plus enregistré deux fois** ([#840](https://github.com/guycorbaz/kesh/issues/840)) :
  un double clic ou un envoi rejoué enregistrait deux relevés identiques sans confirmation ; le second est désormais refusé comme
  fichier déjà importé.
- **Toutes les factures fournisseur sont atteignables** ([#870](https://github.com/guycorbaz/kesh/issues/870)) : la liste se pagine
  (au-delà des cent premières, les plus anciennes n'apparaissaient nulle part) et se filtre par statut depuis l'adresse ; le lien du
  contrôle de clôture ouvre les seuls brouillons.
- **Le menu garde toutes ses entrées** ([#851](https://github.com/guycorbaz/kesh/issues/851)) : déplié, il dépassait la hauteur de
  l'écran et ses dernières entrées d'administration devenaient inaccessibles ; il défile désormais *(à ajuster selon AC-B5)*.
- **Un brouillon de facture dont un taux de TVA a été désactivé se corrige** ([#882](https://github.com/guycorbaz/kesh/issues/882)) :
  la ligne affiche son taux « (désactivé) » au lieu d'un champ vide, et le refus dit de choisir un taux actif.
- **Les dates des écrans comptables s'affichent `29.01.2026`** ([#880](https://github.com/guycorbaz/kesh/issues/880)) : journal,
  exercices, relevés bancaires, rapprochement et clés d'API.
- Points de forme de la revue de la 0.17.1 ([#883](https://github.com/guycorbaz/kesh/issues/883)).

`### Ajouté`
- **La date d'une facture fournisseur créée depuis un document est proposée** ([#885](https://github.com/guycorbaz/kesh/issues/885)) :
  la date du jour, **marquée « à vérifier »** jusqu'à ce que vous la confirmiez ou la changiez ; l'échéance suit la date, plus le délai
  de paiement du fournisseur s'il en a un. Une échéance antérieure à la date reste signalée.

`refs #876`, `refs #840`, `refs #870`, `refs #851`, `refs #882`, `refs #880`, `refs #883`, `refs #885` aux commits ; **aucun mot-clé de
fermeture** (contrôle : `git log origin/main..HEAD --format=%B | /usr/bin/grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
vide). La PR de release porte les `closes`.

## Dev Notes

- **Jamais de donnée réelle** : fixtures CAMT/CSV synthétiques (celles de `bank_imports_e2e.rs`), contacts et factures de démonstration.
- **#876, pourquoi aucun test ne l'a vu** : les E2E durent moins de quinze minutes par spec, et le seul test du renouvellement
  (`api-client.test.ts:204-`) simule un refresh **qui réussit** — il n'a jamais confronté les deux durées.
- **#876, la course résiduelle sans `navigator.locks`** : deux onglets qui décident à la même milliseconde peuvent encore présenter
  le même cookie ; la détection de vol déconnecterait alors tout. Les navigateurs visés par Kesh (Chromium, Firefox, Safari ≥ 15.4)
  portent l'API ; le repli est pour jsdom. À déclarer au Dev Agent Record, pas à résoudre.
- **#840** : le verrou est celui du rapprochement — un import attend donc un rapprochement en cours sur le même compte (≤ 5 s), et
  inversement. C'est le comportement de la saisie manuelle depuis la 0.16.
- **#851** : ne pas toucher `expandedState` ni la persistance des groupes.
- **#880** : ne pas toucher aux `<input type="date">` ni aux valeurs envoyées à l'API.
- **Aucune migration** : `ls crates/kesh-db/migrations/*.sql | wc -l` = 86 au dernier commit ; pas de ligne à
  `docs/migrations-idempotence-audit.md`.

## Change Log

- 2026-11-04 — fiche créée (Opus 5.5, autonomie). Prémisses établies au code (`d7ec2ea2`) : #876 **confirmée et aggravée** (le
  renouvellement sur 401 échoue toujours avec 15/15, § 1-4) ; #840 confirmée (§ 8) ; #870 confirmée, serveur déjà paginé (§ 10) ;
  #851 : hypothèse de l'issue (rôle nul) **réfutée**, cause lue au code = débordement de la barre latérale (§ 13-14), à reproduire ;
  #882 confirmée, le serveur refuse aussi (§ 15-16) ; #880 inventaire clos de 24 sites (§ 23-26) ; #883 : .2 **sans objet**
  (aucun Prettier, § 28) ; #885 : prémisse « l'échéance vient des conditions du fournisseur » **fausse** (§ 20), date du jour en
  UTC (§ 19). Découpage en trois sous-stories A/B/C (C-v0172b-1). Registre C-v0172b-1 à 22. Aucune migration. Prochaine :
  validation P1 (Sonnet, lentille complète, `v0172-correctifs-validate-prompt-p1.md`).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
