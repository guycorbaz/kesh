# Prompt de la passe 4 de `bmad-code-review` — passe ciblée, lentille unique

**Story** : 24-4a, la contre-passation d'une écriture (issue #380).
**Passe** : 4, **ciblée** — même dispositif que la passe 3.
**Modèle** : Haiku 4.5 (rotation : P1 Sonnet ×2 + Haiku → P2 Opus → P3 Sonnet → **P4 Haiku**).
**Cible** : le **seul** commit `cabf18d8`, remédiation de la passe 3.
**Écrit le** : 2026-08-28.

## Pourquoi cette passe a lieu

La passe 3 a rendu 0 CRITICAL, 0 HIGH, 2 MEDIUM, et sa remédiation **touche du
code de production** (le composant Svelte) : le critère de clôture du `CLAUDE.md`
n'est donc pas atteint. Arbitrage du Project Lead : faire la quatrième passe.

Le motif qui la justifie est mesuré : sur les trois passes précédentes,
**quinze findings sur vingt-cinq** portent sur une remédiation, aucun sur une
décision de conception d'origine. Le patch de la passe 3 est petit — deux
gestes — mais c'est la dernière chose que personne n'a relue, et le précédent
immédiat est éloquent : la passe 3 a trouvé qu'un correctif de la passe 2, écrit
pour *renforcer* une garde, avait en réalité **changé le comportement
d'exécution** dans le mauvais sens.

⚠️ **Haiku est le cas pathologique connu du dépôt** sur l'indexation des diffs
multi-commits (`CLAUDE.md` § *Haiku-specific guardrails*). Le prompt lui donne
donc **un commit unique** et lui impose le `grep -nF` de vérification avant tout
finding.

## Le prompt, tel qu'il a été donné

> Tu es « chasseur de régressions » sur le projet Kesh (comptabilité suisse,
> Rust/Axum + SvelteKit + MariaDB), dépôt `/home/gcorbaz/devel/kesh`, branche
> `story/24-4a-contre-passation-ecriture`. Réponds en français.
>
> MISSION : **passe 4, CIBLÉE**, de `bmad-code-review` sur la Story 24-4a (#380).
>
> ⛔ **Ton périmètre est UN SEUL COMMIT** : `cabf18d8`. `git show cabf18d8`.
>
> Ce commit fait **trois** choses, et rien d'autre :
> 1. dans `frontend/src/routes/(app)/journal-entries/[id]/+page.svelte`, le
>    `default` du `switch` de `blockedLabel` passe de `return _exhaustif` à
>    `void _exhaustif; return '';` ;
> 2. dans `crates/kesh-api/tests/journal_entry_reversal_e2e.rs`, une assertion
>    est ajoutée sur `reversalBlockedLabel` ;
> 3. deux comptes rendus sont mis à jour — le doc-comment de
>    `frontend/src/lib/shared/i18n-libelle-en-dur.test.ts` (et son assertion de
>    ventilation), et le journal de revue du story file.
>
> CE QUE TU CHERCHES :
>
> 1. **Le `void _exhaustif; return '';` est-il correct ET suffisant ?** La garde
>    de compilation rougit-elle encore si l'on ajoute un neuvième code au type
>    `ReversalBlocker` sans l'ajouter au `switch` ? Vérifie-le **en le faisant** :
>    ajoute le code au type, lance `npm run check` depuis `frontend/`, lis le
>    résultat, **puis restaure le fichier** (`git checkout --`) et vérifie que
>    l'arbre est propre. Et à l'exécution, que voit l'utilisateur ?
> 2. **L'assertion ajoutée peut-elle rougir ?** Échouerait-elle vraiment si le
>    numéro du compte cessait d'être remonté ? Le montage la met-il dans l'état
>    qu'elle croit ?
> 3. **Les deux comptes rendus disent-ils vrai ?** L'assertion de ventilation
>    (`ecartee` / `conforme`) correspond-elle au relevé réel — recompte-le en
>    lançant le test. Le journal de revue du story file énonce des décomptes
>    (« de dix findings à deux », « 0 CRITICAL, 0 HIGH, 2 MEDIUM ») : sont-ils
>    cohérents avec ce que les passes ont produit ?
> 4. **Un résidu du symptôme.** Le `return _exhaustif` corrigé a-t-il des jumeaux
>    ailleurs dans `frontend/src` — un autre `switch` avec garde `never` qui
>    rendrait la valeur brute ?
>
> ⚠️ RÈGLE DE VÉRIFICATION ABSOLUE — LIS-LA DEUX FOIS. Avant de rapporter tout
> finding affirmant qu'un code **est absent** ou **est présent**, tu DOIS le
> vérifier par `grep -nF "<chaîne exacte>" <fichier>` (le drapeau `-F` est
> OBLIGATOIRE) ou en lisant le fichier. Cite la commande et son résultat. Ne
> devine JAMAIS un numéro de ligne : lis le fichier. Un finding sans preuve est
> REJETÉ.
>
> FORMAT : identifiant (`P4-1`…), SÉVÉRITÉ, site, défaut en une phrase, preuve,
> correctif. Décompte par sévérité, puis un **verdict explicite sur la clôture** :
> la remédiation que tu exiges toucherait-elle du code de production ?
>
> Si tu ne trouves rien, **dis-le franchement** : « 0 finding » est un résultat
> légitime et attendu d'une passe ciblée qui converge. N'invente pas un finding
> pour avoir quelque chose à rendre.
>
> N'écris AUCUN fichier hors de la vérification décrite au point 1, et laisse
> l'arbre de travail PROPRE.
