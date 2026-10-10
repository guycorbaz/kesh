# Revue de code P2 — issue #566 (`scripts/prepare-release.sh` déjà bumpé)

Prompt versionné, commun aux deux lentilles de la passe 2. Modèle : Opus, contexte frais, lecture seule, UNE lentille qui porte B et E à la fois.

## Objet

Worktree : `/home/gcorbaz/devel/kesh-566`, branche `fix/566-prepare-release-deja-bumpe`.
Diff à relire : `git -C /home/gcorbaz/devel/kesh-566 diff origin/main...HEAD` (commits `bbcc0e3c` puis `a7ca2ac1` — remédiation de la passe 1 —, et tout commit
postérieur de la branche).

L'issue #566 : `scripts/prepare-release.sh X.Y.Z` s'arrêtait (`exit 1`, « version cible identique ») quand les crates
portaient déjà la version cible — cas normal depuis la règle P2-bis du `CLAUDE.md` (bump des crates dans le commit de
la migration qui relève `kesh_version_min_required`). Il sautait alors son pré-vol : le contrôle des exemptions
périssables d'`EXEMPT_MIGRATIONS` (`crates/kesh-db/src/post_restore.rs`, `ExemptionBasis::PerishableSince`, lu par
`crates/kesh-db/examples/perishable_exemptions.rs`) — seul filet de ce contrôle — et la datation du CHANGELOG.

Correctif attendu (option 1 de l'issue) : déjà bumpé ⇒ bump sauté, TOUT le reste exécuté et annoncé en clair ;
version inférieure refusée ; datation idempotente (relancé, il ne date pas deux fois et ne casse rien) ; test
automatisé couvrant ces cas ; `CLAUDE.md` point 6 de la liste pré-release mis à jour d'une phrase.

Fichiers : `scripts/prepare-release.sh`, `scripts/tests/prepare-release.test.sh`,
`crates/kesh-db/tests/prepare_release_script.rs`, `CLAUDE.md`, `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`
(entrée C-566-1).

## ⛔ Interdits

- **N'exécute JAMAIS `scripts/prepare-release.sh`**, ni directement ni par aucun détour (il écrit : bump des dix
  crates, CHANGELOG). N'exécute pas non plus `scripts/tests/prepare-release.test.sh`, ni `cargo` sous aucune forme,
  ni `scripts/test-fast.sh`, ni aucun script du dépôt.
- N'écris, ne modifie, ne crée aucun fichier — sauf ton rapport. Aucune commande git qui écrit (`commit`, `checkout`,
  `stash`, `reset`, `tag`…), aucun `gh` qui écrit.
- Commandes autorisées : `git diff`, `git show`, `git log`, `git ls-tree`, `grep`, `sed -n`, `cat`, lecture de fichiers.

## Lentille

- **B — Blind Hunter** : lis le diff comme un relecteur qui ne connaît pas le dépôt. Correction du shell
  (`set -euo pipefail`, citations, codes de sortie, `grep -c` à 0 sous `pipefail`, regex, échappements `sed`/`grep -E`
  sur `[`, `]`, `.`), cohérence des messages avec ce qui se passe réellement, documentation (en-tête du script,
  `CLAUDE.md`, registre) fidèle au code, test qui prouve ce qu'il dit.
- **E — Edge Case Hunter** : parcours chaque branche et chaque bord. Matrice à couvrir au minimum :
  version courante {égale, inférieure, supérieure} × CHANGELOG {« Non publié », daté, absent, en double, daté avec
  suffixe} × crates {tous alignés, partiel} × pré-vol {aucune exemption, exemption tenue, exemption démentie,
  inventaire illisible} × dépôt {propre, sale, sur `main`}. Pour chaque cellule : le script écrit-il avant de pouvoir
  refuser ? annonce-t-il ce qu'il fait ? le test la couvre-t-il ? Versions à plusieurs chiffres (`0.9.0` vs `0.13.0`),
  `sort -V`. Le test : peut-il passer à vide (test muet) ? toucher le dépôt réel ? dépendre de la configuration git
  de la station, de la date (minuit), de l'environnement CI (bash, git, `TMPDIR`) ?

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/566-review-p2-R.md` :

1. **Axes exercés** et **axes NON exercés** — liste explicite. Un « 0 finding » sans cette liste ne compte pas.
2. Findings numérotés `P2-n`, chacun avec sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier:ligne,
   description, preuve, correctif proposé. **Toute affirmation de présence ou d'absence d'un code s'appuie sur une
   sortie `grep -nF` copiée dans le rapport.**
3. Verdict : nombre de findings par sévérité.

## Spécifique à la passe 2

Exerce les deux lentilles ci-dessus (B et E). Puis braque une attention particulière sur `a7ca2ac1`, la remédiation
de la passe 1 : *la sévérité se déplace vers ce qu'on vient d'écrire*. Points connus de la P1 : P1-E-1 (MEDIUM, tag
déjà publié non refusé — corrigé par une garde `git rev-parse -q --verify refs/tags/vX.Y.Z`) ; LOW P1-B-1..5,
P1-E-2..8 (P1-E-2, asymétrie de la garde de bump partiel dans le bump NORMAL, est laissé en l'état : « bump normal
inchangé » est une contrainte de l'issue). Rapports P1 : `/home/gcorbaz/devel/kesh-gate-logs/566-review-p1-{B,E}.md`
— ne les recopie pas, vérifie les correctifs et cherche ce qu'ils ont cassé ou manqué (renvois ajoutés dans
`15-1a-i-marque-du-lettrage.md` et le registre compris).
