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
  *(Rectifié en validation P1 : 32 bandeaux hors `components/ui/` — C-28-1-25.)*
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
  *(Rectifié en validation P1 : 32 bandeaux — C-28-1-25.)*
- **Réversible** : oui (une ligne).

### C-28-1-3 — `#b91c1c` (rouge 700), distinct de `--color-error` (`#dc2626`)

- **Contexte** : réutiliser `--color-error` était le plus simple.
- **Mesuré** : `#dc2626` tient sur le bouton (blanc dessus 4,83:1) mais **échoue** sur les 33 bandeaux
  `bg-destructive/10 text-destructive` (4,13:1 < 4,5) que le jeton activera ; `#b91c1c` : bouton 6,47:1, bandeau
  `/10` 5,45:1, `/20` 4,59:1, texte sur blanc 6,47:1.
  *(Rectifié en validation P1 : 32 bandeaux, non 33 ; `/20` vaut 4,57:1, et n'est plus cité — C-28-1-25.)*
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
  *(Rectifié en validation P1 : 16 fichiers exercent `color-contrast` — C-28-1-25.)*
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
- **Précision (validation P1, 2026-10-13)** : les deux issues existent — **#645** (jetons non définis, P3 ; la
  lentille R y ajoute `error-soft`, `warning-soft`, `danger`) et **#646** (confirmation, P3). Un élargissement de
  la 28-1 à ces deux sujets a été envisagé puis **annulé** par l'orchestrateur (C-28-1-17) : le choix tient.

### C-28-1-14 — La spec UX nomme le rouge d'action

- **Retenu** : une ligne à la palette de `ux-design-specification.md` et deux ratios mesurés (AC11) : la palette
  est la référence du thème, elle ne doit pas ignorer un jeton du thème.
- **Réversible** : oui.

### C-28-1-15 — Pas de découpage *(remplacé par C-28-1-24 en validation P1)*

- **Retenu** : cinq modules de premier niveau au plus, aucune règle métier, aucune migration ; les 24 sites de la
  variante ne sont pas édités.
- **Réversible** : —.

### C-28-1-16 — Le montage d'import des E2E extrait dans un helper

- **Retenu** : `login`, `mount`, `importByScreen` de `bank-import-delete.spec.ts:35-85` passent dans
  `tests/e2e/helpers/bank-import-fixtures.ts`, importé par cette spec et par la neuve (DRY ; une spec importée par
  une autre y enregistrerait ses tests deux fois).
- **Réversible** : oui.

---

### Choix de la remédiation de la validation P1 (2026-10-13)

Rapports `kesh-gate-logs/28-1-validate-p1-{R,F}.md` (hors dépôt). Bilan au Change Log de la fiche.

### C-28-1-17 — Périmètre : #640 seulement (décision de l'orchestrateur)

- **Contexte** : l'orchestrateur avait proposé aux lentilles P1 d'examiner un élargissement — définir les neuf
  autres jetons shadcn, et ajouter une confirmation à la suppression d'une personne de contact. Les lentilles ont
  mesuré qu'il n'est pas neutre : `input` sert à la fois de bordure et de fond désactivé (R1), le risque visuel
  touche l'écran entier sans filet (R2), `muted-foreground` au ton shadcn échoue AA sur `muted` (R3, F3), les
  `dark:` inertes s'activeraient (F4), la confirmation engage i18n, test de composant et manuel (R7, F5), et le
  tout porte la story au-delà de cinq modules (R6).
- **Retenu (décision de l'orchestrateur, non de l'agent)** : l'élargissement est **annulé**. La 28-1 se limite
  au jeton `destructive` (et `-foreground`, `-hover`), à la variante, aux boutons rouges stylés à la main
  (AC5), aux gardes et à la recette visuelle. Les autres jetons → **#645** ; la confirmation → **#646**. La fiche
  les cite comme angles morts 1 et 2, avec renvoi.
- **Effet sur les findings** : F3, F5, R1, R2, R3, R7, R8 se ferment par le retrait du périmètre ; F4 n'est
  traité que pour la part qui touche la variante. Le commentaire déjà posté sur #640 (« la Story 28-1 traite
  l'ensemble ») est devenu faux : à rectifier par l'orchestrateur.
- **Réversible** : oui — #645 et #646 restent ouvertes et portent les mesures des lentilles.

### C-28-1-18 — Les gardes AC7 (1)(2) lisent la variante seule

- **Contexte** (F1, HIGH) : `buttonVariants({ variant: 'destructive' })` rend base + variante fusionnées ; la base
  porte `dark:aria-invalid:…`, `bg-clip-padding`, `text-sm`, `focus-visible:border-ring` : « aucun `dark:` » et
  « tout jeton est défini » auraient rougi au premier lancement, sur des classes hors story.
- **Retenu** : lire `buttonVariants.variants.variant.destructive` (exposé par tailwind-variants,
  `chunk-RZF76H2U.js:254`) ; à défaut, la ligne `destructive:` de `button.svelte`. La sortie fusionnée ne sert qu'à
  (1-bis) : présence des classes de la variante, absence de `focus-visible:border-ring` remplacée.
- **Écartée** : filtrer la sortie fusionnée en retirant les classes de la base — une liste de plus à tenir, qui
  dérive avec `button.svelte`.
- **Réversible** : oui.

### C-28-1-19 — Valeurs lues après stabilisation ; `parseRgb` refuse ce qu'il ne sait pas lire

- **Contexte** (F2 = R9, MEDIUM ; F8, LOW) : la base porte `transition-all` (150 ms) ; une lecture juste après
  `hover()`/`focus` rend une couleur intermédiaire. Et Chromium rend une couleur `color-mix(in oklab, …)` sous une
  syntaxe `oklab(`/`color(` que `parseRgb` ne lit pas.
- **Retenu** : `expect(locator).toHaveCSS(...)` pour toute valeur ; `expect.poll` quand la valeur alimente un
  calcul de contraste. `parseRgb` lit `rgb(`/`rgba(` et **lève** sur toute autre syntaxe en nommant la chaîne ;
  tests unitaires des deux formes, des deux refus et de trois ratios connus.
- **Écartées** : `page.waitForTimeout(200)` (dépend de la durée, fragile) ; désactiver les transitions par une
  feuille injectée (la spec mesurerait un rendu que l'utilisateur n'a pas).
- **Réversible** : oui.

### C-28-1-20 — La variante fixe sa propre bordure de focus

- **Contexte** (R11, MEDIUM) : la base porte `focus-visible:border-ring` ; `ring` n'est pas défini (#645), si bien
  que la bordure de focus reste transparente aujourd'hui — et deviendrait **bleue** sur un bouton rouge plein le
  jour où #645 définira `ring`, sans que rien ne le décide.
- **Retenu** : `focus-visible:border-destructive` dans la variante (tailwind-merge remplace la classe de la base).
  Le focus de la variante ne dépend plus d'aucun jeton absent : anneau rouge 40 % (non asserté : syntaxe
  `color-mix`), bordure `rgb(185, 28, 28)` (assertée, AC3, AC8 (a)), contour global bleu (asserté). Gardes AC7 (1)
  et (1-bis), mutation M19.
- **Écartées** : retirer `focus-visible:border-ring` de la base (touche toutes les variantes, relève de #645) ;
  ne rien faire et l'écrire en angle mort (le défaut se déclencherait par une story sans rapport).
- **Réversible** : oui (une classe).

### C-28-1-21 — *Annuler le rapprochement* du détail d'import passe à `outline`, comme sur la fiche facture

- **Contexte** (R14, LOW, à trancher) : la fiche prévoyait `destructive xs` pour `detail-tx-cancel-reconciliation`,
  alors que le même bouton est `outline sm` sur la fiche de la facture (`InvoiceSettlements.svelte:106-111`) et que
  le manuel dit « le même bouton apparaît aussi sur la fiche de la facture » (`user-manual.tex:2019-2020`).
- **Retenu** : `<Button variant="outline" size="sm" class="ml-2">`. C'est un déclencheur : il ouvre
  `CancelReconciliationDialog`, dont la confirmation est `destructive` (classe A) — même hiérarchie que les
  déclencheurs neutres des Dev Notes. Le défaut de #640 (« se lit comme du texte », souligné à côté d'un lien) est
  corrigé par la forme de bouton ; le manuel reste vrai ; aucun module de plus.
- **Écartées** : `destructive` sur les deux écrans (un bouton rouge plein par ligne de tableau — écarté pour la
  classe C par C-28-1-9 —, et `features/invoices` en huitième module) ; garder les deux formes et documenter la
  différence (le manuel promet l'identité).
- **Trié, non tranché** : la confirmation `invoice-settlement-cancel-confirm` (`invoices/[id]/+page.svelte:1666-1674`)
  est `default`, quand les deux autres confirmations de la page sont `destructive`. Elle se voit ; ce n'est pas
  #640. Signalée à l'orchestrateur, sans demande d'issue.
- **Réversible** : oui (une valeur de `variant`).

### C-28-1-22 — Garde AC6 : clé non ambiguë, seconde table des sites migrés, assertion (e)

- **Contexte** (R10, MEDIUM ; F9, LOW) : la clé `fichier:élément` confondait les deux `<button>` de
  `supplier-invoices/[id]` ; un site migré rendu à un `<button class="ml-2">` nu ou à un `Button` sans `variant`
  ne rougissait rien ; deux des cinq sites n'étaient vus par aucun test visuel.
- **Retenu** : clé `chemin#data-testid` (texte source, gabarit compris), sinon `chemin@élément:rang` ; échec si
  deux clés coïncident ; pas de numéro de ligne. Seconde table fermée des cinq sites migrés (élément `Button`,
  `variant` et `class` littéraux), mutation M18. Assertion (e) dans `reconciliation-cancel.spec.ts:139`
  (`border-top-width: 1px`, `border-top-color` de `--color-border` — non `border-top-style`, déjà `solid` par le
  préflight). `contact-person-delete-{id}` : par transitivité (garde structurelle + scénario (c)), angle mort 8.
- **Écartées** : clé par numéro de ligne (rougit sur toute édition sans rapport) ; monter une personne de contact
  en E2E (dialogue d'édition, montage lourd pour un bouton que la garde structurelle tient déjà).
- **Réversible** : oui.

### C-28-1-23 — Garde d'ensemble clos des classes sans jeton, facultative, en table fermée renvoyée à #645

- **Contexte** (R4, MEDIUM) : « neuf jetons » était une liste ; la lentille R en a trouvé trois de plus
  (`error-soft`, `warning-soft`, `danger`). Une liste de formes ne révèle pas ce qu'elle omet.
- **Retenu** : AC7 (4), **facultatif** (décision de l'orchestrateur) : l'inventaire des classes de couleur que
  **Tailwind lui-même** ne résout pas (`@tailwindcss/node` : `compile` + `build`, ou
  `__unstable__loadDesignSystem(...).candidatesToCss`), comparé à une table fermée de douze noms renvoyés à #645 ;
  rouge dans les deux sens (nom neuf, nom devenu défini). La table ne se remplit pas depuis la sortie de la garde :
  tout écart avec la liste des lentilles est examiné nom par nom. Abandonnée, et dite, si l'API ne se charge pas.
- **Écartée** : une liste de noms à exclure (palette Tailwind, mots-clés, utilitaires non chromatiques) — une
  énumération de plus, ouverte par nature.
- **Réversible** : oui (fichier de test isolé).

### C-28-1-24 — Sept modules : dérogation à la règle de découpage (remplace C-28-1-15)

- **Contexte** (R6, MEDIUM) : C-28-1-15 disait « cinq modules au plus » ; sa propre liste en comptait sept.
  Recompté sur le périmètre réduit : sept modules de code (`ui/button`, `app.css`, `features/contacts`,
  `features/bank-import`, `routes/(app)/bank-import`, `routes/(app)/settings`, `routes/design-system`).
- **Retenu** : ne pas découper, et l'écrire (fiche, § *Dérogation règle de splitting*) : cinq des sept modules
  reçoivent une édition mécanique d'une à trois lignes ; le découpage prescrit (patron puis rollout) scinderait
  la garde AC6, qui est l'inventaire des sites ; le rollout est revu fichier par fichier. Risque accepté écrit.
  Signal déclaré au Project Lead par l'orchestrateur.
- **Écartée** : 28-1 (jetons, variante, gardes) + 28-1b (cinq sites) — deux tables AC6 à écrire deux fois, et
  l'issue ne se clôt qu'à la seconde.
- **Réversible** : oui, tant que le développement n'a pas commencé.

### C-28-1-25 — Décomptes rectifiés (validation P1)

- **R13** : 33 → **32** bandeaux hors `components/ui/` (`grep -rn 'bg-destructive/10' src --include=*.svelte
  --include=*.ts | grep -v '\.test\.' | grep -v components/ui | wc -l` → 32 ; les 32 portent aussi
  `border-destructive` et `text-destructive`). Vaut pour C-28-1-1, C-28-1-2, C-28-1-3.
- **R12** : `/20` → **4,57:1** (mélange `(241 ; 209,6 ; 209,6)`), et retiré du commentaire d'`app.css` et de
  l'AC4 : plus aucun usage applicatif. Vaut pour C-28-1-3.
- **F7** : 5,45:1 est exact sur le mélange non arrondi `(248 ; 232,3 ; 232,3)` — ce que rend Chromium et ce que
  calcule `blend` ; 5,46 si l'on arrondit à `#f8e8e8`. Écrit à l'AC1.
- **R5** : « dix specs axe » → **19** fichiers à `.analyze(` ; **16** exercent `color-contrast` au moins une fois
  (15 sans `disableRules`, plus `invoices` par `:96` et `:231` — la lentille R en comptait 15) ; **3** la
  neutralisent (`users:134`, `email-templates:203`, `reminders:298`). Vaut pour C-28-1-8.
- **Bilan de la lentille R** : déclaré 2 HIGH / 8 MEDIUM / 6 LOW, titré 2 / 9 / 5 (même total de 16).
- **Réversible** : —.
