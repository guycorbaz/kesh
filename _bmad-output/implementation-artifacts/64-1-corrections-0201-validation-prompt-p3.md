# Prompt — validation P3 de la fiche `64-1-corrections-0201` (#1178, #1758, #1712), passe complète

Tu valides, **en lecture seule**, la fiche de story `64-1-corrections-0201` de Kesh (comptabilité suisse ; Rust/Axum, MariaDB,
SvelteKit 5 runes), seule story de l'Epic 64, visée **v0.20.1**, **sans migration**. Kesh tient les livres réels du Project
Lead. Trois défauts : le formulaire des taux de TVA n'envoie rien (#1178, P2), la page `/invoices/{id}/edit` s'ouvre sur une
facture validée et sur un rôle sans droit d'écriture (#1758, P3), `PUT /api/v1/profile/mode` sans garde de rôle (#1712, P3).

- **Modèle : Opus** (la P2 était Sonnet). **Contexte frais. Passe complète** : la P2 a rendu 2 MEDIUM (D2 : clé unique et message
  d'échec) et 7 LOW, tous corrigés ; la garde de rôle de la page `edit` est entrée dans la story (D3, AC5-bis) — règle métier
  ajoutée, une passe ciblée ne suffit pas. Langue : français.
- Worktree `kesh-64-1`, branche `story/64-1-corrections-0201` ; code de référence `origin/main` (`0e99cd2db`). Fiche :
  `_bmad-output/implementation-artifacts/64-1-corrections-0201.md` (lis son Change Log, entrées P1 et P2) ; epic :
  `_bmad-output/planning-artifacts/epic-64-corrections-0201.md`. Rapports précédents : `$GATE_LOGS/e64/64-1-validation-p1.md`,
  `$GATE_LOGS/e64/64-1-validation-p2.md`.
- Lis le `CLAUDE.md` du worktree (Review Iteration Rule, Inventorier les sites NON RÉSOLUS, Propagation post-patch, Priorités des
  défauts, Versions de l'API, Documentation à synchroniser) et les issues (`gh issue view 1178|1758|1712`).
- **Décisions du Project Lead du 2026-11-27, à ne pas rediscuter** : Q1 Administrateur seul ; Q2 pas une rupture, ⚠️ en tête
  des notes, release `c`. **Décision de l'orchestrateur** : la garde de rôle de la page `edit` est dans la story.

## Interdits

Aucun fichier écrit hors de ton rapport. Ni `cargo`, ni `npm` / `npx`, ni `make`, ni `scripts/*` (dont `prepare-release.sh`,
`test-fast.sh`, `gate-ci.sh`, `mem-guard.sh`, `recette-montee.sh`, `install-hooks.sh`), ni `docker`, `mariadb`, `sqlx`, `curl`,
`wget`, ni `git commit|checkout|stash|restore|reset|push|worktree`, ni `gh` en écriture, ni `sed -i`, `perl -pi`, `tee`, `>`,
`>>`, ni `pkill`, `killall`, `kill` par motif, ni interpréteur. Rien sous `~/kesh/`. Permis : `git log|show|diff|grep`, `grep`,
`sed -n`, `cat`, `ls`, `wc`, `pdftotext … -` en tube, `gh … view|list`.

Rapport : **`$GATE_LOGS/e64/64-1-validation-p3.md`** (le seul fichier que tu écris, par l'outil d'écriture).

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

1. **Remédiation de la P2, relue au code.** Chaque ligne citée par les passages ajoutés est-elle exacte ? `20260428000001_vat_rates.sql:26`
   (clé unique) et `:31` (collation) ; `lib.rs:590`, `:736-737`, `:1214` (`comptable_routes`, `PUT /invoices/{id}`) ;
   `supplier-invoices/[id]/edit/+page.svelte:54-56`, `:74`, `:149-155` ; `(app)/+layout.svelte:357` (déclencheur du menu) ;
   `docs/testing.md:112` ; `fr-CH/messages.ftl:648`.
2. **D2 et AC3 (#1178).** Relis `create_vat_rate` et `update_vat_rate` (`vat.rs`) ET les contraintes de la table : l'ensemble des
   refus du `POST` qui suit la clôture est-il, cette fois, clos (clé unique, chevauchement, taux, dates, `borner`, catégorie,
   libellé) ? La comparaison de catégorie « sans casse ni accents » est-elle réalisable et juste (`localeCompare` `sensitivity`) ?
   La comparaison « de même taux, en nombre » pour la clé unique est-elle juste (`rate` en `DECIMAL`) ? Le drapeau « clos »
   distingue-t-il bien les deux échecs, `ApiError` comme erreur non typée ? Chaque mutation du tableau est-elle tuée par un test nommé ?
3. **#1758 (D3, D4, AC4, AC5, AC5-bis).** La garde de rôle : le rôle lit-il `authState` comme la page fournisseur (course au
   chargement : `currentUser` nul un instant ?) ; ordre rôle puis statut ; aucun appel à `getInvoice` pour Consultation ; clés i18n
   `invoice-edit-*` des quatre locales et `sitesTotal` ; test vitest réalisable (mock de `authState`).
4. **#1712 (D5-D7, AC6-AC7, tâches 4-6).** Relecture de bout en bout après les ajouts : AC7 est-elle réalisable à la lettre
   (`data-testid="user-menu-trigger"`, `waitForResponse` armé avant le clic, connexion, mode de départ) ? Rien d'autre ne code le
   groupe de droits de la route (`grep -rn "profile/mode\|profile::set_mode\|KESH-ADMIN-ROUTES" crates/*/tests`).
5. **Manuel, PDF compris.** `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (idem `admin-manual.pdf`) :
   AC8 et la tâche 7 couvrent-ils chaque phrase qui dira autre chose que le code après la story (« bascule », « Mode expert »,
   tableau des rôles) ? La page `edit` change-t-elle une phrase du manuel (modification d'une facture, rôle Consultation) ?
6. **Cohérence d'ensemble.** Notes de version (numéro de release, aucun epic, ⚠️ en tête) ; aucun mot-clé de fermeture dans la
   fiche, l'epic, les prompts et `git log origin/main..HEAD --format=%B` ; `sprint-status.yaml` (YAML lu) ; aucun chemin de la
   station dans le versionné (`grep -n "/home/" <fiche> <epic> <prompts>`) ; décomptes du Change Log P1 (6 MEDIUM, 10 LOW) et P2
   (2 MEDIUM, 7 LOW) recomptés ; la fiche ne se contredit pas (V1-16 périmé par V2-8, annoté ; tâche 8 ; AC ; tableau des tests).
7. **Epic et recoupements** : cinq modules (la garde de rôle n'en ajoute pas) ; recoupements avec `origin/epic/50-corrections` et
   la PR #1696 exacts.
8. **Recyclage** : les défauts de la P2 sont-ils de la même famille que ceux de la P1 (D2 incomplet) ? Dis si la fiche recycle ;
   sinon la boucle converge.

## Rapport

Pour chaque finding : `V3-n`, sévérité (CRITICAL, HIGH, MEDIUM, LOW), axe, constat **avec la ligne lue citée** (`chemin:ligne` et
extrait), conséquence, remède. Pour tout CRITICAL ou HIGH qui affirme une absence ou un anti-pattern, la commande `grep -nF` qui
le prouve et sa sortie. En fin : décompte **CRITICAL / HIGH / MEDIUM / LOW** recompté, **axes exercés et non exercés** (avec la
raison), commandes lancées. Un « 0 finding » sans cette liste ne compte pas comme passe.
