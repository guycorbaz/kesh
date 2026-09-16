# Prompt — passe 2 de `bmad-code-review`, Story 25-1c-zero — PASSE CIBLÉE

*Versionné le 2026-09-15. Une seule lentille en contexte frais (Opus), braquée sur **la seule
remédiation de la passe 1**. Protocole : `CLAUDE.md` § *La passe ciblée*.*

## Pourquoi une passe ciblée

La passe 1 (trois lentilles) n'a retenu qu'**un** MEDIUM, et il ne mettait pas en cause le code de
production : le test de caractérisation de l'AC 8 écrivait `test.identiques` **avant** l'export, si
bien que son sous-cas « identiques » était vrai par construction. La remédiation touche **un seul
fichier de test** et le montage correspondant de la spec. *La sévérité se déplace vers ce qu'on vient
d'écrire* : c'est ce correctif, et lui seul, qu'il reste à relire.

## Ton objet

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-1c-zero-audit-company-id`. **Le diff de la
remédiation, et rien d'autre** :

```sh
git diff 9c04aa08 -- crates/kesh-api/tests/admin_full_import_e2e.rs \
  _bmad-output/implementation-artifacts/25-1c-zero-audit-company-id.md
```

`9c04aa08` est le commit d'implémentation, antérieur à la passe 1. **Nomme cette base dans ton
rapport.** Hors de ce diff, ne lis le code que pour vérifier ce que le diff affirme.

## Les axes, et tu déclareras lesquels tu as exercés

1. **Le test `characterization_full_import_keeps_company_ids_as_written`, exécuté mentalement
   contre le code, ligne à ligne.** Pour chaque assertion : sur quelle ligne de quelle table
   porte-t-elle, quelle valeur y trouve-t-on après l'import, et **qu'est-ce qui la rendrait
   fausse** ? En particulier :
   - le `UPDATE companies SET name = ?` local est-il bien **écrasé** par le restore — `companies`
     est-elle vidée puis réinsérée depuis l'archive (`crates/kesh-db/src/backup.rs`,
     `restore_body`) ? Une contrainte (`version`, `ON UPDATE`, unicité du nom) peut-elle faire échouer
     le renommage ou l'import ?
   - l'assertion « autre nom » tranche-t-elle, ou le nom restauré pourrait-il coïncider avec
     `NOM_LOCAL` pour une raison du montage ?
   - `test.identiques`, écrite après l'export, peut-elle malgré tout se retrouver en deux copies ?
     `admin.full_export` ou une autre entrée d'audit écrite pendant l'export ou l'import perturbe-t-elle
     les décomptes par action ?
   - les assertions « différents », « verbatim » et `admin.full_import` tiennent-elles toujours avec
     le nouvel ordre ?
2. **Le doc-comment et les commentaires réécrits** disent-ils exactement ce que le code fait ?
3. **La spec** : l'AC 8 réécrit est-il cohérent avec le test, et ne contredit-il pas un autre passage
   de la spec (tâches, Dev Notes, Change Log) ?

## Ce que tu rends

- **Les findings**, chacun avec sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), `fichier:ligne`,
  **la commande ou l'extrait qui l'établit**, et le correctif. Pour un scénario, montre que **son état
  de départ est atteignable**.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » non
  adossé à cette liste ne clôt rien. Ne qualifie jamais de « vérifié » ce que tu n'as pas exécuté.

⛔ **Tout finding `CRITICAL` ou `HIGH` affirmant l'absence ou la présence d'un code doit être établi
par un `grep -nF` cité.**

## Interdits

⛔ **N'écris AUCUN fichier du dépôt et n'exécute AUCUNE commande qui mute** — nommément
`scripts/prepare-release.sh` (il bumpe les versions Cargo), `scripts/regen-test-schema.sh`,
`scripts/install-hooks.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, tout
`git commit`/`push`/`checkout`/`add`/`stash`/`reset`, `sqlx migrate`, et toute écriture dans les
bases `kesh` ou `kesh_e2e`. `cargo nextest run -p kesh-api --test admin_full_import_e2e` **en
lecture** est autorisé : il ne travaille que sur des bases éphémères.
