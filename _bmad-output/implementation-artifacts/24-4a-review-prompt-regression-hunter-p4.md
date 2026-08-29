# Prompt de la passe 4 — passe ciblée, lentille unique « chasseur de régressions »

**Story** : 24-4a, la contre-passation d'une écriture (issue #380).
**Passe** : 4, **ciblée** — même dispositif que la passe 3, dont le prompt est
versionné dans `24-4a-review-prompt-regression-hunter-p3.md`.
**Modèle** : Haiku 4.5 (rotation : P1 Sonnet + Haiku → P2 Opus → P3 Sonnet → **P4 Haiku**).
**Cible** : le **seul** commit `6de043cc`, remédiation de la passe 3.
**Écrit le** : 2026-08-28.

## Pourquoi cette passe a lieu

La passe 3 a rendu **0 CRITICAL, 2 HIGH**, et sa remédiation ne touche aucune
décision de conception ni aucune ligne de code — ce qui, au titre de la clause de
passe ciblée du `CLAUDE.md`, autorisait à clore. **Arbitrage du Project Lead : on
applique la lettre de la *Review Iteration Rule* et on fait la quatrième passe.**

C'est le choix prudent, et il a un motif mesuré : sur les trois passes précédentes,
**douze findings sur seize sont nés d'un patch de remédiation**, jamais de la
conception d'origine. La remédiation de la passe 3 est petite, mais elle est la
dernière chose que personne n'a relue.

⚠️ **Haiku est le cas pathologique connu du dépôt** sur l'indexation des diffs
multi-commits (`CLAUDE.md` § *Haiku-specific guardrails*). Le prompt lui donne donc
**un commit unique** et lui impose le `grep -nF` de vérification avant tout finding.

## Le prompt, tel qu'il a été donné

> Tu es « chasseur de régressions » sur le projet Kesh (comptabilité suisse,
> Rust/Axum + SvelteKit + MariaDB), dépôt `/home/gcorbaz/devel/kesh`. Réponds en
> français.
>
> MISSION : **passe 4, CIBLÉE**, sur `_bmad-output/implementation-artifacts/24-4a-contre-passation-ecriture.md`.
>
> ⛔ **Ton périmètre est UN SEUL COMMIT** : `6de043cc`.
>
> ```
> git -C /home/gcorbaz/devel/kesh show 6de043cc
> ```
>
> Ne relis pas la conception d'origine : trois passes l'ont couverte. Le « Journal de
> revue » en fin de spec dit ce qui a déjà été trouvé **et ce qui a été réfuté au sol**
> — ne re-signale aucun point réfuté sans preuve nouvelle et contraire.
>
> Ce commit fait quatre choses, et rien d'autre : il retire « et la liste » d'une
> phrase de D6 ; il corrige l'affirmation « ce chemin sert `reset_demo` (production) »,
> qui était fausse ; il inscrit aux tâches la correction d'un doc-comment périmé ; il
> ajoute la section « Passe 3 » au journal de revue.
>
> CE QUE TU CHERCHES :
>
> 1. **Le symptôme a-t-il été grepé PARTOUT ?** C'est le défaut que ce commit
>    corrigeait, et il pourrait le reproduire. Les deux symptômes sont « la liste »
>    (pour les champs dérivés) et « `reset_demo` ». Cherche-les dans **tout** le
>    fichier de spec, dans `sprint-status.yaml`, et dans le prompt de la passe 3.
>    Reste-t-il une occurrence qui affirme encore le contraire ?
> 2. **La correction de `reset_demo` est-elle exacte ?** Vérifie toi-même ce que
>    `reset_demo` fait (`crates/kesh-seed/src/lib.rs`) et où `delete_all_by_company`
>    est appelée. La nouvelle formulation dit-elle vrai ?
> 3. **La tâche « corriger le doc-comment » est-elle actionnable ?** Le
>    doc-comment visé existe-t-il à la ligne annoncée, et dit-il bien ce que la spec
>    lui reproche ?
> 4. **Le journal de revue dit-il vrai ?** Ses décomptes (« de onze findings à
>    deux », « 19 critères ») se recomptent depuis la source — c'est une règle
>    explicite de ce dépôt : un total doit être cohérent avec sa propre ventilation.
>
> ⚠️ RÈGLE DE VÉRIFICATION ABSOLUE — LIS-LA DEUX FOIS. Avant de rapporter tout
> finding affirmant qu'un texte ou un code **est absent** ou **est présent**, tu DOIS
> le vérifier par `grep -nF "<chaîne exacte>"` (le drapeau `-F` est OBLIGATOIRE) ou
> par lecture directe du fichier, et **citer la commande et son résultat**. Ne devine
> jamais un numéro de ligne : lis le fichier. Un finding sans preuve est REJETÉ.
>
> FORMAT : pour chaque finding, un identifiant (`P4-1`…), une SÉVÉRITÉ, le site, le
> défaut en une phrase, la preuve, le correctif. Termine par le décompte par sévérité
> **et par un verdict explicite sur la clôture de la boucle**.
>
> Si tu ne trouves rien, **dis-le franchement** : « 0 finding » est un résultat
> légitime et attendu d'une passe ciblée qui converge. N'invente pas un finding pour
> avoir quelque chose à rendre.
>
> N'écris AUCUN fichier, ne modifie RIEN, ne commits RIEN.
