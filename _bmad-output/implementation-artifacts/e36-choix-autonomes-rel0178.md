# Registre des choix autonomes — intégration de la release 0.17.8 (`rel0178`)

Agent d'intégration (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-rel0178`, branche `release/0.17.8` partie de
`origin/main` = `cefa3e566` le 2026-11-15. Prompt : `kesh-gate-logs/prompts/rel0178-integration.md`. Chaque choix porte
son contexte, l'option retenue, les autres et sa réversibilité.

## C-rel0178-1 — Le kickoff fusionné d'abord

- **Contexte** : la PR #1160 (kickoff de l'Epic 36, `chore/epic-36-kickoff`, tête `c9c759daf`) est ouverte, non fusionnée
  (`gh pr view 1160` : `OPEN`) ; `git merge-base --is-ancestor chore/epic-36-kickoff origin/main` faux.
- **Retenu** : `git merge --no-ff chore/epic-36-kickoff` en tête (`6b64a78fc`), comme le prévoit le § *Cadre* du prompt.
- **Autres** : attendre la fusion de #1160 (bloque la release).
- **Réversibilité** : totale ; si #1160 est fusionnée avant la PR de release, la fusion devient sans objet au squash.

## C-rel0178-2 — `sitesTotal` relevé par la garde après chaque fusion qui le touche

- **Contexte** : `frontend/src/lib/shared/i18n-keys.test.ts`, `sitesTotal`, en conflit aux fusions de 36-3, 36-3b et 36-2.
  `rerere` a rejoué à la fusion de 36-3 la résolution de l'orchestrateur faite dans la branche de la 36-2 (3041).
- **Retenu** : commentaires des branches gardés tous, une ligne d'union « intégration 0.17.8 », et la valeur relevée par
  la garde : **3045** après 36-3b (« expected 3045 to be 3044 »), **3050** après 36-2 (« expected 3050 to be 3046 ») ;
  inchangé après 36-4 et 36-5 (garde verte).
- **Autres** : additionner les deltas des branches (interdit, C-e350-18).
- **Réversibilité** : un nombre ; la garde le contrôle à chaque Vitest.

## C-rel0178-3 — Rubrique *Modifié* pour #1151 seulement

- **Contexte** : le prompt demande les rubriques *Corrigé* et *Modifié* ; les six fiches rangent tout en *Corrigé*.
  #1151 (fenêtre *Éclater* plus large) porte `enhancement, P4`, sans `bug` (`gh issue view 1151`).
- **Retenu** : #1151 sous *Modifié* (ce n'est pas un défaut corrigé, c'est une présentation changée) ; tout le reste
  sous *Corrigé*, dans l'ordre des stories (36-1, 36-3, 36-3b, 36-2, 36-4, 36-5), un point par issue, texte des fiches
  repris en phrases pour l'utilisateur.
- **Autres** : tout sous *Corrigé*, comme les fiches (laisse la rubrique *Modifié* vide).
- **Réversibilité** : déplacement d'un point, avant le tag.

## C-rel0178-4 — La recette du chemin de la 36-4 dans une recette complémentaire, jouée des deux côtés

- **Contexte** : le § 3 du prompt demande une base peuplée par l'image 0.17.7 où une clé d'API a passé une écriture **et
  annulé un règlement** ; le peuplement de `scripts/recette-montee.sh` ne porte ni clé ni rapprochement.
- **Retenu** : `scripts/recette-montee.sh --image gcorbaz/kesh:0.17.7 --migrations-attendues 0 --import` tel quel, puis
  `kesh-gate-logs/rel0178/recette-complement-rel0178.sh`, reprise de celle de la 0.17.7 : en 0.17.7, la clé passe une
  écriture, rapproche une transaction, règle une facture et annule ce règlement ; montée par le binaire 0.17.8, empreinte
  identique ; la clé règle et annule de nouveau **en 0.17.8**, et les six entrées d'audit de l'annulation doivent porter
  `api_key` (elles portent `user` en 0.17.7) ; nouvelle empreinte, référence du retour ; l'image 0.17.7 relancée sur la
  même base, sans restauration, doit la retrouver.
- **Autres** : un crochet `--apres` dans `recette-montee.sh` (son peuplement n'a pas de clé d'API) ; comparer le retour à
  l'empreinte d'avant (ne prouverait pas que la 0.17.7 relit ce que la 0.17.8 a écrit).
- **Réversibilité** : un script de journal, hors du dépôt.

## C-rel0178-5 — Le manuel d'administration décrit les deux recettes telles qu'elles ont tourné

- **Contexte** : la sous-section *Passer à la 0.17.8* avait été écrite avant la recette (§ 1 avant § 3).
- **Retenu** : la phrase des constats réécrite après la recette, en deux recettes, sur le modèle de *Passer à la 0.17.7* ;
  PDF régénéré après.
- **Réversibilité** : une phrase.
