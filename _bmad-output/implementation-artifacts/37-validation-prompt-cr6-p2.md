# Prompt de validation CR-6 P2 — fiches de l'Epic 37 (couverture API et MCP)

Deuxième passe du cycle CR-6, **modèle différent** de P1 (Sonnet), contexte frais, sur une fiche du dépôt
`/home/beat/limu/compta/kesh` (Kesh : Rust/Axum, SvelteKit, MariaDB). À lire d'abord :

- `_bmad-output/implementation-artifacts/37-couverture-api-mcp.md` (audit, V1–V13) et la section « CR-6 — couverture API
  et MCP » du fichier d'epic `_bmad-output/planning-artifacts/epic-37-multi-devises.md` ;
- la fiche visée, ses entrées « révision CR-6 » et « validation CR-6 P1 » du Change Log ;
- la remédiation de P1 : `git diff 16f1cdb2 19039295 -- <fiche>` (lecture seule) — c'est là que la sévérité se déplace ;
  relis aussi le diff de la révision (`git diff b6e73cc3 d01ad946 -- <fiche>`) pour le contexte ;
- `CLAUDE.md` : *Pattern batch*, *Review Iteration Rule* ; les autres fiches `37-*.md` pour les points de contact.

## Ce que tu cherches

1. Défauts **introduits par la remédiation P1** : prémisse fausse au code (vérifie chaque ligne citée), contradiction
   dans la fiche ou avec une autre fiche (37-3a ↔ 37-3b surtout : DTO `foreign { amount, rate?, chf?, rateSource?, side? }`,
   `debit`/`credit` facultatifs, qualification d'une ligne sans `rateSource`, `rate_from_chf` et ses bornes, route de
   conversion V3 et choix de l'écriture à compléter, ordre des refus et des verrous), exigence irréalisable, renvoi faux.
2. **Comptabilité** de la conversion V3 : la ligne de différence ne peut-elle jamais masquer une erreur du client ?
   Écriture à compléter bien déterminée ? Gain/perte au bon compte et au bon sens ?
3. **Contrat de l'API seule** : chaque combinaison de champs absents/présents a un comportement unique, un code d'erreur
   existant et un test.
4. MCP (37-1 : `champ_enum!`, test de taille dédié ; 37-5 : clé MCP `books:read`, serveur activé) et RBAC (Bearer).
5. Tests manquants.

## Interdits

N'écris rien sauf, en phase 2, dans **la seule fiche visée** (Edit). Aucun script (`scripts/*`, `cargo`, `npm`, ni
script ad hoc), aucun `git` en écriture, aucun appel réseau.

## Rapport (français)

Findings `<story>-CR6-P2-N` (sévérité, lieu, constat avec la ligne lue, correctif) ; **axes exercés / non exercés** ;
décompte. Phase 2 : applique les correctifs propres à la fiche ; Change Log « 2026-11-14 — validation CR-6 P2
(<modèle>) : <décompte> ; remédiés dans la fiche sauf <ids inter-fiches> » ; liste les findings inter-fiches avec le
texte proposé. Message final bref.
