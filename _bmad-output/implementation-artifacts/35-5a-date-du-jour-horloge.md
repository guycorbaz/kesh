# Story 35-5a : la date du jour est celle de Zurich pour ce qui écrit dans les livres ou les borne — solde d'ouverture, contre-passation, annulation de règlement, verrou de période — par une horloge injectable (#1022, #907, #929)

Status: ready-for-dev

*Fiche rédigée le 2026-11-12 par l'agent de kickoff de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, par le workflow
`bmad-create-story`. Née du **découpage préventif** de la 35-5 (C-e350-14) : la 35-5a pose le motif et convertit les sites
qui écrivent dans les livres ou les bornent ; la **35-5b** convertit le reste et part de celle-ci fusionnée. Release
**0.17.7**. **Aucune migration** (C-e350-16). Boucle complète : règle comptable, `kesh-db` touché (gate complet même en
cours de boucle). Code lu sur **`release/0.17.6`** (`4e1c768a`). Registres : `e35-choix-autonomes-e350.md` (C-e350-8, -14,
-16) ; `e35-choix-autonomes-e355a.md` pour la story, à créer au premier choix. Validation à lancer par l'orchestrateur.*

## Story

En tant que titulaire qui travaille parfois tard, je veux qu'un solde d'ouverture, une contre-passation ou une annulation
faits entre minuit et 1 h (2 h en été) soient datés **du jour suisse**, et que « verrouiller jusqu'à hier » soit accepté
à cette heure-là, afin qu'aucune écriture ne tombe la veille, voire dans l'exercice précédent la nuit du 1er janvier.

## Ce qui se lit au code (`release/0.17.6`)

**L'inventaire complet** — `git grep -nE 'Utc::now\(\)\.(date_naive|naive_utc\(\)\.date)\(\)|UTC_DATE|CURDATE|Local::now'`
sur `crates/*/src`, recompté fichier par fichier (module `#[cfg(test)]` séparé) ; aucune autre forme (`now.date_naive()` sur
une variable : aucune). **28 sites de production** :

| # | Site | Ce qu'il date ou borne | Story |
|---|---|---|---|
| 1 | `kesh-api/src/routes/bank_accounts.rs:1007` | solde d'ouverture à la création d'un compte (`OpeningRequest.today`) | **35-5a** |
| 2 | `kesh-api/src/routes/bank_accounts.rs:1252` | solde d'ouverture d'un compte existant | **35-5a** |
| 3 | `kesh-api/src/routes/opening_balances.rs:244` | statut du complément des *Soldes de départ* (`complement_status`) | **35-5a** |
| 4 | `kesh-api/src/routes/opening_balances.rs:604` | date du complément des *Soldes de départ* | **35-5a** |
| 5 | `kesh-db/src/repositories/journal_entries.rs:2935` | contre-passation datée du jour (`ReversalDating::Today`) | **35-5a** |
| 6 | `kesh-db/src/repositories/settlement_cancellation.rs:172` | exercice ouvert au jour de l'annulation d'un règlement (rang 5) | **35-5a** |
| 7 | `kesh-db/src/repositories/companies.rs:1013` | `lock_books` : borne refusée si ≥ aujourd'hui (#907) | **35-5a** |
| 8 | `kesh-db/src/repositories/companies.rs:1100` | `unlock_books` : même borne | **35-5a** |
| 9 | `kesh-api/src/routes/admin.rs:99` | nom du fichier de sauvegarde | 35-5b |
| 10 | `kesh-api/src/routes/audit_log.rs:420` | nom du CSV du journal d'audit | 35-5b |
| 11 | `kesh-api/src/routes/bank_accounts.rs:1802` | `closedOn` par défaut à la clôture d'un compte bancaire | 35-5b |
| 12 | `kesh-api/src/routes/exports.rs:58` | nom de l'export de souveraineté | 35-5b |
| 13 | `kesh-api/src/routes/invoice_email.rs:381` | jours de retard d'un rappel | 35-5b |
| 14–16 | `kesh-api/src/routes/invoices.rs:333`, `:425`, `:1459` | facture échue (fiche, liste) ; CSV de l'échéancier | 35-5b |
| 17 | `kesh-api/src/routes/onboarding.rs:1043` | année du premier exercice à l'assistant | 35-5b |
| 18 | `kesh-api/src/services/postes_ouverts.rs:74` | `asOf` par défaut des postes ouverts | 35-5b |
| 19–21 | `kesh-api/src/services/rapports.rs:732`, `:780`, `:870` | balance âgée ; fin d'un rapport de projet ; accueil (#929) | 35-5b |
| 22 | `kesh-db/src/repositories/dunning_eligibility.rs:98` | éligibilité aux rappels | 35-5b |
| 23–25 | `kesh-db/src/repositories/invoices.rs:404`, `:970`, `:971` | `UTC_DATE()` en SQL : filtre « échues », synthèse de l'échéancier | 35-5b |
| 26 | `kesh-seed/src/lib.rs:279` | année de la démonstration | 35-5b |
| 27–28 | `kesh-seed/src/pieges.rs:378`, `:461` | données des pièges de démonstration | 35-5b |

Hors production : **56** sites dans les modules de tests de `src` (`journal_entries.rs` 50, dont les tests de contre-passation ;
`invoices.rs` 6, dont `fn today()` à `:3498` et cinq `CURDATE()` de montage), **107** dans `crates/*/tests` (38 fichiers ;
les plus chargés : `period_lock_e2e.rs` 19, `journal_entry_reversal_e2e.rs` 19, `bank_accounts_closed_e2e.rs` 6,
`invoice_unvalidate_e2e.rs` 6), **36** dans 26 fichiers des specs E2E (`toISOString().slice(0, 10)`, 35-5b).

**Les appelants des fonctions de `kesh-db` qui recevront la date** (35-5a) : `reverse_in_tx`, `reverse_owned_in_tx`,
`reverse`, `settlement_entry_cancel_blocker`, `lock_books`, `unlock_books` — appelés par `kesh-api/src/routes/companies.rs`
(2), `kesh-api/src/routes/journal_entries.rs` (1), `kesh-db/src/repositories/invoice_settlements_write.rs` (2),
`journal_entries.rs` (3), `reconciliation_cancel.rs` (2), `supplier_invoices.rs` (4) ; le compilateur donne la liste close.
`reverse_at_next_fiscal_year_start*` ne lit pas le jour.

**Le précédent de l'écran** : `frontend/src/lib/shared/utils/local-date.ts` (`todayLocal`, #889, 0.17.4), gardé par
`local-date-guard.test.ts`, qui tient l'inventaire des sites clos. La 35-5a en est le pendant serveur.

**`AppState`** (`crates/kesh-api/src/lib.rs:61-107`) ; constructeur de test `new_for_tests` (`:109-…`), qui défaute les champs
dans son corps pour ne pas toucher les ~33 sites de test.

**Les docs qui disent UTC ou « du jour »** : `docs/api-external.md:272` (« sinon du jour **UTC** »), `:443` (postes ouverts,
« la date du jour, en UTC », 35-5b) ; le manuel utilisateur parle partout de « la date du jour » sans dire le fuseau
(`user-manual.tex:259-261`, `:355`, `:709`, `:722`, `:994`, `:1449-1482`, `:1900`, `:2011`, `:2085`).

## Décisions (C-e350-16)

- **D1 — Europe/Zurich, constante unique**, dans `kesh-core` : `pub fn jour_suisse(instant: DateTime<Utc>) -> NaiveDate`,
  pure (`chrono-tz` 0.10.4, déjà au `Cargo.lock` : ajouté aux dépendances de `kesh-core` à la même version, sans nouveau
  paquet). Pas de `TZ`, pas de `chrono::Local`.
- **D2 — `kesh-db` ne lit plus l'horloge** pour une date : `reverse_in_tx`, `reverse_owned_in_tx`, `reverse`,
  `settlement_entry_cancel_blocker`, `lock_books`, `unlock_books` (et leurs intermédiaires) reçoivent `aujourd_hui:
  NaiveDate`. `ReversalDating::Today` porte la date ou la reçoit.
- **D3 — `kesh-api` tient l'horloge** : un champ `horloge: Arc<dyn Horloge>` dans `AppState` (trait `Horloge { fn
  maintenant(&self) -> DateTime<Utc> }` et la méthode fournie `aujourd_hui()` = `jour_suisse(maintenant())`) ; `HorlogeSysteme`
  en production (`main.rs`), défaut de `new_for_tests` ; `HorlogeFixe(instant)` pour les tests qui datent quelque chose. Le
  trait vit dans `kesh-core` (sans I/O) ; `HorlogeSysteme`, seul appel à `Utc::now()` pour une date, dans `kesh-api`.
- **D4 — La garde de l'inventaire** : un test (`crates/kesh-api/tests/date_du_jour_garde.rs`, sur le modèle de
  `local-date-guard.test.ts`) parcourt `crates/*/src/**/*.rs` et échoue sur toute occurrence des formes de l'inventaire
  (`Utc::now().date_naive()`, `.naive_utc().date()` sur `Utc::now()`, `UTC_DATE`, `CURDATE`, `Local::now`, `Local::today`)
  hors d'une **liste close d'exemptions**, écrite dans le test **par fichier et par nombre** : les 20 sites de la 35-5b et
  les 56 des modules de tests de `src` (que la 35-5b traite aussi), chacun avec la story qui le videra. Un site en plus, ou
  un compte qui ne baisse pas comme prévu, rougit. Sonde positive au Change Log.
- **D5 — Les tests ne dépendent plus de l'heure réelle** pour les sites convertis : un test de `kesh-db` passe la date qu'il
  asserte ; un test HTTP qui asserte une date fixe l'horloge de son `AppState`. La nuit se teste par instants fixés.
- **D6 — Docs** : `api-external.md:272` dit « du jour, à Zurich » ; le manuel dit une fois, à l'endroit où il définit les
  écritures datées du jour, que « la date du jour est celle de Zurich, même la nuit » (paragraphe à choisir, renvoyé depuis
  le solde d'ouverture et le verrou de période).

## Critères d'acceptation

1. **AC1 — La fonction pure** (`kesh-core`, tests unitaires) : `2025-12-31T23:30:00Z` → `2026-01-01` (hiver, +1 h) ;
   `2026-07-01T22:30:00Z` → `2026-07-02` (été, +2 h) ; `2026-03-29T00:30:00Z` et `2026-10-25T00:30:00Z` (jours de
   changement d'heure) → le jour suisse attendu ; `2026-06-15T10:00:00Z` → `2026-06-15`.
2. **AC2 — Solde d'ouverture la nuit** (#1022) : horloge fixée à `2025-12-31T23:30:00Z`, exercice 2026 ouvert et premier
   exercice non concerné (montage où la date retenue est « aujourd'hui ») : `POST /bank-accounts` avec `openingBalance`, puis
   `POST /bank-accounts/{id}/opening-balance` sur un autre compte → écriture datée `2026-01-01`, dans l'exercice 2026. Même
   chose pour le complément des *Soldes de départ* (`opening_balances.rs:604`) et le statut (`:244`).
3. **AC3 — Contre-passation la nuit** (#929) : horloge fixée à la même heure : `POST /journal-entries/{id}/reverse` → écriture
   inverse datée `2026-01-01`. L'annulation d'un règlement (rang 5, `settlement_cancellation.rs:172`) juge l'exercice de
   `2026-01-01`.
4. **AC4 — Verrou de période** (#907) : horloge fixée à `2026-03-10T23:30:00Z` (00 h 30 le 11 mars à Zurich) :
   verrouiller jusqu'au `2026-03-10` est **accepté** ; jusqu'au `2026-03-11`, refusé (`BOOKS_LOCK_BOUND_NOT_PAST`) ; idem
   pour `unlock_books`.
5. **AC5 — `kesh-db` sans horloge** : les fonctions de D2 reçoivent la date ; leurs tests passent une date fixe.
6. **AC6 — La garde** (D4) : rouge sur le code de départ (28 sites hors exemptions vs 20 exemptés : les 8 de la 35-5a la font
   rougir) ; verte après ; sonde positive (un `Utc::now().date_naive()` ajouté à un fichier non exempté la fait rougir).
7. **AC7 — Tests convertis** : les tests qui assertent une date produite par un site de la 35-5a passent une date ou fixent
   l'horloge (verdict par fichier au Change Log : au moins `journal_entries.rs` `mod tests`, `period_lock_e2e.rs`,
   `journal_entry_reversal_e2e.rs`, `bank_account_opening_balance_e2e.rs`, `opening_balances_e2e.rs`,
   `opening_complement_repository.rs`, `invoice_settlement.rs`, `reconciliation_cancel.rs`, `invoice_unvalidate_e2e.rs`,
   `supplier_invoices_repository.rs`, `filet_bilan_clos*.rs`, `credit_card_e2e.rs`). Ceux qui ne dépendent pas d'un site
   converti restent pour la 35-5b.
8. **AC8 — Le chemin de jour ne change pas** : en journée (`2026-06-15T10:00:00Z`), les dates écrites sont celles d'avant ;
   tests existants verts sans retouche de leurs attendus, hors AC7.
9. **AC9 — Docs, manuel, CHANGELOG** : D6 ; PDF régénéré, contrôlé aplati ; CHANGELOG `## [0.17.7] — Non publié`,
   *Corrigé*, un point pour #1022 et un pour #907, et une phrase pour la part de #929 (contre-passation) — la 35-5b ferme
   #929 (C-e350-8).
10. **AC10 — Aucune migration** : `git diff --stat` ne touche pas `crates/kesh-db/migrations/` ; le dire au Change Log.

## Tests à écrire, et la preuve du rouge

Écrire d'abord, lancer sur le code de départ, journal `kesh-gate-logs/e355a/rouges.log` : AC1 (la fonction n'existe pas :
rouge par **squelette compilable** commité d'abord — fonction qui rend le jour UTC, trait et horloge déclarés, sites
inchangés —, puis rouge par attendu) ; AC2 à AC4 avec l'horloge fixée (rouges : dates UTC, la veille) ; AC6.

**Mutations** : M1 `jour_suisse` rend le jour UTC ; M2 l'été sans +2 h (fuseau fixe +1) ; M3 un site de la 35-5a qui lit
`Utc::now()` ; M4 `lock_books` comparé au jour UTC ; M5 une exemption de la garde ajoutée sans site (la garde rougit sur le
compte).

## Tâches

- [ ] Étape 0 : recompter l'inventaire (28 / 56 / 107 / 36) et les appelants ; Change Log.
- [ ] Squelette compilable, tests, rouges consignés.
- [ ] `kesh-core` : `jour_suisse`, trait `Horloge`. `kesh-db` : date en paramètre (D2). `kesh-api` : `AppState.horloge`,
      `HorlogeSysteme`, `HorlogeFixe` pour les tests, sites 1–8 ; garde D4.
- [ ] Tests convertis (AC7) ; docs, manuel, CHANGELOG.
- [ ] Gates : `test-fast.sh --ci` **complet à chaque commit de code** (exception `kesh-db`), `scripts/gate-ci.sh` au dernier
      commit de code, frontend complet, E2E complet au dernier commit de code.

## Dev Notes

### Hors périmètre

- Les 20 sites restants, les modules de tests de `src` non liés aux sites convertis, les tests de `crates/*/tests` restants,
  les specs E2E : **35-5b**.
- Les horodatages (`created_at`, `UTC_TIMESTAMP(6)`, jetons) restent en UTC : ce sont des instants, non des dates (#669 pour
  le `time_zone` de la session).

### Fichiers communs — règle d'union C-e350-8

`opening_balances.rs` avec la **35-6** (ici `:244`, `:604` ; la 35-6, `:515-530`) ; `bank_accounts.rs` avec la **35-5b** (ici
`:1007`, `:1252` ; la 35-5b `:1802`) ; `journal_entries.rs` avec la **35-8**, qui attend cette story ; `docs/api-external.md:272`
avec la **35-6** (même ligne, à unir à la main) ; `user-manual.tex` ; `CHANGELOG.md`. `AppState` (`lib.rs`) : la 35-5b n'y
ajoute rien.

### Ce qui doit rester vrai

- Aucune migration ; aucune donnée existante réécrite. Une écriture passée la nuit avant la 0.17.7 garde sa date.
- Les refus existants (exercice fermé, période verrouillée) ne changent pas, seule la date qu'ils jugent. Fixtures
  synthétiques seulement.

### Références

- Issues #929, #1022, #907 ; #889 (pendant de l'écran, `local-date.ts`) ; classement `epic35-classement-bugs.md` § 8.3.
- `CLAUDE.md` : § *Exception `kesh-db`* ; § *Règle de splitting préventif* ; § *Inventorier les sites NON RÉSOLUS*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

- Fiche rédigée par le workflow `bmad-create-story` (analyse du code de `release/0.17.6`) ; validation à lancer.

### File List

## Change Log

- 2026-11-12 — Fiche rédigée (Epic 35, découpage préventif C-e350-14). Validation non lancée.
