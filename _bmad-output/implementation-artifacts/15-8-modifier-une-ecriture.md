# Story 15.8 : Modifier ou supprimer une écriture tant que son exercice est ouvert

## Status

split

⛔ **CORPS VIDÉ — cette fiche ne contient plus ni critères, ni tâches, ni inventaire.** Elle ne garde que les pointeurs
vers ses deux sous-stories et l'historique des passes qui ont conduit au découpage. *(La définition du statut `split`
l'impose ; précédents : 15-1, 15-5, 17-2.)* La version complète d'avant découpage se lit au commit `0c43bd1d`.

**Issue : [#532]** — CR du Project Lead, 2026-10-08, constat de recette de la v0.12.1. ⚠️ **URGENTE.** Révise la
Story 24-4b (gel inconditionnel, #380).

## Les deux sous-stories

| | fiche | ce qu'elle porte | issue |
|---|---|---|---|
| **15-8a** | `15-8a-modifier-une-ecriture.md` | **La modification** (`PUT`) d'une écriture manuelle et de l'écriture d'ouverture : le cadre (exercice ouvert, aucun exercice postérieur clos, ni contre-passée ni contre-passation, aucune pièce, pas un paiement détaché, hors période verrouillée), la garde `modification_guard` et son garde-fou d'inventaire, l'audit avant/après par utilisateur ou clé d'API, le rejeu sur interblocage, le motif d'écran du détail, « Modifier » sur la fiche et le formulaire en mode édition, le manuel et la documentation de ce geste | `refs #532` |
| **15-8b** | `15-8b-supprimer-une-ecriture.md` | **La suppression** (`DELETE`) dans le même cadre, l'historique visible sur la fiche (« Modifiée », « Historique »), la suppression de l'ouverture qui rouvre la génération, le retrait d'`ENTRY_IS_POSTED`, et ce qui reste du manuel | `closes #532` |

⚠️ **L'ordre n'est pas indifférent** : **15-5a → 15-8a → 15-8b**. La 15-8a s'écrit contre le contrat de refus des
comptes posé par la 15-5a (`ACCOUNT_NOT_POSTABLE`, `exempt_ids` retiré, section `## [0.13.0]` du CHANGELOG) et se
rebase sur `main` après son merge — **fait** le 2026-10-08, sur `b11a074a`, à la validation P3 ; la 15-8b réutilise la garde, les types et l'écran de la 15-8a, et ne commence
qu'après son merge.

## Pourquoi le découpage

La validation P2 (deux lentilles Opus) a relevé (finding F6, MEDIUM) que la story franchissait le **critère de
périmètre** de la § *Règle de splitting préventif* : comptée à la granularité de la règle — « modules métier de premier
niveau » —, elle touchait bien plus de cinq modules, là où la fiche n'en comptait que cinq « zones » et omettait
`kesh-report`. Le signal a été déclaré ; l'orchestrateur a découpé selon la ligne **pré-déclarée** par C-15-8-9 — la
modification d'abord, qui est le geste que Guy attend (corriger son ouverture inversée), la suppression ensuite
(choix **C-15-8-17**). Les remédiations de la P2 ont été appliquées **dans les deux fiches filles**, pas ici.

## Décisions prises pour la story et où elles vivent

| choix | objet | fiche |
|---|---|---|
| C-15-8-1 | la clé 15-8 passe à #532 | les deux |
| C-15-8-2 | la suppression rouverte, dans le même cadre | 15-8b |
| C-15-8-3 | ni numéro ni exercice ne changent | 15-8a |
| C-15-8-4 | contrôles de saisie stricts, sans *grandfathering* de compte (réversibilité corrigée par C-15-8-18) | 15-8a |
| C-15-8-5 | le motif de refus réutilise l'inventaire de la contre-passation | 15-8a (pose), 15-8b (emploie) |
| C-15-8-6 | l'historique se lit au journal d'audit, depuis la fiche | 15-8b |
| C-15-8-7 | modifier et supprimer depuis la fiche, pas depuis la liste | les deux |
| C-15-8-8 | `PUT` et `DELETE` ouverts aux clés `read-write`, trace portant la clé | les deux |
| C-15-8-9 | découpage non déclenché, ligne pré-déclarée — **révisé par C-15-8-17** | — |
| C-15-8-10 | ordre des verrous du `PUT` | 15-8a |
| C-15-8-11 | `PUT` identique sur compte archivé → 400 — **révisé par C-15-8-18** (deux refus) | 15-8a |
| C-15-8-12 | après suppression de l'ouverture seule, numéro 2 | 15-8b |
| C-15-8-13 | garde d'écriture et motif d'écran | 15-8a |
| C-15-8-14 | rôle Consultation : boutons masqués | les deux |
| C-15-8-15 | formulaire rétabli par inversion du gel | 15-8a |
| C-15-8-16 | préparation extraite du `POST` avant son pré-contrôle d'exercice | 15-8a |
| C-15-8-17 | découpage 15-8a / 15-8b | les deux |
| C-15-8-18 | dépendance à la 15-5a, deux refus de compte | 15-8a |
| C-15-8-19 | rejeu sur interblocage, cycles nommés, Pattern 5, projets lus en `LOCK IN SHARE MODE` — **révisé par C-15-8-23** | 15-8a (`PUT`), 15-8b (`DELETE`) |
| C-15-8-20 | paiement détaché d'une facture fournisseur annulée, gelé par la trace d'audit | 15-8a (pose), 15-8b (emploie) |
| C-15-8-21 | modale de conflit écartée de l'inversion, prop `onStale`, clés `journal-entries-*` | 15-8a |
| C-15-8-22 | modification et suppression refusées dès qu'un exercice **postérieur** est clos (`LATER_FISCAL_YEAR_CLOSED`) | 15-8a (pose), 15-8b (emploie) |
| C-15-8-23 | projets existants lus par une lecture ordinaire après le verrou de l'écriture — **révise C-15-8-19** | 15-8a |
| C-15-8-24 | `modification_guard` sur une connexion, `pool.acquire()` à l'écran ; corrige les renvois de C-15-8-5, 10, 12, 13 | les deux |
| C-15-8-25 | paiement détaché : trois réserves écrites, dette #541, requête bornée par société | 15-8a (pose), 15-8b (emploie) |
| C-15-8-26 | `README.md:29` édité par la 15-8b seule | 15-8b |
| C-15-8-27 | contrôles de manuel sur les PDF aplatis, motifs complétés, `.ftl` ; message d'`ENTRY_IS_POSTED` réécrit | les deux |
| C-15-8-28 | dérogation écrite à la règle de splitting, repli 15-8a-1 / 15-8a-2 | 15-8a |
| C-15-8-29 | l'exercice postérieur clos ne garde le `DELETE` que sur le chemin de la route | 15-8b |

## Change Log

| date | ce qui s'est passé |
|---|---|
| 2026-10-08 | **Spec** (`bmad-create-story`, Opus 5.5, autonomie Epic 15). Choix C-15-8-1 à C-15-8-9. Aucune migration. Découpage examiné, non déclenché (cinq zones), ligne de découpe pré-déclarée. |
| 2026-10-08 | **Validation P1** (deux lentilles Sonnet, contexte frais, prompt versionné `15-8-validate-prompt-p1.md` ; rapports `target/gate-logs/15-8-p1-{R,F}.md`). **R** : 0 CRITICAL, 1 HIGH, 4 MEDIUM, 6 LOW ; **F** : 0 CRITICAL, 1 HIGH, 3 MEDIUM, 4 LOW ; R1 = F1 (même défaut), R7 = F7, R3 ≈ F4 + F8 — **tout appliqué**. **HIGH (R1/F1)** : la fiche plaçait deux lectures ordinaires (projets, borne de verrou) avant le `FOR UPDATE` de l'écriture ; en `REPEATABLE READ` la garde lisait alors une vue antérieure à l'attente du verrou, et un `PUT` pouvait réécrire une écriture contre-passée pendant cette attente — le motif même que la 24-4b fermait. Remédié : `FOR UPDATE` de l'écriture premier acte (D2, D4), ordre des verrous écriture → [sentinelle → projets] → exercice (C-15-8-10), refus projet différé à l'étape 6, test à deux connexions avec mutation à tuer (AC 8). **MEDIUM** : audit par clé d'API sur `update` et `delete_in_tx` (`for_actor`, AC 3 et 9 — F2) ; formulaire rétabli par **inversion du commit du gel** `08e20353` sur cinq fichiers, la page de liste, la spec E2E et `i18n-keys.test.ts` exclus, mode édition spécifié (dates bornées à l'exercice de l'écriture, `FISCAL_YEAR_CLOSED` hors `notifyMissingFiscalYearOrFallback`, 409 de course) — F3, C-15-8-15 ; `modification_guard` / `modification_blocker` nommées, `ModificationBlocker`, table de correspondance écran ↔ écriture qui rend l'AC 14 testable (R5, C-15-8-13) ; inventaires : huit tests unitaires de l'ancien `update` (sept rétablis, un **inversé**), quatre specs Playwright **remplacées** par des parcours depuis la fiche plus trois neuves, AC 3 / AC 4 tranché (C-15-8-11), sites périmés ajoutés (`user-manual.tex:380`, table des matières, `admin-manual.tex:1834-1838`, `period_lock_e2e.rs:3-10`, `balance_sheet.rs:31-34`, `accounts.rs:544-545`), grep de l'AC 17 sur le dépôt entier (R2, R3, F4) ; T0, T7-bis (Playwright) et Vitest ajoutés (R4). **LOW** : repli `code()` (R6), « sept motifs sur sept colonnes » (R7/F7), clés i18n neuves nommées et replis Svelte localisés (R8), AC 5 exercice clos seul, AC 20 export, complément 25-7 et numéro 2 asserté à l'AC 13, I2 borné hors restauration (R9, R10, C-15-8-12), rôle Consultation sans boutons (R10, C-15-8-14), `UpdateJournalEntryRequest` et `prepare_new_journal_entry` définis (R11, C-15-8-16), garde-fou D3 monté sur le squash avec un troisième contrôle par nom de colonne (F5, F6), T0 d'inventaire au sol (F8). **Propagation** : symptômes grepés dans la fiche (`huit`, numéros d'étape de D4, `d2910022`, `NewAuditLogEntry::user`, ordre `IS_A_REVERSAL`/`ALREADY_REVERSED`, `test_schema_guard`, lignes `:1835-1838`). **Recompte** (fiche, après remédiation) : 20 AC, 12 tâches (T0 à T10 et T7-bis), 4 invariants, 10 décisions, choix C-15-8-1 à 16. **Signalé à l'orchestrateur** : audit par clé absent de `create`, `reverse` et `invoices::unvalidate` (préexistant, hors périmètre) ; collision attendue de D3 avec la 15-1a. |
| 2026-10-08 | **Validation P2** (deux lentilles **Opus**, contexte frais, lecture seule — rotation D6 : Sonnet en P1, Opus en P2 ; prompt versionné `15-8-validate-prompt-p2.md` ; rapports `target/gate-logs/15-8-p2-{R,F}.md`). **R** (regression hunter) : 0 CRITICAL, 0 HIGH, 4 MEDIUM, 13 LOW ; **F** (adversaire de la conception) : 0 CRITICAL, **1 HIGH**, 5 MEDIUM, 6 LOW. Doublons : R2-1 = F4, R2-3 ≈ F2, R2-4 ≈ F5, R2-2 ≈ F11 (LOW), R2-7 = F9, R2-13 = F10 — soit **1 HIGH, 6 MEDIUM et 16 LOW distincts**. **Tout appliqué**, dans les fiches filles. **HIGH (F1)** : la 15-5a, ordonnée avant, a déjà changé le contrat du refus de compte (`ACCOUNT_NOT_POSTABLE`, `exempt_ids` retiré, `## [0.13.0]`) — les deux refus séparés partout, rebase après son merge (C-15-8-18, C-15-8-11 révisé). **MEDIUM** : rejeu sur interblocage, trois cycles hérités nommés, phrase « jamais une attente infinie » retirée, Pattern 5, projets existants lus en `LOCK IN SHARE MODE` pour que la vue s'ouvre après le dernier verrou (F2, R2-2, R2-3 — C-15-8-19) ; paiement détaché d'une facture fournisseur annulée gelé par la trace d'audit `supplier_invoice.cancelled`, faute de marqueur structurel d'origine (F3 — C-15-8-20) ; modale de conflit écartée de l'inversion, prop `onStale` (R2-1, F4 — C-15-8-21) ; inventaire des sites élargi (migration `20260830000001` en « À NE PAS toucher », `admin-manual.tex:1969`, `trial_balance.rs:42-45`, `reconciliation_cancel.rs:5-8`, `journal_entry_number_sequences.rs:17`, repli `errors.rs:1146`, commentaires de `+page.svelte` et de `journal_entry_reversal_e2e.rs`, `user-manual.tex:1331-1337`), contrôle aplati élargi et appliqué aussi aux `.tex` (R2-4, F5) ; **découpage** (F6 — C-15-8-17). **LOW** appliqués tels que proposés (R2-5 à R2-17, F7 à F12). **Signal D5 déclaré** : sévérité HIGH en P1 et en P2 (« égale ») ; le HIGH de P2 est un défaut d'origine neuf (frontière avec la 15-5a), mais **trois MEDIUM naissent de la remédiation P1** (R2-1 : l'inversion du gel ; R2-2 et R2-3 : le nouvel ordre des verrous) — c'est le recyclage que la règle désigne ; le découpage est décidé, par la ligne pré-déclarée. **Propagation** : symptômes grepés dans les deux fiches filles (`attente infinie`, `neuf codes`/`neuf valeurs`, `exempt_ids`, clés `journal-entry-edit-conflict` / `journal-entry-line-account-unusable`, `23-27`, `1059-1068`, `invoices.rs:2521`, l'ancienne commande `grep -rn "acquire_company_sentinel_lock"`, renvois d'AC et de tâches renumérotés) — résidus seulement aux sites qui citent l'erreur corrigée. **Recompte** : 15-8a — 18 AC, 11 tâches (T0 à T10), 4 invariants, 10 décisions, 10 codes d'écran ; 15-8b — 12 AC, 9 tâches (T0 à T8), 5 décisions ; registre — C-15-8-1 à 21 (`grep -c '^## C-15-8-'` → 21). **Signalé à l'orchestrateur** : issue pour une colonne structurelle du paiement détaché (C-15-8-20) ; audit par clé absent de `create`, `reverse` et `invoices::unvalidate` (déjà signalé en P1) ; collision attendue de D3 avec la 15-1a (point 2, et clé vers une ligne). |
