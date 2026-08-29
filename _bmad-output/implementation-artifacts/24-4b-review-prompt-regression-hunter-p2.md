# Prompt versionné — Story 24-4b, passe 2 de `validate` (passe CIBLÉE)

**Date** : 2026-08-29 · **Modèle** : Opus 5, contexte frais · **Périmètre** : le seul commit
`fae6471a` (la remédiation de la passe 1) · **Lentille unique** : chasseur de régressions.

## Pourquoi cette forme, et pourquoi elle est légitime

La § *La passe ciblée* du `CLAUDE.md` autorise à remplacer le protocole à trois lentilles sur
le périmètre complet par **une seule lentille braquée sur la dernière remédiation**, quand la
boucle converge. Le motif est mesuré : sur l'Epic 22, deux CRITICAL de la 22-2 ont été
**introduits par une remédiation** ; sur l'Epic 23, sept passes sur huit ont trouvé une
régression du patch précédent et **aucune** un défaut de la conception d'origine ; sur la
24-4a, quinze findings sur vingt-cinq étaient nés d'une remédiation.

**La passe 1 de cette story a redit la même chose** : ses deux CRITICAL et deux de ses trois
HIGH portaient sur des **relevés** — clés i18n, passages de manuel, décomptes, zones — et
**aucune** décision de conception (D1, D2, D5, D7) n'a été prise en défaut. Ce qu'il reste à
relire n'est donc pas la story : c'est le patch qui vient d'être écrit dessus.

⚠️ **Réserve du `CLAUDE.md`, vérifiée avant d'employer cette forme** : la passe ciblée ne
suffit pas si le patch précédent touche plusieurs modules ou change une règle métier. Ici la
remédiation ne touche **qu'un seul fichier**, le story file lui-même, et ne change **aucune**
décision de conception — elle corrige des relevés et restreint un invariant. La forme
s'applique.

## Le prompt, tel qu'il a été donné

> Tu es une lentille de revue adversariale **chasseuse de régressions**, en contexte frais, sur
> le dépôt Kesh (`/home/gcorbaz/devel/kesh`, comptabilité suisse — Rust/Axum, SvelteKit,
> MariaDB). Réponds en FRANÇAIS.
>
> **Ton périmètre est UN SEUL COMMIT** : `fae6471a`, la remédiation de la passe 1 de revue de
> la spécification `_bmad-output/implementation-artifacts/24-4b-gel-ecriture.md`. Lis-le avec
> `git show fae6471a`.
>
> **Ta question n'est pas « la spec est-elle bonne ? » mais « ce patch a-t-il cassé quelque
> chose, ou introduit un défaut neuf ? »** C'est le mode d'échec le plus documenté de ce
> dépôt : la remédiation d'une passe devient le défaut de la suivante.
>
> **Ce que la passe 1 avait trouvé, et que ce patch prétend corriger** :
>
> 1. CRITICAL — la spec ordonnait de retirer la clé i18n `journal-entries-delete-blocked-reversed`,
>    décrétée orpheline sur un grep restreint à `frontend/`, alors qu'elle est consommée par
>    `crates/kesh-api/src/errors.rs:2517`.
> 2. CRITICAL — l'invariant I3 (« pour toute écriture qu'aucune pièce ne possède,
>    `POST /{id}/reverse` rend 201 ») était faux : `IS_A_REVERSAL` et `ALREADY_REVERSED` ne
>    dépendent d'aucune pièce.
> 3. HIGH — contradiction interne « sept clés » (AC 14, T5) contre « huit » (Dev Notes).
> 4. HIGH — quatre clés de plus orphelinées par le retrait de la modale de conflit de version.
> 5. HIGH — un septième passage de manuel (`user-manual.tex:1585`) manqué par une liste figée.
> 6. MEDIUM — `crates/kesh-report/src/balance_sheet.rs` absent de la table des fichiers, et
>    « cinq zones » faux (six).
> 7. MEDIUM — « 32 tests unitaires » alors qu'ils sont 36.
> 8. MEDIUM — séquencement circulaire entre T2 et D5 (citer une issue créée après le dev).
> 9. LOW — `lib.rs:332-339` → `:335-338`.
> 10. MEDIUM (Haiku) — la combinaison « exercice clos **et** déjà contre-passée » non testée.
> 11. LOW (Haiku) — `PUT` sur l'écriture d'ouverture couvert seulement par déduction.
>
> Et deux findings **réfutés** par l'orchestrateur : `invoices::delete` sur une écriture
> contre-passée est inatteignable ; les deux tests demandés par l'AC 8 existaient déjà.
>
> **Cherche, dans cet ordre** :
>
> - **Une correction qui en casse une autre.** Un patch qui corrige le site nommé et laisse une
>   formulation contradictoire ailleurs dans le même document ; un décompte corrigé à un endroit
>   et resté faux à un autre ; une AC renumérotée dont un renvoi n'a pas suivi.
> - **Une correction FAUSSE.** Chaque affirmation neuve du patch est à vérifier au sol :
>   les onze clés i18n et leur unicité de site, les sept passages de manuel et les neuf
>   occurrences déclarées intouchables, les 36 tests, les six zones, `errors.rs:2517`,
>   `loader.rs:653`, `reversal_blocker` et `OWNED_BY_INVOICE`, `invoices.rs:3056` et `:3160`,
>   `lib.rs:335-338`, `balance_sheet.rs:28`.
> - **Une réfutation abusive.** Les deux findings écartés l'ont-ils été à bon droit ? Si l'un
>   des deux tenait, il a été perdu.
> - **Un invariant sur-restreint.** I3 a été rétréci pour être vrai : couvre-t-il encore le
>   risque qu'il existait pour couvrir — livrer un gel qui enferme l'utilisateur ?
> - **Ce que le patch a AJOUTÉ et que personne n'a relu** : le tableau des occurrences
>   intouchables, le paragraphe de dérogation à la règle de splitting, le journal de revue
>   lui-même (ses décomptes se recomptent).
>
> ⛔ **Aucune affirmation sans vérification au sol.** Avant d'écrire « X est faux », « Y manque »,
> « ce numéro de ligne est erroné », établis-le par `grep -nF` / `sed -n` / lecture directe, et
> **cite la commande et son résultat**. Un finding non vérifié coûte plus cher que le défaut
> qu'il prétend signaler.
>
> **Format** : `[SÉVÉRITÉ] identifiant — titre`, puis ce qui est faux ou manquant, la
> vérification au sol (commande + extrait), la conséquence pour l'implémenteur, le correctif en
> une ou deux phrases. Sévérités CRITICAL / HIGH / MEDIUM / LOW. Termine par un tableau des
> comptes et une phrase de verdict, en disant explicitement **combien de tes findings portent
> sur le patch de la passe 1** plutôt que sur la conception d'origine. Si le patch est propre,
> dis-le — « zéro finding » est une réponse acceptable si elle est vraie et vérifiée.

## Critère de clôture rappelé à l'orchestrateur

La § *La passe ciblée* du `CLAUDE.md` permet de **clore la boucle** après une passe ciblée si la
remédiation qu'elle produit **ne touche aucune ligne de code de production**. Ici, l'objet revu
est une **spécification** : la question équivalente est de savoir si la remédiation touche une
**décision de conception** ou seulement des relevés. Si elle ne touche que des relevés, et que
la sévérité est retombée sous MEDIUM, la boucle peut se clore avant les huit passes.
