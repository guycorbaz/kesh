# Prompt — revue de code P2 ciblée de la Story 35-7 (#1000), une lentille, braquée sur la remédiation de la P1

Passe **ciblée**, une seule lentille, contexte frais, **Sonnet**, en lecture seule. Worktree `/home/gcorbaz/devel/kesh-e357`, branche
`story/35-7-autocomplete-compte-hors-liste`.

## Périmètre : le seul commit `0a64b0f8`

`git -C /home/gcorbaz/devel/kesh-e357 show 0a64b0f8` — quatre fichiers : `JournalEntryForm.svelte` (le `Ctrl+S` rend le focus avant
`handleSubmit`), `AccountAutocomplete.svelte` (commentaire de `handleBlur`), `account-options.ts` (commentaire), le test
`JournalEntryForm.edit.test.ts`. Il remédie trois findings de la P1 (rapports `/home/gcorbaz/devel/kesh-gate-logs/e357-review-p1-E.md`
§ E1, `…-p1-A.md` § A3 et A5) : **E1 (MEDIUM)** — `Ctrl+S` soumettait sans `blur`, l'ancien compte partait pendant que le champ
montrait le brouillon. Le compte rendu de la remédiation est la dernière entrée du Change Log de la fiche
`_bmad-output/implementation-artifacts/35-7-autocomplete-compte-hors-liste.md` (commit `a9e85e79`), avec l'**inventaire** des chemins
de soumission sans `blur`.

## Ce que tu vérifies

1. **Le correctif** : `(document.activeElement as HTMLElement | null)?.blur()` avant `if (canSubmit) handleSubmit();`. Le `blur` est-il
   **synchrone** (l'`onblur` d'`AccountAutocomplete` s'exécute-t-il avant `handleSubmit`) ? `handleBlur` ne change pas `value` (champ
   strict) : la charge envoyée est-elle bien celle que le champ réaffiche ? `canSubmit` lu **après** le `blur` : un `blur` d'un autre
   champ (description, montant, date) peut-il changer `canSubmit` ou une valeur (un `onblur` qui normalise un montant, par exemple) —
   et si oui, dans quel sens, et est-ce un défaut ? `git grep -n 'onblur' frontend/src/lib/features/journal-entries/` à l'appui. Le cas
   où l'élément actif est `document.body` ou dans un autre composant (un dialogue ouvert par-dessus le formulaire : le `Ctrl+S` de la
   fenêtre soumettrait-il le formulaire derrière ? était-ce déjà le cas ?).
2. **L'inventaire** : vérifie au code chaque affirmation — aucun `<form>` autour d'un consommateur strict (`RuleFormModal.svelte:120`
   sans `AccountAutocomplete` ?) ; le `Ctrl+S` du layout (`routes/(app)/+layout.svelte:34`) émet `kesh:save` que personne n'écoute
   (`git grep -n 'kesh:save'`) ; les cinq consommateurs stricts et leurs boutons de soumission ; `VatPurchaseAssistant` couvert par le
   `Ctrl+S` de `JournalEntryForm` — vrai (est-il monté dans le formulaire) ? Un chemin manque-t-il (Entrée sur un bouton dans un dialogue
   bits-ui, raccourci d'un autre écran) ?
3. **Le test** : rouge avant (`kesh-gate-logs/e357/rem-p1-rouge.log`), vert après, mutation M8 tuée (`e357/mut/M8.log`) ; le test
   simule-t-il un vrai `keydown` sur la fenêtre avec le focus dans le champ (sinon il passerait à vide) ? Asserte-t-il la charge envoyée
   **et** le texte affiché ?
4. **Les commentaires** (A5, A3) : `handleBlur` borne-t-il l'invariant à ce que le code tient ? « sept emplacements, dans cinq écrans »
   (`account-options.ts:37`) : recompte par `git grep -n 'AccountAutocomplete' frontend/src --  '*.svelte'`.
5. **Les gates** au commit `0a64b0f8` (Vitest 3202, E2E 404/7 KF-029, base recréée : `e357/e2e-r6-rem-p1.driver.log`) : les chiffres
   sont-ils aux journaux ? Le CHANGELOG et le manuel doivent-ils mentionner `Ctrl+S` (`git grep -n 'Ctrl' docs/manual/fr/user-manual.tex
   CHANGELOG.md`) ?

## Interdits absolus

Lecture seule : aucun fichier écrit hormis ton rapport, aucune redirection (`>`, `>>`, `tee`, `2>`, `> /dev/null` compris), aucun
fichier temporaire. Aucun `rm`, `pkill`, `kill`, aucun `git` qui écrit, aucun interpréteur ni script (`python3`, `node`, `npm`, `npx`,
`bash -c`, `jq`, `scripts/*` dont `scripts/prepare-release.sh`, `make`, `cargo`, `docker`, `mariadb`, `curl`, `claude`, `sed -i`,
`vitest`, `playwright`), aucune commande `gh`, aucun accès réseau. Ne lis rien sous `~/kesh/`. Permis : `git show/diff/log/grep`,
`/usr/bin/grep`, `sed -n`, `awk` sans redirection, `cat`, `ls`, `find` sans `-exec`, `wc`. ⚠️ `grep` est une fonction vers `ugrep` :
emploie `/usr/bin/grep` ou `git grep`. Toute affirmation de présence ou d'absence s'appuie sur une sortie copiée au rapport.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/e357-review-p2-ciblee.md` : findings numérotés (`C1`…), sévérité (CRITICAL/HIGH/MEDIUM/LOW), lieu, ligne
lue copiée, preuve, remède ; **axes exercés et non exercés** ; décompte final recompté depuis la liste.
