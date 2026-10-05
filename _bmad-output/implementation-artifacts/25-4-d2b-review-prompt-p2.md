# Prompt — revue de code P2 ciblée, Story 25-4-d2b

*Versionné le 2026-10-02. Une lentille (Opus), contexte frais — passe ciblée sur la remédiation de P1. **La sévérité se
déplace vers ce qu'on vient d'écrire** : cherche d'abord ce que ce patch a cassé.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-d2b-bouton-solder-le-reste`. **Diff à revoir : `git diff 50bdfd49 c575efa3`**
(un commit). Fiche : `_bmad-output/implementation-artifacts/25-4-d2b-bouton-solder-le-reste.md` (Change Log, entrée
« Revue de code P1 »).

## Lentille — regression hunter

La remédiation :
1. `WriteOffDialog.svelte` : `escapeKeydownBehavior` / `interactOutsideBehavior` à `'ignore'` et croix masquée pendant
   l'envoi ; `amountDue` peut être `null` (aucune nature, confirmation désactivée, « Reste dû en cours de calcul… ») ;
2. `+page.svelte` : le dialogue est **toujours monté** ; garde `onOpenChange` du parent ; notification du refus quand le
   dialogue n'est plus ouvert ;
3. `due-dates/+page.svelte` : `formatExactAmount` pour le reste dû ;
4. trois tests.

Vérifie :
- le dialogue **toujours monté** : son `$effect` de réinitialisation (« si `open`… ») se déclenche-t-il au montage de la
  page, ou à chaque changement de facture ? Le `$effect` qui retire une nature non proposée, quand `amountDue` passe par
  `null` puis revient (envoi d'e-mail), efface-t-il une saisie en cours ? Le dialogue monté sur une facture **brouillon**
  ou **payée** (le bouton est masqué, mais `writeOffOpen` peut-il rester vrai d'une action antérieure) ?
- les propriétés `escapeKeydownBehavior` / `interactOutsideBehavior` / `showCloseButton` : passent-elles réellement au
  primitif bits-ui à travers le composant `dialog-content.svelte` du projet (`...restProps`) ? Bits-ui 2 les accepte-t-il
  sur `Dialog.Content` ? Une fois l'envoi fini, la fermeture redevient-elle possible ?
- la notification « si le dialogue n'est plus ouvert » : peut-elle doubler le message (dialogue ET toast), ou ne
  jamais partir ?
- l'échéancier : `formatExactAmount` sur un reste **négatif** (trop-perçu hérité, le type le permet) ou à `null` ? Le
  résumé (`unpaidTotal`) reste au centime — incohérent avec la colonne ?
- les trois tests : chacun aurait-il échoué avant le patch ?

## Ce que tu rends

Findings avec sévérité, `fichier:ligne`, **preuve** (commande et sortie, ou code cité relu). Pour tout finding affirmant
qu'un code est absent ou présent : la sortie d'un `grep -nF`. ⛔ La liste des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base : `scripts/*` (dont
`scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx migrate`,
`cargo test`/`nextest`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`,
`git log`/`show`/`diff`, lecture de `node_modules`.
