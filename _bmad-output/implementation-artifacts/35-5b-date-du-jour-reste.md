# Story 35-5b : la date du jour est celle de Zurich partout ailleurs — accueil, échéances, rappels, noms de fichiers, assistant, démonstration — et les tests ne dépendent plus de l'heure réelle (#929)

Status: ready-for-dev

*Fiche rédigée le 2026-11-12 par l'agent de kickoff de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, par le workflow
`bmad-create-story`. Seconde moitié du découpage préventif de la 35-5 (C-e350-14) : elle **part de la 35-5a fusionnée**, dont
elle emploie `jour_suisse`, le trait `Horloge`, `AppState.horloge` et la garde `date_du_jour_garde.rs`. Release **0.17.7**.
**Aucune migration** (C-e350-16). Boucle complète (`kesh-db` touché). Code lu sur **`release/0.17.6`** (`4e1c768a`).
Registres : `e35-choix-autonomes-e350.md` (C-e350-8, -14, -16) ; `e35-choix-autonomes-e355b.md` pour la story, à créer au
premier choix. Validation à lancer par l'orchestrateur, après celle de la 35-5a.*

## Story

En tant que titulaire, je veux que l'accueil, les factures échues, les rappels, l'assistant et les noms de fichiers suivent
le jour suisse, même entre minuit et 1 h (2 h en été), afin que l'accueil du 1er janvier montre le nouvel exercice et qu'une
facture ne paraisse pas échue ou à l'heure selon l'heure de la nuit ; et en tant que mainteneur, je veux que les tests ne
rougissent plus selon l'heure à laquelle on les lance.

## Ce qui se lit au code (`release/0.17.6`)

L'inventaire complet est dans la fiche 35-5a (tableau des 28 sites, recompté). Les **20 sites de cette story** :

| # | Site | Ce qu'il date |
|---|---|---|
| 9 | `kesh-api/src/routes/admin.rs:99` | nom du fichier de sauvegarde (`kesh-installation-AAAA-MM-JJ.keshbackup`) |
| 10 | `kesh-api/src/routes/audit_log.rs:420` | nom du CSV du journal d'audit |
| 11 | `kesh-api/src/routes/bank_accounts.rs:1802` | `closedOn` par défaut à la clôture d'un compte bancaire |
| 12 | `kesh-api/src/routes/exports.rs:58` | nom de l'export de souveraineté |
| 13 | `kesh-api/src/routes/invoice_email.rs:381` | jours de retard d'un rappel (`days_overdue`) |
| 14–15 | `kesh-api/src/routes/invoices.rs:333`, `:425` | `is_overdue` de la fiche et de la liste |
| 16 | `kesh-api/src/routes/invoices.rs:1459` | CSV de l'échéancier (`export_due_dates_csv_handler`, `:1411`) |
| 17 | `kesh-api/src/routes/onboarding.rs:1043` | année du premier exercice, « Exercice {année} », à la finalisation |
| 18 | `kesh-api/src/services/postes_ouverts.rs:74` | `asOf` par défaut des postes ouverts |
| 19 | `kesh-api/src/services/rapports.rs:732` | `as_of` de la balance âgée |
| 20 | `kesh-api/src/services/rapports.rs:780` | fin d'un rapport de projet sans date de fin |
| 21 | `kesh-api/src/services/rapports.rs:870` | synthèse de l'accueil (#929) |
| 22 | `kesh-db/src/repositories/dunning_eligibility.rs:98` | éligibilité aux rappels |
| 23 | `kesh-db/src/repositories/invoices.rs:404` | filtre « échues » de la liste, `i.due_date < UTC_DATE()` |
| 24–25 | `kesh-db/src/repositories/invoices.rs:970`, `:971` | compte et total des échues de la synthèse de l'échéancier, `UTC_DATE()` |
| 26 | `kesh-seed/src/lib.rs:279` | année de la démonstration |
| 27–28 | `kesh-seed/src/pieges.rs:378`, `:461` | données des pièges de démonstration |

Les commentaires qui justifient l'UTC deviennent faux : `dunning_eligibility.rs:9` (« **UTC** via `chrono::Utc`, cohérent
avec `UTC_DATE()` »), `invoices.rs:378` et `:396`, `invoice_email.rs:206`, `services/postes_ouverts.rs:55`,
`companies.rs:988` (35-5a). `docs/api-external.md:443` : `asOf` par défaut « la date du jour, en UTC ».

**Tests** : modules de tests de `src` — `journal_entries.rs` (50 sites, dont ceux que la 35-5a n'a pas convertis) et
`invoices.rs` (6 : `fn today()` `:3498`, cinq `CURDATE()` de montage `:4379`, `:4393`, `:4418`, `:4679`, `:4745`) ;
`crates/*/tests` : 107 sites dans 38 fichiers (liste et décompte à la 35-5a), moins ceux que la 35-5a a convertis ; specs
E2E : **36** occurrences de `toISOString().slice(0, 10)` dans 26 fichiers de `frontend/tests/e2e/` (dont
`helpers/api-fixtures.ts` 2, `reports.spec.ts` 3, `supplier-internal-number.spec.ts` 3) — la date UTC, non celle de
Zurich que le serveur retient désormais.

**Le précédent de l'écran** : `frontend/src/lib/shared/utils/local-date-guard.test.ts` (#889) relève
`toISOString().slice(0, 10)` dans `src/**` hors tests, contre une allowlist d'entrées `{ fichier, fragment, motif }`.

## Décisions

- **D1 — Chaque site lit `state.horloge.aujourd_hui()`** (routes, services) ou reçoit la date (`kesh-db`, `kesh-seed`).
- **D2 — Le SQL ne lit plus `UTC_DATE()`** : `invoices.rs:404`, `:970`, `:971` lient la date du jour en paramètre (`?`),
  reçue par la fonction du dépôt.
- **D3 — `kesh-seed`** reçoit la date (ou l'année) de son appelant ; s'il n'en a pas (outil en ligne de commande), il
  prend `jour_suisse(Utc::now())`, le seul site exempté de la garde avec `HorlogeSysteme`, justification écrite.
- **D4 — La garde de la 35-5a se ferme** : sa liste d'exemptions de `src` se vide (hors D3), et elle **s'étend aux tests**
  (`crates/*/tests/**/*.rs` et modules `#[cfg(test)]`) : une date du jour y vient d'une horloge fixée, d'une date passée en
  dur, ou de `jour_suisse(Utc::now())` dans **un** helper de test partagé (pour les montages qui veulent « un exercice qui
  couvre aujourd'hui ») — jamais de `Utc::now().date_naive()`. Un test qui asserte une date produite par le serveur fixe
  l'horloge.
- **D5 — Les specs E2E** prennent la date de Zurich par **un** helper (`frontend/tests/e2e/helpers/`, par exemple
  `jourZurich()` : `Intl.DateTimeFormat('fr-CA', { timeZone: 'Europe/Zurich' })`), indépendant du fuseau du poste qui lance
  Playwright ; la garde `local-date-guard.test.ts` s'étend à `tests/e2e/**` (allowlist vide à la fin).
- **D6 — Les commentaires et la doc qui disaient UTC** sont réécrits (liste ci-dessus) ; `api-external.md:443` dit « la
  date du jour, à Zurich ».

## Critères d'acceptation

1. **AC1 — Accueil la nuit du 1er janvier** (#929) : horloge fixée à `2025-12-31T23:30:00Z`, exercices 2025 et 2026 ouverts :
   `GET /reports/homepage-summary` lit l'exercice **2026** (rouge sur le code de départ : 2025).
2. **AC2 — Échéances** : facture échue le `2026-03-10`, horloge à `2026-03-10T23:30:00Z` (00 h 30 le 11 mars) : la fiche et la
   liste la disent échue ; le filtre « échues » et la synthèse de l'échéancier la comptent (D2) ; le CSV de l'échéancier
   aussi. À `2026-03-10T22:30:00Z` (23 h 30 le 10), elle ne l'est pas.
3. **AC3 — Rappels** : jours de retard (`invoice_email.rs:381`) et éligibilité (`dunning_eligibility.rs:98`) comptés depuis
   le jour suisse (même montage).
4. **AC4 — Noms de fichiers et défauts** : à `2025-12-31T23:30:00Z`, la sauvegarde, l'export et le CSV d'audit portent
   `2026-01-01` ; `closedOn` par défaut vaut `2026-01-01` ; `asOf` par défaut des postes ouverts et `as_of` de la balance
   âgée aussi ; la fin d'un rapport de projet sans date de fin aussi.
5. **AC5 — L'assistant** : finalisé à `2025-12-31T23:30:00Z`, le premier exercice s'appelle « Exercice 2026 » et couvre 2026.
6. **AC6 — La garde fermée** (D4) : aucune exemption dans `src` hors D3 ; les tests aussi gardés ; sonde positive (un
   `Utc::now().date_naive()` dans un test la fait rougir).
7. **AC7 — E2E** (D5) : plus aucun `toISOString().slice(0, 10)` dans `tests/e2e/**` hors allowlist motivée ; la garde
   étendue rougit sur le code de départ (36 relevés) et passe à la fin.
8. **AC8 — Le jour ne change rien** : à `2026-06-15T10:00:00Z`, mêmes résultats qu'avant ; tests existants verts.
9. **AC9 — Docs, CHANGELOG** : D6 ; manuel (accueil, échéances : une phrase « date de Zurich » au même endroit que la 35-5a
   l'a écrite, renvoyée) ; PDF régénéré, contrôlé aplati ; CHANGELOG `## [0.17.7] — Non publié`, *Corrigé*, le point #929
   (C-e350-8). La PR porte `closes #929`.
10. **AC10 — Aucune migration**.

## Tests à écrire, et la preuve du rouge

AC1 à AC5 avec l'horloge fixée, rouges sur la 35-5a fusionnée (les sites lisent encore le jour UTC) ; AC6 et AC7, rouges sur
la garde de la 35-5a (exemptions encore présentes, tests non gardés). Journal `kesh-gate-logs/e355b/rouges.log`.

**Mutations** : M1 un site remis à `Utc::now().date_naive()` (la garde) ; M2 `UTC_DATE()` rétabli à `:404` (AC2) ; M3 le
helper E2E en `toISOString` (AC7) ; M4 `onboarding.rs:1043` sur l'année UTC (AC5).

## Tâches

- [ ] Étape 0 : partir de la 35-5a fusionnée ; recompter les sites restants et les tests ; Change Log.
- [ ] Tests, rouges consignés ; sites 9–28 ; SQL ; seed ; tests de `src` et de `crates/*/tests` ; specs E2E et leur garde.
- [ ] Commentaires, docs, manuel, CHANGELOG.
- [ ] Gates : `test-fast.sh --ci` complet à chaque commit de code, `scripts/gate-ci.sh` au dernier, frontend complet, E2E
      complet au dernier commit de code.

## Dev Notes

### Fichiers communs — règle d'union C-e350-8

`bank_accounts.rs` avec la 35-5a (ici `:1802`) ; `journal_entries.rs` (tests) avec la **35-8**, qui attend cette story ;
`invoices.rs` (`kesh-db`) avec la **35-1** si son correctif touche `:2334-2340` (hunks distincts) ; `docs/api-external.md`
(ici `:443`) ; `user-manual.tex` ; `CHANGELOG.md` ; `frontend/tests/e2e/helpers/api-fixtures.ts` avec la **35-2a** (qui y
ajoute un paramètre `city`) : hunks distincts.

### Ce qui doit rester vrai

- Les horodatages restent en UTC (#669). Aucune donnée existante réécrite. Fixtures synthétiques seulement.

### Références

- Issue #929 ; fiche 35-5a ; #889 et `local-date-guard.test.ts`.
- `CLAUDE.md` : § *Exception `kesh-db`* ; § *Inventorier les sites NON RÉSOLUS* ; § *Test Locally First* (`cargo test`
  contre `nextest`).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

- Fiche rédigée par le workflow `bmad-create-story` (analyse du code de `release/0.17.6`) ; validation à lancer.

### File List

## Change Log

- 2026-11-12 — Fiche rédigée (Epic 35, découpage préventif C-e350-14). Validation non lancée.
