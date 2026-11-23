# Release 0.20.0 — choix autonomes de la Story 49-1c (clé `e491c`)

Registre du développement de `49-1c-types-de-lignes-ecran` (Opus 5.5, `bmad-dev-story`, 2026-11-19).
Chaque choix : contexte, option retenue, autres options, réversibilité.

## C-e491c-1 — Les deux prédicats dans un module sans dépendance

- **Contexte** : D5–D7 placent `estChiffree` et `estLigneDeTexte` dans `invoice-lines.ts`, et D6 le
  filtre du total dans `invoice-helpers.ts`. Or `invoice-lines.ts` importe `readableAmount` de
  `supplier-invoice-form.ts`, qui importe `invoice-helpers.ts` : importer `invoice-lines.ts` depuis
  `invoice-helpers.ts` créerait un cycle (celui que `unit-price.ts` évite déjà, 36-3b V1-5).
- **Retenu** : `frontend/src/lib/features/invoices/invoice-line-type.ts`, sans dépendance, porte les
  deux prédicats ; `invoice-lines.ts` les réexporte (l'API de la fiche est tenue), `invoice-helpers.ts`
  et les deux fiches les importent du module léger. Une seule définition.
- **Écartées** : le cycle d'imports (fonctionne en ESM pour des fonctions, mais fragile et contraire au
  précédent) ; dupliquer le filtre dans `invoice-helpers.ts` (DRY).
- **Réversibilité** : totale, déplacement de deux fonctions.

## C-e491c-2 — `ligneDepuisReponse(l, uiKey)` reçoit la clé d'interface

- **Contexte** : la fiche nomme `ligneDepuisReponse(l: InvoiceLineResponse)` ; la clé `_uiKey` est
  produite par `nextUiKey()`, propre à l'instance d'`InvoiceForm`.
- **Retenu** : second paramètre `uiKey: string` ; les deux appelants passent `nextUiKey()`. La fonction
  reste pure et testable à l'égalité exacte.
- **Écartée** : un compteur global au module (état partagé entre instances, tests non déterministes).
- **Réversibilité** : totale.

## C-e491c-3 — Rangée du commentaire dans le formulaire : `colspan` 6, non 5

- **Contexte** : D7 écrit « couvre les colonnes description à total (`colspan` 5) ». Le tableau a sept
  colonnes (`LINE_TABLE_COLUMNS = 7`) : description, quantité, prix, TVA, compte, total, action. De la
  description au total, il y en a **six**.
- **Retenu** : `colspan={LINE_TABLE_COLUMNS - 1}` (6), la cellule de suppression restant la septième —
  l'intention de D7 (« couvre … à total ; la cellule de suppression reste »). Le chiffre 5 de la fiche
  est une erreur de compte.
- **Réversibilité** : totale.

## C-e491c-4 — Positions des lignes dans T3 : 0, 1, 2

- **Contexte** : AC1 écrit « positions 1 à 3 ». L'API numérote les positions à partir de **0** (le test
  existant `ligne_conforme` de la 49-1a affirme `position == 0` pour une ligne unique).
- **Retenu** : T3 affirme `[0, 1, 2]` — trois lignes consécutives à leur place, ce qu'AC1 veut établir.
- **Réversibilité** : sans objet (constat du code).

## C-e491c-5 — Une ligne de texte relue porte des montants vides

- **Contexte** : D5 dit qu'une ligne de commentaire porte `''` dans ses champs de montant ; la réponse de
  l'API, elle, porte `0.0000` / `0.00`.
- **Retenu** : `ligneDepuisReponse` pose `''` pour `quantity`, `unitPrice`, `vatRate` et `null` pour le
  compte d'une ligne de texte (un `'0.00'` hérité serait trompeur et ne part jamais).
- **Réversibilité** : totale.

## C-e491c-6 — Rejeu d'AC8 (c) : copies déplacées, non supprimées ; vitest lancé depuis `frontend/`

- **Contexte** : AC8 (c) (1) demande de supprimer les copies `*.rejeu.test.ts` ; la consigne interdit
  toute forme de `rm`. La commande de la fiche (`npm --prefix frontend exec -- vitest run …`) lance
  vitest depuis la racine du worktree, sans la configuration du frontend (constaté :
  `kesh-gate-logs/e491c/vitest-cible-1.log`, premier essai, « Cannot find module `$lib/…` »).
- **Retenu** : copies déplacées (`mv`) dans `kesh-gate-logs/e491c/rejeu/` après le rejeu (elles y
  restent comme pièces) ; vitest lancé par `cd frontend && ../scripts/mem-guard.sh npx vitest run …`.
- **Réversibilité** : sans objet.

## C-e491c-7 — Manuels compilés hors du dépôt

- **Contexte** : `make` écrit les PDF des manuels en place, et ils sont suivis par git ; ils ne se
  commitent pas dans une story.
- **Retenu** : l'arbre `docs/manual/` est copié dans `kesh-gate-logs/e491c/manuel/` et compilé là
  (`make -B user admin`) ; les contrôles aplatis lisent ces PDF. Aucun PDF du dépôt ne bouge.
- **Réversibilité** : sans objet.

## C-e491c-8 — Voie de base

- **Contexte** : le prompt d'appel donne la clé `e491c` sans voie.
- **Retenu** : voie 1 (`kesh-mariadb-dev`, verrou `.gate.lock`), celle de `gate-ci.sh`.

## C-e491c-9 — Un point de code `U+NNNN` n'est pas un compte (garde G4-bis)

- **Contexte** : `test-fast-1.log` (2026-11-20, 04:21) rougit sur
  `textes_coherents::les_comptes_cites_en_exemple_existent_dans_les_plans_livres` : le paragraphe des
  caractères du manuel utilisateur (`user-manual.tex:1594`, AC10 a) cite le signe moins « (U+2212) »,
  que `nombres_isoles` lit comme le compte 2212 (`+` ne borne rien ; U+2010, U+2011 et U+2013 passaient
  comme des années).
- **Retenu** : la garde apprend la forme `U+NNNN` — un nombre précédé de `+`, lui-même précédé de `U`,
  n'est pas lu ; un `+` seul ne change rien. Doc-comment de `nombres_isoles` et test
  `nombres_isoles_ne_prend_pas_un_point_de_code_pour_un_compte` (négatif `U+2212`, positifs `+2212`,
  `X+2212`). Le texte du manuel, juste, ne change pas.
- **Alternatives** : retirer « (U+2212) » du manuel (AC10 a nomme les points de code pour identifier les
  caractères ; U+2010 et U+2011 n'ont pas d'autre nom lisible) ; écrire le point de code autrement
  (contournement de la garde, que la 49-4 retrouverait).
- **Réversibilité** : totale (code de test seul).
