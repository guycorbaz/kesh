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

## Remédiation de la revue P1 (2026-11-23, Sonnet 5.5, agent de remédiation)

Revue P1 : Opus, trois lentilles (B, E, A) exercées l'une après l'autre par un seul agent, contexte frais ;
**0 CRITICAL, 0 HIGH, 1 MEDIUM (A-1), 10 LOW** (A-2 à A-5, B-1 à B-4, E-1, E-2).

## C-e491c-10 — A-1 : gates de passage en `review` rejoués au HEAD, Rust cité sans rejeu

- **Contexte** : l'agent de développement est mort au crash du 2026-11-20 avant d'écrire la section des
  gates ; `front-1.log` était rouge (garde KF-043, corrigée par `2003e23ef`), `e2e-complet-1.log` vide.
- **Retenu** : frontend complet rejoué au HEAD de la remédiation (`front-2.log`, `c00858669`) ; T10 rejoué
  (`e2e-T10-p1.log`). `git diff --stat 7f8bf6a00 HEAD -- crates` est **vide** : `test-fast-2.log` et
  `gate-ci-2.log`, pris à `7f8bf6a00`, restent valables et sont cités sans rejeu. Section du Dev Agent
  Record écrite, « C-e491c-1 à 8 » corrigé en « 1 à 9 » (puis « 1 à 17 » avec la présente remédiation),
  `Status: review`.
- **Alternative écartée** : rejouer `test-fast` et `gate-ci` (le code Rust n'a pas bougé ; la voie base est
  prise par la 48-11).
- **Réversibilité** : sans objet.

## C-e491c-11 — B-2 et E-2 corrigés dans la story

- **Contexte** (prémisses relues au code) : `estChiffree` testait `=== undefined` (donc faux pour `null`,
  que `ligneDepuisReponse` et le serveur lisent comme absent) ; `estLigneDeTexte` testait `comment` ou
  `heading` : pour une valeur hors de l'union les deux prédicats disaient « non », et la ligne s'affichait
  chiffrée mais partait en texte. Prémisses exactes (`invoice-line-type.ts:19`, `:28`).
- **Retenu** : `estChiffree` : `l.lineType == null || l.lineType === 'priced'` ; `estLigneDeTexte(t)` =
  `!estChiffree({ lineType: t })`, contraire exact par construction. Deux cas de test neufs
  (`invoice-lines.test.ts`). Une valeur inconnue devient une ligne de texte partout (affichage, total
  écarté, envoi `{ lineType, description }`), refusée par le serveur avec son message.
- **Écartée** : rendre `ligneDepuisReponse` total sur l'union (refus explicite) : plus de code pour le même
  effet.
- **Réversibilité** : totale.

## C-e491c-12 — B-3 et B-4 corrigés dans la story

- **Retenu** : constante `COLONNES_LIGNES = 6` à côté des déclarations de chacune des deux fiches
  (`colspan={COLONNES_LIGNES}`), doc-comment de `invoices.types.ts:32` renvoyé à `invoice-line-type.ts`.
  Les assertions `toBe('6')` des tests des fiches restent vraies. La constante ne se dérive pas du
  `<thead>` (il est écrit en dur) : elle nomme le nombre et son lien au tableau, sans l'y lier par code.
- **Réversibilité** : totale.

## C-e491c-13 — B-1 : issue P3 demandée, non corrigé

- **Contexte** : `CreateInvoiceLineRequest` garde `quantity?`, `unitPrice?`, `vatRate?` pour tous les types ;
  un seul constructeur (`ligneAEnvoyer`), aucun scénario fautif aujourd'hui.
- **Retenu** : le remède (union discriminée) touche les types de requête et leurs appelants, hors du
  « petit et sûr » ; issue `technical-debt`, P3, demandée à l'orchestrateur.

## C-e491c-14 — E-1 : issue P3 demandée, non corrigé

- **Contexte** : un `heading` relu (inatteignable par l'API, qui le refuse) s'affiche comme un commentaire
  et rend le brouillon inenregistrable sous un refus qui parle de « heading ».
- **Retenu** : l'affichage en lecture seule d'un titre est une décision d'écran qui appartient à la 49-3
  (qui ouvre `heading`) ; issue P3 demandée, avec renvoi à la 49-3. Rien dans la fiche ne le promet.

## C-e491c-15 — A-2 et A-3 déclarés à la fiche

- **A-2** : le Dev Agent Record dit que T10 de `e2e-T10-vert.log` est à `4cb6633e9`, antérieur à
  `2003e23ef` ; T10 est rejoué au HEAD (`e2e-T10-p1.log`, vert) ; l'E2E complet de clôture fait la preuve.
- **A-3** : les captures sont sur la graine E2E (`with-company`), non sur une base de démonstration (AC11).
  Écart déclaré à la fiche, non repris : les états touchés y sont bien visibles, le constat de la revue
  confirme que cela ne change rien à ce qu'elles prouvent.
- **Réversibilité** : sans objet.

## C-e491c-16 — A-4 : issue P4 demandée

- Le constat cosmétique de la fiche avoir (marges des colonnes « TVA % » et « Compte de produit ») n'a pas
  d'issue ; demandée à l'orchestrateur (P4, `bug`), son numéro à porter à la fiche.

## C-e491c-17 — A-5 : ventilation de `sitesTotal` recomptée

- Recompté à la source : `git show <commit>:frontend/src/lib/components/invoices/InvoiceForm.svelte | grep -o
  "i18nMsg(" | wc -l` rend **21** à `5156f2b7b` (base de la story) et **25** à `cea6910db`..`54471b566`
  (avant la remédiation, qui ne touche pas ce fichier) ; l'écart **+4** (3495 → 3499) était juste, le « 19 → 23 »
  de l'agent de développement ne se retrouve pas faute de périmètre. Phrase du Dev Agent Record corrigée.
