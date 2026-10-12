# Prompt — validation P4 **ciblée** de la spec, Story 27-1 (supprimer un import bancaire non rapproché)

*Versionné le 2026-10-12. **Une seule lentille**, **Haiku**, contexte frais, lecture seule, braquée sur **le seul
commit de la remédiation P3** (`CLAUDE.md`, § *La passe ciblée* : quand une boucle converge, ce qu'il reste à relire
n'est plus la story, c'est la dernière remédiation). Trend des MEDIUM distincts : P1 7 → P2 4 → P3 2, 0 HIGH.*

Dépôt `/home/gcorbaz/devel/kesh-27-1`, branche `story/27-1-supprimer-import`.
**Périmètre : `git show 269bb704`** (remédiation P3), soit un **diff unique** — ne lis pas la séquence des commits
antérieurs. Fichiers touchés : la fiche `_bmad-output/implementation-artifacts/27-1-supprimer-import-non-rapproche.md`,
le registre `epic-27-choix-autonomes.md`, `sprint-status.yaml`, l'epic `_bmad-output/planning-artifacts/epic-27-recette-v0-13-1.md`.
Rapports P3 que ce commit remédie : `/home/gcorbaz/devel/kesh-gate-logs/27-1-validate-p3-{R,F}.md`.

## La lentille — chasseur de régressions de la remédiation P3

Pour **chaque hunk** du commit, vérifie contre le code au sol :

1. **Angle mort « restauration contre suppression »** (réduit à une phrase, C-27-1-20) : chaque mot est-il vrai à
   `crates/kesh-db/src/backup.rs:492-500` et l. 60-68 (ordre de `TABLES_TO_TRUNCATE`) ? La phrase ne dit-elle **rien**
   de plus que ces lignes (pas d'`audit_log`, pas de victime, pas d'« installation restaurée ») ?
2. **AC3 (b), trajectoires** : codes `BANK_TRANSACTION_NOT_FOUND` / `RECONCILIATION_ALREADY_RECONCILED` et lignes citées
   (`routes/reconciliation.rs:65, 950-963, 1442-1455, 1935-1941, 3049-3064, 3188, 3218-3225, 3656-3660`) — exacts ?
3. **AC13 / AC14, propagation de la fermeture aux clés** : `docs/api-external.md` l. 77-83 et 205 ;
   `docs/manual/fr/admin-manual.tex` l. 2262-2285 ; les phrases prescrites sont-elles vraies, et la liste des sites
   relevée au Change Log (« seuls 205 et 2272 énumèrent les exceptions hors administration ») est-elle complète ?
   Refais : `grep -rn "fermées aux clés\|fermée aux clés\|exceptions près\|en est exclue\|de même que le journal"
   docs/ crates/kesh-api/src frontend/src README.md website/`.
4. **Manuel contre code (obligatoire), `.tex` ET PDF aplati** : `docs/manual/fr/user-manual.tex` l. 248-252 (tuile
   *Relevé du …*, phrase à aligner) contre `crates/kesh-db/src/repositories/bank_accounts.rs:1011-1017` ; le PDF
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` et
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '` (aplatir avant de greper).
5. **Tests et mutations neufs** : H9 (montage `list_bank_accounts_exposes_the_last_statement`,
   `crates/kesh-api/tests/bank_accounts_e2e.rs:1206`), assertion de message de H6, test de message de
   `crates/kesh-i18n/src/loader.rs:663`, M19, M20, rollback explicite (patron `routes/imported_supplier_invoices.rs:315`) —
   faisables, et chacun ferait-il rougir la mutation qu'on lui attribue ?
6. **Recomptes** écrits par le commit : 2 MEDIUM distincts / 13 LOW en P3, 20 mutations à jouer, H1-H9, C-27-1-1 à 20,
   ligne de `sprint-status.yaml` — recompte depuis les fichiers, ne relis pas les nombres.

⚠️ **Erreur de catégorie** (consigne 5 des agents) : un livrable **prescrit par la fiche mais absent du code** n'est
**pas** un défaut — la fiche est une spec, le code n'est pas encore écrit (« H9 n'existe pas dans
`bank_accounts_e2e.rs` », « la phrase n'est pas dans `api-external.md` » sont attendus). Un finding ne porte que sur
ce que **le texte du commit affirme du code existant**, ou sur ce qu'il **prescrit de faux ou d'infaisable**.
⚠️ Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée ; un CRITICAL ou HIGH qui affirme qu'un
patch « n'a pas été appliqué » se vérifie au `grep -nF` sur le fichier **final** avant d'être écrit.

## Ce que tu rends

Findings numérotés `P4-1…` avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve** (commande et
sortie copiée), correction proposée. ⛔ **La liste des axes exercés ET non exercés** (les six ci-dessus) — un
« 0 finding » sans elle ne compte pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/27-1-validate-p4-R.md` ;
dernier message : le chemin du rapport, le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`,
`latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`,
`cargo`, `npm`, `npx`, `node`, `docker`, `bash` sur un script du dépôt, `gh` en écriture, toute requête SQL. Rien de
`~/kesh-test/` (relevé réel de Guy). Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`/`blame`, `gh api`
en lecture, `pdftotext` vers la sortie standard.
