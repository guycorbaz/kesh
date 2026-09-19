# Prompt — passe 1 de `bmad-code-review`, Story 25-2-b-zero

*Versionné le 2026-09-19. Une lentille en contexte frais (Sonnet), orthogonale à l'auteur de
l'implémentation (Opus 5). Une seule lentille : le diff tient en une fonction, ses tests et deux
paragraphes de manuel.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-2-b-zero-verrou-periode-suppression`.

## L'objet

**Le diff `main...HEAD` APLATI, hors documentation BMAD et PDF** :

```sh
git diff main...HEAD -- . ':(exclude)_bmad-output' ':(exclude)*.pdf'
```

Copie figée : `/tmp/claude-1000/-home-gcorbaz-devel-kesh/b4a4f4f4-facb-4366-8d5d-2d8994016d0b/scratchpad/review-zero.diff`.

La spécification est `_bmad-output/implementation-artifacts/25-2-b-zero-verrou-periode-suppression.md`
— huit critères, validée en une passe. Son Change Log déclare quatre mutations tuées, un gate à
2401/2401 et un E2E dont trois échecs sont attribués à la pollution : **ne les crois pas sur parole.**

## Les axes, et tu déclareras lesquels tu as exercés

1. **La garde.** Est-elle exacte — seuil inclusif, bonne société, bonne date (celle de l'écriture,
   ramenée par la requête `FOR UPDATE` de l'étape 2) ? Est-elle au bon endroit — après le gel, avant
   le snapshot et l'audit, de sorte qu'un refus ne laisse **aucune** trace d'audit orpheline ni
   aucune écriture partielle ? La lecture non verrouillante de `companies` est-elle sûre ici ?
2. **Les appelants.** Inventorie tous les appelants de `delete_in_tx` et dis ce que la garde change
   pour chacun. `invoices::delete` rend-il bien l'erreur au client, et **annule-t-il** la
   suppression de la facture déjà exécutée dans la même transaction ?
3. **Les tests prouvent-ils ce qu'ils disent ?** Cherche l'assertion qui passerait si le code était
   faux. Le helper `supprimer_sous_borne` peut-il laisser une borne posée si un `unwrap` panique
   avant son retrait ? Les dates (`il_y_a(30)`, `il_y_a(29)`) tombent-elles toujours dans l'exercice
   ouvert que `setup` garantit, quel que soit le jour d'exécution (début janvier, par exemple) ?
   Le test de bout en bout mesure-t-il bien la garde, et non une autre raison de refuser ?
4. **Les commentaires et messages.** Chaque doc-comment ajouté ou modifié dit-il vrai ? Le
   doc-comment de `delete_in_tx` décrit-il encore fidèlement ses gardes ?
5. **Les manuels, le CHANGELOG, le README.** Les phrases ajoutées disent-elles vrai sans
   sur-promettre ? ⚠️ Contrôle les **PDF aplatis** (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`)
   des manuels utilisateur **et** administrateur. Cherche un **autre** site qui décrit le verrou de
   période ou la suppression d'une facture validée et qui serait désormais faux.

## Ce que tu rends

- **Les findings** : sévérité, `fichier:ligne`, **la commande ou l'extrait qui l'établit**, le
  correctif. Pour un scénario, montre que son état de départ est atteignable.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

⛔ Tout finding `CRITICAL` ou `HIGH` affirmant l'absence d'un code ou la présence d'un anti-pattern
doit citer un `grep -nF` qui l'établit.

## Interdits

⛔ **N'écris AUCUN fichier du dépôt et n'exécute AUCUNE commande qui mute** — nommément
`scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout `git commit`/`push`/
`checkout`/`add`/`stash`/`reset`, `sqlx migrate`, `cargo fmt` sans `--check`, `cargo test`/
`cargo nextest` (ils écrivent dans la base partagée), et toute écriture dans `kesh` ou `kesh_e2e`.
Lecture, `grep`, `git diff`/`log`/`show`, `pdftotext` et `cargo check` sont autorisés.
