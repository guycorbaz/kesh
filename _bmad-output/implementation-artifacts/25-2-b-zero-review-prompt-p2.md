# Prompt — passe 2 de `bmad-code-review`, Story 25-2-b-zero — passe CIBLÉE

*Versionné le 2026-09-19. Une lentille « Regression Hunter » en contexte frais (Opus), modèle
différent de la passe 1 (Sonnet). Passe ciblée au sens du `CLAUDE.md` : la remédiation relue ne
touche aucune ligne de code de production.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-2-b-zero-verrou-periode-suppression`.

## L'objet — le SEUL commit de remédiation

```sh
git show 1c9ca464 -- . ':(exclude)_bmad-output' ':(exclude)*.pdf'
```

La passe 1 (rapport au Change Log de la fiche, entrée « revue P1 ») a relevé : des tests datés
« il y a trente jours » qui paniquaient en janvier, un `unwrap` entre pose et retrait de la borne,
une liste de refus du manuel incomplète, un doc-comment d'étapes périmé. Tu peux lire le code
environnant autant que nécessaire.

## Les axes, et tu déclareras lesquels tu as exercés

1. **Les tests réécrits prouvent-ils encore ce qu'ils disent ?** Écriture datée du jour, borne sur
   le jour ou la veille : chacun des quatre tests rougirait-il si la garde était retirée, si `<=`
   devenait `<`, si la garde passait avant le gel ? Un test peut-il désormais passer **à vide** —
   par exemple parce qu'une borne du jour empêche la **création** même de l'écriture, ou parce que
   la garde de création et celle de suppression se confondent ?
2. **L'auto-réparation de `setup`.** Retirer toute borne au début de chaque test de ce module :
   masque-t-elle un défaut qu'un test existant devait voir ? Un test du module pose-t-il une borne
   qu'il compte retrouver ? Et les AUTRES modules qui partagent la base de dev ?
3. **Le bloc `async` qui recueille les erreurs.** Le `?` sur `begin`, `commit`, `rollback` rend-il
   bien le contrôle au retrait de la borne dans tous les cas ? Un chemin de panique subsiste-t-il
   entre la pose et le retrait ?
4. **Le manuel.** La quatrième entrée de la liste des refus dit-elle vrai, et renvoie-t-elle à la
   bonne section ? Contrôle le **PDF aplati** (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`).
5. **Le doc-comment de `delete_by_id`** dit-il désormais vrai, étape par étape ?

## Ce que tu rends

- **Les findings** : sévérité, `fichier:ligne`, **la preuve**, le correctif.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Ne qualifie jamais de
  « vérifié » un point que tu n'as pas exécuté.

## Interdits

⛔ **N'écris AUCUN fichier du dépôt et n'exécute AUCUNE commande qui mute** — nommément
`scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout `git commit`/`push`/
`checkout`/`add`/`stash`/`reset`, `sqlx migrate`, `cargo fmt` sans `--check`, `cargo test`/
`cargo nextest`, et toute écriture dans `kesh` ou `kesh_e2e`. Lecture, `grep`, `git show`/`diff`,
`pdftotext` et `cargo check` sont autorisés.
