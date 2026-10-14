# Prompt — revue de code P2 ciblée, Story 29-1a (préparer la clôture : contrôles préalables)

*Versionné le 2026-10-14. Passe ciblée (`CLAUDE.md` § « La passe ciblée ») : **une seule lentille, Haiku**, contexte
frais, **lecture seule stricte**, braquée sur le seul diff de la remédiation de la revue de code P1.*

Worktree `/home/gcorbaz/devel/kesh-29-1`, branche `story/29-1` (rebasée sur `origin/main` = `c36f39bc`, Story 29-2).

**Objet — un diff unique, aplati** (ne lis pas les commits un par un : `CLAUDE.md` § *Haiku-specific guardrails*) :

```sh
git diff 34750f1f 1c15363e -- . ':(exclude)_bmad-output'
```

`34750f1f` = état revu en P1 (après rebase) ; `8b5e1a3f` = tests et doc-comments (dernier commit de **code**) ;
`1c15363e` = CHANGELOG, `api-external.md`, manuel `.tex` et PDF. Le compte rendu de la remédiation est dans la fiche
`_bmad-output/implementation-artifacts/29-1-preparer-cloture.md` (Change Log, entrée « revue de code P1 remédiée » ;
Dev Agent Record, section « remédiation de la revue de code P1 » ; table *Mutations*, lignes M-a23 à M-a41) et au
registre `epic-29-choix-autonomes-29-1.md` (C-29-1-43 à 47). Rapports remédiés :
`/home/gcorbaz/devel/kesh-gate-logs/29-1a-review-p1-{B,E,A}.md`. Journaux des mutations rejouées :
`kesh-gate-logs/291a-p1-mutations-{front,back}.log`.

⛔ **DONNÉES PERSONNELLES — DÉPÔT PUBLIC.** N'ouvre aucun fichier hors du dépôt et de `/home/gcorbaz/devel/kesh-gate-logs/`.

## La lentille unique — chasseur de régressions de la remédiation

1. **Aucune ligne exécutable de production n'a changé — à vérifier, pas à croire.** Sur le diff ci-dessus restreint à
   `crates/*/src` et `frontend/src` hors `*.test.ts`, toute ligne ajoutée ou retirée est-elle un commentaire (`//`,
   `///`, `//!`) ? Commande attendue, en pipe :
   `git diff -U0 34750f1f 1c15363e -- crates frontend/src ':(exclude)*.test.ts' ':(exclude)crates/*/tests' | grep -E '^[+-]' | grep -vE '^(\+\+\+|---)' | grep -vE '^[+-]\s*//'`
   (sortie vide attendue). Les doc-comments neufs de `closing_checks.rs` (formes que T-a2 ne voit pas, avoirs, listes
   tronquées, coût de K6, K7 après un trou, écart au demi-centime, `///` des types publics, équivalence de M-a33)
   disent-ils **vrai** contre le code ? Un doc-comment peut-il faire rougir la garde T-a2 (mots de verrou ou appels
   `query*` dans un commentaire — le source est-il bien décommenté avant lecture) ?
2. **Les tests neufs prouvent-ils ce qu'ils nomment ?** `crates/kesh-db/tests/closing_checks.rs` :
   `t_a5_k3_bornes_de_cloture_et_ordre_des_motifs`, `t_a9_k6_perimetre_de_y_et_bornes`, `t_a7_k4_liste_bornee_et_ordonnee`,
   et les ajouts à `t_a3_…` et `t_a10_…` ; `fiscal-years-page.test.ts`, bloc « Jeton de requête ». Pour chacun : la
   fixture pose-t-elle bien l'état annoncé (assertions de montage) ; l'attente est-elle exacte (dates, motifs, ordre,
   `count`) ; le test peut-il passer **à vide** ; la mutation qu'il revendique (journaux) le fait-elle rougir pour la
   **bonne** raison ? Pour les Vitest : le `flush()` laisse-t-il réellement à la réponse tardive le temps d'écrire
   l'état (sinon le test serait vert même sans garde — les journaux montrent-ils M-a23 à M-a26 tuées ?) ; la réouverture
   du dialogue de clôture après « Annuler » appelle-t-elle bien une seconde lecture ?
3. **M-a33 déclarée équivalente** (C-29-1-44) : la démonstration tient-elle contre `INCONSISTENT_SQL` tel qu'écrit
   (chacun des quatre drapeaux implique-t-il la population) ? Une autre mutation plausible du `WHERE` ou du `HAVING`
   resterait-elle verte ?
4. **Textes** (CHANGELOG, `docs/api-external.md` note ¹⁰, manuel). ⛔ **Le manuel se contrôle sur le PDF**, aplati en
   pipe : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oF "<phrase>"`. Le PDF à
   `1c15363e` porte-t-il les phrases neuves du `.tex` (écart au demi-centime, listes de vingt, avoirs, K7 après un trou,
   bouton *Créer l'exercice suivant*, glossaire *Report à nouveau*) ? Chaque phrase neuve est-elle **vraie** contre le
   code (arrondi `MidpointAwayFromZero` de `statement_gap` ; `MAX_LISTED` ; `credit_notes::create` au statut
   `issued` ; `NEXT_AFTER_SQL`) ? CHANGELOG : l'entrée de la 29-1a est-elle sous `## [0.13.4] — Non publié`, après celle
   de la 29-2, sans rien perdre de l'une ni de l'autre (compare à `git show c36f39bc:CHANGELOG.md`) ; la section
   `## [0.14.0]` vide est-elle un défaut pour `scripts/prepare-release.sh` (**lis-le, ne l'exécute pas**) ?
   Propagation : reste-t-il, hors `_bmad-output` et hors historique daté, une mention « v0.14.0 » pour la 29-1a, ou
   « inférieur au centime », « Nouvel exercice » pour le lien K7, « à la clôture » pour le report ?
5. **Comptes rendus** : les décomptes de l'entrée du Change Log (tests neufs aux deux bornes `34750f1f` → `8b5e1a3f`,
   mutations rejouées, tuées, équivalentes ; chiffres des gates contre `kesh-gate-logs/291a-p1-final-*.log`) se
   recomptent-ils depuis la source ?

## Ce que tu rends

Findings numérotés (CRITICAL / HIGH / MEDIUM / LOW), `fichier:ligne`, **preuve** — sortie de `grep -nF` copiée, ou code
relu cité —, correction proposée. ⛔ **Liste des axes exercés ET non exercés** : un « 0 finding » sans elle ne compte
pas comme passe. Tout finding qui affirme l'**absence** d'un code attendu ou la **présence** d'un défaut s'appuie sur
une sortie `grep -nF` copiée ; un MEDIUM qui reproche l'absence d'un livrable se vérifie d'abord au `grep -nF`.

Rapport complet dans `/home/gcorbaz/devel/kesh-gate-logs/29-1a-review-p2-ciblee.md` — **le seul fichier que tu
écris**. Dernier message : le chemin, le bilan, une ligne par MEDIUM+.

## Interdits — lecture seule stricte

- ⛔ **Aucun fichier temporaire**, nulle part (ni `/tmp`, ni `kesh-gate-logs/`, ni le dépôt) : toute comparaison se
  fait **en pipe** (`diff <(…) <(…)`, `comm <(…) <(…)`, `| grep`), **jamais** par redirection vers un fichier. Les deux
  lentilles de la passe P1 ont écrit des fichiers temporaires : c'est précisément ce qu'il ne faut pas refaire.
- Aucune commande qui écrit, compile ou exécute : `scripts/*` — **dont `scripts/prepare-release.sh`** —, `make`,
  `latexmk`, `cargo`, `npm`, `npx`, `sqlx`, `docker`, `python3`, `sed -i`, `perl -pi`, `git commit` / `add` /
  `checkout` / `switch` / `restore` / `stash` / `reset` / `rebase`, `gh` en écriture, toute requête SQL.
- Autorisés : lecture de fichiers, `grep`, `sed -n`, `awk` sans redirection, `git log` / `show` / `diff` / `grep`,
  `pdftotext … -` en pipe.
