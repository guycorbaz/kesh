# Prompt versionné — Story 24-4c, passe 2 de `bmad-code-review` (passe CIBLÉE)

**Date** : 2026-08-30 · **Modèle** : Opus 5, contexte frais · **Périmètre** : le seul commit
`f5c71403` (la remédiation de la passe 1 de revue de **code**) · **Lentille unique** : chasseur
de régressions.

⚠️ **Ne pas confondre avec les prompts `24-4c-review-prompt-*`** : ceux-là portaient la revue de
la **spécification** (4 passes, closes). Celui-ci porte la revue du **code**.

## Où en est la boucle de revue de code

| passe | modèle | CRIT | HIGH | MED | LOW | total |
|---|---|---|---|---|---|---|
| 1 | Sonnet 4.6 + Haiku 4.5 | 2 | 1 | 1 | 2 | **6** |
| 2 | Opus 5 *(ciblée)* | — | — | — | — | — |

⛔ **Le finding le plus grave de la passe 1 ne vivait dans AUCUN fichier que la story touchait** :
`DbError::PeriodLocked`, variant neuf, tombait dans le repli `_` du pattern batch de
`reconciliation.rs` et sortait en `DATABASE_ERROR`. *Introduire un variant d'erreur change le
comportement de tous les `match` qui ne le nomment pas.* **C'est la piste à pousser plus loin :
ce `match`-là est-il le seul ?**

⛔ **Second motif de la passe 1** : `lock_books` portait une garde de date, son jumeau
`unlock_books` ne l'avait pas. *Une garde posée sur un chemin et pas sur son symétrique.*

## Le prompt, tel qu'il a été donné

> Tu es une lentille de revue adversariale **chasseuse de régressions**, en contexte frais, sur
> le dépôt Kesh (`/home/gcorbaz/devel/kesh`, comptabilité suisse — Rust/Axum, SvelteKit,
> MariaDB). Réponds en FRANÇAIS.
>
> **Ton périmètre est UN SEUL COMMIT** : `f5c71403`, la remédiation de la passe 1 de revue de
> code de la Story 24-4c (« le verrou de période », #380). Lis-le avec `git show f5c71403`.
>
> **Ta question n'est pas « le code est-il bon ? » mais « ce patch a-t-il cassé quelque chose,
> ou introduit un défaut neuf ? »** Sur ce dépôt, mesuré sur quatre epics, la remédiation d'une
> passe devient le défaut de la suivante.
>
> **Ce que fait la story** : `companies.books_locked_through` — une date **inclusive**. Toute
> écriture dont la date lui est antérieure ou égale est refusée à la création, en 400
> `PERIOD_LOCKED`. La garde vit dans `create_in_tx_inner`. Poser/avancer : Admin + Comptable.
> Reculer/retirer : Admin seul, motif obligatoire.
>
> **Ce que la passe 1 avait trouvé, et que ce patch prétend corriger** :
>
> 1. CRITICAL — `PeriodLocked` tombait dans le repli `_` du pattern batch → `DATABASE_ERROR` au
>    lieu du code canonique. Et l'AC 4 n'avait aucun test.
> 2. CRITICAL / MEDIUM — l'AC 14 (`books.restored`) sans test, alors que **deux cases de tâches**
>    l'affirmaient faite.
> 3. HIGH — `unlock_books` sans garde de date : un Admin pouvait poser une borne **future** et
>    refuser toute écriture du jour, contre-passation comprise (invariant I2 cassé).
> 4. LOW — un doc-comment déplacé dans deux fichiers.
> 5. LOW — « quatre endroits » alors qu'il y en a cinq.
>
> **Cherche, dans cet ordre** :
>
> - ⛔ **D'AUTRES `match` sur `DbError` que le variant neuf traverse sans être nommé.** La
>   passe 1 en a trouvé deux dans `reconciliation.rs`. `grep -rn "DbError::" crates/ --include=*.rs`
>   puis inspecte **chaque** `match` exhaustif ou avec repli `_` qui peut recevoir une erreur de
>   `create_in_tx_inner` : les routes de factures, d'avoirs, de factures fournisseurs, de
>   règlements, d'ouverture. **Un repli qui étiquette mal est muet.**
> - ⛔ **D'autres gardes posées d'un côté et pas de l'autre.** `lock_books` / `unlock_books` en
>   étaient un cas. Y en a-t-il d'autres — au handler, au frontend (le formulaire de
>   déverrouillage a-t-il un `max` ?), dans les deux chemins de l'import ?
> - **Une correction FAUSSE.** Vérifie au sol chaque affirmation neuve : le bras `PeriodLocked`
>   est-il placé **avant** le repli `_` et **après** les bras plus spécifiques ? Le `if let` de
>   `accept_one_rule` intercepte-t-il bien avant le `return` générique ? La garde d'`unlock_books`
>   traite-t-elle `None` correctement (retirer le verrou doit rester permis) ?
> - **Les tests neufs mesurent-ils ce qu'ils prétendent ?** Cinq ont été ajoutés, dont deux
>   contre-tests. Un contre-test qui passerait aussi sans la correction ne vaut rien : vérifie
>   qu'il rougirait. ⚠️ En particulier :
>   `full_import_does_not_trace_an_advancing_books_lock` et `other_errors_keep_their_mapping`.
> - **Ce que le patch a AJOUTÉ et que personne n'a relu** : le journal de revue du story file,
>   dont les décomptes se recomptent (2270 → 2275, six findings, cinq tests).
>
> ⛔ **Aucune affirmation sans vérification au sol.** `grep -nF` (le `-F` est obligatoire),
> `sed -n`, lecture du code — **commande et résultat cités**. Tu peux exécuter des tests ciblés
> (`cargo nextest run -E 'test(...)'`, MariaDB de dev démarré). ⛔ **Ne lance PAS la suite
> complète ni Playwright.**
>
> **Format** : `[SÉVÉRITÉ] identifiant — titre`, le défaut, la vérification au sol, la
> conséquence, le correctif en une ou deux phrases. CRITICAL / HIGH / MEDIUM / LOW. Termine par
> le tableau des comptes, **combien de tes findings portent sur le patch de la passe 1**, et un
> verdict sur la clôture. **« Zéro finding » est acceptable si c'est vrai et vérifié.**

## Critère de clôture

La § *La passe ciblée* du `CLAUDE.md` permet de clore si la remédiation produite ne touche
**aucune ligne de code de production**. La passe 1 en a touché beaucoup ; c'est donc la passe 2
qui décidera.
