# Prompt de la passe 3 de `bmad-code-review` — passe ciblée, lentille unique

**Story** : 24-4a, la contre-passation d'une écriture (issue #380).
**Passe** : 3, **ciblée** au sens du `CLAUDE.md` § *La passe ciblée*.
**Modèle** : Sonnet 4.6 (rotation : P1 Sonnet ×2 + Haiku → P2 Opus → **P3 Sonnet**).
**Cible** : le **seul** commit `04c30535`, remédiation de la passe 2.
**Écrit le** : 2026-08-28.

## Pourquoi cette passe, et pourquoi ciblée

Sur les deux premières passes, **treize findings sur vingt-trois portent sur une
remédiation**, aucun sur une décision de conception d'origine. Ce qu'il reste à
relire n'est donc plus la story : c'est ce que la passe 2 vient d'écrire.

La passe 2 a elle-même nommé les deux endroits les plus probables, et ce prompt
les lui répète :

1. **la propagation d'un décompte** — la passe 2 avait trouvé « sept » laissé à
   trois sites dont un cinq lignes sous le « huit » qui venait d'être écrit ; la
   remédiation refait exactement ce type de geste ;
2. **le `switch` Svelte**, seul patch de la série à toucher un chemin
   d'affichage.

⚠️ Le prompt est versionné pour que la passe soit **rejouable et contestable**.
Une passe qui s'écarte du protocole à trois lentilles doit laisser de quoi la
refaire — sans quoi son verdict ne vaut pas mieux qu'une passe non faite.

## Le prompt, tel qu'il a été donné

> Tu es « chasseur de régressions » sur le projet Kesh (comptabilité suisse,
> Rust/Axum + SvelteKit + MariaDB), dépôt `/home/gcorbaz/devel/kesh`, branche
> `story/24-4a-contre-passation-ecriture`. Réponds en français.
>
> MISSION : **passe 3, CIBLÉE**, de `bmad-code-review` sur la Story 24-4a (#380)
> — la contre-passation d'une écriture comptable.
>
> ⛔ **Ton périmètre est UN SEUL COMMIT** : `04c30535`, la remédiation de la
> passe 2. `git show 04c30535`.
>
> Ne relis pas la conception d'origine : deux passes complètes l'ont couverte, et
> **treize findings sur vingt-trois** y portaient déjà sur une remédiation, aucun
> sur une décision d'origine. Le « Journal de revue de code » de
> `_bmad-output/implementation-artifacts/24-4a-contre-passation-ecriture.md` dit
> ce qui a été trouvé aux passes 1 et 2 ; le « Journal de revue » plus haut dans
> le même fichier dit ce que quatre passes de revue de SPEC ont trouvé **et
> réfuté**. ⛔ Ne re-signale rien qui y figure comme réfuté sans preuve nouvelle.
>
> CE QUE TU CHERCHES, dans cet ordre :
>
> 1. **Une propagation incomplète.** C'est le défaut que ce commit corrigeait, et
>    il peut le reproduire. Il a changé des décomptes (`sept` → `huit`,
>    `conforme`/`ecartee`, `2248` → `2257`, `63` → `64`) et renommé un champ
>    (`archived_account_id` → `archived_account_number`). Chaque valeur
>    a-t-elle été grepée comme **jeton**, sur tout le dépôt — code, tests,
>    doc-comments, spec, `sprint-status.yaml`, manuels ?
> 2. **Le `switch` Svelte et son chemin d'affichage.** `blockedLabel` est passée
>    à `code: ReversalBlocker` avec une affectation à `never` ; une fonction
>    `blockedMessage` est née. Le rendu est-il juste dans tous les cas — motif
>    sans étiquette, étiquette sans motif, code inconnu venant d'un serveur plus
>    récent que le navigateur ? Le `never` fait-il réellement rougir, ou seulement
>    en apparence ?
> 3. **Le contrat du refus.** `document_label` porte désormais **deux choses** :
>    un numéro de pièce, ou un numéro de compte. `document_id` reste `None` pour
>    le compte. Un consommateur — écran, test, futur client d'API — peut-il
>    confondre les deux ? La sous-requête SQL qui choisit le compte archivé
>    (`ORDER BY a.number LIMIT 1`) est-elle déterministe, et son choix est-il le
>    bon quand plusieurs comptes sont archivés ?
> 4. **Le manuel.** Une section neuve et une FAQ modifiée décrivent la fonction.
>    Décrivent-elles ce que le code fait **réellement** ? Le PDF versionné
>    correspond-il au `.tex` ? Une autre page du manuel, ou du site
>    `website/`, dit-elle encore le contraire ?
>
> RÈGLE DE VÉRIFICATION NON NÉGOCIABLE : tout finding `CRITICAL` ou `HIGH`
> affirmant l'absence ou la présence d'un code se vérifie par `grep -nF`
> (fixed-string obligatoire) ou par lecture directe, AVANT d'être rapporté,
> commande et résultat cités. Tu peux exécuter des tests ciblés — MariaDB de dev
> est démarré, le gate est vert à 2257/2257 et le frontend à 747/747.
>
> FORMAT : identifiant (`P3-1`…), SÉVÉRITÉ, site, défaut en une phrase, scénario
> d'échec concret, preuve, correctif. Précise pour chacun s'il porte sur **la
> remédiation de la passe 2** ou sur **le code antérieur**. Termine par le
> décompte par sévérité et un **verdict explicite** sur la clôture.
>
> ⚠️ **Le critère de clôture du dépôt** : la boucle se clôt quand la remédiation
> qu'une passe produit ne touche plus **aucune ligne de code de production**.
> Dis-le franchement si c'est le cas ; dis-le tout aussi franchement sinon. Si tu
> ne trouves rien, « 0 finding » est un résultat légitime — n'invente pas.
>
> N'écris AUCUN fichier, ne modifie RIEN.
