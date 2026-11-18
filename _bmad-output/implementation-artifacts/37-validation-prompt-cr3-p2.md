# Prompt de validation CR-3 P2 — fiches révisées de l'Epic 37

Tu fais la **deuxième passe complète** du cycle de validation CR-3 (revue de Beat),
sur une fiche du dépôt `/home/beat/limu/compta/kesh` (Kesh, logiciel comptable suisse en production : Rust/Axum,
SvelteKit, MariaDB). À lire d'abord :

- le fichier d'epic `_bmad-output/planning-artifacts/epic-37-multi-devises.md` : décisions **D1–D10** et **R1–R10** et **S1–S6** (sections « Revue de Beat — CR-2 » et « — CR-3 » ; CR-3 prime) (D9 : table des
  cours modifiable ; D10 : réévaluation au cours journalier de clôture ; *Terminologie*) et registre, en particulier
  **CR-3 — amendements du registre et arbitrages** (A9–A16) ;
- la fiche à valider (ta mission), dont l'entrée « révision CR-3 » du Change Log ; le diff de la révision :
  `git diff b6f88c24 d9c0018e -- <fiche>` (remédiation CR-3 P1 et arbitrage A17) (lecture seule) ;
- `CLAUDE.md` : *Migration breaking policy* (P1–P8, fenêtre d'import), *Pattern batch*, *Review Iteration Rule* ;
- les autres fiches `37-*.md` du même dossier pour les points de contact.

## Ce que tu cherches

1. Défauts **introduits par la révision CR-3** : prémisse fausse au code (vérifie chaque ligne citée), contradiction
   avec le reste de la fiche ou avec une autre fiche, exigence irréalisable, renvoi faux.
2. **Sites non résolus** de S1–S6 : sauvegarde et restauration (fenêtre d'import, registre de rejeu `post_restore.rs`,
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

Findings `<story>-CR3-P2-N` (sévérité, lieu, constat avec la ligne de code lue, correctif) ; **axes exercés / non
exercés** ; décompte. Phase 2 : applique les correctifs propres à la fiche ; Change Log « 2026-11-14 — validation CR-3
P1 (<modèle>) : <décompte> ; remédiés dans la fiche sauf <ids inter-fiches> » ; liste les findings inter-fiches non
appliqués avec le texte proposé. Message final bref.


## Spécifique à la CR-3 P2

Cherche d'abord les défauts **introduits par la remédiation CR-3 P1** (diff ci-dessus), puis les incohérences entre
fiches, puis le reste. Arbitrage **A17** (fichier d'epic, arbitrages CR-3) à appliquer en phase 2 s'il touche ta fiche :
- **37-3a** : réécrire le paragraphe de l'AC-4 (point 4) marqué « Arbitrage A17 » en texte propre (supprimer le
  « texte antérieur ») et aligner tests et Tâches (une ligne `afc` inchangée n'est jamais recontrôlée au `PUT` ; après
  restauration ou adoption, un `PUT` du seul libellé passe).
- **37-3b** : aligner l'AC-2 (refus `FX_RATE_NOT_AFC` sur une écriture ancienne) sur A17 : le refus ne vise plus que
  les lignes nouvelles ou modifiées ; garder les actions « Reprendre le cours attendu » / « Garder mon cours » pour
  ces lignes seulement.
