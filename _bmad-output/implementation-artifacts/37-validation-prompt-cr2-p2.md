# Prompt de validation CR-2 P2 — fiches révisées de l'Epic 37

Tu fais la **deuxième passe complète** du cycle de validation CR-2 (revue du responsable de l'epic),
sur une fiche du dépôt `<dépôt>` (Kesh, logiciel comptable suisse en production : Rust/Axum,
SvelteKit, MariaDB). À lire d'abord :

- le fichier d'epic `_bmad-output/planning-artifacts/epic-37-multi-devises.md` : décisions **D1–D10** et **R1–R10** (section « Revue du responsable de l'epic — CR-2 », qui prime) (D9 : table des
  cours modifiable ; D10 : réévaluation au cours journalier de clôture ; *Terminologie*) et registre, en particulier
  **CR-2 — amendements du registre et arbitrages** ;
- la fiche à valider (ta mission), dont l'entrée « révision CR-2 » du Change Log ; le diff de la révision :
  `git diff b6bf53f8 HEAD -- <fiche>` (lecture seule) ;
- `CLAUDE.md` : *Migration breaking policy* (P1–P8, fenêtre d'import), *Pattern batch*, *Review Iteration Rule* ;
- les autres fiches `37-*.md` du même dossier pour les points de contact.

## Ce que tu cherches

1. Défauts **introduits par la révision CR-2** : prémisse fausse au code (vérifie chaque ligne citée), contradiction
   avec le reste de la fiche ou avec une autre fiche, exigence irréalisable, renvoi faux.
2. **Sites non résolus** de R1–R10 : sauvegarde et restauration (fenêtre d'import, registre de rejeu `post_restore.rs`,
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

Findings `<story>-CR2-P2-N` (sévérité, lieu, constat avec la ligne de code lue, correctif) ; **axes exercés / non
exercés** ; décompte. Phase 2 : applique les correctifs propres à la fiche ; Change Log « 2026-11-14 — validation CR-2
P1 (<modèle>) : <décompte> ; remédiés dans la fiche sauf <ids inter-fiches> » ; liste les findings inter-fiches non
appliqués avec le texte proposé. Message final bref.

## Spécifique à la CR-2 P2

La fiche a reçu la passe CR-2 P1 (Sonnet), remédiée, puis des alignements de l'orchestrateur : relis leurs entrées au
Change Log et le diff `git diff b36599d0 HEAD -- <fiche>`. Cherche d'abord les défauts **introduits par cette
remédiation** (verrous et rejeu, ordre des refus, archivage sous `force`, exemptions de `FX_MULTIPLE_CURRENCIES`,
assistant *Conversion*, calendrier de récupération), puis les incohérences entre fiches, puis le reste.
