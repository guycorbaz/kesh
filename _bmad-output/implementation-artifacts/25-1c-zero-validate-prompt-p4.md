# Prompt — passe 4 de `bmad-create-story validate`, Story 25-1c-zero — PASSE CIBLÉE

*Versionné le 2026-09-15. Une seule lentille en contexte frais (Opus), braquée sur **la seule
remédiation de la passe 3**. Protocole : `CLAUDE.md` § *La passe ciblée*.*

## Pourquoi une passe ciblée

Les trois passes ont suivi le motif mesuré du dépôt : *la sévérité se déplace vers ce qu'on vient
d'écrire.* La passe 3 n'a rien trouvé dans la conception d'origine ; son HIGH était **créé par la
remédiation de la passe 2** (un montage de test où deux assertions exigeaient deux valeurs sur la
même ligne). Ce qu'il reste à relire n'est plus la story : c'est **le dernier correctif**.

## Ton objet

Le fichier `_bmad-output/implementation-artifacts/25-1c-zero-audit-company-id.md`, dépôt
`/home/gcorbaz/devel/kesh`, **restreint au diff de la passe 3** :

```sh
git diff c3f91d8a -- _bmad-output/implementation-artifacts/25-1c-zero-audit-company-id.md
```

`c3f91d8a` est le commit de la passe 2 : ce diff montre exactement ce que la passe 3 a réécrit — le
montage de l'AC 8 (deux entrées pour U1, copies locale et d'archive), une ligne de la table de
mutations de l'AC 16, et l'entrée de Change Log. **Nomme cette base dans ton rapport.**

⛔ Hors de ce diff, ne relis la spec **que** pour vérifier qu'un énoncé ancien ne contredit pas une
ligne réécrite.

## Les axes, et tu déclareras lesquels tu as exercés

1. **Le montage réécrit de l'AC 8, exécuté mentalement contre le code, ligne à ligne.** Pour
   chaque assertion : sur quelle ligne d'`audit_log` porte-t-elle, quelle valeur y trouve-t-on
   **après** l'import, et **qu'est-ce qui la rendrait fausse** si le code était fautif ? En
   particulier :
   - l'affirmation « la copie locale est celle de plus petit `id`, la copie d'archive celle d'`id`
     neuf » est-elle vraie **dans tous les cas** ? L'`AUTO_INCREMENT` d'`audit_log` peut-il être
     réinitialisé ou abaissé pendant le restore — `DELETE` d'autres tables, `FOREIGN_KEY_CHECKS`,
     redémarrage, `ALTER TABLE` ? Lis `backup.rs` et la séquence du handler d'import ;
   - l'entrée `admin.full_import` et d'éventuelles entrées `books.restored` écrites pendant
     l'import perturbent-elles le décompte ou l'ordre des `id` que le montage suppose ?
   - l'entrée locale de U2 porte-t-elle bien C2 **au moment de l'écriture** — la société C2
     existe-t-elle alors ?
2. **La ligne réécrite de la table de mutations** : la mutation fait-elle rougir 15 (a) **et**
   15 (b), et seulement sous la condition écrite ?
3. **Cohérence** : un énoncé non réécrit de la spec — tâches, Dev Notes, AC 7, AC 16, Change Log —
   contredit-il le montage à deux entrées ?

## Ce que tu rends

- **Les findings**, chacun avec sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), l'endroit exact,
  **la commande ou l'extrait qui l'établit**, et ce qu'il faut changer. Pour un scénario, montre
  que **son état de départ est atteignable**.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » non
  adossé à cette liste ne clôt rien.

## Interdits

⛔ **N'écris aucun fichier du dépôt et n'exécute aucune commande qui écrit dans le dépôt ou dans
une base persistante** — nommément `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`,
`scripts/install-hooks.sh`, `scripts/test-fast.sh`, `make` dans `docs/manual/`, tout
`git commit`/`push`/`checkout`/`stash`, `sqlx migrate`, `cargo test`/`cargo nextest`. Seule
exception : une base jetable `_p4_scratch` que tu crées et **supprimes** toi-même (accès :
`docker exec kesh-mariadb-dev mariadb -uroot -pkesh_dev_root`) — jamais `kesh` ni `kesh_e2e`.
