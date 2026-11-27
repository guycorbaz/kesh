# Prompt — validation P2 de la fiche `64-1-corrections-0201` (#1178, #1758, #1712), passe complète

Tu valides, **en lecture seule**, la fiche de story `64-1-corrections-0201` de Kesh (comptabilité suisse ; Rust/Axum, MariaDB,
SvelteKit 5 runes), seule story de l'Epic 64, visée **v0.20.1**, **sans migration**. Kesh tient les livres réels du Project
Lead (Guy). Trois défauts : le formulaire des taux de TVA n'envoie rien (#1178, P2), la page `/invoices/{id}/edit` s'ouvre sur une
facture validée (#1758, P3), `PUT /api/v1/profile/mode` sans garde de rôle (#1712, P3).

- **Modèle : Sonnet. Contexte frais. Passe complète** (la P1, Opus, a rendu 6 MEDIUM, tous corrigés dans la fiche ; la remédiation
  réécrit D2, D5, D7, AC3, AC7, AC8 et les tâches 0, 4, 6, 7, 8 : une passe ciblée ne suffit pas). Langue du rapport : français.
- Worktree `kesh-64-1`, branche `story/64-1-corrections-0201` ; code de référence `origin/main` (`0e99cd2db`). Fiche :
  `_bmad-output/implementation-artifacts/64-1-corrections-0201.md` (lis d'abord son Change Log, entrée « Validation P1 ») ; epic :
  `_bmad-output/planning-artifacts/epic-64-corrections-0201.md`.
- Lis le `CLAUDE.md` du worktree (Review Iteration Rule, Inventorier les sites NON RÉSOLUS, Propagation post-patch, Priorités des
  défauts, Versions de l'API, Documentation à synchroniser, Recette visuelle) et les trois issues (`gh issue view 1178|1758|1712`).
- **Décisions du Project Lead du 2026-11-27, à ne pas rediscuter** : Q1 — bascule du mode réservée à l'**Administrateur seul** ;
  Q2 — **pas une rupture du contrat de l'API**, ⚠️ en tête des notes de version, release `c` (v0.20.1). Vérifie seulement que la
  fiche les porte partout de façon cohérente (D5, D6, D7, AC6, AC7, notes de version, section des questions).

## Interdits

Tu n'écris **aucun fichier** hors de ton rapport. Tu ne lances **ni** `cargo` (build, test, nextest, clippy, fmt), **ni** `npm` /
`npx`, **ni** `make`, **ni** aucun `scripts/*` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`,
`scripts/mem-guard.sh`, `scripts/recette-montee.sh`, `scripts/install-hooks.sh`) ni `gate-distant.sh`, **ni** `docker`, **ni**
`mariadb` / `sqlx`, **ni** `curl` / `wget`, **ni** `git commit|checkout|stash|restore|reset|push|worktree`, **ni** `gh` en
écriture (`issue create|edit|comment|close`, `pr …`), **ni** `sed -i`, `perl -pi`, `tee`, `>`, `>>`, **ni** `pkill`, `killall`
ni `kill` par motif, **ni** interpréteur (`python3`, `node`). Rien sous `~/kesh/`. Permis : `git log|show|diff|grep`, `grep`,
`sed -n`, `cat`, `ls`, `wc`, `pdftotext … -` en tube, `gh … view|list`.

Rapport : **`$GATE_LOGS/e64/64-1-validation-p2.md`** (le seul fichier que tu écris, par l'outil d'écriture ; `$GATE_LOGS` est le
dossier `kesh-gate-logs` voisin du dépôt).

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

1. **Remédiation de la P1, relue au code** (`0e99cd2db`) : chaque ligne citée par les passages réécrits est-elle exacte ?
   En particulier `vat-rates/+page.svelte:147`, `:155`, `:166-167`, `:306`, `:342-343` ; `vat.rs:267-279`, `:296`, `update_vat_rate`
   (`validate_dates` contre `existing.valid_from`), `has_overlap_in_category` ; `vat_rates.rs:49-56`, `:65-72` ;
   `admin_pat_denied_e2e.rs:46-48`, `:285-287`, `:341-352`, `:862-891` ; `lib.rs:305-307`, `:564`, `:590`, `:1374` ;
   `profile.rs:41-45` ; `profile_e2e.rs:121-122` ; `app.css:65-79` ; `(app)/+layout.svelte:45`, `:67`, `:376-378`, `:493` ;
   `(app)/+page.svelte:32` ; `supplier-invoices/[id]/edit/+page.svelte:74`, `:86-94` ; `credit_note.rs:1-6` ;
   `docs/testing.md:485-486` ; `mode-expert.spec.ts:36` ; `vat-rates.spec.ts:46`, `:61` ; `api-external.md:82`, `:227`, `:1555` ;
   `admin-manual.tex:1683-1690` ; `user-manual.tex:155`, `:256`.
2. **#1178, D1-D2-AC1-AC3.** Le validateur de D2 couvre-t-il **l'ensemble clos** des refus prévisibles du `POST` après la clôture
   (lis `create_vat_rate` en entier : `borner`, `validate_category`, `validate_label`, `validate_rate`, `validate_dates`,
   chevauchement) et du `PUT` avant elle ? Un cas manque-t-il (catégorie vide au changement, libellé, borne de date de `borner`) ?
   Le chevauchement lu dans `rates` est-il juste (taux inactifs exclus comme au serveur, catégorie de l'ancien) ? Le message
   d'échec après clôture distingue-t-il l'appel qui a échoué ? Chaque mutation du tableau des tests est-elle tuée par un test nommé ?
3. **#1758, D3-D4-AC4-AC5.** Plus d'avoir dans AC4 ; la variante « motif de la page fournisseur » est-elle bien laissée au choix
   et sans contradiction avec l'issue ? Le formulaire ne s'affiche jamais avant la lecture du statut ; clés i18n (préfixe
   `invoice-edit-…` à côté de `invoice-edit-title`, `fr-CH/messages.ftl:648`), quatre locales, `sitesTotal`.
4. **#1712, D5-D6-AC6-AC7.** La tâche 4 nomme-t-elle désormais **tout** ce qui change dans `admin_pat_denied_e2e.rs` et `lib.rs`
   (relis le fichier de test entier, toutes assertions de source comprises) ? Un autre test code-t-il le groupe de droits de la
   route ou le nombre de routes du bloc (`grep -rn "profile/mode\|profile::set_mode\|KESH-ADMIN-ROUTES" crates/*/tests`) ?
   La nouvelle spec d'AC7 est-elle réalisable (déclencheur du menu, `createUser`, sonde positive, `waitForResponse`) ?
5. **Manuel, PDF compris.** Aplatis avant de greper : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`
   (idem `admin-manual.pdf`) ; motifs : « Mode expert », « Guidé », « bascule », « taux de TVA », « Changer le taux »,
   « Modifier », tableau des rôles. AC8 et la tâche 7 couvrent-ils chaque phrase qui dira autre chose que le code après la story ?
6. **Inventaire de la tâche 8.** Refais-le depuis le symptôme (`grep -rn 'type="number"' --include=*.svelte frontend/src`, sonde
   positive `vat-rates/+page.svelte:287`) : l'ensemble est-il clos, les verdicts justes (notamment `dunning/+page.svelte:128`,
   `:189` : champ vidé → `Number(null)` = 0 accepté), l'angle mort déclaré ?
7. **Cohérence d'ensemble.** Notes de version (numéro de release, aucun epic, ⚠️ en tête) ; aucun mot-clé de fermeture
   (`closes`, `fixes`, `resolves` et variantes) dans la fiche, l'epic, ce prompt et `git log origin/main..HEAD --format=%B` ;
   `sprint-status.yaml` (clés `epic-64`, `64-1-corrections-0201`, YAML lu) ; aucun chemin absolu de la station dans ce qui est
   versionné (`grep -n "/home/" <fiche> <epic>`) ; décompte du Change Log P1 recompté (6 MEDIUM, 10 LOW).
8. **Epic.** Cinq modules (recompte au grain crate / domaine) ; recoupements avec `origin/epic/50-corrections` et la PR #1696
   (`gh pr view 1696 --json files`) exacts et complets.

## Rapport

Pour chaque finding : identifiant `V2-n`, sévérité (CRITICAL, HIGH, MEDIUM, LOW), axe, constat **avec la ligne lue citée**
(`chemin:ligne` et l'extrait), conséquence, remède proposé. Pour tout CRITICAL ou HIGH qui affirme l'absence d'un code ou la
présence d'un anti-pattern, la commande `grep -nF` qui le prouve et sa sortie. En fin de rapport : décompte **CRITICAL / HIGH /
MEDIUM / LOW**, recompté depuis tes findings ; la liste des **axes exercés** et des **axes non exercés** (avec la raison) ; les
commandes lancées. Un « 0 finding » sans cette liste ne compte pas comme passe.
