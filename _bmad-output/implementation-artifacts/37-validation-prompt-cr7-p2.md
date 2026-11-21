# Prompt de validation CR-7 P2 — fiches de l'Epic 37

Deuxième passe du cycle CR-7 (**modèle différent** de P1, Sonnet), contexte frais, sur une fiche du worktree
`<dépôt>/.claude/worktrees/kesh-wt37` (branche `epic/37-multi-devises` ; n'écris nulle part ailleurs).

## À lire

- epic `_bmad-output/planning-artifacts/epic-37-multi-devises.md`, section « CR-7 » (A34, A35, A36 KISS) ;
- la fiche visée, sa « Révision CR-7 » et le diff `git diff 882d865b e99364d2 -- <fiche>` (remédiation P1) ;
- les autres fiches 37-* ; `CLAUDE.md` (*Review Iteration Rule*).

## Décisions de l'orchestrateur à appliquer (là où elles touchent ta fiche)

- **D-CR7-1** (déjà en 37-3a) : plus aucun verrou de `companies` pris par les écritures ; tout texte qui s'y appuie est
  réécrit.
- **D-CR7-2 (KISS, remplace la « limite assumée »)** : la création d'un exercice **avant un exercice mono-devise** est
  refusée, comme l'est déjà la création avant un exercice clos (`fiscal_years::create`, `LaterFiscalYearClosed`) — même
  code ou code voisin, le plus simple (vérifie au code). La limite assumée (exercice rouvert puis exercice créé dans un
  trou devant lui), son test et sa phrase au manuel **disparaissent** de 37-1, 37-3a, 37-5 et de l'epic ; un test du refus
  remplace le test de la limite (37-1).
- **D-CR7-3 (KISS)** : 37-5 — la relecture « sous verrou » du mode de la société et de l'exercice, devenue sans course
  possible, est **retirée** (AC-2 (0)-(1), AC-4, FR-37-5-5, Tâches) ; le contrôle reste une lecture simple avant la
  transaction.
- **Epic** (mission 37-1 seulement) : A5 annoté « retiré par CR-7 » ; section CR-7 « Points à faire valider » mise à jour
  (verrou retiré, D-CR7-2, D-CR7-3).
- **37-2** : la branche « société … mono-devise » du 409 `MULTI_CURRENCY_DISABLED` d'une route d'écritures (vers la
  ligne 750) est alignée sur « exercice mono-devise » seul, si elle y est encore.

## Ce que tu cherches

Défauts introduits par la remédiation P1 et par ces décisions (prémisse fausse au code, contradiction, renvoi faux,
test muet ou manquant), résidus du modèle réversible, et tout mécanisme encore présent qui n'existait que pour la
réversibilité (KISS).

## Interdits

Edit seulement, dans la fiche visée (et, pour 37-1, l'epic aux points nommés). Aucun script (ni heredoc, ni python, ni
`cat >>`), aucun `cargo`/`npm`, aucun `git` en écriture.

## Rapport (français)

Findings `<story>-CR7-P2-N` ; axes exercés / non exercés ; décompte. Phase 2 : corrige ; Change Log « 2026-11-16 —
validation CR-7 P2 (<modèle>) : <décompte> » ; inter-fiches signalés.
