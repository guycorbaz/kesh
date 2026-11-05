# Story v0174-correctifs-2 : le serveur — les bornes du verrou de période (#869), le pays d'une adresse (#817), le verrou du compte rendu au pool (#842, #540), la date de la 0.17.1 (#901)

Status: review

*Fiche sœur de `v0174-correctifs.md` (l'index), rédigée le 2026-11-05 en autonomie (Opus 5.5), worktree
`/home/gcorbaz/devel/kesh-0174c`, branche `story/v0174-correctifs` partie de `7bd4d5c0`. Release **0.17.4**, **sans
migration**. Registre : `v0174-choix-autonomes-v0174c.md`, choix C-v0174c-8 à 11 et 20. Clé de statut `v0174-correctifs-2`.
Issues **#869** (P3), **#817** (P2), **#842** (P3), **#540** (P2, intégrée, C-v0174c-11), **#901** (P4), en `refs`
seulement. Numéros de ligne de `7bd4d5c0`. **À développer avant la fiche 3**, dont l'écran de #869 lit les `details`
posés ici.*

## Story

En tant que Comptable qui se voit refuser un geste sous le verrou de période, je veux que la réponse dise jusqu'à quand les
livres sont fermés ; en tant que Comptable qui saisit une adresse, je veux qu'un pays inconnu (« SU », « XX ») soit refusé à
la saisie, non à la QR-facture ; en tant qu'Administrateur, je veux qu'une requête interrompue ne laisse pas un compte
bancaire bloqué pour une demi-heure, ni une connexion sans contrôle des clés étrangères ; en tant que lecteur du CHANGELOG, je
veux la vraie date de la 0.17.1.

## Contrainte

Production 0.17.3 : fixtures synthétiques, bases `#[sqlx::test]` jetables (squash `crates/kesh-db/test-schema`). **Aucune
migration.** Modules : `kesh-api`, `kesh-qrbill`, `kesh-db`, `kesh-reconciliation` (quatre ; la part d'écran de #817 est
portée par la fiche 3, § 3.5, qui touche déjà le frontend). ⚠️ **Exception `kesh-db`** : #842
touche le pool — **gate complet même en cours de boucle** (`CLAUDE.md`, § *Pendant une boucle de revue*).

---

## 2.1 — #869 : `PERIOD_LOCKED` porte les bornes du verrou

### Ce que le code dit

1. Tous les `400 PERIOD_LOCKED` de l'API naissent de `DbError::PeriodLocked { locked_through, attempted }`
   (`crates/kesh-db/src/errors.rs:1283-1289`), traduit à **un** site, `crates/kesh-api/src/errors.rs:4248-4264` :
   `build_response(StatusCode::BAD_REQUEST, "PERIOD_LOCKED", &t_args("journal-entries-period-locked", …))` ;
   `build_response` (`:1367-1378`) ne porte que `code` et `message`. **Prémisse confirmée** : aucun `details`. Le message, lui,
   nomme déjà les deux dates (fr-CH `crates/kesh-i18n/locales/fr-CH/messages.ftl:438`), en `AAAA-MM-JJ`.
2. L'acceptation par lot les porte déjà, par proposition : `period_locked_failed_proposal`
   (`routes/reconciliation.rs:572-595`) — `lockedThrough`, `attempted`, `projectId`.
3. L'écran du repassage en brouillon d'une facture fournisseur compose son texte par code
   (`frontend/src/lib/features/supplier-invoices/supplier-invoice-unvalidate.ts:154-158`) : sans date.
4. `docs/api-external.md:493` : « Les **cinq autres** codes du tableau des refus — 11 lignes en tout, 6 avec `details` et 5
   sans — `FISCAL_YEAR_CLOSED`, `ENTRY_IS_REVERSED`, `PERIOD_LOCKED`, … — n'émettent **aucun** `details`. » ;
   § 10 *Gestion des erreurs* (`:937-`) n'a pas de ligne `PERIOD_LOCKED`. Les autres mentions
   (`/usr/bin/grep -n 'PERIOD_LOCKED' docs/api-external.md` → `:295`, `:300`, `:311`, `:333`, `:336`, `:477`, `:493`, `:516`,
   `:538`, `:570`, `:572`, `:594`, `:602`, `:650`, `:677`) ne disent rien des `details`.

### Conception (C-v0174c-8)

Au site `errors.rs:4248`, un corps `{ "error": { "code": "PERIOD_LOCKED", "message": …, "details": { "lockedThrough":
"AAAA-MM-JJ", "attempted": "AAAA-MM-JJ" } } }`, sur le patron de `later_fiscal_year_closed_response` (`:1385-`). Le message ne
change pas. Le registre écarte le champ « exercice » que l'issue mentionne entre parenthèses (C-v0174c-8 : la date du verrou
est la borne d'une période, non d'un exercice ; le refus d'exercice a ses propres codes). `docs/api-external.md` :
- `:493` : le **début** de la phrase change — « Les **quatre autres** codes … 11 lignes en tout, 7 avec `details` et 4 sans » —
  **et sa liste**, d'où `PERIOD_LOCKED` sort (restent `FISCAL_YEAR_CLOSED`, `ENTRY_IS_REVERSED`, `OPTIMISTIC_LOCK_CONFLICT`,
  `ILLEGAL_STATE_TRANSITION`) ; la phrase est relue **en entière** après correctif
  (recompté depuis le tableau, en-tête `:467`, lignes `:469-479`) ; la fin (« dans trois cas sur cinq ») compte les cinq lignes
  du tableau `documentId` / `documentNumber`, dont trois portent la facture elle-même : elle **reste vraie** et ne change pas ;
- `:298` (refus du `PUT` d'une écriture) et `:336` (« Les `details` sont ceux du `PUT` … ») : une incise « `PERIOD_LOCKED` :
  `details.lockedThrough`, `details.attempted` *(depuis la v0.17.4)* » ;
- § 10 (`:937-`) : une ligne `400 PERIOD_LOCKED`, `details.lockedThrough` / `details.attempted` (`AAAA-MM-JJ`), *(depuis la
  v0.17.4)*, qui dit que `failed[]` de `POST /reconciliation/accept` porte déjà ces deux noms (et `projectId`).
Inventaire de contrôle au Dev Agent Record : `/usr/bin/grep -n 'lockedThrough' docs/api-external.md` (vide aujourd'hui) après
correctif, les quatre sites.

### Acceptance Criteria (2.1)

**AC-2.1** — Dans `crates/kesh-api/tests/period_lock_e2e.rs` (montage existant `setup`, `make_entry`) : une écriture créée dans
la période verrouillée rend `400 PERIOD_LOCKED` **avec** `details.lockedThrough` = la borne, `details.attempted` = la date de
l'écriture ; le `message` est inchangé (assertion d'égalité à la valeur d'aujourd'hui). **Rouge avant correctif.**
**AC-2.2** — Le repassage en brouillon d'une facture fournisseur dont l'achat est sous le verrou
(`POST /api/v1/supplier-invoices/{id}/unvalidate`) rend les mêmes `details` (`supplier_invoice_draft_e2e.rs` ou fichier voisin
qui monte déjà une facture validée). **Rouge avant.**
**AC-2.3** — **Tous** les tests de `crates/kesh-api/tests` qui citent `PERIOD_LOCKED`
(`git grep -n PERIOD_LOCKED -- crates/kesh-api/tests`, sortie jointe au Dev Agent Record — dont `invoice_delete_e2e.rs:327`,
`journal_entry_reversal_e2e.rs:1882`, `:1891`, `:2298-2304`, `:2595`, `:3216`, `:3241`, `period_lock_e2e.rs:343`, `:348`)
lisent `code` seul, aucun ne compare le corps entier : ils restent verts sans changement.

**Mutation (P3, nommée)** : **MB-869** — `details` omis : AC-2.1 et AC-2.2 rougissent.

---

## 2.2 — #817 : le pays d'une adresse, contrôlé sur la liste ISO

### Ce que le code dit

5. `StructuredAddressInput::check_lengths` (`crates/kesh-api/src/address_input.rs:52-73`) refuse un pays qui n'a pas deux
   lettres ASCII (`:66-71`), après normalisation (`trim`, majuscules, défaut `CH`, `:37-43`). **Prémisse confirmée** : « SU »,
   « XX » passent. La liste ISO 3166-1 alpha-2 vit, **privée**, dans `crates/kesh-qrbill/src/validation.rs:276`
   (`is_iso_3166_alpha2`, 249 codes — `awk '/const ISO_3166_ALPHA2/,/\];/' … | grep -oE '"[A-Z]{2}"' | wc -l` → 249 —, triée,
   `binary_search`, garde `debug_assert!` de l'ordre), appelée par `validate_country` (`:258-270`) à la production de la
   QR-facture. `kesh-api` dépend déjà de `kesh-qrbill` (`crates/kesh-api/Cargo.toml:12`).
6. Les écrivains d'une adresse structurée : `validate_common` des contacts (`routes/contacts.rs:392`,
   `validate_optional`), appelée par la création (`:641`) et la mise à jour (`:708`) ; l'assistant d'installation
   (`routes/onboarding.rs:532`, `validate_required`). **Aucune route ne modifie l'adresse de la société après l'assistant**
   (`routes/companies.rs` : `update_company_email` `:102`, `update_company_contact_details` `:240`, le verrou `:395`, `:408`) —
   prémisse de l'issue confirmée.
7. La lecture ne valide rien : `From<&StructuredAddress> for StructuredAddressInput` (`address_input.rs:110-120`) recopie le
   pays stocké.
8. Écrans : `contacts/+page.svelte:995` (`maxlength={2}`, `placeholder="CH"`) et `SupplierCreatePanel.svelte:210`
   (`maxlength="2"`) ; l'assistant (manuel `user-manual.tex:213` : « l'assistant affiche le motif que le serveur donne »).
   Les deux écrans affichent déjà `err.message` (`contacts/+page.svelte` : `formError = err.message` ;
   `SupplierCreatePanel.svelte` : `error = (isApiError(err) && err.message)`).
8 bis. `update_contact` **lit déjà** la ligne avant d'écrire (`routes/contacts.rs:704-706`,
   `find_by_id_in_company`, contrôle d'appartenance) : le pays stocké (`address_country`, `entities/contact.rs:190`) est
   disponible sans lecture de plus. L'issue l'exige : « une adresse déjà enregistrée avec un code invalide ne doit pas devenir
   impossible à modifier ni à relire ».

### Conception (C-v0174c-9)

`is_iso_3166_alpha2` devient `pub` (réexportée `kesh_qrbill::is_iso_3166_alpha2`, documentée `///`) ; `check_lengths` la
consulte **après** le contrôle de forme : refus `AppError::Validation` (`400 VALIDATION_ERROR`) au message « adresse : le pays
« {code} » n'est pas un code ISO 3166-1 alpha-2 connu (ex. CH, FR, DE) ». Le message de forme (`:68-70`) reste. Lecture
inchangée. **Pays inchangé toléré** (C-v0174c-9, révisé à la validation P1) : `validate_common` reçoit le pays stocké
(`Some(_existing.address_country)` à la mise à jour, `None` à la création) et le contrôle ISO accepte un code **égal**, après
normalisation (majuscules, `trim`), au pays stocké ; un pays **nouveau ou modifié** hors liste est refusé. Une adresse stockée
invalide se lit, se liste, s'affiche, et le contact se modifie (téléphone, courriel…) sans toucher au pays ; changer le pays
exige un code valide. La création et l'assistant (aucune valeur stockée) refusent tout code hors liste. **Le signal à
l'écran** : `ContactResponse` porte un champ additif `addressCountryKnown: bool` (calculé par `is_iso_3166_alpha2` sur le pays
lu ; `true` sans adresse) — l'écran des contacts l'affiche (fiche 3, § 3.5) sans recopier la liste ISO. Le message reste en
français, comme les refus de forme de l'adresse (`AppError::Validation`, manuel `:213`, #710).

### Acceptance Criteria (2.2)

**AC-2.4** — Tests unitaires d'`address_input.rs` : `CH`, `ch`, ` li `, vide (→ `CH`) acceptés ; `SU`, `XX`, `ZZ`, `UK`
refusés, le message nomme le code ; `C1`, `CHE` refusés par le contrôle de forme (message d'aujourd'hui). **Rouges avant**
(`SU`, `XX`, `ZZ`, `UK`).
**AC-2.5** — E2E (`crates/kesh-api/tests/`, fichier de contacts voisin ou neuf `contact_address_country_e2e.rs`) : `POST
/api/v1/contacts` avec `country: "XX"` → `400 VALIDATION_ERROR`, rien d'écrit ; avec `"FR"` → `201`.
**AC-2.6 — Les données existantes** : un contact dont l'adresse porte `SU`, posé par SQL au montage (comme une installation
0.17.3) : `GET /api/v1/contacts/{id}` et la liste rendent `200`, `"country": "SU"` et `addressCountryKnown: false` ; `PUT` qui
ne change que le téléphone, adresse renvoyée telle quelle (`SU`, ou `su`) → `200`, pays inchangé ; `PUT` avec `XX` →
`400 VALIDATION_ERROR` nommant `XX`, contact inchangé en base ; `PUT` avec `CH` → `200`, `addressCountryKnown: true`. **Rouge
avant** : `addressCountryKnown` absent, `XX` accepté.
**AC-2.7** — Assistant : l'étape des coordonnées avec `country: "XX"` → `400` (`onboarding_e2e.rs`) ; rien d'écrit.
**AC-2.8** — Côté écran : fiche 3, § 3.5 (AC-3.14, AC-3.15).

**Mutations (P2, nommées)** : **MB-817** — le contrôle n'appelle pas `is_iso_3166_alpha2` : AC-2.4, AC-2.5, AC-2.6 (`XX`)
rougissent. **MB-817-b** — la tolérance du pays stocké est retirée (`None` passé à la mise à jour) : AC-2.6 (`PUT` du
téléphone sur `SU`) rougit. **MB-817-c** — la tolérance accepte tout code à la mise à jour : AC-2.6 (`XX`) rougit.

---

## 2.3 — #842 (et #540) : la session est remise à zéro au retour au pool

### Ce que le code dit

9. `with_account_lock` (`crates/kesh-reconciliation/src/mutex.rs:66-`) prend `GET_LOCK` (`:89-94`) sur la connexion de la
   `Transaction` qu'on lui prête, exécute la fermeture (`:124`), puis `RELEASE_LOCK` (`:131-135`). Deux chemins laissent le
   verrou tenu : **(a)** la future est abandonnée pendant la fermeture (client déconnecté : Axum abandonne la future du
   gestionnaire) — `RELEASE_LOCK` ne s'exécute jamais ; **(b)** `RELEASE_LOCK` ne rend pas 1 ou échoue —
   `LockReleaseFailed` (`:137-170`), l'appelant abandonne la transaction. Dans les deux cas, la `Transaction` abandonnée
   rend sa connexion au pool (`sqlx-core-0.8.6/src/pool/connection.rs`, `return_to_pool` : `after_release` s'il existe, puis
   `ping`, puis remise au pool) ; `GET_LOCK` est **de session** et survit au `ROLLBACK`. La documentation l'ignore :
   « le lock advisory sera libéré à fin de session MariaDB cf. L22 » (`:61-64`), « lock will leak until session end »
   (`:150`, `:165`) — la session ne finit pas, le pool la garde. **Prémisse confirmée.** Huit appels hors tests
   (`routes/bank_imports.rs:1461` — partagé par la saisie manuelle et les deux imports —, `routes/reconciliation.rs:1761`,
   `:4468`, `:4966`, `:5456`, `:5789`, `:5979`, `:6282`).
10. Le pool de production est construit dans `crates/kesh-api/src/main.rs:82-85` (`MySqlPoolOptions::new().max_connections(5)
    .acquire_timeout(…)`), **sans** crochet ni `create_pool` (`crates/kesh-db/src/pool.rs:28-`, qui n'est pas appelé par
    `main.rs`). Le pool du MCP ferme déjà chaque connexion à sa restitution (`after_release` → `Ok(false)`, doc `pool.rs:70-`,
    `:114`) : c'est le précédent du crochet.
11. **#540** : la restauration (`crates/kesh-db/src/backup.rs:611-650`) désactive `FOREIGN_KEY_CHECKS` sur sa connexion et le
    rétablit à la fin ; une future abandonnée rend la connexion avec `FOREIGN_KEY_CHECKS=0`. `reset_demo` ferme déjà la sienne
    (commentaire de #540) ; reste la restauration.

### Conception (C-v0174c-10, 11)

Dans `kesh-db/src/pool.rs`, une fonction publique documentée, par exemple
`pub fn avec_remise_a_zero_de_session(options: MySqlPoolOptions) -> MySqlPoolOptions`, qui pose un `after_release` :
`SELECT RELEASE_ALL_LOCKS()` puis `SET SESSION foreign_key_checks = 1` (une ou deux instructions, au choix du développeur),
`Ok(true)` ; une erreur rend `Err` (sqlx ferme alors la connexion, `close_hard`, et la fin de session libère tout).
`main.rs:82` l'applique au pool de production. `mutex.rs` : l'en-tête (`:30-50`, motif d'appel), la doc (`:61-64`) et les deux
messages de journal (`:150`, `:165`) disent désormais que le verrou est rendu **au retour au pool** de la connexion ;
les autres sites du même symptôme aussi — `crates/kesh-reconciliation/src/errors.rs:19-25` (doc **et** message
`#[error(… lock may persist until session end)]` de `LockReleaseFailed`), `crates/kesh-api/src/errors.rs:919-921` (doc de
`ReconciliationLockReleaseFailed`), `crates/kesh-api/src/routes/reconciliation.rs:1809-1810` (« Lock advisory libéré à fin de
session ») ; inventaire de contrôle, **deux** greps rejoués au Dev Agent Record : `/usr/bin/grep -rn 'fin de session\|session end\|L22'
crates --include=*.rs` (seuls restent des sites sans rapport : `config.rs:202`, la session d'authentification) et
`/usr/bin/grep -rn 'rendue au pool\|retour au pool\|connexion au pool\|suit la session' crates --include=*.rs` (validation P3, P4). `create_pool` (`pool.rs:28`) applique **aussi** la fonction (même
comportement pour tout pool de `kesh-db`).

### Acceptance Criteria (2.3)

Les tests neufs montent le squash, comme l'exige `crates/kesh-db/tests/test_schema_guard.rs` (`:34`, `:921`) :
`#[sqlx::test(migrations = "./test-schema")]` dans `kesh-db`, `#[sqlx::test(migrations = "../kesh-db/test-schema")]` dans
`kesh-api`. Ils montent un pool **avec** la fonction, d'**une** connexion (pour que la connexion réacquise soit celle qui a servi),
depuis les options de la base de test (`MySqlPoolOptions::new().max_connections(1).connect_with((*pool.connect_options()).clone())`,
patron de `backup.rs:1772-1778`), et une **seconde** connexion indépendante pour observer :

**AC-2.9 — Verrou de session rendu** (`crates/kesh-db/tests/pool_session_reset.rs`, neuf) : connexion empruntée,
`SELECT GET_LOCK('kesh-test-<base>', 0)` = 1, puis abandonnée **sans** `RELEASE_LOCK` ; depuis la seconde connexion,
`IS_FREE_LOCK('kesh-test-<base>')` vaut 1 au plus tard 2 s après (sondage à 50 ms, la restitution étant asynchrone). Rouge
**sans** la fonction (témoin dans le même test : le même montage sans crochet laisse le verrou tenu au-delà de 2 s).
**AC-2.10 — `FOREIGN_KEY_CHECKS` rétabli** (même fichier) : `SET SESSION foreign_key_checks = 0`, connexion abandonnée,
réacquise : `SELECT @@session.foreign_key_checks` = 1. Témoin sans crochet : 0.
**AC-2.11 — La requête abandonnée** (`crates/kesh-api/tests/`, fichier voisin de `reconciliation_e2e.rs` ou neuf
`account_lock_session_reset_e2e.rs`) : sur le pool d'une connexion avec la fonction, `with_account_lock(&mut tx, …)` dont la
fermeture attend indéfiniment, enveloppé dans `tokio::time::timeout(100 ms)` (abandon) ; la transaction abandonnée ; depuis
une connexion indépendante, `GET_LOCK('reconcile:<base>:<société>:<compte>', 0)` = 1 au plus tard 2 s après. Témoin sans
crochet : 0 au-delà de 2 s. **Rouge avant correctif** (aucun crochet n'existe).
**AC-2.12 — `RELEASE_LOCK` raté, angle mort déclaré.** Le chemin (b) n'a pas de montage propre : un `RELEASE_LOCK` qui rend
0 suppose un verrou déjà rendu (rien ne fuit alors), et une erreur de `RELEASE_LOCK` suppose une connexion cassée, que sqlx
ferme. Ce qui compte — une connexion rendue au pool avec un verrou tenu, quelle qu'en soit la cause — est couvert par AC-2.9.
Le Dev Agent Record le déclare ; la revue de code relit `mutex.rs:137-170`.
**AC-2.13** — Les tests de verrou existants (`bank_statements_manual_e2e.rs:1431`, `:1519` ; `bank_imports_e2e.rs`, AC-A7 de
la 0.17.2 ; `reconciliation_e2e.rs:7497-7571`) restent verts sans changement d'assertion.

**Mutations (P3, nommées)** : **MB-842-a** — le crochet n'exécute pas `RELEASE_ALL_LOCKS` : AC-2.9 et AC-2.11 rougissent.
**MB-842-b** — `main.rs` n'applique pas la fonction : aucun test ne le voit (les tests construisent leur pool) — **angle mort
déclaré** ; contrôle de forme à la revue de code (`grep -nF 'avec_remise_a_zero_de_session' crates/kesh-api/src/main.rs`,
ligne citée au Dev Agent Record). **MB-540** — le crochet ne rétablit pas `foreign_key_checks` : AC-2.10 rougit.

---

## 2.4 — #901 : la date de la 0.17.1

12. `CHANGELOG.md:60` : `## [0.17.1] — 2026-11-03` ; tag annoté `v0.17.1` du 2026-11-04 10:35 +0100
    (`git for-each-ref --format='%(taggerdate:iso)' refs/tags/v0.17.1`), release GitHub du 2026-11-04 09:44 UTC. Les autres
    sections de la série sont justes (`v0.17.0` 2026-11-03, `v0.17.2` 2026-11-04, `v0.17.3` 2026-11-05). Aucun manuel, ni le
    README, ni `website/` ne porte la date de la 0.17.1 (`git grep -n '2026-11-03' -- docs/manual README.md website` : vide).

**AC-2.14** — `CHANGELOG.md:60` devient `## [0.17.1] — 2026-11-04` ; rien d'autre de publié ne change (C-v0174c-20). La cause
(`scripts/prepare-release.sh:9-10` date la section à la préparation) est demandée en issue au rapport.

---

## Tâches

1. Rouges d'abord : AC-2.1, 2.2, 2.4 à 2.7, 2.9 à 2.11 (constatés, notés).
2. #869 : corps du `PERIOD_LOCKED` ; `docs/api-external.md` (début et liste de `:493`, incises `:298` et `:336`, ligne au § 10).
3. #817 : `kesh-qrbill` (`pub`, réexport, doc) ; `address_input.rs` ; tolérance du pays stocké dans `validate_common` ;
   `addressCountryKnown` ; manuel.
4. #842/#540 : la fonction de `pool.rs`, appliquée par `create_pool` et `main.rs` ; commentaires de `mutex.rs`,
   `kesh-reconciliation/src/errors.rs`, `kesh-api/src/errors.rs:919-921`, `routes/reconciliation.rs:1809-1810`, et deux sites
   du même symptôme écrit autrement (validation P2, P11) : `routes/bank_imports.rs:1479-1481` (« Le verrou suit la session de
   la connexion », dans le bras d'échec du `RELEASE_LOCK`, après `drop(lock_tx)`, `:1472`) → « rendu au retour de la connexion au pool (`avec_remise_a_zero_de_session`) » ;
   `crates/kesh-db/src/backup.rs:618-620` (« rétabli systématiquement … pour ne jamais rendre une connexion au pool avec les FK
   désactivées ») → « rétabli ici ; à l'abandon d'une requête, par le crochet du pool » ; et `crates/kesh-api/src/inbox_import.rs:177-186`
   (« une connexion rendue au pool garderait le verrou dans sa session ») — devenu faux avec le crochet ; la justification de
   `close_on_drop` se récrit en défense en profondeur (la connexion du verrou n'est jamais réutilisée), `close_on_drop` reste. Le grep de contrôle de la conception
   (`fin de session|session end|L22`) ne les voit pas : ils se relisent nommément.
5. #901 : la ligne du CHANGELOG.
6. Mutations MB-869, MB-817, MB-817-b, MB-817-c, MB-842-a, MB-540 ; gates (fiche-index).

## Dev Notes

- **Pattern batch** : inchangé — `PERIOD_LOCKED` par proposition garde sa forme (`reconciliation.rs:572-595`).
- **#817, MCP** : l'outil `kesh_contacts` a son propre DTO (`crates/kesh-api/src/mcp/outils/contacts.rs:117`, `:185`,
  `pays`) et **ne porte pas** `addressCountryKnown` — choix assumé (le MCP ne prépare pas de QR-facture), écrit au Dev Agent
  Record.
- **#817** : le message est en français (`AppError::Validation`), comme tous les refus de forme de l'adresse ; le manuel le dit
  déjà (`user-manual.tex:213`, #710).
- **#842** : la fonction s'applique à **toute** restitution — une instruction de plus par connexion rendue ; mesurer n'est pas
  demandé, le déclarer l'est. `RELEASE_ALL_LOCKS` existe depuis MariaDB 10.5 ; la CI est en `mariadb:10.11` (`.github/workflows/ci.yml:50`) ;
  l'image de dev se relit dans `docker-compose.dev.yml` et se cite au Dev Agent Record.
- **Ne pas** fermer la connexion (`Ok(false)`) comme le pool du MCP : le pool de l'API rouvrirait une connexion par requête.
- **Captures** : celles de #817 sont à la fiche 3 (§ 3.5).
- **Angles morts déclarés, issue demandée** (pour qu'ils ne restent pas seulement écrits ici) : AC-2.12 (chemin `RELEASE_LOCK`
  raté sans montage propre) et MB-842-b (l'application de la fonction par `main.rs` n'est vue par aucun test, les tests
  construisant leur pool).

## Change Log

- 2026-11-05 — fiche créée (Opus 5.5, autonomie). Prémisses : #869 **confirmée** (site unique, `errors.rs:4248`) ; #817
  **confirmée** (forme seule, liste privée dans `kesh-qrbill`, aucune route d'adresse de la société) ; #842 **confirmée et
  précisée** (la doc de `mutex.rs` affirme une libération « à fin de session » qui n'a pas lieu : le pool garde la session) ;
  #901 **confirmée** (seul le CHANGELOG porte la date). #540 intégrée (mécanique, C-v0174c-11). Aucune migration.
- 2026-11-05 — **Validation P1 (Sonnet, R et F)** : sur cette fiche, R2 (MEDIUM, #817 : prémisse « relire la ligne » fausse —
  `update_contact` la lit déjà, `contacts.rs:704-706` — et contrainte de l'issue) → pays inchangé toléré, `addressCountryKnown`,
  MB-817-b et c, écran en fiche 3 § 3.5 ; F1 (la fin de `api-external.md:493` reste vraie), F2 (`:298`, `:336`, § 10),
  F3 (symptôme « fin de session » à trois sites de plus) ; LOW R5 (squash), R8 (« et l'exercice » écarté, C-v0174c-8), R10 = F9
  (quatre modules, l'écran passe à la fiche 3), F6 (#540 P2), F8 (`:641`, `:466-478` → `:467`/`:469-479`), F10 (angles morts
  demandés en issue).
- 2026-11-05 — **Validation P2 (Opus, lentille complète)** : sur cette fiche, LOW seulement — P5 (tâche 2 alignée sur la
  conception de `api-external.md`), P10 (AC-2.3 : inventaire clos par `git grep`), P11 (deux sites du symptôme « fin de
  session » écrits autrement : `bank_imports.rs:1478-1481`, `backup.rs:618-620`), P15 (le MCP ne porte pas
  `addressCountryKnown`, dit ; « Pour l'API » à la puce CHANGELOG, index).
- 2026-11-05 — **Validation P3 (Sonnet, passe ciblée)** : LOW seulement — P3 (`bank_imports.rs:1479-1481`, après `drop(lock_tx)`),
  P4 (`inbox_import.rs:177-186`, grep de contrôle élargi), P5 (liste de `api-external.md:493` : `PERIOD_LOCKED` retiré).
  *(Ces deux entrées, écrites sous « File List », sont déplacées au développement vers ce Change Log ; texte inchangé.)*
- 2026-11-05 — **Développement** (Opus 5.5, autonomie), commit `524fda2a`. Étape 0 : `RELEASE_ALL_LOCKS` sur l'image de dev —
  `docker-compose.dev.yml:56` `image: mariadb:10.11` ; joué sur le conteneur (`etape0-release-all-locks.log`) :
  `SELECT VERSION(), GET_LOCK('kesh-v0174c-sonde',0), RELEASE_ALL_LOCKS(), IS_FREE_LOCK('kesh-v0174c-sonde')` →
  `10.11.19-MariaDB-ubu2204  1  1  1`. LOW P2 de la validation P4 : chaque site du second grep classé (Completion Notes).
  Statut → review.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`claude-opus-5-5`), autonomie, worktree `/home/gcorbaz/devel/kesh-0174c`.

### Completion Notes List

- **Rouges d'abord** (`kesh-gate-logs/v0174c/rouges.log`, `rouges-fiche2.log`) : 6 rouges / 8 — `unknown_countries_…_817`,
  `contact_address_country_e2e` ×2 (`addressCountryKnown` absent, `XX` accepté en `201`), `coordinates_rejects_unknown_iso_
  country_817` (`200`), `period_locked_carries_its_bounds_in_details_869` et `unvalidate_under_the_period_lock_names_its_
  bounds_869` (`details.lockedThrough` = `Null`). `pool_session_reset.rs` et `account_lock_session_reset_e2e.rs` ne
  compilaient pas (fonction absente) ; leurs **témoins sans crochet** sont verts après — ils établissent le défaut (verrou
  tenu au-delà de 2 s, `foreign_key_checks = 0` sur la connexion réacquise).
- **#869** : corps `PERIOD_LOCKED` au site unique (`errors.rs`), message égal à celui d'aujourd'hui (AC-2.1 l'assert
  littéralement). AC-2.3 : `git grep -n PERIOD_LOCKED -- crates/kesh-api/tests` rendait **21 lignes** avant les tests
  neufs (`ac-2-3-period-locked-tests.txt`, relevé au développement avant leur écriture), toutes lisant `code` (ou
  `modificationBlockedBy`), aucune ne comparant le corps entier ; **25** après (les quatre lignes des deux tests neufs
  de #869, `period_lock_e2e.rs`, `supplier_invoice_draft_e2e.rs`). Les 21 restent vertes au gate. `docs/api-external.md` : `grep -n lockedThrough` → `:298`, `:336`, `:493`, `:970` (quatre sites).
- **#817** : `kesh_qrbill::is_iso_3166_alpha2` publique et réexportée ; `check_lengths(stored_country)` ; `validate_optional`
  (création) et `validate_required` (assistant) sans tolérance ; `validate_optional_keeping` (mise à jour,
  `existing.address_country`) ; `is_known_country` → `ContactResponse.address_country_known` (`true` sans adresse). Le DTO
  MCP `kesh_contacts` ne porte pas le champ (choix de la fiche : le MCP ne prépare pas de QR-facture).
- **#842, #540** : `kesh_db::pool::avec_remise_a_zero_de_session` (`DO RELEASE_ALL_LOCKS()`, puis
  `SET SESSION foreign_key_checks = 1`, `Ok(true)` ; une erreur ferme la connexion), posée par `create_pool` et par
  `main.rs` — `grep -nF 'avec_remise_a_zero_de_session' crates/kesh-api/src/main.rs` →
  `85:    let pool = match kesh_db::pool::avec_remise_a_zero_de_session(MySqlPoolOptions::new())` (MB-842-b, angle mort
  déclaré : aucun test ne le voit). Coût : deux instructions par restitution, non mesuré.
- **Les deux greps du symptôme**, rejoués après correctif :
  - `grep -rn 'fin de session\|session end\|L22' crates --include=*.rs` (`grep-842-1.txt`) → 4 lignes : `config.rs:202`
    (session d'authentification, sans rapport), `pool.rs:62`, `:71` (la doc neuve, qui dit pourquoi la fin de session
    n'arrive pas), `mutex.rs:32` (« libéré à la fin de session si l'application s'arrête » : vrai) — **aucun résidu**.
  - `grep -rn 'rendue au pool\|retour au pool\|connexion au pool\|suit la session' crates --include=*.rs`
    (`grep-842-2.txt`) → 30 lignes, classées : **corrigés** — `routes/bank_imports.rs` (bras d'échec du `RELEASE_LOCK`),
    `inbox_import.rs:177-…` (défense en profondeur, `close_on_drop` gardé), `backup.rs:618-…`, `reconciliation.rs:1810`,
    `mutex.rs` (en-tête et doc), `errors.rs` (doc), `inbox_import_e2e.rs:3254-…` (le pool du test, « comme la production »,
    est dit **sans** le crochet : il prouve `close_on_drop` seul) ; **neufs** — `main.rs:82`, `pool.rs:29`, `:55`,
    `pool_session_reset.rs`, `account_lock_session_reset_e2e.rs` ; **sans rapport, restés vrais** —
    `bank_imports.rs:82`, `:1264`, `:1637`, `bank_accounts.rs:1126` (attente des transactions), `snapshot.rs:18`
    (isolation), `backup.rs:2025`, `:2027` (connexion fermée), `kesh-seed/src/lib.rs:471`, `:538` (connexion fermée) ;
    **pools de test sans crochet, restés vrais** — `test_fixtures.rs:393`, `exports_global_e2e.rs:1893` (pools de
    `#[sqlx::test]`).
- **Tests neufs** : `address_input.rs` 5 unitaires ; `contact_address_country_e2e.rs` 2 ; `onboarding_path_b_e2e.rs` 1
  (C-v0174c-27) ; `period_lock_e2e.rs` 1 ; `supplier_invoice_draft_e2e.rs` 1 ; `pool_session_reset.rs` 4 ;
  `account_lock_session_reset_e2e.rs` 2. Ciblé `cible-fiche2.log` : 16/16 verts.
- **Mutations** (`mut/mutations.log`) : MB-869, MB-817, MB-817-b, MB-817-c, MB-842-a (kesh-db et kesh-api), MB-540 —
  **toutes tuées** (7/7). MB-842-b non jouable par un test (angle mort, issue demandée).
- **AC-2.12** (chemin `RELEASE_LOCK` raté) : sans montage propre, déclaré ; couvert par AC-2.9 pour la conséquence.
- **AC-2.14 (#901)** : `## [0.17.1] — 2026-11-04`.
- Gates : au dernier commit de code de la story (exception `kesh-db` : gate complet) — fiche-index § *Dev Agent Record
  (gates)*.

### File List

- `crates/kesh-api/src/errors.rs`, `address_input.rs`, `main.rs`, `inbox_import.rs`
- `crates/kesh-api/src/routes/contacts.rs`, `routes/bank_imports.rs`, `routes/reconciliation.rs`
- `crates/kesh-api/tests/period_lock_e2e.rs`, `supplier_invoice_draft_e2e.rs`, `onboarding_path_b_e2e.rs`,
  `inbox_import_e2e.rs`, `contact_address_country_e2e.rs` (neuf), `account_lock_session_reset_e2e.rs` (neuf)
- `crates/kesh-qrbill/src/lib.rs`, `validation.rs`
- `crates/kesh-db/src/pool.rs`, `backup.rs`, `crates/kesh-db/tests/pool_session_reset.rs` (neuf)
- `crates/kesh-reconciliation/src/mutex.rs`, `errors.rs`
- `docs/api-external.md`, `docs/manual/fr/user-manual.tex` (et PDF), `CHANGELOG.md`
