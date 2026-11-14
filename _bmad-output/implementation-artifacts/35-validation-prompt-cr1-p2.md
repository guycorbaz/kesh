# Prompt de validation CR-1 P2 — fiches révisées de l'Epic 35

Tu fais la **deuxième passe complète** du cycle de validation, ouvert par un changement de périmètre (CR-1),
sur une fiche du dépôt `/home/beat/limu/compta/kesh` (Kesh, logiciel comptable suisse en production : Rust/Axum,
SvelteKit, MariaDB). À lire d'abord :

- le fichier d'epic `_bmad-output/planning-artifacts/epic-35-multi-devises.md` : décisions **D1–D10** (D9 : table des
  cours modifiable ; D10 : réévaluation au cours journalier de clôture ; *Terminologie*) et registre, en particulier
  **CR-1 — amendements du registre** ;
- la fiche à valider (ta mission), dont l'entrée « révision CR-1 » du Change Log ; le diff de la révision :
  `git diff 0fb6636e b8c73b2b -- <fiche>` (lecture seule) ;
- `CLAUDE.md` : *Migration breaking policy* (P1–P8, fenêtre d'import), *Pattern batch*, *Review Iteration Rule* ;
- les autres fiches `35-*.md` du même dossier pour les points de contact.

## Ce que tu cherches

1. Défauts **introduits par la révision CR-1** : prémisse fausse au code (vérifie chaque ligne citée), contradiction
   avec le reste de la fiche ou avec une autre fiche, exigence irréalisable, renvoi faux.
2. **Sites non résolus** de D9/D10 : sauvegarde et restauration (fenêtre d'import, registre de rejeu `post_restore.rs`,
   `TABLES_TO_TRUNCATE`, export CSV, remise à zéro), audit (`audit_labels.rs`, registre des routes), RBAC, i18n,
   manuels, MCP, E2E — énumère au code, ne suppose pas.
3. Comportement de la source journalière (`xmldaily?d=AAAAMMJJ`) : tu peux faire **au plus trois GET** de lecture pour
   vérifier un cas (date future, week-end, date ancienne).
4. Défauts comptables (cours effectif, provenance `afc`/`manual`, réévaluation au cours de clôture).
5. Tests manquants pour un AC nouveau ou modifié.

## Interdits

N'écris rien sauf, en phase 2, dans **la seule fiche visée** (Edit). Aucun script (`scripts/*`, `cargo`, `npm`), aucun
`git` en écriture.

## Rapport (français)

Findings `<story>-CR1-P2-N` (sévérité, lieu, constat avec la ligne de code lue, correctif) ; **axes exercés / non
exercés** ; décompte. Phase 2 : applique les correctifs propres à la fiche ; Change Log « 2026-11-14 — validation CR-1
P1 (<modèle>) : <décompte> ; remédiés dans la fiche sauf <ids inter-fiches> » ; liste les findings inter-fiches non
appliqués avec le texte proposé. Message final bref.

## Spécifique à la CR-1 P2

La fiche a reçu la passe CR-1 P1 (Sonnet), remédiée : relis son entrée au Change Log et le diff
`git diff b8c73b2b HEAD -- <fiche>`. Cherche d'abord les défauts introduits par cette remédiation, puis le reste.
Arbitrages de l'orchestrateur à appliquer en phase 2 s'ils touchent ta fiche :
- **35-2** : à l'écran *Cours de change*, le bouton **↑** s'affiche pour **tout cours saisi existant**, même égal au
  cours AFC (D9 : il doit toujours pouvoir être retiré et suivre une republication AFC) ; et la fonction des bornes
  d'un cours seul rend une erreur de type propre (p. ex. `RateBoundsError { Scale, TooLarge, NotPositive }`), non une
  variante de `CoreError` liée à un code : le `PUT` d'un cours saisi et 35-5 la traduisent en `FX_RATE_INVALID`, 35-3a
  en `FX_VALUE_OUT_OF_RANGE { line }` (35-3a-CR1-P1-IF1).
