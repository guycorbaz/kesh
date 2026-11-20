# Revue de code P1 — 43-7 (#823), le plafond de l'écart d'un règlement groupé réglé par l'administrateur, trois lentilles B, E, A (Opus, lecture seule)

Tu es **une** des trois lentilles de la passe 1 de `bmad-code-review` de la story **43-7** de Kesh (release 0.19.0, Epic 43,
issue #823). Modèle : **Opus**. Contexte frais. Travaille en français soigné. L'orchestrateur te dit laquelle tu es :

- **B — Blind Hunter** : le diff seul, sans la fiche ; cherche les défauts que le code porte en lui-même (un `PUT` qui
  écrit sans changer la version, un champ qui manque à l'audit, une valeur qui se perd entre l'écran et la base, une
  échelle fausse, un refus qui ne nomme pas le champ, un champ de saisie sans état d'erreur).
- **E — Edge Case Hunter** : chaque branche et chaque frontière — valeur 0, 0.01, 100, 100.00, 100.01, −0.01, 1.005,
  « 1.250 », « 1'250 », vide, espaces, virgule décimale, `5.0000` (quatre décimales), valeur stockée à quatre décimales par
  SQL direct (`2.5049`, `5.0060`), champ **absent** du corps (client ancien) × champ présent, version périmée × plafond
  seul, deux `PUT` concurrents, relecture sur conflit de version à l'écran.
- **A — Acceptance Auditor** : la fiche contre le code, AC par AC (AC1 à AC6), test par test, mutation par mutation
  (M1 à M3, journaux ci-dessous), manuel `.tex`, `docs/api-external.md`, i18n des quatre locales, captures.

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-43-7 log --oneline 241839568..HEAD
git -C /home/gcorbaz/devel/kesh-43-7 diff 241839568 HEAD -- crates frontend/src docs
```

Lis le diff **aplati** (un seul diff de `241839568`, le commit de contrat de la 43-1a, à `HEAD`). Fichiers principaux :
`crates/kesh-db/src/repositories/company_invoice_settings.rs` (`is_no_op_change`, l'`UPDATE`, `settings_snapshot_json`),
`crates/kesh-db/src/entities/company_invoice_settings.rs` (`CompanyInvoiceSettingsUpdate`),
`crates/kesh-api/src/routes/company_invoice_settings.rs` (réponse, requête, contrôle, `GROUP_SETTLEMENT_MAX_DIFFERENCE_CEILING`),
`frontend/src/routes/(app)/settings/invoicing/+page.svelte`, `frontend/src/lib/features/invoices/group-settlement-max.ts`,
catalogues `crates/kesh-i18n/locales/*/messages.ftl` (six clés `settings-invoicing-group-max-*`),
`docs/manual/fr/admin-manual.tex` (§ *Écart admis pour un règlement de plusieurs factures*), `docs/api-external.md`.
Tests : `crates/kesh-api/tests/group_settlement_max_setting_e2e.rs` (neuf), `crates/kesh-db/tests/grouped_settlements.rs`
(`the_ceiling_set_through_the_settings_is_read_by_the_gesture`), `company_invoice_settings_repository.rs`,
`settings-invoicing-page.test.ts`, `group-settlement-max.test.ts`.

Fiche : `_bmad-output/implementation-artifacts/43-7-plafond-difference-reglage.md` (lentille A ; B la lit **après** son
examen à l'aveugle, pour classer). Contrat de la 43-1a : `/home/gcorbaz/devel/kesh-gate-logs/e43/t3.md`. Rapports de
validation : `/home/gcorbaz/devel/kesh-gate-logs/e43/validation-fiches-contre-t3.md` et `validation-fiches-p-ciblee-2.md`.
Journaux : mutations `/home/gcorbaz/devel/kesh-gate-logs/e437/mut/M{1,2,3}.log` et `mut-e437.sh`, gates
`/home/gcorbaz/devel/kesh-gate-logs/e437/*.log` (le Dev Agent Record de la fiche les cite).

## Axes obligatoires

1. **Les trois sites d'écriture** (`CompanyInvoiceSettingsUpdate`, `UPDATE`, `is_no_op_change`) et l'audit `{before, after}` :
   un `PUT` qui ne change que le plafond écrit-il, incrémente-t-il la version, audite-t-il avec l'acteur utilisateur ? Une
   comparaison de `Decimal` (`5.0000` contre `5.00`) juge-t-elle « sans changement » à tort ou à raison ?
2. **Les contrôles du `PUT`** : bornes (0 à 100.00), échelle (deux décimales sur la forme normalisée), refus
   `400 VALIDATION_ERROR` qui **nomme le champ**, champ absent préservé **sans contrôle** (une valeur stockée à quatre
   décimales ne bloque pas l'enregistrement d'autre chose). Le contrôle est-il placé avant la persistance ? La `CHECK`
   de la colonne reste-t-elle un filet jamais atteint ?
3. **L'échelle à l'affichage** (`GET` à deux décimales, la colonne à quatre) et la limite L-4 écrite (5.0060 affiché
   5.01, écart 5.01 refusé) : est-elle dite au manuel et à la fiche, et le test la couvre-t-il ?
4. **L'écran** : la grammaire de saisie (`readGroupMax` sur `readJournalAmount` : « 1.250 » ambigu, « 1'250 » lu), les cinq
   refus (vide, ambigu, illisible, échelle, plage), le bouton désactivé, `aria-invalid`, `aria-describedby`, la relecture
   sur conflit de version (`OPTIMISTIC_LOCK_CONFLICT`), la valeur envoyée (`payload`).
5. **L'effet** (AC5) : le test du dépôt pose le plafond par `company_invoice_settings::update` (le chemin du `PUT`) puis le
   geste groupé refuse ±1.01 avec et sans compte, accepte ±1.00. Prouve-t-il le chemin, ou lit-il une valeur posée par SQL ?
   (La partie « la recherche de la 43-2 rend `maxDifference` » relève de la 43-2 : hors périmètre ici.)
6. **Le manuel** — `docs/manual/fr/admin-manual.tex` : le paragraphe dit-il vrai contre le code (défaut 5.00, de 0 à 100.00,
   dans les deux sens, compte choisi, arrondi sous le centime exclu, lecture au moment du geste, audit, limite L-4) ?
   **Le PDF n'est pas régénéré dans la story** (règle des branches d'epic : seul le `.tex` change ; les PDF se régénèrent à
   la release) : ne le compte pas comme manque. S'il t'en faut un contrôle, forme unique :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'` (le PDF actuel est celui
   de la 0.18.0 : le paragraphe n'y figure pas, c'est attendu).
7. **i18n** : six clés dans les quatre locales, mêmes noms, sens égaux, aucun doublon ; `sitesTotal` 3086 → 3092.
8. **Captures** : `/home/gcorbaz/devel/kesh-gate-logs/e437/captures/` (`1-defaut.png`, `2-ambigu.png`, `3-au-dela.png`,
   `4-enregistre.png`, thème clair) : lisibles, le refus se voit-il (bordure, message), le champ est-il étiqueté ?
9. **Tests et mutations** : M1 (borne haute retirée) tuée par `the_ceiling_bounds` ; M2 (`is_no_op_change`) et M3 (`UPDATE` qui
   réécrit `before`) tuées par `a_put_that_changes_only_the_ceiling_writes_it` et ses voisins : pour la bonne raison (lis les
   `M*.log`) ? Un test passe-t-il à vide ?

## Hors périmètre (déjà tranché)

Aucune migration (la colonne vient de la 43-1a). L'écran de la recherche et du geste (43-2, 43-3), le champ `maxDifference`
du refus (43-1b, 43-2). « Frais bancaires » ne désigne que le compte prérempli de la 43-3. Le refus d'une clé d'API et
d'un Comptable sur le `PUT` : `admin_pat_denied_e2e.rs` (la route y figure) et `a_comptable_cannot_set_the_ceiling`.
Deux défauts de **test** hérités de la 43-1a (`empreinte` : `SUM(version)` en `DECIMAL` ; une requête sur `api_key_id`) sont
signalés au Dev Agent Record : l'un est corrigé ici à l'identique de la branche de la 43-1a, l'autre n'est pas de cette story.

## ⛔ Ground-truth obligatoire

Toute affirmation de présence ou d'absence d'un texte s'appuie sur une commande
`/usr/bin/grep -nF "<chaîne exacte>" <fichier>` (ou `git -C … show HEAD:<fichier> | /usr/bin/grep -nF …`), **dont tu copies
la sortie**. Bloc multi-lignes : `-A5`. (`grep` est une fonction vers `ugrep` : utilise **`/usr/bin/grep`**.) Tout chemin
cité se vérifie (`git cat-file -e HEAD:<chemin>` ou `ls`), ligne lue citée.

## Sévérités

CRITICAL (écriture comptable ou plafond faux silencieusement, perte de données), HIGH (AC faux, `PUT` qui n'écrit pas, test qui
ne prouve rien), MEDIUM (incohérence code/fiche/manuel/i18n, mutation tuée pour une mauvaise raison), LOW (rédaction, nit).

## Interdits — lecture seule stricte

Aucune commande qui écrit, compile ou exécute : pas de `cargo`, `npm`, `make`, `latexmk`, `docker`, `curl`, `sqlx`, `mariadb` ;
**pas de `scripts/prepare-release.sh`** ni d'aucun script de `scripts/` ; pas de `git checkout`/`switch`, `git reset`,
`git stash`, `git commit`, `git push`, `git tag`, `git rebase` ; aucune commande `gh` qui écrit ; pas de `claude`, `pkill`,
`killall`, `rm`, `sed -i`, `perl -pi`, `tee`, **aucune redirection `>` ni fichier temporaire**. **Tu n'écris qu'un seul
fichier** : ton rapport.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/e437/review-p1-<B|E|A>.md` (jamais sous `target/`) :

1. **Décompte** CRITICAL / HIGH / MEDIUM / LOW, recompté depuis ta liste.
2. **Findings** : identifiant (`B1-1`, `E1-1`, `A1-1`…), sévérité, fichier et ligne, sortie `grep -nF` copiée, correctif proposé.
3. **Axes exercés et non exercés** — liste explicite ; un « 0 finding » sans cette liste ne compte pas.
