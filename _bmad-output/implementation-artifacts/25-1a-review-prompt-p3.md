# Story 25-1a — prompt de la passe 3 de `bmad-code-review`

**Modèle** : Sonnet 4.6 (rotation : P1 Sonnet/Haiku/Sonnet sur trois lentilles,
P2 Haiku seul, P3 Sonnet seul).
**Contexte** : frais.
**Périmètre** : le commit de remédiation de la passe 2, **plus** une reprise des
axes que la passe 2 a mal couverts.

## Pourquoi cette passe existe, et ce qu'elle doit corriger de la précédente

La passe 2 a rendu 1 CRITICAL, 1 HIGH, 1 MEDIUM. **Deux ont été réfutés au sol**
et le troisième visait à côté :

- son CRITICAL affirmait que les manuels n'avaient pas été touchés — elle avait
  comparé le mauvais commit, et l'extrait qu'elle citait en preuve contenait la
  phrase qui la réfutait ;
- son HIGH reposait sur un état de base **inatteignable** (aucun chemin de
  production n'écrit dans `audit_log` hors du repository) ;
- son MEDIUM raisonnait sur un test où l'`UPDATE` visé **ne tourne pas**.

⛔ **Elle a par ailleurs déclaré l'axe « manuels » NON EXERCÉ tout en en tirant
son CRITICAL.** Tiens cette contradiction pour ce qu'elle est : le signal d'un
axe réellement non couvert. **Reprends-le toi-même, entièrement.**

Le défaut réel de la passe 2 n'a été trouvé qu'en vérifiant ses faux positifs :
rien n'exerçait la garde `WHERE actor_label = ''` là où le rejeu tourne
vraiment. Un test l'a fermé (`full_import_replay_does_not_overwrite_a_local_actor_label`).

## Interdits — ce sont des COMMANDES, pas seulement de l'écriture

⛔ N'exécute **aucune** commande qui écrit dans le dépôt. Nommément interdits :
`scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`,
`scripts/install-hooks.sh`, tout `git add`/`commit`/`checkout`/`stash`/`restore`,
tout `sed -i`, tout `cargo fmt` sans `--check`, `make` dans `docs/manual/`, toute
commande `sqlx migrate`. Lecture, `grep` et `cargo nextest run` sont permis.
⛔ Ne modifie aucun fichier. Ton livrable est un **rapport**.

## Discipline de vérification — non négociable

Avant de déclarer un finding affirmant l'**absence** d'un code ou d'un texte
attendu :

1. **Nomme la base de comparaison et justifie-la.** Une story se construit en
   plusieurs commits ; l'état livré est `HEAD`, et le diff de la story est
   `git diff main...HEAD`. Comparer un fichier à un commit intermédiaire ne prouve
   rien sur ce qui a été livré. *(C'est l'erreur exacte de la passe 2.)*
2. **Vérifie que l'état de départ de ton scénario est ATTEIGNABLE** par un chemin
   réel du code. Un scénario qui suppose un état qu'aucun chemin ne produit n'est
   pas un défaut. *(C'est l'erreur exacte du HIGH de la passe 2.)*
3. **Cite l'extrait** obtenu par `grep -nF` ou lecture directe.

## Axes — déclare pour chacun « exercé » ou « non exercé »

Un « 0 finding » sans cette liste ne compte pas comme passe.

1. **Le test neuf de la passe 2** — `full_import_replay_does_not_overwrite_a_local_actor_label`.
   Mesure-t-il ce qu'il prétend ? Son montage atteint-il l'état visé ? Existe-t-il
   une mutation plausible du code de production qu'il ne tuerait pas ?
   Les **trois** cas `actor_label` couvrent-ils ensemble le produit
   {sentinelle présente / absente} × {libellé local convergent / divergent} ?
2. **⚠️ LES MANUELS ET LEURS PDF — l'axe que la passe 2 a manqué.**
   Compare `git diff main...HEAD -- docs/manual/` au comportement réellement
   livré. Six sites ont été amendés au volet B, un septième à la passe 2.
   Reste-t-il, dans les `.tex` **ou dans les PDF**, une affirmation que le code
   ne tient pas — inaltérabilité, conformité, exhaustivité du journal ?
   Aplatis avant de greper : `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`.
   ⚠️ **Calibrage** : le Project Lead a tranché qu'il ne faut **pas** viser une
   application stricte de l'OLICo — très peu de logiciels comptables la suivent
   strictement. Ce qu'on traque est une **promesse fausse**, pas une couverture
   réglementaire incomplète. Ne produis pas de findings réclamant plus de
   conformité ; produis-en sur ce que le texte **affirme** et que le code dément.
3. **`website/` et `README.md`** — mêmes affirmations, autre support. Ont-ils été
   traités ? Le site est publié automatiquement au push sur `main`.
4. **La cohérence du Change Log de la passe 2** avec ce qui a réellement été fait.
   Ses décomptes sont-ils vrais ? Affirme-t-il quelque chose qui n'a pas tourné ?
5. **Le rayon d'impact non relu** : `crates/kesh-db/src/backup.rs` (la fusion),
   `entities/audit_log.rs`, `repositories/audit_log.rs`, `crates/kesh-api/src/lib.rs`
   (le déplacement de route). Ces fichiers ont été relus en passe 1 mais pas en
   passe 2. Un regard neuf sur la **conception**, pas seulement sur les patches.
6. **La fusion elle-même, côté cas limites** : que se passe-t-il si le backup
   porte une entrée d'audit dont le `user_id` ne correspond à aucun utilisateur
   après restore ? Si `audit_log` est vide d'un côté ou de l'autre ? Si le même
   `id` collisionne ? Cherche le cas que les tests ne couvrent pas.
7. **Les garde-fous P5/P6/P7/P8** de la migration, revérifiés depuis la source
   plutôt que depuis les déclarations de la story.

## Livrable

Findings par sévérité, chacun avec fichier:ligne, le défaut, le **scénario
d'échec concret** (entrées → sortie fausse), la base de comparaison employée, et
la vérification citée. Plus la liste des axes exercés et non exercés.
