---
status: ready-for-dev
epic: 18
story: 18-1f
type: chore
parent: 18-1
issue: 180
created: 2026-06-26
depends_on: [18-1a, 18-1b, 18-1c, 18-1d, 18-1e]
baseline_commit: 6d5b52c
stepsCompleted: []
---

# Story 18-1f — Tests E2E + documentation (clôture épopée 18-1)

> Extraite de l'umbrella convergée [`18-1-comptabilisation-tva-achats.md`](18-1-comptabilisation-tva-achats.md).
> **Axe (f)** — AC11 (E2E + couverture complète) + AC12 (manuels user/admin + CHANGELOG + README).
> **Dernière sous-story de l'épopée 18-1.** Une fois `done`, ouvrir la **PR umbrella** de l'épopée (pattern
> 17-3 #171). Aucune nouvelle logique métier : tests E2E + synchronisation documentaire (politique CLAUDE.md
> « Synchroniser TOUTES les docs avant tout push »).

## User Story

**En tant que** mainteneur / fiduciaire de Kesh,
**je veux** un test E2E du décompte TVA et une documentation à jour (manuels, CHANGELOG, README),
**afin de** livrer l'épopée TVA (Epic 18) avec une couverture de bout-en-bout et des supports utilisateurs
fidèles aux fonctionnalités réellement disponibles.

## Contexte ground-truth (vérifié `main`-branche @ `6d5b52c`, après 18-1a..e)

### Couverture de tests EXISTANTE (ne pas réinventer — vérifier + compléter le gap)

L'essentiel d'AC11 (intégration) est **déjà couvert** par les sous-stories précédentes :

| Cas AC11 (umbrella) | Test existant | Fichier |
|---------------------|---------------|---------|
| Facture `vat_rate=0`/exempt → **aucune** ligne 2200 | `validate_zero_rate_no_vat_line` | `crates/kesh-db/tests/invoices_validate_vat.rs:113` |
| Facture multi-taux (positif + 0 %) → ligne 2200 pour taux > 0 seul | `validate_mixed_positive_and_zero_rate` (l.132) + `validate_multi_rate_two_vat_lines` (l.167) | idem |
| Ligne 0 %/exonérée présente dans le rapport | `vat_report_includes_zero_rate_exempt_row` | `crates/kesh-api/tests/vat_report_e2e.rs:353` |
| Écart de réconciliation `delta != 0` (+ isolation OD, anti-IDOR, seuil) | 9 tests `reconciliation_*` | `crates/kesh-report/tests/vat_report_reconciliation.rs` |
| TVA récupérable (achats seuls, bornes, NULL, anti-IDOR) | 7 tests `recoverable_*` | `crates/kesh-report/tests/vat_report_recoverable.rs` |
| Assistant achat TVA (E2E round-trip) | `vat-purchase-assistant.spec.ts` | `frontend/tests/e2e/` |

**Gap potentiel à vérifier (AC11 umbrella)** : « un test où l'**arrondi** d'un taux > 0 donne `0.00` → pas de
ligne 2200 ». À confirmer dans `invoices_validate_vat.rs` ; **si absent, l'ajouter** (T-F2). Un montant HT
minuscule (p.ex. `0.01` à 8.1 % → `line_vat_amount = 0.00` après arrondi) ne doit générer aucune ligne 2200
(contrainte `chk_jel_debit_credit_exclusive`, F-OPUS-1).

### E2E Playwright existant (pattern à répliquer)

- Harness : `frontend/tests/e2e/vat-purchase-assistant.spec.ts` — `seedTestState('with-company')` +
  `clearAuthStorage`, login `admin`/`admin123`, navigation `page.goto`. Pré-requis : MariaDB up +
  `KESH_TEST_MODE=true` + seed `with-company` + browsers Playwright
  (`PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` sur Ubuntu 26.04+, cf. mémoire projet).
- Page rapports : `frontend/src/routes/(app)/reports/+page.svelte` — onglet `{ id: 'vat', labelKey:
  'reports-vat' }` (l.245) ; `<VatReportView dto={vatReport} />` (l.338). Un bouton « Générer » produit le
  rapport pour la période/exercice sélectionnés.
- ⚠️ **Dette connue #172** (Playwright double-instance) : ne PAS multiplier les specs lourdes ; **un seul**
  spec E2E focalisé suffit pour l'épopée.

### Documentation EXISTANTE à synchroniser

- **CHANGELOG** : `CHANGELOG.md` — dernière entrée `## [0.2.0] — 2026-06-12`. Epic 18 est postérieur à
  0.2.0 → **nouvelle section `## [Non publié]`** (ou version cible à la prochaine release ; ne PAS inventer
  un numéro de tag — utiliser `[Non publié]` car le tag est décidé par Guy au moment de la release). Format
  Keep a Changelog FR (sections `### Ajouté` / `### Modifié` / `### Corrigé`), rédigé pour
  fiduciaires/PME.
- **README** : `README.md` — l.30 `**TVA suisse** — calcul et rapports par période *(à venir)*` → **retirer
  `*(à venir)*`** (livré). Ligne roadmap l.182 (`| v0.2 (suite) | E11 TVA Suisse, … | 🚧 En cours |`) :
  Epic 11 est clos (rétro 2026-06-14) et Epic 18 (Comptabilisation TVA) n'y figure pas (L2) → **refléter
  l'état** : la TVA est calculée + comptabilisée + réconciliée (E11 + E18 livrés), seul le décompte AFC
  officiel reste à venir. Action concrète : mentionner E18 / marquer la base TVA livrée sans sur-promettre le
  format e-décompte ESTV.
- **Site web (politique CLAUDE.md pré-push, M2)** : `website/roadmap.html` (l.234-235 « E11 — Swiss VAT :
  Per-period VAT computation, official VAT report formats, … », **sans badge Done**) + `website/index.html`.
  ⚠️ **E11 n'est que PARTIELLEMENT livré** : calcul + comptabilisation + décompte + réconciliation faits
  (E18), mais « official VAT report formats » (e-décompte AFC/ESTV) = **hors scope v0.2**. Donc **NE PAS**
  passer E11 « Done » automatiquement — c'est une **décision produit (gate Guy)** au moment de la PR umbrella.
  Scope 18-1f : **vérifier** la cohérence, ne pas sur-claim, et **signaler** l'arbitrage E11 Done/in-progress
  à Guy (pas de modification de badge sans son aval). Pas d'over-claim « official VAT report formats » livré.
- **Manuel utilisateur** : `docs/manual/fr/user-manual.tex` — section `\section{Rapports comptables}`
  (l.709) avec sous-sections Bilan/Compte de résultat/Balance/Journal/Visualisation. **Ajouter une
  sous-section `\subsection{Décompte TVA}`** : TVA due (sur ventes), TVA récupérable (impôt préalable sur
  achats via l'assistant), solde net dû à l'AFC, et le **bandeau de réconciliation** (signale un décompte qui
  ne correspond pas aux écritures — écriture validée modifiée à la main). PDF à régénérer (`latexmk -xelatex`,
  convention versionner le PDF cf. PR #102).
  - ⚠️ **M3 (Pass 1) — incohérences internes à corriger** : 3 mentions obsolètes « TVA à venir Epic 11 »
    contredisent l.485 (qui dit déjà « écriture créée automatiquement … + 2200 TVA »). À mettre à jour :
    `l.300` (retirer « (TVA à venir Epic 11) »), `l.445` (remplacer « TVA Epic 11 à venir » par les taux
    actuels seuls), `l.1014` (« TVA non encore implémentée (Epic 11 v0.1) » → renvoi « voir §Décompte TVA »).
    Re-grep `grep -nE "à venir|pas encore|non encore" docs/manual/fr/user-manual.tex` avant de figer (les
    n° de ligne peuvent bouger).
- **Manuel administrateur** : `docs/manual/fr/admin-manual.tex` — a déjà `\subsection{Loi sur la TVA (LTVA)}`
  (l.1645). ⚠️ **M1 (Pass 1) — le corps existant (l.1647 « Kesh v0.1 ne couvre pas encore la TVA (prévue
  Epic 11) … ») est OBSOLÈTE → le RÉÉCRIRE** (pas seulement ajouter), sinon le manuel devient contradictoire.
  Nouveau contenu : la TVA est comptabilisée (Epic 18) ; documenter la **configuration des comptes TVA**
  (`Paramètres → Facturation` expose les 3 comptes TVA due `2200`, impôt préalable `1171`, décompte `2206`
  posés par 18-1a, requis pour la comptabilisation) + renvoi au user-manual pour l'usage. Conserver/adapter
  la mention LTVA Art. 70 (e-décompte ESTV reste hors scope v0.2).

## Décisions figées (héritées umbrella — NE PAS re-litiger)

- Pas de nouvelle logique métier (tests + doc uniquement).
- CHANGELOG : section `[Non publié]` (le tag de version est un gate Guy, hors scope dev).
- E2E : **un seul** spec focalisé (dette #172) — décompte TVA happy-path. Le bandeau de réconciliation
  (qui exige une édition manuelle d'une écriture validée) est couvert au niveau **intégration** (9 tests
  18-1e) ; ne PAS forcer un E2E fragile de falsification d'écriture.
- Régénération PDF des manuels : si `latexmk`/`xelatex` n'est pas disponible dans l'environnement, **éditer
  le `.tex` + documenter que le PDF doit être régénéré** (ne pas bloquer la story sur l'outillage LaTeX ;
  noter dans le Change Log). Vérifier la disponibilité de `latexmk` avant (T-F5).

## Acceptance Criteria

- **AC1 — Gap test arrondi** : un test d'intégration vérifie qu'une facture à taux > 0 dont la TVA arrondie
  tombe à `0.00` ne génère **aucune** ligne 2200 (F-OPUS-1). Si déjà présent → le référencer ; sinon
  l'ajouter dans `invoices_validate_vat.rs`.
- **AC2 — E2E décompte TVA** : un spec Playwright (`frontend/tests/e2e/vat-report.spec.ts` ou similaire)
  couvre le happy-path : login admin → page Rapports → onglet TVA → générer → le décompte affiche TVA due,
  TVA récupérable et solde. Suit le pattern `seedTestState('with-company')` + harness existant. Le spec passe
  en local (pré-requis Playwright documentés).
- **AC3 — CHANGELOG** : section `## [Non publié]` ajoutée en tête (sous le titre), documentant l'épopée TVA
  Epic 18 (#180) en FR pour fiduciaires/PME : comptes TVA dans le plan comptable + configuration ;
  comptabilisation automatique de la TVA due à la validation des factures de vente ; assistant de saisie des
  achats avec impôt préalable ; décompte TVA (TVA due − récupérable = solde net AFC) ; **réconciliation**
  rapport ↔ grand livre avec alerte d'écart. Format Keep a Changelog FR.
- **AC4 — README + site web** : marqueur `*(à venir)*` retiré de la ligne « TVA suisse — calcul et rapports
  par période » ; ligne roadmap l.182 mise à jour (E11 + E18 base TVA livrée, décompte AFC officiel à venir).
  `website/roadmap.html` + `index.html` vérifiés : pas d'over-claim « official VAT report formats » livré ; le
  badge « Done » de E11 **n'est pas modifié sans aval Guy** (gate produit, M2) — l'arbitrage est signalé dans
  le Change Log / la description de PR umbrella. Aucune autre feature livrée ne reste marquée `(à venir)` à
  tort du fait de l'épopée.
- **AC5 — Manuel utilisateur** : sous-section « Décompte TVA » ajoutée sous « Rapports comptables » (TVA due,
  récupérable, solde net AFC, bandeau de réconciliation), cohérente avec l'UI réelle (`VatReportView`). **+ les
  3 mentions obsolètes « TVA à venir Epic 11 » (l.300/445/1014) corrigées** (M3) — plus aucune contradiction
  interne (`grep -nE "à venir|pas encore" user-manual.tex` ne doit plus rien retourner sur la TVA).
- **AC6 — Manuel administrateur** : sous-section LTVA (l.1645) **réécrite** — la phrase obsolète « ne couvre
  pas encore la TVA » (l.1647) supprimée (M1) ; configuration des comptes TVA (`Paramètres → Facturation`,
  3 comptes) documentée. PDF régénérés si l'outillage est disponible, sinon édition `.tex` + note de
  régénération dans le Change Log.
- **AC7 — Quality gate** : `cargo fmt/clippy/build/test` (au moins les crates touchés + non-régression) ;
  frontend `check`/`lint-i18n-ownership`/`test:unit`/`build` ; E2E lancé localement (best-effort selon
  l'outillage Playwright). Aucune régression.

## Tasks (T-F1..T-F7)

- **T-F1 — Vérifier la couverture AC11** : confirmer par lecture que les cas vat_rate=0 / multi-taux / delta /
  récupérable sont couverts (cf. tableau ground-truth). Documenter le mapping dans le Change Log.
- **T-F2 — Gap test arrondi→0.00** (AC1) : `grep`/lire `invoices_validate_vat.rs` ; si aucun test ne couvre
  « taux > 0 mais TVA arrondie = 0.00 → pas de ligne 2200 », ajouter `validate_rounds_to_zero_no_vat_line`
  (HT ≈ `0.01`, taux 8.1 % → `line_vat_amount = 0.00` → écriture = créance + produit, **0** ligne 2200).
  Réutiliser le helper `create_and_validate` (l.46) + fixture.
- **T-F3 — E2E décompte TVA** (AC2) : nouveau `frontend/tests/e2e/vat-report.spec.ts`. Seed `with-data`
  (= `with-company` + contact + produit ; **aucun preset ne contient de facture validée** — L1, donc le test
  crée et valide la facture lui-même), login, créer + valider une facture avec TVA via l'UI, aller à
  `/reports`, onglet TVA, générer, asserter l'affichage de TVA due / récupérable / solde. S'inspirer de
  `vat-purchase-assistant.spec.ts` et `invoices.spec.ts`. Garder **un seul** spec (dette #172).
- **T-F4 — CHANGELOG** (AC3) : insérer `## [Non publié]` sous l'en-tête, avant `## [0.2.0]`.
- **T-F5 — Manuels LaTeX** (AC5/AC6) : éditer `user-manual.tex` — **ajouter** sous-section Décompte TVA
  **ET corriger** les 3 mentions obsolètes « TVA à venir » (M3, re-grep `grep -nE "à venir|pas encore"
  user-manual.tex`) ; `admin-manual.tex` — **réécrire** la sous-section LTVA (l.1645, supprimer « ne couvre
  pas encore la TVA » l.1647, M1) + config des 3 comptes TVA. Vérifier `which latexmk xelatex` ; si dispo →
  régénérer les PDF (`cd docs/manual/fr && latexmk -xelatex user-manual.tex admin-manual.tex`) et committer
  les `.pdf` ; sinon noter la régénération requise dans le Change Log. **Vérification finale** : `grep -nE
  "à venir|pas encore|non encore" docs/manual/fr/*.tex` ne doit plus rien retourner concernant la TVA.
- **T-F6 — README + site web** (AC4) : retirer `*(à venir)*` TVA (l.30) + mettre à jour la ligne roadmap
  l.182 (E11+E18 base TVA livrée). Vérifier `website/roadmap.html` (l.234-235) + `index.html` : pas
  d'over-claim ; **ne pas** toucher le badge « Done » de E11 sans aval Guy (M2) — signaler l'arbitrage dans le
  Change Log + description PR umbrella.
- **T-F7 — Quality gate + Change Log final** (AC7) : `cargo fmt/clippy/build/test` (crates touchés +
  non-régression) ; frontend `check`/`lint-i18n-ownership`/`test:unit`/`build` ; E2E best-effort (rapporter
  fidèlement s'il n'a pas pu tourner). Change Log final + bilan doc-sweep.

## Hors-scope

- Décompte AFC officiel / e-décompte ESTV, TDFN, multi-période (hors v0.2, cf. umbrella).
- Traductions DE/IT/EN des manuels (v0.2+ ; FR seul comme l'existant).
- Nouvelle logique métier (tout est livré 18-1a..e).

## Risques

- **Outillage LaTeX** : `latexmk`/`xelatex` peut manquer dans l'environnement → ne pas bloquer la story ;
  éditer le `.tex` et documenter la régénération PDF requise (Guy régénère à la release).
- **E2E flaky / dette #172** : un seul spec, pré-requis Playwright documentés ; si l'environnement ne permet
  pas de lancer Playwright, livrer le spec + documenter qu'il n'a pas pu être exécuté localement (ne pas
  prétendre l'avoir fait — politique CLAUDE.md « rapporter fidèlement »).
- **CHANGELOG version** : ne PAS inventer de numéro de tag — `[Non publié]` (le tag est un gate Guy).

## Prochaine étape

`bmad-create-story validate 18-1f` (rotation Sonnet→Haiku→Opus, contexte frais) avant `bmad-dev-story`. Une
fois `done` → **PR umbrella de l'épopée 18-1** (toutes les sous-stories a..f), pattern 17-3 #171.

## Change Log

### `bmad-create-story validate 18-1f` — cycle adversarial (CLAUDE.md Review Iteration Rule)

| Passe | Modèle | Findings > LOW | Points clés |
|-------|--------|----------------|-------------|
| 1 | Sonnet 4.6 | 3 (3M) | **Ground-truth complet** (tests `invoices_validate_vat.rs` 113/132/167 + helper l.46, `vat_report_e2e.rs:353`, 9 reconciliation + 7 recoverable confirmés ; gap arrondi→0.00 RÉELLEMENT absent, math `0.01×8.1%=0.00` vérifiée ; README l.30 `(à venir)` présent ; CHANGELOG dernière `[0.2.0]` ; manuels sections confirmées ; harness E2E + onglet `vat` l.245). **M1** : `admin-manual.tex:1647` « Kesh v0.1 ne couvre pas encore la TVA » → à RÉÉCRIRE (pas ajouter). **M2** : `website/roadmap.html` (l.234-235 E11 sans badge Done) absent du scope alors que CLAUDE.md l'impose pré-push → ajouté T-F6, badge E11 Done = **gate Guy** (E11 partiel : décompte AFC officiel hors scope). **M3** : `user-manual.tex` 3 mentions obsolètes « TVA à venir Epic 11 » (l.300/445/1014) contredisent l.485 (TVA déjà postée) → à corriger. LOW : L1 `with-data` n'a pas de facture validée (seed précisé) ; L2 README l.182 action E11/E18 précisée. **Tous patchés.** |
| 2 | Haiku 4.5 | 0 ✅ | **0 hallucination, ground-truth complet.** Patches Pass 1 confirmés par grep : M3 mentions user-manual l.300/445/1014 exactes (+ vérifié qu'il n'y en a pas d'autres TVA ; l.511 « envoi email à venir » hors-scope, correctement écarté) ; M1 admin-manual l.1647 exact ; gap arrondi→0.00 réellement absent ; README l.30 ; CHANGELOG format `### Added/Sécurité` ; preset E2E `with-data` existe (`test-state.ts:32`) ; onglet `vat` `+page.svelte:245`. **Numéros comptes TVA vérifiés corrects** : `2200` (3 charts), `1171`+`2206` (migration `20260614000001_vat_accounts_config.sql`). Sous-section « Décompte TVA » bien absente (à créer). Badge E11 = gate Guy respecté. **0 > LOW.** |

**Trend findings > LOW** : Pass 1 (Sonnet) 3 (3M) → Pass 2 (Haiku) **0 ✅**. Rotation Sonnet→Haiku, contexte frais, grep ground-truth. Pass 1 = doc-sweep (3 incohérences manuels/website ratées initialement) ; Pass 2 confirme + vérifie les numéros de comptes. **Cycle validate CONVERGÉ Pass 2, 0 > LOW. Prochaine : `bmad-dev-story 18-1f` (Opus 4.8).**
