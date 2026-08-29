# Prompt versionné — Story 24-4b, passe 3 de `validate` (passe CIBLÉE)

**Date** : 2026-08-29 · **Modèle** : Sonnet 4.6, contexte frais · **Périmètre** : le seul commit
`d32a2d08` (la remédiation de la passe 2) · **Lentille unique** : chasseur de régressions.

## Où en est la boucle

| passe | modèle | CRIT | HIGH | MED | LOW | nés d'une remédiation |
|---|---|---|---|---|---|---|
| 1 | Sonnet 4.6 + Haiku 4.5 | 2 | 3 | 4 | 2 | — |
| 2 | Opus 5 *(ciblée)* | 0 | 3 | 2 | 3 | **8 / 8** |
| 3 | Sonnet 4.6 *(ciblée)* | — | — | — | — | — |

⚠️ **Deux passes, et pas une seule décision de conception prise en défaut** — D1 (pas de colonne
`status`), D2 (le refus au repository, la précédence des refus), D3, D5 (`invoices::delete` non
gelée), D7 (le bilan d'ouverture) tiennent toutes. Ce qui cède à chaque passe, ce sont les
**relevés** : clés i18n, passages de manuel, décomptes, numéros de ligne, portée des greps.

⚠️ **Et le motif s'est reproduit DANS sa propre remédiation en passe 2** : le patch de la passe 1
avait corrigé les décomptes là où on les lui avait nommés, laissé leurs jumeaux dans la table
des fichiers, produit trois décomptes neufs faux, et arrêté au premier résultat le grep dont son
message de commit proclamait l'avoir refait en grand. C'est exactement ce qu'il faut chercher
une fois de plus.

⚠️ La rotation ramène Sonnet, qui a tenu une lentille en passe 1 — mais en **contexte frais**,
donc sans mémoire de ses propres findings. C'est la condition posée par la § *Review Iteration
Rule* du `CLAUDE.md`, et elle est respectée.

## Le prompt, tel qu'il a été donné

> Tu es une lentille de revue adversariale **chasseuse de régressions**, en contexte frais, sur
> le dépôt Kesh (`/home/gcorbaz/devel/kesh`, comptabilité suisse — Rust/Axum, SvelteKit,
> MariaDB). Réponds en FRANÇAIS.
>
> **Ton périmètre est UN SEUL COMMIT** : `d32a2d08`, la remédiation de la passe 2 de revue de la
> spécification `_bmad-output/implementation-artifacts/24-4b-gel-ecriture.md`. Lis-le avec
> `git show d32a2d08`.
>
> **Ta question n'est pas « la spec est-elle bonne ? » mais « ce patch a-t-il cassé quelque
> chose, ou introduit un défaut neuf ? »** Sur cette story, la passe 2 a rendu huit findings et
> les **huit** portaient sur le patch de la passe 1. Le patch que tu relis est celui qui les
> corrige : c'est donc là, statistiquement, que se trouve le prochain défaut.
>
> **Ce que la passe 2 avait trouvé, et que ce patch prétend corriger** :
>
> 1. HIGH — `reversal_blocker` cité à `journal_entries.rs:1449`, au milieu d'un littéral SQL, au lieu de `:1416` ; deux sites.
> 2. HIGH — le grep de `journal_entries::update` s'était arrêté au premier résultat : trois sites manquaient (`accounts.rs:444`, `admin_full_import_e2e.rs:1354`, et ⛔ `crates/kesh-db/migrations/20260729000001_invoice_lines_revenue_account_backfill.sql:54`, **migration publiée depuis v0.9.0**, dont toute retouche casse le checksum et empêche le boot — garde-fou P8).
> 3. HIGH — trois décomptes corrigés dans les AC et restés faux dans la **table des fichiers** (« 7 clés », « six passages » deux fois).
> 4. MEDIUM — « NEUF autres occurrences » pour un tableau qui en listait sept ; `:1585` n'est pas un hit du grep, `:1594` l'est.
> 5. MEDIUM — le journal de la passe 1 annonçait « 3 MEDIUM » et dix findings pour un tableau de onze.
> 6. LOW — « la troisième puce » de la FAQ est la deuxième.
> 7. LOW — aucune tâche ne portait l'ouverture de l'issue de D5.
> 8. LOW — l'invariant I3, rétréci, ne nommait plus qui reste enfermé après le gel.
>
> **Cherche, dans cet ordre** :
>
> - **Une correction qui en casse une autre**, ou qui laisse un jumeau non corrigé ailleurs dans
>   le document. C'est le défaut qui s'est produit deux fois de suite sur cette story : vérifie
>   que chaque nombre, chaque pointeur, chaque liste modifiés par ce patch sont cohérents avec
>   **toutes** leurs autres occurrences. Cherche le symptôme, pas le site.
> - **Une correction FAUSSE.** Vérifie au sol chaque affirmation neuve : `journal_entries.rs:1416`,
>   les trois sites de `journal_entries::update` ajoutés à la table des fichiers, la migration
>   `20260729000001…sql:54` et sa publication (`git tag --contains`), « SEPT autres occurrences »,
>   « quinze lignes » rendues par le grep du manuel, `:454` et `:1594` comme lignes de
>   continuation, « onze clés », « sept passages », les décomptes des DEUX journaux de revue.
> - **Une AC devenue incohérente.** L'AC 9 a été amendée pour exclure les migrations : l'exclusion
>   est-elle formulée de façon qu'un implémenteur ne puisse pas se tromper ? N'a-t-elle pas
>   ouvert un trou ailleurs (par exemple : un doc-comment de migration qui, lui, DEVRAIT être
>   corrigé) ?
> - **Ce que le patch a AJOUTÉ et que personne n'a relu** : le paragraphe P8 des pièges, le
>   paragraphe « qui reste enfermé » des hors-périmètre, la tâche d'ouverture d'issue, et le
>   journal de la passe 2 lui-même — dont les décomptes se recomptent comme les autres.
> - **Une sur-correction.** Le patch a-t-il retiré ou affaibli quelque chose de juste ? En
>   particulier : le tableau des occurrences « intouchables » du manuel a-t-il été rétréci au
>   point de laisser un passage faux ?
>
> ⛔ **Aucune affirmation sans vérification au sol.** Avant d'écrire « X est faux », « Y manque »,
> « ce numéro de ligne est erroné », établis-le par `grep -nF` / `sed -n` / lecture directe, et
> **cite la commande et son résultat**. Un finding non vérifié coûte plus cher que le défaut
> qu'il prétend signaler.
>
> **Format** : `[SÉVÉRITÉ] identifiant — titre`, puis ce qui est faux ou manquant, la
> vérification au sol (commande + extrait), la conséquence pour l'implémenteur, le correctif en
> une ou deux phrases. Sévérités CRITICAL / HIGH / MEDIUM / LOW. Termine par un tableau des
> comptes, en disant explicitement **combien de tes findings portent sur le patch de la passe 2**
> plutôt que sur la conception d'origine, et par une phrase de verdict sur la clôture de la
> boucle. Si le patch est propre, dis-le — « zéro finding » est une réponse acceptable si elle
> est vraie et vérifiée.

## Critère de clôture

La § *La passe ciblée* du `CLAUDE.md` permet de clore la boucle si la remédiation produite ne
touche **aucune ligne de code de production**. L'objet revu étant une **spécification**, la
question équivalente est : la remédiation touche-t-elle une **décision de conception**, ou
seulement des relevés ? Deux passes de suite n'ont trouvé que des relevés. Si la passe 3 retombe
sous MEDIUM sans entamer une décision, la boucle peut se clore avant les huit passes.
