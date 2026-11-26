# Prompt de validation CR-6 P1 — fiches révisées de l'Epic 37 (couverture API et MCP)

Tu fais la **première passe complète** d'un nouveau cycle de validation, ouvert par l'audit de couverture API et MCP
(CR-6), sur une fiche du dépôt `~/limu/compta/kesh` (Kesh, logiciel comptable suisse en production :
Rust/Axum, SvelteKit, MariaDB). À lire d'abord :

- `_bmad-output/implementation-artifacts/37-couverture-api-mcp.md` : l'audit, les trous T1–T13 et les correctifs
  retenus **V1–V13** ;
- le fichier d'epic `_bmad-output/planning-artifacts/epic-37-multi-devises.md`, section « CR-6 — couverture API et MCP »,
  et les décisions et arbitrages antérieurs (D1–D10, R, S, T, U, A2–A30) ;
- la fiche à valider (ta mission), dont l'entrée « révision CR-6 » du Change Log ; le diff de la révision :
  `git diff b6e73cc3 d01ad946 -- <fiche>` (lecture seule) ;
- `CLAUDE.md` : *Pattern batch*, *Migration breaking policy*, *Review Iteration Rule* ;
- les autres fiches `37-*.md` du même dossier pour les points de contact.

## Ce que tu cherches

1. Défauts **introduits par la révision CR-6** : prémisse fausse au code (vérifie chaque ligne citée), contradiction avec
   le reste de la fiche ou avec une autre fiche, exigence irréalisable, renvoi faux.
2. **Contrat de l'API seule** : une écriture en devise (création, modification, ouverture et complément, conversion)
   peut-elle être passée par HTTP sans aucun calcul côté client ? `foreign { amount, chf }` sans `rate`, `rateSource`
   absent, `debit`/`credit` absents : chaque combinaison a-t-elle un comportement unique, un code d'erreur existant et
   un test ? Le calcul cours = CHF ÷ montant est-il **une seule** fonction de `kesh-core`, partagée par fixture avec le
   client (DRY) ?
3. **Choix de l'auteur à trancher** (non ancrés au code) : (a) refus de `rate_from_chf` par `FX_AMOUNT_MISMATCH` avec
   `expected` ; (b) mauvaise paire de la ligne de différence de conversion → 400 `Validation`, `details { entry, line }`,
   sans étendre `FX_ACCOUNT_INVALID` ; (c) repérage de la ligne de différence par son compte (compte de gains ou de
   pertes de la société ou d'un compte en devise) ; (d) extension de V1/V2 aux routes d'ouverture et de complément ;
   (e) une ligne CHF sans `debit` passe de 422 à 400. Pour chacun : correct, cohérent, comptablement sûr ? Si non,
   propose le correctif.
4. **MCP** : `kesh_exercices` étendu (type scellé, taille, test), exclusions écrites (société, cours, aperçu de
   réévaluation, `kesh_postes_ouverts`) ; garde-fous `mcp_gardes_lexicales`, `mcp_contrat`, `MCP_ENVOI_OCTETS_MAX`.
5. RBAC et authentification : activation en session Bearer jamais par clé d'API (`rbac.rs`, `middleware/auth.rs`), test
   de bout en bout par l'API (37-5 AC-7).
6. Tests manquants pour un AC nouveau ou modifié.

## Interdits

N'écris rien sauf, en phase 2, dans **la seule fiche visée** (Edit). Aucun script (`scripts/*`, `cargo`, `npm`), aucun
`git` en écriture, aucun appel réseau.

## Rapport (français)

Findings `<story>-CR6-P1-N` (sévérité, lieu, constat avec la ligne de code lue, correctif) ; **axes exercés / non
exercés** ; décompte. Phase 2 : applique les correctifs propres à la fiche ; Change Log « 2026-11-14 — validation CR-6
P1 (<modèle>) : <décompte> ; remédiés dans la fiche sauf <ids inter-fiches> » ; liste les findings inter-fiches non
appliqués avec le texte proposé. Message final bref.
