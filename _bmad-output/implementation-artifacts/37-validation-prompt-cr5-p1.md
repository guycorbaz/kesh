# Prompt de validation CR-5 P1 — fiches révisées de l'Epic 37

Tu fais la **première passe complète** d'un nouveau cycle de validation, ouvert par la revue 3 de Beat (CR-5),
sur une fiche du dépôt `~/limu/compta/kesh` (Kesh, logiciel comptable suisse en production : Rust/Axum,
SvelteKit, MariaDB). À lire d'abord :

- le fichier d'epic `_bmad-output/planning-artifacts/epic-37-multi-devises.md` : décisions **D1–D10** et **R1–R10** et **S1–S6** , **T1–T6** et **U1–U6** (sections CR-2 à CR-5 ; « Revue 3 de Beat — CR-5 » prime) (D9 : table des
  cours modifiable ; D10 : réévaluation au cours journalier de clôture ; *Terminologie*) et registre, en particulier
  **CR-5 — amendements du registre et arbitrages** (A23–A29) ;
- la fiche à valider (ta mission), dont l'entrée « révision CR-5 » du Change Log ; le diff de la révision :
  `git diff 0ae43432 d59a1de9 -- <fiche>` (lecture seule) ;
- `CLAUDE.md` : *Migration breaking policy* (P1–P8, fenêtre d'import), *Pattern batch*, *Review Iteration Rule* ;
- les autres fiches `37-*.md` du même dossier pour les points de contact.

## Ce que tu cherches

1. Défauts **introduits par la révision CR-5** : prémisse fausse au code (vérifie chaque ligne citée), contradiction
   avec le reste de la fiche ou avec une autre fiche, exigence irréalisable, renvoi faux.
2. **Sites non résolus** de U1–U6 (en particulier U1 : tables renommées `exchange_rates` et `_kesh_fetched_exchange_rates`, `republished_*` retirées — chercher tout reste des anciens noms dans les six fiches) : sauvegarde et restauration (fenêtre d'import, registre de rejeu `post_restore.rs`,
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

Findings `<story>-CR5-P1-N` (sévérité, lieu, constat avec la ligne de code lue, correctif) ; **axes exercés / non
exercés** ; décompte. Phase 2 : applique les correctifs propres à la fiche ; Change Log « 2026-11-14 — validation CR-5
P1 (<modèle>) : <décompte> ; remédiés dans la fiche sauf <ids inter-fiches> » ; liste les findings inter-fiches non
appliqués avec le texte proposé. Message final bref.


## Arbitrages à appliquer en phase 2 (s'ils touchent ta fiche)

A23 (↑ direct tant que l'option « toutes les sociétés » est masquée), A24 (copie limitée aux devises des comptes de la
société ; copie déclenchée aussi quand un compte reçoit une devise — 37-1 et 37-2), A25, A26, A27, A28 (`source` sans
CHECK d'énumération), A29 (37-4 : « totaux en devise principale »). Ils sont dans le fichier d'epic, section
« CR-5 — amendements du registre et arbitrages ».
