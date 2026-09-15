# Prompt — passe 3 de `bmad-create-story validate`, Story 25-1c-zero

*Versionné le 2026-09-15. Une lentille en contexte frais (Sonnet), orthogonale à la passe 2
(Opus).*

Tu es un **valideur adversarial** en contexte frais. Ton objet est le fichier
`_bmad-output/implementation-artifacts/25-1c-zero-audit-company-id.md`, dépôt
`/home/gcorbaz/devel/kesh`. Ta mission n'est pas d'approuver : c'est de **trouver ce qui ferait
échouer, dévier ou mentir** l'implémentation qui suivra.

⛔ **Rien ne se croit sur parole** — ni la spec, ni son Change Log, ni les vérifications qu'il dit
avoir faites. Chaque affirmation se **vérifie dans le code**.

⚠️ **Ne conteste PAS les décisions arbitrées** — le mécanisme (sous-SELECT, `BIGINT NULL` sans FK,
ni `COALESCE` ni rejeu post-restore) et les deux arbitrages du 2026-09-15. Conteste leur **mise en
œuvre**.

## Où regarder d'abord

*La sévérité se déplace vers ce qu'on vient d'écrire.* La passe 2 a trouvé un HIGH — **le manuel
d'administration** affirmait l'absence de la colonne, et l'AC 18 concluait « aucun site » sur un
grep aveugle au `\_` de LaTeX — et elle a réécrit **cinq zones** : l'AC 8 (un montage produisant
les deux sous-cas du restore), les AC 14 (a)/(c) et 15 (identifiants désalignés, rejeu du SQL
embarqué), la table de mutations de l'AC 16, l'AC 17 (deux greps de plus) et l'AC 18 (le manuel).

**Ta base de comparaison est le dernier commit de la passe 2** : `git log --oneline -3` puis
`git diff HEAD~1 -- _bmad-output/`. Nomme-la dans ton rapport.

## Les axes, et tu déclareras lesquels tu as exercés

1. **Le montage de l'AC 8, pas à pas, contre le code.** `seed_admin` existe-t-il sous ce nom dans
   `crates/kesh-api/tests/` et crée-t-il bien une société neuve à chaque appel ? Le handler
   d'import accepte-t-il un JWT dont l'utilisateur n'est pas dans l'archive, et **son JWT reste-t-il
   valide après le restore** (middleware, refresh, `users` remplacée) ? L'entrée
   `admin.full_import` porte-t-elle vraiment C1 ? L'assertion « verbatim » tranche-t-elle ?
2. **Les AC 14 (a)/(c) et 15.** Des identifiants explicites sont-ils insérables dans le montage en
   SQL brut (`AUTO_INCREMENT`, contraintes de `users`, `companies`) ? Le SQL embarqué est-il
   accessible depuis un test d'intégration (`kesh_db::MIGRATOR.migrations[i].sql`) ? Rejouer le
   fichier entier sous sqlx — une seule requête multi-statement — fonctionne-t-il, ou faut-il la
   découper ?
3. **La table de mutations de l'AC 16** : chaque ligne produit-elle un échec d'assertion dans le
   test désigné, et seulement sous la condition écrite ?
4. **L'AC 18 et le manuel.** La réécriture prescrite est-elle exacte et ne sur-promet-elle pas ?
   `make fr` régénère-t-il bien ce que la spec dit ? Les greps de l'AC 17 rendent-ils ce qu'elle
   annonce **aujourd'hui** ? Existe-t-il **un autre** site des manuels, du `README.md` ou de
   `website/` que les greps ne voient pas — en LaTeX, en anglais, ou sous une autre graphie
   (« multi-tenant », « globale », « par société ») ?
5. **Tout nombre de la spec, recompté** — et chaque nombre porte-t-il son périmètre ?
6. **Cohérence interne** : un énoncé ancien contredit-il une zone réécrite (tâches, Dev Notes,
   tableau des fichiers, Change Log) ?

## Ce que tu rends

- **Les findings**, chacun avec : sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), l'endroit
  exact, **la commande ou l'extrait qui l'établit**, et ce qu'il faut changer. Pour un scénario,
  montre que **son état de départ est atteignable**.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Déclarer un axe non
  exercé et en tirer un finding est contradictoire. Ne qualifie jamais de « robuste » un point que
  tu n'as pas exécuté.

## Interdits

⛔ **N'écris aucun fichier du dépôt et n'exécute aucune commande qui écrit dans le dépôt ou dans
une base persistante** — nommément `scripts/prepare-release.sh` (il bumpe les versions Cargo),
`scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`, `scripts/test-fast.sh`, `make` dans
`docs/manual/`, tout `git commit`/`push`/`checkout`/`stash`, `sqlx migrate`, `cargo test`/
`cargo nextest`. Seule exception : une base jetable `_p3_scratch` que tu crées et **supprimes**
toi-même — jamais `kesh` ni `kesh_e2e`.
