# Prompt versionné — Story 24-4b, passe 4 de `validate` (passe CIBLÉE)

**Date** : 2026-08-29 · **Modèle** : Haiku 4.5, contexte frais · **Périmètre** : le seul commit
`8bc61265` (la remédiation de la passe 3) · **Lentille unique** : chasseur de régressions.

## Où en est la boucle

| passe | modèle | CRIT | HIGH | MED | LOW | total | nés d'une remédiation |
|---|---|---|---|---|---|---|---|
| 1 | Sonnet 4.6 + Haiku 4.5 | 2 | 3 | 4 | 2 | 11 | — |
| 2 | Opus 5 *(ciblée)* | 0 | 3 | 2 | 3 | 8 | **8 / 8** |
| 3 | Sonnet 4.6 *(ciblée)* | 0 | 1 | 1 | 0 | 2 | **2 / 2** |
| 4 | Haiku 4.5 *(ciblée)* | — | — | — | — | — | — |

⚠️ **Trois passes, et pas une seule décision de conception prise en défaut.** D1 (pas de colonne
`status`), D2 (le refus au repository, la précédence), D3, D5 (`invoices::delete` non gelée) et
D7 (le bilan d'ouverture) tiennent toutes. Ce qui cède à chaque passe, ce sont les **relevés**.

⚠️ **Le symptôme récurrent de cette story est le GREP TROP ÉTROIT** — et il s'est manifesté
trois fois : périmètre `frontend/` seul en passe 1, arrêt au premier résultat en passe 2,
périmètre `crates/` seul en passe 3. La remédiation de la passe 3 élargit le scope de la
commande à `.` et ajoute une leçon générale (« grepper large **et** trier à la main »). C'est
cette remédiation-là qu'il faut mettre en doute.

## ⛔ Garde-fou spécifique au modèle

Le `CLAUDE.md` documente un mode d'échec propre à Haiku 4.5 : affirmer `CRITICAL` ou `HIGH`
qu'un code est **absent** alors qu'il est présent, par mauvaise indexation des numéros de ligne
d'un diff. Le prompt impose donc, pour toute affirmation d'absence ou de présence d'un motif,
une vérification par `grep -nF` (`-F` obligatoire : le code Rust et TypeScript est plein de
métacaractères) avec commande et résultat cités. Un finding sans vérification citée est écarté
sans être lu.

## Le prompt, tel qu'il a été donné

> Tu es une lentille de revue adversariale **chasseuse de régressions**, en contexte frais, sur
> le dépôt Kesh (`/home/gcorbaz/devel/kesh`, comptabilité suisse — Rust/Axum, SvelteKit,
> MariaDB). Réponds en FRANÇAIS.
>
> **Ton périmètre est UN SEUL COMMIT** : `8bc61265`, la remédiation de la passe 3 de revue de la
> spécification `_bmad-output/implementation-artifacts/24-4b-gel-ecriture.md`. Lis-le avec
> `git show 8bc61265`.
>
> **Ta question n'est pas « la spec est-elle bonne ? » mais « ce patch a-t-il cassé quelque
> chose, ou introduit un défaut neuf ? »** Sur cette story, les passes 2 et 3 ont rendu dix
> findings et les **dix** portaient sur le patch de la passe précédente. Le patch que tu relis
> est celui qui corrige la passe 3 : c'est donc là que se trouve, statistiquement, le prochain
> défaut.
>
> **Ce que la passe 3 avait trouvé, et que ce patch prétend corriger** :
>
> 1. HIGH — le grep de `journal_entries::update` était resté scopé `crates/` ; un quatrième site
>    de code vit dans `frontend/tests/e2e/fiscal-years.spec.ts:114`.
> 2. MEDIUM — `docs/optimistic-locking-patterns.md` (`:49`, `:75`) décrira une fonction
>    supprimée.
>
> **Ce que le patch a ajouté, et que personne n'a encore relu** :
>
> - l'AC 9 passe de « trois » à « **quatre** » sites, et affirme que les neuf tests de
>   `fiscal-years.spec.ts` **survivent** parce qu'ils passent par « Nouvelle écriture » ;
> - la commande de D2(b) devient `grep -rn "journal_entries::update" .` sur le dépôt entier ;
> - un piège neuf déclare **six** story files historiques de `_bmad-output/` **intouchables** ;
> - deux lignes s'ajoutent à la table des fichiers ;
> - le journal de la passe 3, dont les décomptes se recomptent.
>
> **Cherche, dans cet ordre** :
>
> - **Un décompte faux.** « quatre » sites de code, « six » story files historiques, « neuf »
>   tests dans `fiscal-years.spec.ts`, les totaux du journal (11, 8, 2). Recompte **chacun**
>   depuis la source.
> - **Une affirmation fausse.** Les neuf tests de `fiscal-years.spec.ts` passent-ils VRAIMENT
>   tous par « Nouvelle écriture » et jamais par l'édition ni la suppression ? C'est
>   l'affirmation la plus risquée du patch : si un seul test emprunte le chemin gelé, la story
>   casse une suite qu'elle ne liste pas.
> - **Un jumeau non corrigé.** Le patch corrige « trois » en « quatre » : toutes les autres
>   occurrences de ce nombre, et toutes les autres listes de ces sites, ont-elles suivi ?
> - **Une sur-correction.** Déclarer les story files de `_bmad-output/` intouchables est-il juste
>   ? Y en a-t-il un qui, au contraire, devrait être corrigé ? Et le compte de six est-il exact ?
> - **Un site encore manquant.** Refais toi-même le grep sur le dépôt entier, **sans filtre de
>   type de fichier**, et compare à ce que la spec liste.
>
> ⛔ **RÈGLE ABSOLUE — vérification au sol avant toute affirmation.** Avant d'écrire « X manque »,
> « Y est faux », « ce site n'est pas listé », tu DOIS l'établir par `grep -nF "<chaîne exacte>"`
> (le drapeau `-F` est **obligatoire**), `sed -n` ou lecture directe, et **citer la commande et
> son résultat**. Un finding sans vérification citée sera écarté sans être lu.
>
> **Format** : `[SÉVÉRITÉ] identifiant — titre`, puis ce qui est faux, la vérification au sol
> (commande + extrait), la conséquence, le correctif en une phrase. Sévérités CRITICAL / HIGH /
> MEDIUM / LOW. Termine par un tableau des comptes, en disant **combien de tes findings portent
> sur le patch de la passe 3**, et par un verdict sur la clôture de la boucle. **« Zéro
> finding » est une réponse acceptable et attendue si elle est vraie et vérifiée** — n'invente
> rien pour remplir.

## Critère de clôture

L'objet revu est une **spécification** : la question de clôture est de savoir si la remédiation
touche une **décision de conception** ou seulement des relevés. Trois passes de suite n'ont
trouvé que des relevés, et le volume s'effondre (11 → 8 → 2). Si la passe 4 ne trouve rien
au-dessus de LOW, la boucle se clôt et la story part en `dev-story`.
