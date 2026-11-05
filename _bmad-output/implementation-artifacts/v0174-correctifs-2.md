# Story v0174-correctifs-2 : le serveur — les bornes du verrou de période (#869), le pays d'une adresse (#817), le verrou du compte rendu au pool (#842, #540), la date de la 0.17.1 (#901)

Status: ready-for-dev

*Fiche sœur de `v0174-correctifs.md` (l'index), rédigée le 2026-11-05 en autonomie (Opus 5.5), worktree
`/home/gcorbaz/devel/kesh-0174c`, branche `story/v0174-correctifs` partie de `7bd4d5c0`. Release **0.17.4**, **sans
migration**. Registre : `v0174-choix-autonomes-v0174c.md`, choix C-v0174c-8 à 11 et 20. Clé de statut `v0174-correctifs-2`.
Issues **#869** (P3), **#817** (P2), **#842** (P3), **#540** (P3, intégrée, C-v0174c-11), **#901** (P4), en `refs`
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
migration.** Modules : `kesh-api`, `kesh-qrbill`, `kesh-db`, `kesh-reconciliation` (quatre). ⚠️ **Exception `kesh-db`** : #842
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
change pas. `docs/api-external.md` : la phrase de `:493` est récrite (« Les **quatre autres** … 11 lignes en tout, 7 avec
`details` et 4 sans — recompté depuis le tableau `:466-478` ») et la phrase qui suit (« dans trois cas sur cinq ») aussi ; une
ligne `400 PERIOD_LOCKED` entre au tableau du § 10, *(depuis la v0.17.4)*.

### Acceptance Criteria (2.1)

**AC-2.1** — Dans `crates/kesh-api/tests/period_lock_e2e.rs` (montage existant `setup`, `make_entry`) : une écriture créée dans
la période verrouillée rend `400 PERIOD_LOCKED` **avec** `details.lockedThrough` = la borne, `details.attempted` = la date de
l'écriture ; le `message` est inchangé (assertion d'égalité à la valeur d'aujourd'hui). **Rouge avant correctif.**
**AC-2.2** — Le repassage en brouillon d'une facture fournisseur dont l'achat est sous le verrou
(`POST /api/v1/supplier-invoices/{id}/unvalidate`) rend les mêmes `details` (`supplier_invoice_draft_e2e.rs` ou fichier voisin
qui monte déjà une facture validée). **Rouge avant.**
**AC-2.3** — Les tests qui figent le corps d'un `PERIOD_LOCKED` (`invoice_delete_e2e.rs:327`,
`journal_entry_reversal_e2e.rs:1882`, `:1891`, `:2298-2304`, `:2595`) restent verts sans changement (ils lisent `code`).

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
   `validate_optional`), appelée par la création (`:640`) et la mise à jour (`:708`) ; l'assistant d'installation
   (`routes/onboarding.rs:532`, `validate_required`). **Aucune route ne modifie l'adresse de la société après l'assistant**
   (`routes/companies.rs` : `update_company_email` `:102`, `update_company_contact_details` `:240`, le verrou `:395`, `:408`) —
   prémisse de l'issue confirmée.
7. La lecture ne valide rien : `From<&StructuredAddress> for StructuredAddressInput` (`address_input.rs:110-120`) recopie le
   pays stocké.
8. Écrans : `contacts/+page.svelte:995` (`maxlength={2}`, `placeholder="CH"`) et `SupplierCreatePanel.svelte:210`
   (`maxlength="2"`) ; l'assistant (manuel `user-manual.tex:213` : « l'assistant affiche le motif que le serveur donne »).

### Conception (C-v0174c-9)

`is_iso_3166_alpha2` devient `pub` (réexportée `kesh_qrbill::is_iso_3166_alpha2`, documentée `///`) ; `check_lengths` la
consulte **après** le contrôle de forme : refus `AppError::Validation` (`400 VALIDATION_ERROR`) au message « adresse : le pays
« {code} » n'est pas un code ISO 3166-1 alpha-2 connu (ex. CH, FR, DE) ». Le message de forme (`:68-70`) reste. Lecture
inchangée. Une adresse stockée invalide : lue, listée, affichée ; toute **écriture** de ce contact exige un pays valide — le
refus le dit (C-v0174c-9, écartée b). Écrans : le refus s'affiche par leur chemin d'erreur existant — le développeur le
vérifie par un test Vitest par écran (contacts, panneau fournisseur ; mock `400 VALIDATION_ERROR` au message ci-dessus) : si
le message est déjà affiché, le test le fige ; sinon, correction.

### Acceptance Criteria (2.2)

**AC-2.4** — Tests unitaires d'`address_input.rs` : `CH`, `ch`, ` li `, vide (→ `CH`) acceptés ; `SU`, `XX`, `ZZ`, `UK`
refusés, le message nomme le code ; `C1`, `CHE` refusés par le contrôle de forme (message d'aujourd'hui). **Rouges avant**
(`SU`, `XX`, `ZZ`, `UK`).
**AC-2.5** — E2E (`crates/kesh-api/tests/`, fichier de contacts voisin ou neuf `contact_address_country_e2e.rs`) : `POST
/api/v1/contacts` avec `country: "XX"` → `400 VALIDATION_ERROR`, rien d'écrit ; avec `"FR"` → `201`.
**AC-2.6 — Les données existantes** : un contact dont l'adresse porte `SU`, posé par SQL au montage (comme une installation
0.17.3) : `GET /api/v1/contacts/{id}` et la liste rendent `200` et `"country": "SU"` ; `PUT` qui ne change que le téléphone,
adresse renvoyée telle quelle → `400 VALIDATION_ERROR` nommant `SU`, contact inchangé en base ; `PUT` avec `CH` → `200`.
**AC-2.7** — Assistant : l'étape des coordonnées avec `country: "XX"` → `400` (`onboarding_e2e.rs`) ; rien d'écrit.
**AC-2.8** — Vitest : les deux écrans affichent le message du refus (AC de conception).

**Mutation (P2, nommée)** : **MB-817** — `check_lengths` n'appelle pas `is_iso_3166_alpha2` : AC-2.4, AC-2.5, AC-2.6 rougissent.

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
`errors.rs:911-` (doc de `AccountLocked`) relu. `create_pool` (`pool.rs:28`) applique **aussi** la fonction (même
comportement pour tout pool de `kesh-db`).

### Acceptance Criteria (2.3)

Les tests montent un pool **avec** la fonction, d'**une** connexion (pour que la connexion réacquise soit celle qui a servi),
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
2. #869 : corps du `PERIOD_LOCKED` ; `docs/api-external.md` (`:493`, phrase suivante, ligne au § 10).
3. #817 : `kesh-qrbill` (`pub`, réexport, doc) ; `address_input.rs` ; Vitest des deux écrans ; manuel.
4. #842/#540 : la fonction de `pool.rs`, appliquée par `create_pool` et `main.rs` ; commentaires de `mutex.rs`.
5. #901 : la ligne du CHANGELOG.
6. Mutations MB-869, MB-817, MB-842-a, MB-540 ; gates (fiche-index).

## Dev Notes

- **Pattern batch** : inchangé — `PERIOD_LOCKED` par proposition garde sa forme (`reconciliation.rs:572-595`).
- **#817** : le message est en français (`AppError::Validation`), comme tous les refus de forme de l'adresse ; le manuel le dit
  déjà (`user-manual.tex:213`, #710).
- **#842** : la fonction s'applique à **toute** restitution — une instruction de plus par connexion rendue ; mesurer n'est pas
  demandé, le déclarer l'est. `RELEASE_ALL_LOCKS` existe depuis MariaDB 10.5 ; la CI est en `mariadb:10.11` (`.github/workflows/ci.yml:50`) ;
  l'image de dev se relit dans `docker-compose.dev.yml` et se cite au Dev Agent Record.
- **Ne pas** fermer la connexion (`Ok(false)`) comme le pool du MCP : le pool de l'API rouvrirait une connexion par requête.
- **Captures** (thème clair, base de démonstration jetable `kesh_v0174c`, `kesh-gate-logs/v0174c/captures/`) : le refus d'un
  contact au pays `XX` (écran des contacts) et le même refus dans le panneau de création d'un fournisseur.

## Change Log

- 2026-11-05 — fiche créée (Opus 5.5, autonomie). Prémisses : #869 **confirmée** (site unique, `errors.rs:4248`) ; #817
  **confirmée** (forme seule, liste privée dans `kesh-qrbill`, aucune route d'adresse de la société) ; #842 **confirmée et
  précisée** (la doc de `mutex.rs` affirme une libération « à fin de session » qui n'a pas lieu : le pool garde la session) ;
  #901 **confirmée** (seul le CHANGELOG porte la date). #540 intégrée (mécanique, C-v0174c-11). Aucune migration.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
