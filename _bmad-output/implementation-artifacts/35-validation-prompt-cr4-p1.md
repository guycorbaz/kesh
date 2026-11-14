# Prompt de validation CR-4 P1 — fiches révisées de l'Epic 35

Tu fais la **première passe complète** d'un nouveau cycle de validation, ouvert par la revue 2 de Beat (CR-4),
sur une fiche du dépôt `/home/beat/limu/compta/kesh` (Kesh, logiciel comptable suisse en production : Rust/Axum,
SvelteKit, MariaDB). À lire d'abord :

- le fichier d'epic `_bmad-output/planning-artifacts/epic-35-multi-devises.md` : décisions **D1–D10** et **R1–R10** et **S1–S6** et **T1–T6** (sections « Revue de Beat — CR-2 », « — CR-3 » et « Revue 2 de Beat — CR-4 » ; CR-4 prime) (D9 : table des
  cours modifiable ; D10 : réévaluation au cours journalier de clôture ; *Terminologie*) et registre, en particulier
  **CR-4 — amendements du registre et arbitrages** (A18–A22) ;
- la fiche à valider (ta mission), dont l'entrée « révision CR-4 » du Change Log ; le diff de la révision :
  `git diff 2c0f5afc bd7d0ec7 -- <fiche>` (lecture seule) ;
- `CLAUDE.md` : *Migration breaking policy* (P1–P8, fenêtre d'import), *Pattern batch*, *Review Iteration Rule* ;
- les autres fiches `35-*.md` du même dossier pour les points de contact.

## Ce que tu cherches

1. Défauts **introduits par la révision CR-4** : prémisse fausse au code (vérifie chaque ligne citée), contradiction
   avec le reste de la fiche ou avec une autre fiche, exigence irréalisable, renvoi faux.
2. **Sites non résolus** de T1–T6 (en particulier T3 : retrait de `journal_entry_lines.currency` — chercher tout reste de cette colonne dans les six fiches) : sauvegarde et restauration (fenêtre d'import, registre de rejeu `post_restore.rs`,
   `TABLES_TO_TRUNCATE`, export CSV, remise à zéro), audit (`audit_labels.rs`, registre des routes), RBAC, i18n,
   manuels, MCP, E2E — énumère au code, ne suppose pas.
3. Comportement de la source journalière (`xmldaily?d=AAAAMMJJ`) : tu peux faire **au plus trois GET** de lecture pour
   vérifier un cas (date future, week-end, date ancienne).
4. Défauts comptables (mode par exercice, compte de conversion, une devise par écriture, cours d’ouverture et de clôture, une écriture par compte réévalué).
5. Tests manquants pour un AC nouveau ou modifié.

## Interdits

N'écris rien sauf, en phase 2, dans **la seule fiche visée** (Edit). Aucun script (`scripts/*`, `cargo`, `npm`), aucun
`git` en écriture.

## Rapport (français)

Findings `<story>-CR4-P1-N` (sévérité, lieu, constat avec la ligne de code lue, correctif) ; **axes exercés / non
exercés** ; décompte. Phase 2 : applique les correctifs propres à la fiche ; Change Log « 2026-11-14 — validation CR-4
P1 (<modèle>) : <décompte> ; remédiés dans la fiche sauf <ids inter-fiches> » ; liste les findings inter-fiches non
appliqués avec le texte proposé. Message final bref.

