# Prompt — passe 5 de `bmad-create-story validate`, Story 25-1c-a — PASSE CIBLÉE

*Versionné le 2026-09-15. Une seule lentille en contexte frais (Sonnet), orthogonale à la passe 4 (Opus).
Protocole : `CLAUDE.md` § *La passe ciblée*.*

## Pourquoi cette passe, et ce qu'elle a de différent

Les passes 2, 3 et 4 ont chacune trouvé un MEDIUM **dans la correction de la précédente**, toujours sur
les bornes de date. La forme « `created_at < date_to + 1 jour` » a donc été **abandonnée** : l'AC 2 lie
désormais des bornes **inclusives** `date_from 00:00:00.000` et **`date_to 23:59:59.999`**, sans aucune
arithmétique de date. L'orchestrateur a éprouvé cette forme sur base jetable avant de l'écrire. **Ta
mission est de la prendre en défaut** — pas de relire la story.

## Ton objet

Le fichier `_bmad-output/implementation-artifacts/25-1c-a-journal-audit-route.md`, dépôt
`/home/gcorbaz/devel/kesh`, **restreint au diff de la passe 4** :

```sh
git diff c25999f7 -- _bmad-output/implementation-artifacts/25-1c-a-journal-audit-route.md
```

`c25999f7` est le commit de la passe 3. **Nomme cette base dans ton rapport.** Hors de ce diff, ne relis
la spec que pour vérifier qu'un énoncé ancien ne contredit pas une ligne réécrite.

## Les axes, et tu déclareras lesquels tu as exercés

1. **La borne inclusive, attaquée** — sur une base jetable **et** par une sonde Rust aux versions du
   `Cargo.lock` :
   - `created_at` est-il **vraiment** `DATETIME(3)` partout où la route lit — squash de test **et**
     migrations réelles ? Une entrée peut-elle porter plus de trois décimales (fusion d'archive, insertion
     manuelle, `NOW(6)`) et se glisser entre `23:59:59.999` et minuit ?
   - `NaiveDate::and_hms_milli_opt(23, 59, 59, 999)` existe-t-il, et que rend-il ? Comment sqlx encode-t-il
     ce `NaiveDateTime` (précision microseconde, `DATETIME(6)`) et la comparaison au `DATETIME(3)`
     reste-t-elle exacte **en protocole binaire** — pas seulement en `PREPARE` SQL ?
   - `1000-01-01 00:00:00.000` et `9999-12-31 23:59:59.999` se lient-ils sans panique ni avertissement ?
2. **Les tests réécrits** — AC 19 (d), AC 20 (dates encodées `%2B`, `-0001-01-01`, `0999-12-31` des deux
   côtés, `9999-12-31` avec items), AC 21 (mutation de la borne) : chacun **tranche**-t-il ? Qu'est-ce qui
   le rendrait faux ? `-0001-01-01` passe-t-il l'extracteur `Query` et le parsing du handler ?
3. **Cohérence** : un énoncé non réécrit — AC 3, Dev Notes, tableau des fichiers, Change Log des passes
   précédentes, en-tête — cite-t-il encore « `+ 1 jour` », `checked_add_days`, `Days`, l'omission de la
   borne ou la borne exclusive **comme prescription actuelle** ?

## Ce que tu rends

- **Les findings**, chacun avec sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), l'endroit exact, **la
  commande ou l'extrait qui l'établit**, et le correctif. Pour un scénario, montre que **son état de
  départ est atteignable**.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » non adossé
  à cette liste ne clôt rien.

## Interdits

⛔ **N'écris aucun fichier du dépôt et n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante** — nommément `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`,
`scripts/install-hooks.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, tout
`git commit`/`push`/`checkout`/`add`/`stash`/`reset`/`rebase`, `sqlx migrate` sur `kesh` ou `kesh_e2e`,
`cargo test`/`cargo nextest`. Seules exceptions : une base jetable `_v5_scratch` que tu crées et
**supprimes** toi-même, et une crate de sonde **dans le scratchpad**
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/cb9f9ce3-4808-472f-93d0-698c43110e0e/scratchpad/`.
