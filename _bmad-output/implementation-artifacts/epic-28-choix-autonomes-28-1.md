# Epic 28 — registre des choix autonomes, Story 28-1 (à fusionner par union dans `epic-28-choix-autonomes.md`)

*Registre tenu par union (jamais par écrasement). Chaque choix : contexte, option retenue, alternatives, réversibilité.*

---

## Story 28-1 — Les boutons d'action destructive se voient (#640)

Fiche : `28-1-boutons-destructifs.md`. Choix faits à la création de la fiche (2026-10-13), en autonomie, sur
`6f971d76` (rétrospective de l'Epic 27), branche `story/28-1`.

### C-28-1-1 — La cause : le jeton `destructive` n'existe pas dans le thème (prémisse de l'issue corrigée)

- **Contexte** : #640 et l'epic attribuent le défaut à `bg-destructive/10` « fond rouge à 10 % sous texte rouge ».
- **Constat au sol** : `frontend/src/app.css:10-28` ne définit aucun `--color-destructive` ; Tailwind v4 n'engendre
  pas de classe pour un jeton absent ; le build du 2026-10-06 contient **0** règle `destructive` dans ses sept
  feuilles. Le bouton n'a ni fond ni couleur : il hérite du texte courant. 201 occurrences du jeton dans 42
  fichiers sont dans le même cas (33 bandeaux d'erreur compris).
- **Retenu** : corriger à la racine — définir le jeton (AC1) **et** refaire la variante (AC2).
- **Écartées** : ne corriger que la variante avec des couleurs de palette (`bg-red-700`) — laisserait les autres
  usages du jeton (201 occurrences, 42 fichiers) sans effet, et le défaut d'origine en place.
- **Réversible** : oui.

### C-28-1-2 — Fond plein plutôt que contour

- **Contexte** : l'epic laisse le choix « fond plein ou contour marqué ».
- **Retenu** : fond plein `bg-destructive`, texte blanc — la convention shadcn d'origine, la plus forte distinction
  d'avec *Annuler* (`outline`) et d'avec un texte d'erreur, et la seule qui rende l'action irréversible saillante
  dans un dialogue.
- **Écartées** : contour rouge (bordure + texte) — reste proche d'un cadre d'erreur (`border-destructive
  text-destructive`, 33 bandeaux) ; deux variantes (pleine + contour) — élargit la story sans besoin établi.
- **Réversible** : oui (une ligne).

### C-28-1-3 — `#b91c1c` (rouge 700), distinct de `--color-error` (`#dc2626`)

- **Contexte** : réutiliser `--color-error` était le plus simple.
- **Mesuré** : `#dc2626` tient sur le bouton (blanc dessus 4,83:1) mais **échoue** sur les 33 bandeaux
  `bg-destructive/10 text-destructive` (4,13:1 < 4,5) que le jeton activera ; `#b91c1c` : bouton 6,47:1, bandeau
  `/10` 5,45:1, `/20` 4,59:1, texte sur blanc 6,47:1.
- **Retenu** : `--color-destructive: #b91c1c`, `--color-destructive-foreground: #ffffff`. `--color-error` inchangé.
- **Écartées** : `#dc2626` (AA perdu sur les bandeaux, et les specs axe pourraient rougir) ; `#991b1b` au repos
  (8,31:1 mais trop proche du brun, survol plus foncé impossible à distinguer).
- **Réversible** : oui (une valeur).

### C-28-1-4 — Survol : un jeton dédié plus foncé

- **Retenu** : `--color-destructive-hover: #991b1b` (blanc dessus 8,31:1), `hover:bg-destructive-hover`.
- **Écartées** : `hover:bg-destructive/90` (éclaircit, 5,58:1 mesuré, différence peu perceptible et contraste en
  baisse) ; pas de survol (la variante `default` n'en a pas sur `<button>`, mais l'issue demande des états
  distincts).
- **Réversible** : oui.

### C-28-1-5 — Désactivé : l'opacité commune, exemptée par WCAG, et « grisé » → « désactivé » au manuel

- **Contexte** : la consigne demande un contraste mesuré pour l'état désactivé.
- **Mesuré** : `disabled:opacity-50` de la base rend un fond effectif `#dc8e8e`, texte 2,53:1. WCAG 1.4.3 exempte
  le texte d'un composant inactif ; l'état reste distinct du repos (6,47:1).
- **Retenu** : garder l'opacité commune à toutes les variantes ; le manuel cessait d'être exact en disant
  « grisé » (c'est un rouge estompé) — deux phrases passent à « désactivé » (AC10).
- **Écartées** : un désactivé gris dédié (`disabled:bg-secondary`) — une seconde règle pour une seule variante,
  et l'action perd son identité ; un désactivé à 4,5:1 — se confondrait avec l'état actif.
- **Réversible** : oui.

### C-28-1-6 — Focus : le contour global bleu, vérifié au style calculé

- **Contexte** : la base porte `outline-none` et `focus-visible:ring-ring/50` (jeton `ring` absent, sans effet).
- **Lecture** : la règle `:focus-visible { outline: 2px solid #3b82f6; outline-offset: 2px }` d'`app.css:62` est
  hors couche et l'emporte sur `outline-none` (`@layer utilities`). Contraste contre la page 3,68:1 (≥ 3).
- **Retenu** : compter sur elle, ajouter `focus-visible:ring-destructive/40` à la variante, et **le prouver** à
  l'E2E (AC8 a) ; correction prescrite si la lecture est fausse.
- **Réversible** : oui.

### C-28-1-7 — « Mode sombre » : il n'existe pas ; la variante ne porte aucune classe `dark:`

- **Contexte** : l'epic demande des captures clair et sombre. Kesh n'a pas de thème sombre
  (`sonner.svelte:3`), mais le variant `dark` de Tailwind v4 est `@media (prefers-color-scheme: dark)` — vérifié
  dans le build (`dark:bg-amber-950/30`, `dark:bg-green-900`… émis sous cette requête).
- **Piège** : la variante actuelle porte `dark:bg-destructive/20` ; une fois le jeton défini, elle rendrait le
  bouton **pâle à nouveau** chez qui a son système en sombre.
- **Retenu** : aucune classe `dark:` dans la variante (garde AC7, mutation M3) ; l'E2E rejoue chaque scénario sous
  `colorScheme: 'dark'` émulé et exige le même rendu.
- **Écartée** : instaurer un thème sombre — hors périmètre.
- **Réversible** : oui.

### C-28-1-8 — Les autres usages du jeton deviennent rouges, sans édition de site

- **Contexte** : définir le jeton change l'apparence des bandeaux, textes et bordures d'erreur, et des champs
  `aria-invalid`.
- **Retenu** : l'accepter — c'est leur intention d'origine, et le contraste est garanti par C-28-1-3. Contrôle
  indépendant : les dix specs E2E qui lancent axe restent vertes.
- **Écartée** : neutraliser ces usages (les écrire en `text-error`) — des dizaines d'éditions pour garder un défaut.
- **Réversible** : oui (avec le jeton).

### C-28-1-9 — Sites stylés à la main : B et D migrés, C et E laissés et inscrits

- **Retenu** : migrer vers la variante les 2 `<Button class="bg-red-600 …">` (DRY) et les 3 `<button>` sans bordure
  qui se lisent comme du texte — dont *Annuler le rapprochement*, sur l'écran de la recette. Laisser les 6
  déclencheurs **bordés** (ils se lisent comme des boutons) et les 3 icônes ✕ de retrait de ligne, inscrits avec
  leur motif à la table fermée de la garde AC6.
- **Écartées** : tout migrer (6 boutons rouges pleins de plus dans des lignes de tableau, chargés visuellement, et
  plus de cinq modules touchés) ; une variante `destructive-outline` pour la classe C (élargit la story ; à
  proposer par issue si l'harmonisation est voulue).
- **Réversible** : oui.

### C-28-1-10 — Recette visuelle : style calculé + axe + captures non comparées

- **Contexte** : B2 de la rétrospective de l'Epic 27 — un test qui trouve un bouton ne prouve pas qu'on le voit.
- **Retenu** : une spec Playwright qui lit `getComputedStyle` (fond non transparent, valeur exacte, contraste
  recalculé ≥ 4,5, survol, focus, désactivé, clair et sombre), lance axe `color-contrast` sur les dialogues, et
  prend des captures **avant** ses assertions — « avant » sur le code inchangé (T0), « après » à la fin — dans
  `kesh-gate-logs/28-1-captures/`, hors dépôt.
- **Écartée** : `toHaveScreenshot` — références au pixel dépendantes de la plateforme et des polices, PNG à
  versionner et à régénérer, aucun précédent au dépôt, et la CI ne lance pas l'E2E ; il mesure la ressemblance,
  non la visibilité. Les propriétés calculées visent exactement le défaut (fond transparent).
- **Réversible** : oui.

### C-28-1-11 — Deux gardes Vitest, parce que la CI n'exécute pas l'E2E

- **Retenu** : AC7 (variante, jetons définis, contrastes recalculés depuis `app.css`) et AC6 (inventaire clos des
  boutons rouges posés à la main, par `svelte/compiler`, patron `bank-name-inventory.test.ts`). La CI lance
  `npm run test:unit` (`ci.yml:214`), non Playwright (#577).
- **Écartée** : une garde par expression régulière sur les balises (`onclick={() => …}` contient `>`).
- **Réversible** : oui.

### C-28-1-12 — Un seul calcul de contraste, dans `tests/e2e/helpers/contrast.ts`

- **Retenu** : fonctions pures partagées par la spec E2E et la garde AC7, testées par `contrast.test.ts` (Vitest
  ramasse `tests/**/*.test.ts`, Playwright l'ignore par `testMatch`).
- **Écartées** : deux copies (DRY) ; le placer dans `src/lib/` (code de test dans le produit, et aucune spec E2E
  n'importe aujourd'hui de `src/`).
- **Réversible** : oui.

### C-28-1-13 — Les neuf autres jetons shadcn absents : hors périmètre, issue à ouvrir

- **Contexte** : `muted`, `muted-foreground`, `foreground`, `background`, `ring`, `input`, `accent`,
  `accent-foreground`, `card` ne sont pas définis non plus ; `outline` et `ghost` n'ont pas de survol.
- **Retenu** : ne pas les traiter ici (ils ne masquent aucune fonction : *Annuler* garde sa bordure) ; demander à
  l'orchestrateur une issue P3. Même chose pour la suppression sans confirmation d'une personne de contact.
- **Réversible** : —.

### C-28-1-14 — La spec UX nomme le rouge d'action

- **Retenu** : une ligne à la palette de `ux-design-specification.md` et deux ratios mesurés (AC11) : la palette
  est la référence du thème, elle ne doit pas ignorer un jeton du thème.
- **Réversible** : oui.

### C-28-1-15 — Pas de découpage

- **Retenu** : cinq modules de premier niveau au plus, aucune règle métier, aucune migration ; les 24 sites de la
  variante ne sont pas édités.
- **Réversible** : —.

### C-28-1-16 — Le montage d'import des E2E extrait dans un helper

- **Retenu** : `login`, `mount`, `importByScreen` de `bank-import-delete.spec.ts:35-85` passent dans
  `tests/e2e/helpers/bank-import-fixtures.ts`, importé par cette spec et par la neuve (DRY ; une spec importée par
  une autre y enregistrerait ses tests deux fois).
- **Réversible** : oui.
