# Story 15.5a : Le refus « compte non imputable », sous son vrai nom

Status: ready-for-dev

<!-- Issue de la story 15-5, DÉCOUPÉE le 2026-10-08 après la passe de validation P1 (choix C7 de
     `epic-15-choix-autonomes.md`). Sous-story « zéro » du patron « story-zéro qui pose le patron +
     rollout » du CLAUDE.md : elle pose la variante d'erreur que la 15-5b déploiera sur les surfaces
     neuves. Choix applicables : C3 (forme du refus), C7 (découpage). Validation P2 : à lancer. -->

**Issues** : `refs #427`, `refs #429` — **ne ferme rien** (c'est la 15-5b qui ferme #427 et #429).
Elle traite le **commentaire de #429** : toutes les gardes de postabilité refusent aujourd'hui avec
`InactiveOrInvalidAccounts`, « Un ou plusieurs comptes sont archivés ou invalides », message que le
dépôt avait déjà jugé trompeur en 16-1a pour ce motif.

**Dépendances** : aucune. **La 15-5b dépend de cette story** (elle émet la variante posée ici).

## Story

En tant que **comptable ou intégrateur qui appelle Kesh par l'écran ou par clé d'API**,
je veux qu'**un refus pour compte non imputable le dise sous son vrai nom, en nommant le ou les
comptes**,
afin de **savoir quoi corriger** — choisir un sous-compte imputable — au lieu de chercher un compte
« archivé ou invalide » qui ne l'est pas.

### Pourquoi une story à part

La 15-5 d'origine mêlait ce changement de contrat (un code d'erreur neuf, visible des intégrations, et
la réécriture de tous les tests qui figeaient l'ancien) et le déploiement de gardes sur sept surfaces
neuves. La passe P1 a compté plus de cinq modules (C-4/M5) et la seule dérogation codifiée — les cycles
Cargo — ne s'appliquait pas. Cette moitié est le **socle** : la variante, son message, et les quatre
gardes **existantes** (saisie manuelle 14-3b, trois gardes de la 24-5) converties. Elle ne crée aucune
garde neuve.

## Acceptance Criteria

### La variante et son message

1. **AC1 — Une variante dédiée, un code, un détail structuré.** `crates/kesh-db/src/errors.rs` porte
   `DbError::AccountsNotPostable(Vec<String>)` — les **numéros** des comptes refusés, **triés et
   dédupliqués** à la construction (un constructeur `DbError::accounts_not_postable(impl
   IntoIterator<Item = String>)` le garantit ; la variante n'est construite que par lui). Son
   `error_code()` rend `"ACCOUNT_NOT_POSTABLE"`. `crates/kesh-api/src/errors.rs` la mappe en
   **HTTP 400**, code `ACCOUNT_NOT_POSTABLE`, avec un corps
   `{ "error": { "code", "message", "details": { "accountNumbers": ["…"] } } }` (forme du bras
   `InvalidRevenueAccounts`, `errors.rs:~3130`). Les numéros sont exposés sans risque : la variante
   n'est émise que pour un compte **de la société et actif** (anti-énumération KF-002 préservée — un
   compte inconnu ou d'une autre société reste `InactiveOrInvalidAccounts`, ou `404` là où il l'est).
   ⚠️ La clé `accountNumbers` est celle que la 15-5b réutilisera dans `failed[].details` — **une seule
   clé** pour un même refus (finding L2/B4).
2. **AC2 — Le message nomme la cause et les comptes, dans les quatre locales.** Clé neuve
   `error-account-not-postable` dans `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`,
   variables `$numbers` (numéros joints par « , ») et `$count` (nombre de comptes, passé comme
   **nombre** Fluent pour que le sélecteur de pluriel fonctionne), **sélecteur `[one]` / `*[other]`**
   (patron `contact-payment-terms-days-label`, `fr-CH/messages.ftl:440`) et **apostrophe
   typographique** `’` (patron `fr-CH/messages.ftl:975`). Texte FR **exact** :
   ```ftl
   error-account-not-postable = { $count ->
       [one] Le compte { $numbers } n’est pas imputable (compte de regroupement ou de clôture) : choisissez un compte imputable.
      *[other] Les comptes { $numbers } ne sont pas imputables (comptes de regroupement ou de clôture) : choisissez des comptes imputables.
   }
   ```
   Les trois autres locales suivent le vocabulaire déjà en place pour « non imputable »
   (`error-opening-complement-retained-earnings-not-postable`, ligne 925 de chaque fichier) :
   **DE** « bebuchbar », **EN** « postable », **IT** « registrabile ». Aucune ne dit « archivé » ni
   « invalide ». Le repli Rust du `t_args` reprend le texte FR au singulier/pluriel selon `count`.

### Les gardes existantes nomment la cause

3. **AC3 — Saisie manuelle** (`validate_lines_accounts_in_tx`,
   `crates/kesh-db/src/repositories/journal_entries.rs:84`). La requête lit
   `id, number, active, postable` **sans** clause `active` ni `postable` dans le `WHERE` (seulement
   `company_id` et `id IN (…)`), et la décision se prend en Rust, dans cet ordre :
   1. un identifiant demandé **absent** du résultat (inconnu ou d'une autre société) **ou** un compte
      **archivé** → `InactiveOrInvalidAccounts` (inchangé) ;
   2. sinon, si `enforce_postable`, tout compte `postable = FALSE` **hors** `exempt_ids` →
      `DbError::accounts_not_postable(numéros)` nommant **tous** ces comptes.
   Quand les deux défauts coexistent dans une même écriture, **le premier gagne** (patron « une seule
   raison, la plus bloquante d'abord » de `validate_line_revenue_accounts_in_tx`,
   `crates/kesh-db/src/repositories/invoices.rs:631-643`). Le chemin `enforce_postable = false` est
   **strictement inchangé** (même ensemble de comptes acceptés, même erreur). La fonction reste
   rollback-agnostique.
4. **AC4 — Les trois gardes de la 24-5**, qui lisent aujourd'hui `active, postable` et rendent
   `InactiveOrInvalidAccounts` pour tout écart, distinguent désormais, **dans cet ordre** :
   - (a) compte **inconnu, d'une autre société, archivé**, ou — pour le compte de charge seulement —
     **d'un autre type que `Expense`** → `InactiveOrInvalidAccounts` (inchangé) ;
   - (b) sinon, compte **non imputable** → `DbError::accounts_not_postable([numéro])`.
   Un compte qui cumule (a) et (b) — p. ex. un compte d'actif non imputable proposé comme compte de
   charge — rend **(a)** (finding M4). Les trois sites, qui ajoutent `number` à leur `SELECT` :
   - `crates/kesh-db/src/repositories/invoice_settlements_write.rs:156-167` (règlement client, compte
     interne) ;
   - `crates/kesh-db/src/repositories/supplier_invoices.rs:336-347` (compte de charge, **dans une
     boucle sur les lignes**) : une ligne en défaut (a) rend `InactiveOrInvalidAccounts`
     immédiatement ; les comptes en défaut (b) sont **collectés** et, si aucune ligne n'est en (a),
     un seul `AccountsNotPostable` les nomme tous après la boucle ;
   - `crates/kesh-db/src/repositories/supplier_invoices.rs:639-650` (règlement fournisseur, compte
     interne).
5. **AC5 — L'écriture d'ouverture hérite du nom juste ; le complément ne bouge pas.**
   `create_opening_entry` (`journal_entries.rs:634`, `enforce_postable = true`) rend désormais
   `ACCOUNT_NOT_POSTABLE` pour une ligne sur un compte non imputable de la société ; un compte inconnu,
   archivé ou d'une autre société garde `INACTIVE_OR_INVALID_ACCOUNTS` (test existant
   `post_cross_tenant_account_inactive_or_invalid`, `crates/kesh-api/tests/opening_balances_e2e.rs:704`,
   **inchangé et vert**). Le complément de soldes (`opening_complement.rs:641`) n'est **pas** concerné :
   son contrôle préalable (`check_lines`, `:645-660`) rend `OPENING_COMPLEMENT_ACCOUNT_INVALID` avant
   `create_in_tx` — vérifié par son test existant, inchangé.

### Ce que voit l'utilisateur, et ce qui ne change pas

6. **AC6 — Les écrans affichent le message du serveur.** `JournalEntryForm.svelte:178`
   (`frontend/src/lib/features/journal-entries/`) ajoute `case 'ACCOUNT_NOT_POSTABLE':` au groupe qui
   affiche `err.message`. Les autres écrans atteints par les gardes de l'AC4 et de l'AC5 — dialogue de
   règlement d'une facture client, saisie et règlement d'une facture fournisseur, soldes de départ —
   affichent déjà `err.message` pour un code qu'ils ne connaissent pas : **le dev le vérifie** écran par
   écran (lecture du `catch`) et le consigne au Dev Agent Record ; un écran qui afficherait un repli
   générique à la place est corrigé dans cette story.
7. **AC7 — Les tests qui figeaient l'ancien code pour un compte non imputable sont réécrits à dessein**,
   chacun avec un commentaire qui cite la 15-5a :
   - `crates/kesh-db/src/repositories/journal_entries.rs:3676` (`test_create_manual_rejects_non_postable_line`)
     et `:3755` (`test_create_manual_rejects_result_account`) → `AccountsNotPostable` ;
   - `crates/kesh-db/tests/invoice_settlement.rs:354` (`un_compte_non_imputable_est_refuse`) →
     `AccountsNotPostable` ; **`:311` (compte archivé) reste `InactiveOrInvalid`** ;
   - `crates/kesh-db/tests/supplier_invoices_repository.rs:424` (charge non imputable) et `:470`
     (règlement fournisseur, compte interne non imputable) → `AccountsNotPostable` ; **`:388` (mauvais
     type) et `:672` (autre société) restent `InactiveOrInvalidAccounts`** ;
   - `crates/kesh-api/tests/reports_e2e.rs:1953` → `ACCOUNT_NOT_POSTABLE` ; son commentaire, qui dit
     « aucun code d'erreur neuf n'est introduit par cette story » (AC 13 de la 24-5), est réécrit : le
     code neuf est **délibéré** (choix C3) ;
   - et toute autre occurrence que rend la commande de T0, triée : non imputable → nouveau code ;
     inconnu / archivé / autre société / mauvais type → inchangé.
   Les assertions par `format!("{err:?}").contains(…)` sont remplacées par un `matches!` sur la
   variante — un `contains("InactiveOrInvalid")` passe aussi pour un nom de variante voisin.
8. **AC8 — Non-régression.** Restent verts **sans modification** : `test_create_in_tx_auto_flow_allows_non_postable`
   (`journal_entries.rs:3687`, flux automatique sur compte de configuration devenu non imputable) ;
   `reverse_succeeds_when_an_account_became_non_postable` (`crates/kesh-api/tests/journal_entry_reversal_e2e.rs:1015`) ;
   `invoices_validate_vat.rs:301` (compte TVA **archivé** → `InactiveOrInvalidAccounts`) ;
   `opening_balances_e2e.rs:770` ; les refus de rôles absents de `insert_with_defaults_in_tx`,
   `kesh-seed/src/lib.rs:208` et `onboarding.rs:194,727`, qui ne passent par aucune des gardes modifiées.
9. **AC9 — Le contrat changé est écrit.** `CHANGELOG.md`, section `## [0.13.0] — Non publié` (à créer
   en tête si absente — c'est le motif exact qu'exige `scripts/prepare-release.sh:189`), rubrique
   **Modifié** : pour un compte non imputable, la saisie manuelle, l'écriture d'ouverture, le règlement
   par compte interne et la saisie/le règlement d'une facture fournisseur répondent désormais
   `ACCOUNT_NOT_POSTABLE` (avec `details.accountNumbers`) au lieu de `INACTIVE_OR_INVALID_ACCOUNTS` —
   **changement visible d'une intégration par clé d'API** —, et le message nomme le ou les comptes.
   Le **manuel** ne cite ni l'ancien code ni l'ancien message pour ce motif (vérifié au `.tex` **et** au
   PDF aplati, cf. T7) : il n'y a rien à y changer dans cette story — le dire au Dev Agent Record,
   comme contrôle exercé et sans objet. `docs/manual/fr/admin-manual.tex` : sans objet, à écrire de
   même.

## Tasks / Subtasks

- [ ] **T0 — Refaire l'inventaire des lecteurs de l'ancien code** sur `HEAD` et le comparer à l'AC7 :
  ```sh
  grep -rnE "InactiveOrInvalid[^A]|InactiveOrInvalidAccounts|INACTIVE_OR_INVALID_ACCOUNTS" \
    crates frontend/src frontend/tests --include=*.rs --include=*.ts --include=*.svelte
  ```
  (le motif `InactiveOrInvalid[^A]` attrape les `contains("InactiveOrInvalid")` que le motif plein
  rate — finding C-1). Trier **chaque** ligne (non imputable / autre motif / commentaire) au Dev Agent
  Record ; un site neuf non trié bloque la story.
- [ ] **T1 — La variante** (AC1)
  - [ ] `DbError::AccountsNotPostable(Vec<String>)` + constructeur trieur/dédoublonneur dans
        `crates/kesh-db/src/errors.rs` ; doc-comment : pourquoi elle existe (commentaire de #429,
        précédent `RevenueAccountRejection::NotPostable`, choix C3) ; `error_code()` →
        `"ACCOUNT_NOT_POSTABLE"`. Compléter le `match` exhaustif de `errors.rs:~745`.
  - [ ] Bras dans `crates/kesh-api/src/errors.rs`, à côté de `InactiveOrInvalidAccounts` (`:3067`) :
        400, code, `t_args("error-account-not-postable", repli, args{numbers, count})`, `details`.
- [ ] **T2 — Le message** (AC2) : la clé dans les quatre `messages.ftl` ; `npm run lint-i18n-ownership`
      vert ; un test Rust par locale (patron des tests i18n de `kesh-i18n`) vérifie que la clé se résout
      **au singulier et au pluriel** et contient le numéro passé.
- [ ] **T3 — Saisie manuelle** (AC3) : réécrire `validate_lines_accounts_in_tx` (requête + décision en
      Rust) et son doc-comment (`journal_entries.rs:65-83`, qui annonce encore
      `Err(DbError::InactiveOrInvalidAccounts)` seul).
- [ ] **T4 — Les trois gardes de la 24-5** (AC4) : `SELECT active, postable, number[, account_type]` ;
      ordre (a) puis (b) ; collecte sur la boucle des lignes de facture fournisseur. Mettre à jour les
      commentaires de ces trois sites.
- [ ] **T5 — Ouverture et écrans** (AC5, AC6)
  - [ ] `crates/kesh-api/src/routes/opening_balances.rs:25-26` (doc de module : « non-postable … →
        `INACTIVE_OR_INVALID_ACCOUNTS` ») et les commentaires `:189`, `:332`, `:405` : réécrire ce qui
        concerne le compte **non imputable** ; ce qui concerne l'inconnu / l'autre société reste.
  - [ ] `JournalEntryForm.svelte:178` : le `case`. Lecture des `catch` des autres écrans (AC6),
        consignée.
- [ ] **T6 — Tests** (AC3, AC4, AC5, AC7, AC8)
  - [ ] Réécrire les tests de l'AC7 (liste fermée + ce que T0 aura trouvé).
  - [ ] `kesh-db` — saisie manuelle : (i) un compte non imputable → `AccountsNotPostable(["<n°>"])` ;
        (ii) **deux** comptes non imputables → les deux numéros, triés ; (iii) un archivé **et** un non
        imputable dans la même écriture → `InactiveOrInvalidAccounts` (priorité) ; (iv)
        `enforce_postable = false` sur un non imputable → accepté (déjà couvert par
        `test_create_in_tx_auto_flow_allows_non_postable`, le citer).
  - [ ] `kesh-db` — gardes 24-5 : pour chacune, non imputable → `AccountsNotPostable` ; archivé →
        `InactiveOrInvalidAccounts` ; et pour le compte de charge, **un compte d'actif non imputable**
        → `InactiveOrInvalidAccounts` (cumul (a)+(b), finding M4) ; deux lignes de charge non
        imputables → un seul refus nommant les deux.
  - [ ] `kesh-api` : `opening_balances_e2e.rs` — une ligne d'ouverture sur un compte non imputable →
        400 `ACCOUNT_NOT_POSTABLE` et `details.accountNumbers == ["<n°>"]` ; `reports_e2e.rs:1953`
        asserte aussi `details.accountNumbers`.
  - [ ] Le **compte de test ne diffère d'un compte accepté que par `postable`** (même société, actif,
        bon type), et l'assertion porte sur la **variante / le code**, jamais sur le seul statut 400.
  - [ ] **Mutation** : pour chacune des quatre gardes, retirer la branche (b) une fois → le test
        négatif rougit (il retombe sur `InactiveOrInvalidAccounts` ou passe) ; restaurer **et toucher le
        fichier** (mémoire « mutation restaurée, binaire périmé »). Consigner la liste au Dev Agent
        Record.
- [ ] **T7 — Propagation et documentation** (AC9)
  - [ ] `CHANGELOG.md` (AC9).
  - [ ] Manuel : `grep -n "archivés ou invalides\|INACTIVE_OR_INVALID" docs/manual/fr/*.tex` et
        `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "archivés ou invalides"`
        — attendu : aucune occurrence pour ce motif ; consigner le résultat.
  - [ ] **Grep du symptôme** (règle *Propagation post-patch*) : les commentaires qui disent qu'un compte
        non imputable est refusé en `InactiveOrInvalidAccounts` / « archivés ou invalides » —
        `grep -rnE "non.?postable.*InactiveOrInvalid|InactiveOrInvalid.*non.?postable|INACTIVE_OR_INVALID_ACCOUNTS" crates`
        — sont réécrits, en particulier `journal_entries.rs:3647`,
        `crates/kesh-db/tests/invoices_line_revenue_account.rs:567` et `:708`,
        `crates/kesh-db/src/repositories/accounts.rs:276` s'ils parlent de ce motif.
- [ ] **T8 — Gates** : gate complet backend (`scripts/test-fast.sh`, base remise à zéro avant) —
      **même en cours de boucle de revue** (la story touche des repositories `kesh-db`) ; gate frontend
      complet ; **E2E Playwright complet au dernier commit de code** (décision D7), jugé fichier par
      fichier contre `docs/testing.md` § « Les échecs attendus ».

*(Décompte : 9 AC, 9 tâches T0–T8.)*

## Dev Notes

### Le modèle, et ce qu'il n'est pas

- `validate_line_revenue_accounts_in_tx` (`invoices.rs:550-660`, 16-1a) est le modèle de la
  **priorité** : une seule raison, la plus bloquante d'abord. `RevenueAccountRejection::NotPostable`
  (`crates/kesh-db/src/errors.rs:5-47`) est le précédent d'un refus qui nomme la non-imputabilité ; on
  ne le réutilise pas (il est propre aux lignes de facture, choix C3).
- **Ne pas changer `enforce_postable`** des appelants : cette story change le **nom** du refus, pas son
  **périmètre**. Aucun compte aujourd'hui accepté ne devient refusé, aucun refusé ne devient accepté.
- La décision en Rust (au lieu de la clause SQL) est nécessaire pour savoir **pourquoi** un compte
  manque — la requête actuelle le filtre et perd l'information.

### Ce qui doit être préservé

- Anti-énumération (KF-002) : un compte inconnu ou d'une autre société ne livre jamais son numéro ;
  seul un compte **de la société, actif** est nommé. La construction de la variante n'a donc lieu
  qu'après le contrôle d'appartenance.
- `insert_with_defaults_in_tx`, `seed_demo`, `onboarding.rs:194,727` reconnaissent
  `InactiveOrInvalidAccounts` pour **leurs** refus (comptes de rôle absents) ; ils ne passent par aucune
  des gardes modifiées — le vérifier à T0, ne rien y changer.

### Décisions consignées (registre `epic-15-choix-autonomes.md`)

- **C3** — la forme du refus (variante dédiée, code `ACCOUNT_NOT_POSTABLE`, étendue aux gardes
  existantes).
- **C7** — le découpage : cette story est le socle ; la 15-5b déploie.
- **C13** — la clé de détail `accountNumbers`, commune au 400 et aux `failed[]` de la 15-5b, et l'ordre
  des causes sur une garde à plusieurs critères.

### Fichiers touchés (prévision)

`crates/kesh-db/src/errors.rs`, `crates/kesh-db/src/repositories/{journal_entries,invoice_settlements_write,supplier_invoices}.rs`,
`crates/kesh-api/src/errors.rs`, `crates/kesh-api/src/routes/opening_balances.rs` (commentaires),
`crates/kesh-i18n/locales/*/messages.ftl`, `frontend/src/lib/features/journal-entries/JournalEntryForm.svelte`,
tests (`crates/kesh-db/tests/{invoice_settlement,supplier_invoices_repository}.rs`,
`crates/kesh-api/tests/{reports_e2e,opening_balances_e2e}.rs`, `mod tests` de `journal_entries.rs`),
`CHANGELOG.md`. **Aucune migration** (P1–P8 sans objet). Modules de premier niveau : `kesh-db`,
`kesh-api` (erreurs), `kesh-i18n`, `frontend/journal-entries` — sous le seuil de la règle de splitting.

### Tests — ce qui rendrait un test vert sans rien prouver

- `contains("InactiveOrInvalid")` sur le `Debug` : passe pour n'importe quelle variante dont le nom
  commence ainsi. Asserter la variante par `matches!`.
- Un compte de test non imputable **et** d'un autre type : le refus viendrait du type. Ne changer que
  `postable` — sauf dans le test de priorité, qui cumule exprès.
- Un message asserté en français seulement : le pluriel cassé dans une autre locale passerait. T2 teste
  les quatre.

### References

- Issues : #427, #429 (et son commentaire), #375 (24-5).
- `crates/kesh-db/src/repositories/journal_entries.rs:65-133`, `:3640-3760`.
- `crates/kesh-db/src/repositories/invoices.rs:550-660`.
- `crates/kesh-api/src/errors.rs:52-60` (`t_args`), `:3067-3074`, `:3120-3140`.
- `_bmad-output/implementation-artifacts/24-5-comptes-de-cloture.md`, `14-3b-consommateurs-roles.md`.
- Fiche source : `15-5-gardes-postabilite-serveur.md` (statut `split`, Change Log de la passe P1).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-10-08 — **Créée par découpage de la 15-5** après la passe de validation **P1** (trois lentilles
  Sonnet, contexte frais ; prompt `15-5-validate-prompt-p1.md`). Bilan de P1 sur la 15-5 entière :
  **0 CRITICAL, 0 HIGH** ; bruts par lentille A 5 MEDIUM / 6 LOW, B 2 / 2, C 4 / 4 ; **après fusion des
  doublons, 7 MEDIUM et 8 LOW distincts** (ventilation dans la fiche 15-5). Ce qui en revient à cette
  moitié, et comment :
  - **M1 = C-1** (MEDIUM, lentilles A et C) — le grep de T2 ratait `invoice_settlement.rs:354` ;
    `supplier_invoices_repository.rs:424` et `:470` n'étaient pas nommés → liste fermée à l'AC7, motif
    `InactiveOrInvalid[^A]` à T0.
  - **M4** (MEDIUM, A) — l'ordre « non imputable ET mauvais type » n'était pas tranché → AC4 (a) puis
    (b), test de cumul.
  - **M5 = C-4** (MEDIUM, A et C) — règle de splitting franchie → découpage (choix **C7**).
  - **C-3** (MEDIUM, C) — `## [Unreleased]` inexistant → `## [0.13.0] — Non publié`, rubriques
    françaises (AC9).
  - **L2/B4** (LOW, A et B) — détail structuré non fixé → `details.accountNumbers` (AC1, choix C13).
  - **L3** (LOW, A) — pluriel et apostrophe → AC2.
  - **L4** (LOW, A) — messages des quatre locales non testés → T2.
  - **C-8 / L5** (LOW, A et C) — manuel : contrôle sans objet ici, écrit à l'AC9.
  Les autres findings (B1/M2, C-2/M3, B2/C-6, B3/C-5, C-7, L1, L6) portent sur les surfaces neuves : voir
  la 15-5b. Décisions de l'orchestrateur : C7 à C11 ; ajoutées pendant la remédiation : C12 à C14.
