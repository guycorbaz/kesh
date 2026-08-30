# Prompt versionné — Story 24-4c, passe 3 de `validate` (passe CIBLÉE)

**Date** : 2026-08-30 · **Modèle** : Sonnet 4.6, contexte frais · **Périmètre** : le seul commit
`59a165e6` (la remédiation de la passe 2) · **Lentille unique** : chasseur de régressions.

## Où en est la boucle

| passe | modèle | CRIT | HIGH | MED | LOW | total | nés d'une remédiation |
|---|---|---|---|---|---|---|---|
| 1 | Sonnet 4.6 + Haiku 4.5 | 2 | 1 | 3 | 2 | **8** | — |
| 2 | Opus 5 *(ciblée)* | 0 | 4 | 6 | 3 | **13** | **13 / 13** |
| 3 | Sonnet 4.6 *(ciblée)* | — | — | — | — | — | — |

⚠️ **Le volume MONTE pendant que la sévérité BAISSE** (8 → 13, CRITICAL → HIGH). C'est le signe
d'une spec qui se précise, non d'une conception qui vacille : aucune décision de conception n'a
été prise en défaut en deux passes.

⛔ **Ce que la passe 2 a trouvé dit où chercher.** Deux de ses HIGH étaient des **contradictions
internes au patch lui-même** : trois nombres neufs qui ne se recoupaient pas dans un seul
paragraphe, et une correction écrite douze lignes sous la ligne qu'elle réfutait sans la
toucher. C'est là que le prochain défaut se logera.

## Le prompt, tel qu'il a été donné

> Tu es une lentille de revue adversariale **chasseuse de régressions**, en contexte frais, sur
> le dépôt Kesh (`/home/gcorbaz/devel/kesh`, comptabilité suisse — Rust/Axum, SvelteKit,
> MariaDB). Réponds en FRANÇAIS.
>
> **Ton périmètre est UN SEUL COMMIT** : `59a165e6`, la remédiation de la passe 2 de revue de la
> spécification `_bmad-output/implementation-artifacts/24-4c-verrou-de-periode.md` (Story 24-4c,
> « le verrou de période »). Lis-le avec `git show 59a165e6`.
>
> **Ta question n'est pas « la spec est-elle bonne ? » mais « ce patch a-t-il cassé quelque
> chose, ou introduit un défaut neuf ? »** Sur cette story, la passe 2 a rendu treize findings et
> les **treize** portaient sur le patch de la passe 1. Le patch que tu relis est celui qui les
> corrige.
>
> **Ce que fait la story** : la société porte une date `books_locked_through` ; toute écriture
> dont la date lui est antérieure ou égale est refusée à la **création** (400 `PERIOD_LOCKED`).
>
> **Ce que la passe 2 avait trouvé, et que ce patch prétend corriger** :
>
> 1. HIGH — « tous passent par `create_in_tx` » est faux : `reverse` appelle
>    `create_in_tx_inner` directement (`:1385`). Le point de passage réel est l'inner.
> 2. HIGH — « treize sites » en vaut **douze** ; et le paragraphe disait « neuf » puis « dix ».
> 3. HIGH — la flèche de précédence de D4 était **inversée** par rapport au code
>    (`FiscalYearClosed` `:263` avant `DateOutsideFiscalYear` `:274`).
> 4. HIGH — l'AC 14 exigeait « l'ancienne valeur » de la borne alors que `companies` est vidée
>    avant le seul point où l'audit survit.
> 5. MEDIUM — `DATE_OUTSIDE_FISCAL_YEAR` est inatteignable par la route ; c'est `NO_FISCAL_YEAR`.
> 6. MEDIUM — l'entrée d'audit de l'import aurait été signée par un Admin **de l'archive**.
> 7. MEDIUM — deux résidus du décompte, dont la tâche T2 (« les 22 chemins »).
> 8. MEDIUM — l'AC 14 n'avait ni fichier ni test.
> 9. MEDIUM — la garde neuve de l'AC 3 n'avait aucun test nommé.
> 10. MEDIUM — « seize fonctions d'export » : il y en a **dix-neuf**.
> 11. LOW — le « non nulle » perdu entre D3 et l'AC 6.
> 12. LOW — `< today` ne disait pas quelle horloge.
> 13. LOW — I3 omettait `reset_demo`.
>
> **Cherche, dans cet ordre** :
>
> - **Un jumeau non corrigé, ou une contradiction interne au patch.** C'est ce que la passe 2 a
>   trouvé deux fois. Pour **chaque nombre, chaque nom de fonction, chaque code d'erreur, chaque
>   nom d'action d'audit** que ce patch modifie, cherche **toutes** ses autres occurrences dans
>   le document et vérifie qu'elles ont suivi. Cherche le symptôme, pas le site.
> - **⛔ Le tableau « Fichiers à toucher » CONTRE les AC et les tâches.** La passe 2 l'a trouvé
>   désynchronisé. Chaque AC exige-t-elle un fichier qui y figure ? Chaque fichier du tableau
>   est-il justifié par une AC ou une tâche ? Chaque AC a-t-elle un test nommé en T6 ?
> - **Une correction FAUSSE.** Vérifie au sol : `create_in_tx_inner` et ses appelants,
>   `journal_entries.rs:263` et `:274`, `routes/journal_entries.rs:512-524`, `NO_FISCAL_YEAR`
>   dans `errors.rs`, `backup.rs:76` et `restore_tables_in_tx`, `ActorType`, le `MIN(id)` des
>   admins dans `routes/admin.rs`, `onboarding.rs:256-278`, les 19 fonctions d'export, le
>   décompte « douze sites », les compteurs de migrations (64→65, `N` 30→31, frontière 34).
> - **Une garde neuve non exercée.** Le patch ajoute des tests nommés en T6 — couvrent-ils
>   vraiment chaque garde neuve, et un test manquant rendrait-il la garde muette ?
> - **Ce que le patch a AJOUTÉ et que personne n'a relu** : le journal de la passe 2 lui-même,
>   dont les décomptes se recomptent, et le cas « archive d'une autre installation » laissé « à
>   trancher à l'implémentation » — est-ce une décision reportée légitime, ou un trou ?
>
> ⛔ **Aucune affirmation sans vérification au sol, et la vérification doit SORTIR DU DOCUMENT.**
> `grep -nF` / `sed -n` / lecture du **code**, commande et résultat cités. ⚠️ En passe 1, une
> lentille a rendu zéro finding en vérifiant la spec contre elle-même : citer l'objectif comme
> preuve qu'il est atteint ne compte pas.
>
> **Format** : `[SÉVÉRITÉ] identifiant — titre`, le défaut, la vérification au sol, la
> conséquence, le correctif en une ou deux phrases. CRITICAL / HIGH / MEDIUM / LOW. Termine par
> le tableau des comptes, **combien de tes findings portent sur le patch de la passe 2**, et un
> verdict sur la clôture. **« Zéro finding » est acceptable si c'est vrai et vérifié.**

## Critère de clôture

L'objet revu est une **spécification** : la boucle peut se clore si la remédiation ne touche plus
qu'à des relevés, et non à une décision de conception ni au **lieu où le code va s'écrire**.
Deux passes de suite ont pourtant déplacé ce lieu (P2-1 : le point de passage ; P2-5 : le moment
de la lecture de la borne). Tant que c'est le cas, la boucle continue.
