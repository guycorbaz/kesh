# Prompt versionné — Story 24-4c, passe 4 de `bmad-code-review` (passe CIBLÉE)

**Date** : 2026-08-30 · **Modèle** : Haiku 4.5, contexte frais · **Périmètre** : le seul commit
`4b15f9e9` (la remédiation de la passe 3) · **Lentille unique** : contrôle mécanique et
vérification par mutation.

## Où en est la boucle de revue de code

| passe | modèle | CRIT | HIGH | MED | LOW | total |
|---|---|---|---|---|---|---|
| 1 | Sonnet 4.6 + Haiku 4.5 | 2 | 1 | 1 | 2 | **6** |
| 2 | Opus 5 *(ciblée)* | 0 | 1 | 3 | 3 | **7** |
| 3 | Sonnet 4.6 *(ciblée)* | 0 | 0 | 1 | 0 | **1** |
| 4 | Haiku 4.5 *(ciblée)* | — | — | — | — | — |

⛔ **Le motif de cette story est identifié et il est unique** : *corriger au site nommé, laisser
le jumeau ailleurs.* Cinq occurrences — dont la dernière attrapée non par une lentille mais par
un **geste systématique** (greper chaque correction dans les tests avant d'écrire le journal).

⚠️ **La garde du commit à relire ferme le DERNIER angle** d'une exigence posée en passe 2 :
`books.unlocked` doit avoir **un seul producteur**. Les passes 2 et 3 en ont fermé deux angles
(avancer par la levée ; poser un premier verrou par la levée). **Y en a-t-il un troisième ?**

## Pourquoi ce prompt est MÉCANIQUE et exige des MUTATIONS

⚠️ En passe 1 de la revue de **spécification**, cette même lentille a rendu zéro finding en
vérifiant le document **contre lui-même** — deux CRITICAL lui ont échappé. Le prompt ne lui
demande donc **aucun jugement de conception** : des commandes à exécuter, des mutations à
appliquer puis annuler, et le rapport de ce qui rougit.

⚠️ Les passes 2 et 3 ont toutes deux employé la **mutation** — retirer une garde, jouer le test,
restaurer — et cela a révélé un contre-test inutile puis confirmé quatre gardes. **C'est la
technique la plus productive de cette boucle, et elle est demandée explicitement.**

## Le prompt, tel qu'il a été donné

> Tu es une lentille de **contrôle mécanique et de vérification par mutation**, en contexte
> frais, sur le dépôt Kesh (`/home/gcorbaz/devel/kesh`). Réponds en FRANÇAIS.
>
> **Le commit à relire** : `4b15f9e9` — `git show 4b15f9e9`.
>
> **Ce qu'il fait** : il ajoute à `unlock_books` (`crates/kesh-db/src/repositories/companies.rs`)
> une garde refusant de **poser un premier verrou par l'endpoint de levée** (`before` est `None`
> et `through` est `Some`), plus son test
> `on_ne_pose_pas_un_premier_verrou_par_la_levee`.
>
> **Contexte** : `companies.books_locked_through` est une borne **inclusive** ; aucune écriture ne
> peut être créée à une date antérieure ou égale. `lock_books` pose/avance (Admin + Comptable),
> `unlock_books` recule/retire (Admin seul, motif obligatoire). L'audit distingue trois verbes :
> `books.locked`, `books.unlocked`, `books.restored`.
>
> ⛔ **Ta mission n'est pas de juger la conception.** Elle est d'**exécuter**, et de rapporter
> ce qui rougit.
>
> ### A. Les mutations — applique chacune, joue le test, **restaure**, et rapporte
>
> Pour chaque garde ci-dessous : commente-la, lance le test nommé, note s'il rougit, puis
> **restaure le fichier** et vérifie `git status --porcelain` vide.
>
> | garde à muter | fichier | test attendu rouge |
> |---|---|---|
> | `before.is_none() && through.is_some()` | `companies.rs` | `on_ne_pose_pas_un_premier_verrou_par_la_levee` |
> | `vise > avant` (avancée par la levée) | `companies.rs` | `le_deverrouillage_ne_peut_pas_avancer_la_borne` |
> | `d >= Utc::now()` dans `unlock_books` | `companies.rs` | `le_deverrouillage_refuse_une_borne_future` |
> | `motif.trim().is_empty()` | `companies.rs` | `le_deverrouillage_exige_un_motif_non_blanc` |
>
> ⛔ **Une garde dont la mutation ne fait rougir aucun test est une garde que rien ne protège.**
> C'est le finding le plus utile que tu puisses rendre.
>
> ### B. Le troisième angle — `books.unlocked` a-t-il vraiment UN SEUL producteur ?
>
> ```sh
> grep -rnF '"books.unlocked"' crates/
> grep -rnF '"books.locked"' crates/
> grep -rnF '"books.restored"' crates/
> ```
> Pour **chaque** site, dis quelle condition y mène et si un test l'exerce. Existe-t-il un
> chemin où `unlock_books` réussit en n'ayant **rien** reculé — par exemple `before == through` ?
>
> ### C. Les jumeaux
>
> Pour chaque constante, garde, message et nom de fonction que ce commit touche, cherche
> **toutes** ses occurrences (`grep -rnF`) et dis si elles ont suivi. C'est le motif récurrent
> de cette story.
>
> ### D. Les décomptes du journal
>
> `2280 → 2281`, « un seul finding », « cinq occurrences du motif ». Recompte depuis la source.
>
> ⛔ **Aucune affirmation sans commande citée et sa sortie.** ⛔ **Ne lance PAS la suite complète
> ni Playwright.** ⛔ **Restaure tout ce que tu mutes**, et dis-le.
>
> **Format** : `[SÉVÉRITÉ] identifiant — titre`, la commande, sa sortie, la conséquence, le
> correctif en une phrase. CRITICAL / HIGH / MEDIUM / LOW. Termine par le tableau des mutations
> (garde → rouge ? oui/non), le compte par sévérité, et la réponse à : *une garde de ce commit
> est-elle non protégée par un test ?* **« Aucun écart » est acceptable si c'est vrai et
> vérifié.**

## Critère de clôture

Si la passe 4 ne trouve aucune garde non protégée et aucun jumeau, la boucle se clôt : la
sévérité sera retombée de CRITICAL à rien en quatre passes, et le volume de 6 → 7 → 1 → 0.
