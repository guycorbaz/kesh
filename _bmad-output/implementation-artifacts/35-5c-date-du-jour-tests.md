# Story 35-5c : les tests ne dépendent plus de l'heure à laquelle on les lance — tests Rust et specs E2E au jour suisse, gardes fermées (#929)

Status: ready-for-dev

*Fiche rédigée le 2026-11-12 par l'agent de rédaction des fiches 35-5a et 35-5b (Opus 5.5), worktree `kesh-e355`, à la
remédiation de la validation P1 de la 35-5b (V1-8 : la story ne tenait pas en une boucle). Troisième des trois stories du
découpage de la 35-5 (C-e350-14, C-e350-22) : elle **part de la 35-5b fusionnée** (donc de la 35-5a), dont elle emploie
`jour_suisse`, `HorlogeFixe`, `AppState.horloge`, `Lecture` à horloge, les helpers `aujourd_hui_suisse` et la garde
`date_du_jour_garde.rs`. Release **0.17.7**. **Aucune migration**. **Aucun code de production** : tests, specs E2E, gardes.
Boucle de revue : les fichiers de tests de `kesh-db` sont touchés, donc **gate complet à chaque commit** (`CLAUDE.md`,
§ *Exception `kesh-db`*). Code lu sur **`release/0.17.6`** (`4e1c768a`). Registres : `e35-choix-autonomes-e350.md` (C-e350-14,
-21, -22, -24) ; `e35-choix-autonomes-e355c.md` pour la story, à créer au premier choix.*

## Story

En tant que mainteneur, je veux que les tests Rust et les specs E2E prennent la date du jour comme le serveur — le jour
suisse — ou une date fixée, afin qu'aucun ne rougisse selon l'heure à laquelle on lance le gate ou la CI (entre minuit et
1 h, 2 h en été, à Zurich), et que rien ne réintroduise une date du jour UTC dans un test.

## Ce qui se lit au code (`release/0.17.6`, à recompter sur la 35-5b fusionnée)

- **Modules de tests de `src`** : `kesh-db/src/repositories/journal_entries.rs`, `mod tests` à partir de `:3178` — 50 sites,
  moins ceux que la 35-5a a convertis (appels aux fonctions de D2, dates de contre-passation). Les 6 de `invoices.rs` sont
  à la 35-5b.
- **`crates/*/tests`** : les fichiers marqués **35-5c** au verdict de la fiche 35-5a — `kesh-api/tests/accounts_e2e.rs`
  (1 + 1 coupé, `:243-245`), `admin_full_import_e2e.rs` (4), `credit_card_e2e.rs` (3), `mcp_enveloppe_e2e.rs` (1),
  `reconciliation_link_e2e.rs` (1) — et tout reste qu'une story précédente aurait laissé, à son compte exact dans la liste
  de la garde.
- **Specs E2E** : **34** `toISOString().slice(0, 10)` dans 26 fichiers de `frontend/tests/e2e/`, plus **1** coupé sur trois
  lignes (`invoices.spec.ts:527-528`), soit **35** — moins ceux que la 35-5a et la 35-5b ont convertis. Formes Node locales,
  qui suivent le fuseau du processus Node et non le `timezoneId: 'Europe/Zurich'` du navigateur (`playwright.config.ts:59`) :
  `bank-accounts-closed.spec.ts:144-145` (`getFullYear`/`getMonth`/`getDate` sur `new Date()`), `helpers/api-fixtures.ts:62-63`
  (`overdueDate` : `setDate` local puis `toISOString`, mélange des deux).
- **Arithmétique pure sur une date**, légitime : `reconciliation-show-older.spec.ts:80` (`Date.UTC(2025, 0, 1 + i)`),
  `bank-manual-statement.spec.ts:74`, `:314` (sur `T12:00:00Z`), peut-être `invoices_echeancier.spec.ts:97`,
  `reports.spec.ts:262`, `v0174-correctifs.spec.ts:53`, `invoices.spec.ts:527-528` (date + 30 jours, depuis un jour déjà
  donné) : à trier un par un.
- **Le helper E2E existe** : `todayInZurich()`, `frontend/tests/e2e/helpers/api-fixtures.ts:390-392`.
- **La garde frontend** : `frontend/src/lib/shared/utils/local-date-guard.test.ts` relève `toISOString().slice(0, 10)` dans
  `src/**` hors tests, contre une allowlist `{ fichier, fragment, motif }` ; elle ne masque pas les commentaires.

## Décisions

- **D1 — Une date du jour, dans un test, vient de trois sources** : une `HorlogeFixe` (tout test qui asserte une date
  produite par le serveur) ; une date écrite en dur ; ou le helper `aujourd_hui_suisse()` de sa crate de tests (montage qui
  veut « un exercice qui couvre aujourd'hui »). Jamais `Utc::now().date_naive()` ni `Local::now()`.
- **D2 — La garde Rust se ferme** : la liste d'exemptions de `date_du_jour_garde.rs` ne porte plus que l'entrée permanente
  de `jour_suisse` ; tous les fichiers de `src` et de `crates/*/tests` comptent 0 (égalité stricte, C-e350-24).
- **D3 — Les specs E2E** prennent le jour par `todayInZurich()` ; l'arithmétique sur une date passe par **un** helper
  partagé (`addDaysIso(iso, n)` dans `helpers/`, calcul en `Date.UTC` sur un jour donné, sans lire l'horloge) ; pas de
  second helper du jour.
- **D4 — La garde frontend s'étend aux specs** : `local-date-guard.test.ts` parcourt aussi `tests/e2e/**` et relève
  `toISOString()` suivi de `.slice(0, 10)` **même coupé sur plusieurs lignes**, et `getFullYear(`, `getMonth(`, `getDate(`
  (hors `getUTC*`) sur un `new Date()` sans argument. L'allowlist des specs est **vide à la fin** : l'arithmétique pure passe
  par `addDaysIso`, qui n'est pas relevé.

## Critères d'acceptation

1. **AC1 — Tests Rust à la nuit** : les tests de la liste ci-dessus n'emploient plus que D1 ; la garde (D2) est verte avec
   la seule entrée permanente ; rouge sur la 35-5b fusionnée (entrées 35-5c encore présentes, comptes différents).
2. **AC2 — Verdict par fichier** au Change Log : pour chaque fichier converti, la source retenue (D1) ligne lue citée ; un
   test qui asserte une date produite par le serveur fixe l'horloge, il ne prend pas `aujourd_hui_suisse()`.
3. **AC3 — Specs E2E** (D3) : plus aucun `toISOString().slice(0, 10)` ni forme Node locale du jour dans `tests/e2e/**` ;
   la garde étendue (D4) rougit sur la 35-5b fusionnée (les sites restants) et passe à la fin, allowlist des specs vide.
4. **AC4 — Sondes** au Change Log : un `Utc::now().date_naive()` ajouté à un test Rust rougit ; une forme coupée sur deux
   lignes aussi ; un `new Date().toISOString().slice(0, 10)` ajouté à une spec rougit, coupé ou non ; `getDate()` sur
   `new Date()` aussi.
5. **AC5 — Rien ne change hors des tests** : `git diff --stat` ne touche ni `crates/*/src/**` hors modules `#[cfg(test)]`, ni
   `frontend/src/**` hors `local-date-guard.test.ts`, ni `crates/kesh-db/migrations/`.
6. **AC6 — Gates et E2E** : gate complet, `scripts/gate-ci.sh`, frontend complet et E2E complet au dernier commit de code ;
   les échecs E2E nommés contre `docs/testing.md` § « Les échecs attendus ».
7. **AC7 — CHANGELOG, PR** : aucun point au CHANGELOG (aucun changement visible) ; la PR porte `refs #929`.

## Tests à écrire, et la preuve du rouge

AC1, AC3 : les gardes, rouges sur la 35-5b fusionnée. Journal `kesh-gate-logs/e355c/rouges.log`.

**Mutations** : M1 un test remis à `Utc::now().date_naive()` (garde Rust) ; M2 la même forme coupée sur deux lignes ; M3
`todayInZurich` remplacé par `toISOString().slice(0, 10)` dans une spec (garde frontend) ; M4 `addDaysIso` qui lit l'horloge
(garde : relevé).

## Tâches

- [ ] Étape 0 : partir de la 35-5b fusionnée ; relever la liste d'exemptions restante de la garde Rust et les sites E2E
      restants ; trier l'arithmétique pure ; Change Log.
- [ ] Conversion des tests Rust (D1) ; garde Rust fermée (D2).
- [ ] Specs E2E (D3), `addDaysIso`, garde frontend étendue (D4).
- [ ] Gates : `test-fast.sh --ci` complet à chaque commit (tests de `kesh-db`), `scripts/gate-ci.sh` au dernier, frontend
      complet, E2E complet au dernier commit de code.

## Dev Notes

### Fichiers communs — règle d'union C-e350-8

`kesh-db/src/repositories/journal_entries.rs` (`mod tests`) avec la **35-8**, qui part de la 35-5c fusionnée ;
`frontend/tests/e2e/helpers/api-fixtures.ts` avec la **35-2a** (paramètre `city`) : hunks distincts ;
`frontend/src/lib/shared/utils/local-date-guard.test.ts` avec la 35-5a (entrée du verrou retirée) : la 35-5c part d'elle.

### Ce qui doit rester vrai

- Les tests gardent leur objet : une conversion ne change pas ce qu'un test prouve, seulement d'où vient sa date. Un test
  qui devient vide de sens après conversion se signale au Change Log.
- Fixtures synthétiques seulement.

### Références

- Issue #929 ; fiches 35-5a et 35-5b ; #889 et `local-date-guard.test.ts`.
- `CLAUDE.md` : § *Exception `kesh-db`* ; § *Inventorier les sites NON RÉSOLUS* ; § *Test Locally First* (`cargo test`
  contre `nextest`).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

- Fiche rédigée à la remédiation de la validation P1 de la 35-5b (C-e350-22) ; validation à lancer.

### File List

## Change Log

- 2026-11-12 — Fiche rédigée (découpage de la 35-5b, V1-8 de sa validation P1 ; C-e350-22). Validation non lancée.
