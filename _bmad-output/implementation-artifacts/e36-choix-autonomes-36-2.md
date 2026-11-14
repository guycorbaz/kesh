# Epic 36 — registre des choix autonomes de la story 36-2 (#1150)

Registre propre à la story 36-2, **versé par union**. Fiche : `36-2-eclater-bouton-grise.md`. Code lu à `dabd9045a`.
Ces choix ne sont pas couverts par C-e360 ; ils prolongent l'étape 0 de la 36-2 et C-e360-5 (registre
`e36-choix-autonomes-e360.md`), qui font foi : la cause n'est pas tranchée par la fiche, la reproduction la départage.

### C-36-2-1 — H4 (le catalogue efface l'attendu et l'écart) se corrige sans condition

- **Contexte** : l'étape 0 nomme H1 à H3. La lecture du catalogue (`fr-CH/messages.ftl:1620-1621`) contre
  `i18n.svelte.ts:20` montre un quatrième fait, établi et non supposé : en production, l'indicateur dit « Balance » et le
  message du déséquilibre ne dit ni l'attendu ni l'écart.
- **Retenu** : correction dans la story (D3), parce que l'attendu de #1150 est « ou l'écran dit ce qui manque ».
- **Écartée** : une issue séparée — la story livrerait un écran qui ne dit toujours pas ce qui manque dans le cas le plus
  probable d'un bouton grisé sur des lignes remplies.
- **Réversible** : oui.

### C-36-2-2 — H5 (le montant part en nombre JSON, refusé par `serde-str`) se corrige sans condition

- **Contexte** : `bind:value` sur `type="number"` rend un nombre (Svelte 5.55.1, `bindings/input.js:31`, `:280-289`) ;
  `Decimal` sous `serde-str` exige une chaîne (`rust_decimal-1.43.0/src/serde.rs:289-296`, `serde_json-1.0.154/src/de.rs:1535-1544`).
  Aucun test ne fait traverser la frontière HTTP à un éclatement saisi à l'écran.
- **Retenu** : le montant part en chaîne (D2), prouvé par un Vitest et par l'E2E R0 (a), qui passe l'éclatement de bout en
  bout.
- **Écartée** : une issue séparée — le bouton activé mènerait à un refus, et #1150 resterait vraie en substance.
- **Réversible** : oui.

### C-36-2-3 — Branche H2 : la grammaire du journal, importée, non modifiée

- **Contexte** : si Chromium fr-CH jette une saisie (`70,00`, `1'200.00`), le champ montre un texte que la valeur ignore.
- **Retenu** : champ texte lu par `isValidAmount` / `parseAmount` de `journal-entries/balance.ts`, sans le modifier ; la
  36-3 (C-e360-6) en fixe l'évolution, dont le dialogue hérite à la fusion.
- **Écartées** : `valueAsNumber` (la saisie jetée reste muette) ; la grammaire du solde d'ouverture (`bank-accounts/format.ts`) ;
  une grammaire propre au dialogue.
- **Réversible** : oui.

### C-36-2-4 — R0 se joue aussi contre l'image 0.17.6

- **Contexte** : Guy a vu le défaut le 2026-11-13, en 0.17.6 ; la 35-7 (0.17.7) a changé le `blur` du champ strict, si bien
  qu'une reproduction à la tête seule peut ne plus montrer H1.
- **Retenu** : R0-b rejoue la même spec contre `gcorbaz/kesh:0.17.6` (image présente sur la station), sur une base neuve de
  la voie 2 ; si l'image ne peut pas la jouer, le Record le dit et s'en tient à la lecture de `v0.17.6`.
- **Écartée** : la lecture seule de `v0.17.6` (elle ne dit rien de H2 ni de H3).
- **Réversible** : oui.

### C-36-2-5 — Branche H1 : le champ de compte n'est pas touché

- **Contexte** : à la tête, le `blur` d'un champ strict jette le brouillon (35-7) et le message dit « compte requis » ; un
  numéro exact puis *Tab* qui choisirait le compte est une autre décision (#1084), sur sept consommateurs.
- **Retenu** : aucun changement d'`AccountAutocomplete.svelte` ; si R0-a contredit la lecture, arrêt et amendement de la fiche.
- **Écartée** : choisir l'unique correspondance exacte au `blur` (contraire à D2 de la 35-7, « seul un choix explicite
  sélectionne », et décision qui revient à Guy dans #1084).
- **Réversible** : oui.

### C-36-2-6 — Les clés de l'indicateur : la clé existante pour l'état équilibré, une clé neuve pour l'écart

- **Contexte** : `reconciliation-split-balance-indicator` sert aujourd'hui les deux états (`TransactionSplitModal.svelte:405`,
  `:410`) avec deux replis différents.
- **Retenu** : la clé existante garde l'état équilibré (`{ $expected }`) ; `reconciliation-split-balance-gap` neuve porte
  l'écart ; `reconciliation-split-error-imbalance` reçoit `{ $expected }` et `{ $diff }`.
- **Écartée** : retirer la clé partagée et en créer deux (plus de lignes touchées dans quatre fichiers, sans gain).
- **Réversible** : oui.
