# Prompt — passe 4 de `bmad-create-story validate`, Story 25-1c-a — PASSE CIBLÉE

*Versionné le 2026-09-15. Une seule lentille en contexte frais (Opus), braquée sur **la seule
remédiation de la passe 3**. Protocole : `CLAUDE.md` § *La passe ciblée*.*

## Pourquoi une passe ciblée

La passe 3 n'a trouvé qu'un MEDIUM et un LOW, **tous deux nés de la remédiation de la passe 2** (les
bornes de date de la route). Tout le reste a été confirmé sur pièces. Ce qu'il reste à relire n'est plus
la story : c'est **le dernier correctif**.

## Ton objet

Le fichier `_bmad-output/implementation-artifacts/25-1c-a-journal-audit-route.md`, dépôt
`/home/gcorbaz/devel/kesh`, **restreint au diff de la passe 3** :

```sh
git diff 63e1ce4d -- _bmad-output/implementation-artifacts/25-1c-a-journal-audit-route.md
```

`63e1ce4d` est le commit de la passe 2. **Nomme cette base dans ton rapport.** Hors de ce diff, ne relis
la spec que pour vérifier qu'un énoncé ancien ne contredit pas une ligne réécrite.

## Les axes, et tu déclareras lesquels tu as exercés

1. **La répartition route / repository des bornes de date (AC 2, AC 7)**, exécutée mentalement — ou par
   sonde dans le scratchpad — contre le code :
   - la route valide `[1000-01-01, 9999-12-31]` pour **les deux** dates ; le repository calcule
     `date_to.checked_add_days(Days::new(1))` dans `push_where_clauses` et omet la clause si `None`.
     `chrono::Days` et `checked_add_days` existent-ils sous cette forme dans la version de chrono du
     dépôt (`Cargo.lock`) ?
   - Une fois la plage validée par la route, le `None` de `checked_add_days` ne survient que pour
     `9999-12-31` : est-ce exact ? La borne basse `date_from` à `1000-01-01 00:00:00` s'encode-t-elle
     sans avertissement MariaDB ?
   - `push_where_clauses` recevant des `NaiveDate`, la borne haute se lie-t-elle comme `DATE` ou
     `DATETIME`, et la comparaison à un `DATETIME(3)` reste-t-elle exacte à la milliseconde ?
2. **Les trois cas de test ajoutés à l'AC 20** (`dateFrom=+262142-12-31`, `dateFrom=0999-12-31`, et le
   `dateTo` symétrique) : chacun rougirait-il si la validation de la borne basse était absente ?
   `+262142-12-31` passe-t-il seulement l'extracteur `Query` (chaîne parsée dans le handler) ?
3. **Cohérence** : un énoncé non réécrit de la spec — AC 19 (d), tableau de mutations de l'AC 21, Dev
   Notes, Change Log — contredit-il la répartition « la route valide, le repository calcule » ?

## Ce que tu rends

- **Les findings**, chacun avec sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), l'endroit exact, **la
  commande ou l'extrait qui l'établit**, et le correctif. Pour un scénario, montre que **son état de
  départ est atteignable**.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » non
  adossé à cette liste ne clôt rien.

## Interdits

⛔ **N'écris aucun fichier du dépôt et n'exécute aucune commande qui écrit dans le dépôt ou dans une
base persistante** — nommément `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`,
`scripts/install-hooks.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, tout
`git commit`/`push`/`checkout`/`add`/`stash`/`reset`/`rebase`, `sqlx migrate` sur `kesh` ou `kesh_e2e`,
`cargo test`/`cargo nextest`. Seules exceptions : une base jetable `_v4_scratch` que tu crées et
**supprimes** toi-même, et une crate de sonde **dans le scratchpad**
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/cb9f9ce3-4808-472f-93d0-698c43110e0e/scratchpad/`.
