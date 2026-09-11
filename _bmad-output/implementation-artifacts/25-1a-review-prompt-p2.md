# Story 25-1a — prompt de la passe 2 de `bmad-code-review`

**Modèle** : Haiku 4.5 (rotation — la passe 1 était Sonnet 4.6 / Haiku 4.5 / Sonnet 4.6
sur trois lentilles ; cette passe est une **lentille unique**).
**Contexte** : frais. Ne PAS réutiliser celui de la passe 1.
**Diff à relire** : `git show 9dfbb527` — le **seul commit de remédiation** de la passe 1.

## Pourquoi une lentille unique braquée sur ce commit

La boucle converge : 0 CRITICAL, 0 HIGH en passe 1, et tous les MEDIUM portaient
sur des trous de couverture ou des affirmations périmées, aucun sur la conception.
Le motif mesuré du dépôt est que **la sévérité ne stagne pas, elle se déplace vers
ce qu'on vient d'écrire** : sur les huit passes cumulées des Stories 23-1a et 23-1b,
sept ont trouvé une régression du patch précédent et aucune un défaut d'origine.

⚠️ **Mais la remédiation de la passe 1 touche du CODE DE PRODUCTION** —
`backup.rs` (un test neuf y vit, mais le fichier est de production),
`post_restore.rs` et son extrait SQL. **La boucle ne peut donc pas être close
après cette passe** si elle trouve quoi que ce soit au-dessus de LOW.

## Interdits — et ce sont des COMMANDES, pas seulement de l'écriture

⛔ N'exécute **aucune** commande qui écrit dans le dépôt. Nommément interdits :
`scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`,
`scripts/install-hooks.sh`, tout `git add` / `commit` / `checkout` / `stash`,
tout `sed -i`, tout `cargo fmt` sans `--check`, toute migration `sqlx`.
⛔ Ne modifie aucun fichier. Ton livrable est un **rapport**, pas un patch.

*(Motif : passe 4 de la Story 24-5. Le prompt disait « n'écris aucun fichier » ;
la lentille a lancé `prepare-release.sh`, qui a bumpé les dix crates du workspace.)*

## Axes à exercer — et le rapport DOIT dire lesquels l'ont été

Un « 0 finding » non adossé à la liste des axes réellement exercés **ne compte pas
comme passe**. Déclare explicitement, pour chacun : *exercé* ou *non exercé*.

1. **Les deux tests neufs mesurent-ils ce qu'ils prétendent ?**
   `full_import_replays_actor_label_when_column_is_absent` et
   `full_import_preserves_archived_actor_label_when_column_is_present`.
   Cherche le **test muet** : une assertion qui passerait aussi sur du code cassé,
   un montage qui n'atteint pas l'état visé, un `COUNT(*) = 0` vrai par vacuité.
   Les deux mutations rapportées au Change Log sont-elles les bonnes ? En
   existe-t-il une **troisième**, plausible, qu'aucun des deux ne tuerait ?

2. **Le garde-fou de largeur** `actor_label_is_at_least_as_wide_as_username`.
   Il lit `information_schema` sous `#[sqlx::test(migrations = "./test-schema")]`,
   donc il contrôle le **squash**, pas les migrations réelles. Est-ce un angle
   mort ? Le squash peut-il diverger du schéma réel sur ce point précis sans que
   `test_schema_guard.rs` ne rougisse ?

3. **L'en-tête neuf de l'extrait `post_restore/20260910000001_*.sql`.**
   Il affirme deux choses : (a) le rejeu tourne **après** le remplacement de
   `users`, donc la jointure résout dans le bon espace d'identifiants ;
   (b) la garde `actor_label = ''` n'atteint que les lignes d'archive.
   **Vérifie (a) dans le code** (`routes/admin.rs`), et cherche un cas où (b)
   serait faux — une ligne LOCALE dont `actor_label` vaudrait `''`.

4. **Les décomptes réécrits de `post_restore.rs`.** La passe 1 a retiré deux
   totaux périmés (« neuf », « sept ») plutôt que de les corriger. Reste-t-il,
   **ailleurs dans le dépôt**, un décompte du même objet ? Recompte depuis la
   source : `POST_RESTORE_BACKFILLS`, `RETIRED_BACKFILLS`, `EXEMPT_MIGRATIONS`.

5. **Le compteur i18n `sitesTotal: 1631`** et sa ventilation. Le raisonnement
   écrit est qu'un `{#if}` n'ajoute aucun site. Est-il exact au regard de ce que
   l'extracteur compte réellement ? Les quatre locales portent-elles la clé ?

6. **⚠️ LE MANUEL, ET SON PDF.** La story 25-1a retire des promesses
   d'inaltérabilité et change qui peut réinitialiser une instance de démo.
   Vérifie `docs/manual/fr/{user,admin}-manual.tex` **et les PDF** contre le code
   livré. Aplatis avant de greper :
   `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`.
   C'est l'axe au meilleur rendement du dépôt : sur l'Epic 24, c'est en vérifiant
   si le manuel disait vrai qu'une lentille a trouvé un quatrième chemin
   d'écriture qu'aucune passe n'avait énuméré.

7. **Le Dev Agent Record et le Change Log**, écrits à cette passe. Leurs
   décomptes sont-ils vrais ? Leur **périmètre** est-il déclaré ? Affirment-ils
   quelque chose qui n'a pas tourné ?

## Discipline de vérification — obligatoire avant de déclarer un finding

Pour tout finding CRITICAL ou HIGH affirmant l'**absence** d'un code attendu ou la
**présence** d'un anti-pattern : vérifie par `grep -nF` (le `-F` est obligatoire) ou
par lecture directe du fichier, et **cite l'extrait** dans le rapport. Une
affirmation non vérifiée sera traitée comme un faux positif.

## Livrable

Un rapport : findings par sévérité, chacun avec fichier:ligne, le défaut, le
**scénario d'échec concret** (entrées → sortie fausse), et la vérification qui
l'établit. Plus la liste des axes exercés et non exercés.
