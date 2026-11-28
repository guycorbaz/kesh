# Story 58-5 : Cosmétique des écrans — colonnes collées, débordements, contraste (#253, #933, #996, #1008, #1015, #1016, #1153, #1154, #1184, #1549)

Status: drafted

*Fiche rédigée le 2026-11-28 (Sonnet 5.5, en autonomie), workflow `bmad-create-story`. Worktree `/home/gcorbaz/devel/kesh-58-5`, branche
`story/58-5-cosmetique-ecrans` partie de `main` à `287deea33` (v0.20.1). Epic 65 (nettoyage, suite de l'Epic 58), release visée : vendredi 2026-12-04
(numéro attribué au tag). **Aucune migration, aucune règle métier, aucun changement d'API.** Du balisage, des classes CSS, un jeton de couleur, un libellé
fr-CH, quelques tests et une phrase du manuel ; aucune compilation ni aucun gate n'a été lancé pour écrire la fiche (un agent de fiche n'exécute ni `docker` ni `curl` : ce qui
doit être vu à l'écran est prescrit au développement). Ratios de contraste calculés à la main (formule WCAG 2.x, script jetable hors dépôt) : le
développement les **recalcule** avec `tests/e2e/helpers/contrast.ts`.*

## Story

En tant qu'utilisateur de Kesh, je veux que les colonnes de montant ne touchent plus le texte qui les suit (aperçu d'import, réconciliation, journal d'audit), que les
valeurs longues se coupent dans leur cadre (avis de conflit de la société, tuile des comptes bancaires, champ de compte) et que les textes d'avertissement et les
chips sélectionnés soient lisibles au seuil AA, afin de lire les écrans sans deviner où finit un mot et où commence l'autre.

## Contrôle de couverture (règle du 2026-11-26 : on vérifie qu'une issue n'est pas déjà couverte avant de la retenir)

Chaque issue a été relue (`gh issue view`, lecture seule) puis cherchée au code de `287deea33` (`grep -n`, lignes lues). Les issues voisines ouvertes : #645
(jetons de couleur non définis, dont `bg-warning-soft`) est **hors de cette story** (une issue par problème) ; aucun commit depuis le tag précédent ne cite
l'une des dix issues (`git log 287deea33..HEAD --format=%B | grep -oE '#(253|933|996|1008|1015|1016|1096|1153|1154|1184|1549)\b'` : vide au 2026-11-28, à rejouer au développement). Les doublons #1548, #1550 et #1551 sont déjà fermés vers #1154, #933 et #1016.

| # | Constat relu au code | Verdict |
|---|---|---|
| #253 | Le fond `bg-primary-light` (#3b82f6) porte du texte foncé sur **cinq sites** : `settings/email-templates/+page.svelte:253`, `:276`, `:319` (chips type, niveau, onglets de langue), `:298` (badge « Personnalisé », texte `text-primary`), `invoices/due-dates/+page.svelte:498` (onglets de statut). Les sites que l'issue listait en plus — `(app)/+layout.svelte` (nav), `onboarding/*`, `bank-accounts/+page.svelte` — n'emploient plus le fond plein : `hover:bg-primary-light/10`, `/5`, `/20` avec `text-primary` (≥ 6,9:1), **hors défaut**. `design-system/+page.svelte:41` : étiquette `text-white` sur `bg-primary-light` (3,68:1), page de démonstration des jetons, hors des écrans de l'utilisateur (voir D3). Le contournement existe toujours : `email-templates.spec.ts:203` et `reminders.spec.ts:301` ne passent qu'avec `.disableRules(['color-contrast'])`. | **gardée** |
| #933 | `bank-import/BankImportUpload.svelte:872-873` et `:880-881` (aperçu, table `preview-tx-table`) : `<th class="text-right">Montant</th>` puis `<th class="text-left">Détails</th>`, **aucun** espacement de cellule (Tailwind en préflight remet `padding` à zéro). Même motif au tableau des lignes signalées « ressemblent à une ligne déjà importée » : `:685-686` et `:698-699` (colonne Montant à gauche, suivie de Détails). | **gardée** (deux tableaux du même écran) |
| #996 | `app.css:27` : `--color-warning: #d97706` = **3,19:1** sur blanc (recalcul : 3,19). `text-warning` : **18 sites** (le relevé de l'issue disait 11 ; inventaire en D2). Aucun test de contraste du jeton. | **gardée** |
| #1008 | `homepage/BankAccountsCard.svelte:37` : `<div class="min-w-0 flex-1">` (nom du compte, `<p class="font-medium">` à `:38`) à côté de `:42` `<div class="text-right">` sans borne de largeur : sa largeur est celle de la ligne la plus longue, « Solde non disponible — lier au plan comptable » (`:58-62`, une seule ligne ; branche `currentBalance === null` à `:43`), et le nom, `min-w-0`, déborde dessus. | **gardée** (mécanisme à **confirmer** à la capture d'avant, D5) |
| #1015 | `settings/CompanyConflictNotice.svelte:37` (`text-warning`) et `:44` (`text-text-muted`) : `<p class="mt-1 text-xs …">` portant `{ $value }` — ni `break-words` ni `wrap-anywhere`. Les parents sont, pour l'adresse, des cellules de grille (`settings/+page.svelte:827-840`, dont la seule rue est en `col-span-2`) et, pour l'e-mail (le cas de l'AC10), un `<div class="flex-1">` dans un `flex` (`:907-908`) : dans les deux, un mot long élargit le conteneur. | **gardée** |
| #1016 | `reconciliation/ReconciliationProposals.svelte:708` (`th text-right` Montant) et `:756` (`td text-right`), suivis de Contrepartie `:709`, `:757` : aucun espacement. | **gardée** |
| #1096 | `invoices/[id]/+page.svelte:1093` (fiche validée) et `:1229` (annulée) : `<div class="flex min-w-0 flex-wrap justify-end gap-2">` ; le commentaire `:1089-1092` le dit (« passe à la ligne au lieu de couper « Dévalider » »). Introduit par `773df80da` (v0.19.0, 2026-11-18), **quatre jours après** l'ouverture de l'issue (2026-11-14, qui citait `:1030` d'avant). Le brouillon (`:1069`, `flex gap-2`) n'a que trois boutons. | **déjà corrigée** — sort de la story, preuve `invoices/[id]/+page.svelte:1093` ; **fermée le 2026-11-28, preuve en commentaire** ; capture de confirmation à 1400 px et 1024 px à la recette (D6), non bloquante |
| #1153 | Deux demandes. (1) « MontantContrepartie » : **aucune** chaîne soudée au catalogue (`grep -rn MontantContrepartie` vide hors fiches) : le mot soudé est l'effet visuel de #1016, **même défaut**, corrigé avec lui. (2) « Candidate » : `crates/kesh-i18n/locales/fr-CH/messages.ftl:1718` `reconciliation-cols-candidate = Candidate` ; repli `ReconciliationProposals.svelte:710`. En français, « candidate » est un mot juste (le manuel l'emploie comme nom : `user-manual.tex:3822`, `:3893`) mais, **seul en titre de colonne**, il ne dit pas de quoi : la colonne porte la facture ou la règle proposée. La demande est celle de l'auteur ; D1. | **gardée** (la moitié (2) seule, (1) étant #1016) |
| #1154 | `audit-log/+page.svelte:362` (`th py-2 text-right`, « N° ») et `:387` (`td py-2 text-right`), suivis de Détails `:363`, `:388`. | **gardée** |
| #1184 | `journal-entries/AccountAutocomplete.svelte:508-536` : `<Input>` sans marge droite, le × (`:551`, `absolute right-2`) se pose sur la fin du texte. `Input` (`ui/input/input.svelte`) passe sa classe à `cn` (tailwind-merge) : `pr-9` l'emporte sur le `px-2.5` de base. Consommateurs de `allowClear` : `products/+page.svelte:779`, `GroupSettlementModal.svelte:962`, `InvoiceForm.svelte:1190`. | **gardée** |

| #1549 | `bank-import/[id]/+page.svelte:170-171` (`th` Montant `text-right` puis Référence) et `:183-184` (`td`) : aucun espacement ; issue ouverte (`bug`, P4), **même module** (`bank-import`) et **même correctif** que #933 ; décision de l'orchestrateur du 2026-11-28 (validation P1, F4) : retenue. | **gardée**, ajoutée à la story |

**Bilan : dix issues retenues** (les neuf de l'epic moins #1096, plus #1549), **une sortie (#1096, déjà corrigée et fermée)**. Aucune ne révèle de défaut de production ni de règle métier. Dans la borne de cinq à dix issues par story. **#1552** (écriture, « Crédit » et « Compte analytique » collés) est du même genre mais d'un **autre module** (`journal-entries`) : **angle mort assumé**, elle reste ouverte, hors de cette story.

## Choix consignés (autonomie : aucune question à Guy)

- **C-58-5-1 — le jeton `--color-warning` s'assombrit à `#92400e`** (ambre 800 de Tailwind), au lieu d'un jeton neuf. Alternatives écartées : (a) `#b45309`
  (la piste de l'issue, 5,02:1 sur blanc) : **sous 4,5:1 sur sa propre teinte à 10 %** (4,39:1) et à 20 % (3,81:1) — or `InvoiceForm.svelte:925` pose
  `text-warning` sur `bg-warning/10` et `PaymentStatusBadge.svelte:62-63` sur `warning` à 20 % ; il aurait fallu retoucher ces sites ; (b) un jeton
  `--color-warning-text` à côté : 18 sites à reprendre, et deux couleurs d'avertissement. `#92400e` mesure **7,09:1** sur blanc, 6,78:1 sur `surface-alt`
  (#f8fafc), 6,56 / 6,08 / 5,16 / 4,99:1 sur sa propre teinte à 5 / 10 / 20 / 22 %. Texte `text-text-muted` sur teinte : `#64748b` sur `bg-warning/5` mesure 4,51:1 avec `#d97706` et **4,40:1** avec `#92400e` (la teinte à 5 % est plus sombre) ; sur `/10` 4,27 → 4,08. **Tout texte qui n'est pas `--color-text` et qui repose sur une teinte d'avertissement se mesure** (D2) ; un site qui tombe sous 4,5:1 se corrige par le patron AA-safe `text-text` (`settings/dunning/+page.svelte:295` en est un, AC4). Si le développement trouve un site que `text-text` ne règle pas, il choisit un jeton qui tient partout. Contrepartie visible : les **bordures** `border-warning` (une soixantaine
  de sites, D2) et les teintes `bg-warning/N` passent à un brun plus sombre ; la recette le montre (D6). *Réversible* : une ligne de `app.css`.
- **C-58-5-2 — le chip sélectionné prend `bg-primary text-primary-foreground`** (#1e40af / blanc = **8,72:1**), le patron déjà employé par
  `accounts/groups/+page.svelte:316` et les boutons primaires. Alternative écartée : éclaircir `--color-primary-light` — il sert aussi au contour de focus
  (`app.css:84`, `outline: 2px solid var(--color-primary-light)`) et aux survols `/10`, `/20` ; le changer élargit la story pour rien. Le **badge**
  « Personnalisé » (`:298`) devient `bg-primary-light/20 text-primary` (patron du badge de `bank-accounts/+page.svelte:1447` ; **6,92:1**), non plein : un
  badge n'est pas un état sélectionné.
- **C-58-5-3 — l'espacement des colonnes se pose par `pr-4` sur chaque colonne sauf la dernière** (en-tête **et** cellules), **dans les cinq tableaux que la
  story touche** (aperçu d'import, lignes signalées, détail d'import, réconciliation, journal d'audit) — jamais par une marge globale sur `th`/`td` (les autres tableaux de
  l'application ont déjà leurs `py-2 pr-2`/`pr-4`, ex. `reconciliation/wise/+page.svelte:133`). Même module, même correctif (décision de l'orchestrateur du
  2026-11-28) : les couples voisins du couple nommé par l'issue — journal d'audit, **toutes** les colonnes (`audit-log/+page.svelte:358-363`, `py-2` seul) ; lignes
  « ressemblent à… » (`BankImportUpload.svelte:683-687`) ; réconciliation, Contrepartie → Candidate → Score (`:709-711`) — se corrigent avec lui, sinon le même défaut
  reste à côté. **Pas au-delà de ces cinq tableaux** : #1552 et les autres tableaux collés de l'application restent hors de la story. Seize pixels, comme les tableaux voisins.
- **C-58-5-4 — « Candidate » devient « Proposition » en fr-CH seulement** (`Kandidat`, `Candidato`, `Candidate` restent justes dans leurs langues, et le
  français ne les imite pas). Le mot est celui que l'écran emploie ailleurs : l'astuce `reconciliation-labels-priority-hint` (« Une seule **proposition** est
  affichée par transaction », `ReconciliationProposals.svelte:697`). Cette story est la **seule** de l'epic hors 58-4 à écrire dans un catalogue : elle ne touche
  qu'**une ligne** de `fr-CH/messages.ftl` (1718), à la fusion un conflit se résout en gardant les deux côtés ; l'orchestrateur le sait (§ Questions).
- **C-58-5-5 — la valeur longue se coupe par `wrap-anywhere`** (Tailwind 4.2.2, présent dans `node_modules/tailwindcss/dist/lib.js`), non par `break-words`
  seul : `overflow-wrap: break-word` ne réduit pas la largeur minimale du contenu, donc un mot long continue d'élargir une cellule de grille ; `anywhere` le fait.

## Acceptance Criteria

1. **Chips (#253).** Les cinq sites de `bg-primary-light` en fond plein (`email-templates/+page.svelte:253`, `:276`, `:319`, `due-dates/+page.svelte:498`) n'en
   portent plus : l'état sélectionné est `bg-primary text-primary-foreground` ; le badge `:298` est `bg-primary-light/20 text-primary`. Un état non sélectionné
   ne change pas. Ratio mesuré par `contrastRatio` : **≥ 4,5:1** (attendu 8,72 et 6,92), écrit au Dev Agent Record.
   **Test et mutation** : (a) `due-dates-page.test.ts` (rendu Vitest) : l'onglet sélectionné porte `bg-primary` et `text-primary-foreground` et **non**
   `bg-primary-light` — *mutation* : remettre `class:bg-primary-light` à `due-dates/+page.svelte:498` ; (b) Playwright, `email-templates.spec.ts` : le chip *Type*
   sélectionné a pour `getComputedStyle` fond `rgb(30, 64, 175)` et texte `rgb(255, 255, 255)` (valeurs asserties, l'E2E ne recalcule aucun contraste) — *mutation* :
   remettre `class:bg-primary-light` à `email-templates/+page.svelte:253` ; (c) la garde de jeton `jetons-contraste.test.ts` ne prouve **que** le jeton, pas le site :
   elle ne compte pas comme test d'AC1.
2. **Axe réactivé (#253).** Le `.disableRules(['color-contrast'])` de `email-templates.spec.ts:203` et celui de `reminders.spec.ts:301` sont **retirés**, et les
   deux tests passent. Si la règle remonte un autre défaut de contraste hors des chips (`reminders-list` porte des badges), il s'ouvre en **issue** (« on trace »)
   et le `disableRules` de ce seul test reste, avec son commentaire corrigé pour citer la nouvelle issue au lieu de #253 ; le commentaire de #253 disparaît dans
   les deux cas. Les onglets de langue et le badge `data-testid="email-template-badge"` (hors du `.include` actuel) passent sous axe dans un test neuf.
   **Mutation** : remettre `class:bg-primary-light` au chip sélectionné : l'axe (3,98:1) rougit (AC1 le prouve déjà par la valeur calculée ; l'axe le double).
3. **Jeton d'avertissement (#996).** `--color-warning` vaut `#92400e` (`app.css:27`), et une garde Vitest nouvelle (`jetons-contraste.test.ts`, patron de
   `account-autocomplete-classes.test.ts`, lisant `app.css` par `blocTheme`/`jetonDans`) établit que ce jeton atteint **≥ 4,5:1** sur `--color-surface`, sur
   `--color-surface-alt` **et** sur sa propre teinte à 5, 10, 20 et 22 % posée sur `--color-surface` (les alphas des sites : `login` 5 %, `InvoiceForm` 10 %,
   `PaymentStatusBadge` 20 %, badges `color-mix` 22 %), et que `--color-text-muted` atteint ≥ 4,5:1 sur cette teinte à 5 %. La garde lit le jeton **dans `app.css`**
   (une valeur écrite en dur dans le test ne prouverait rien). **Mutation** : remettre `#d97706` (puis `touch`) : rouge à 3,19:1.
4. **Textes sur teinte d'avertissement (#996, effet de bord).** L'inventaire de D2 est parcouru, **pour tout texte qui n'est pas `--color-text`** et repose sur un
   fond teinté par le jeton, relevé au navigateur par `getComputedStyle` : les 18 sites `text-warning` **et** les textes `text-text-muted` ou autres sur
   `bg-warning/N` ou `color-mix(… var(--color-warning) N%…)` (dont `.row-overdue`, `due-dates/+page.svelte:677`, et `settings/dunning/+page.svelte:295`) mesurent chacun
   **≥ 4,5:1** ; le tableau des mesures va au Dev Agent Record. `settings/dunning/+page.svelte:295` passe de `text-text-muted` à `text-text` (4,40:1 sinon,
   avec `#92400e`) ; tout autre site sous le seuil est corrigé de même, ou le jeton est remplacé par un qui tient partout (C-58-5-1). **Test et mutation** :
   `jetons-contraste.test.ts` mesure `--color-text-muted` sur la teinte à 5 % (mutation : jeton `#d97706`, rouge à 4,51 → seuil non tenu avec `#92400e` seul) ;
   l'assertion `text-text` de `dunning/+page.svelte:295` se pose dans le test de rendu de cette page s'il existe, sinon en Playwright (`getComputedStyle(...).color`
   = `rgb(30, 41, 59)`) — *mutation* : remettre `text-text-muted` à la ligne 295.
5. **Aperçu d'import (#933).** Dans `BankImportUpload.svelte`, **chaque colonne sauf la dernière** des deux tableaux (principal `:871-881` ; lignes signalées
   `:683-700`, C-58-5-3) porte `pr-4` en en-tête **et** en cellule : à 1280 px, « 100.00 CHF » et son libellé sont séparés d'au moins 16 px ; aucun test existant
   ne casse. **Test et mutation** : `BankImportUpload.test.ts` : le `th` **et** le `td` de la colonne Montant portent `pr-4` — *mutation* : retirer `pr-4` du `td`
   seul, puis du `th` seul (chacune rougit).
6. **Réconciliation (#1016).** `ReconciliationProposals.svelte:704-711` et `:752-770` : `pr-4` en en-tête **et** en cellule sur chaque colonne sauf la dernière
   (Montant, Contrepartie, Candidate, Score compris) ; capture : « -1450 CHF » et « Régie du Lac » séparés, en-tête compris. **Test et mutation** :
   `ReconciliationProposals.test.ts` : le `th` et le `td` Montant portent `pr-4` — *mutation* : retirer `pr-4` du `td` seul.
7. **Libellé (#1153).** `fr-CH/messages.ftl:1718` : `reconciliation-cols-candidate = Proposition` ; repli de `ReconciliationProposals.svelte:710` :
   `'Proposition'` ; les trois autres catalogues **inchangés** ; le manuel (`docs/manual/fr/user-manual.tex:3893`) nomme la colonne (§ Tâches). La garde i18n
   (`i18n-keys.test.ts`, `sitesTotal`) ne bouge pas (une valeur change, aucune clé). **Test et mutation** : `ReconciliationProposals.test.ts` asserte que l'en-tête de
   la colonne se lit « Proposition » (rendu avec le repli) ; un test Rust sur le catalogue (`textes_coherents` ou un test `kesh-i18n` qui lit `fr-CH`) asserte la
   valeur — *mutations* : remettre « Candidate » au seul repli Svelte, puis au seul catalogue (chacune rougit un des deux).
8. **Journal d'audit (#1154).** `audit-log/+page.svelte:358-363` et `:374-388` : `pr-4` en en-tête et en cellule sur chaque colonne sauf la dernière (N° compris) ;
   capture : « N° » et « Détails » séparés. **Test et mutation** : `audit-log-page.test.ts` : le `th` et le `td` « N° » portent `pr-4` — *mutation* : retirer `pr-4`
   du `td` seul.
9. **Tuile des comptes bancaires (#1008).** À 1280 px **et** à 1920 px, le lien « Solde non disponible — lier au plan comptable » ne recouvre ni le nom ni
   l'identifiant court du compte : il passe à la ligne dans sa colonne. Correctif attendu (à confirmer à la capture d'avant, D5) : colonne de droite
   `min-w-0 max-w-[60%] text-right` (`BankAccountsCard.svelte:42`) et nom `break-words` (`:38`). **Test et mutation** : `HomepageCards.test.ts` : la colonne de droite
   d'un compte non lié porte `max-w-[60%]` et le nom `break-words` — *mutation* : retirer `max-w-[60%]`.
10. **Avis de conflit (#1015).** `CompanyConflictNotice.svelte:37` **et** `:44` portent `wrap-anywhere` : une adresse e-mail de 320 caractères sans espace se
    coupe dans son cadre à **375 px**. Mesure **au niveau de l'avis et de son conteneur**, avant et après, et non du document : `el.scrollWidth <= el.clientWidth` sur
    `[data-testid="settings-company-email-conflict-changed-email"]` et sur son conteneur (`<div class="flex-1">`, `settings/+page.svelte:908`). **Angle mort assumé :
    #708** (ouverte, P4 : la barre latérale fixe fait déborder la page entière à 390 px) rend `document.documentElement.scrollWidth` > `clientWidth` quel que soit le
    correctif ; il n'est pas mesuré ici. **Test et mutation** : `settings-page.test.ts` (rend les avis, p. ex. `settings-company-address-conflict-changed-city`) :
    l'avis porte `wrap-anywhere` — *mutation* : le retirer de la ligne 37 seule, puis de la 44 seule.
11. **Champ de compte (#1184).** Quand le × est affiché (`allowClear && !disabled && (value !== null || query !== '')`), le champ reçoit `pr-9` : le texte
    d'un libellé long se coupe avant le bouton, **sans** que la largeur du champ change quand le × est absent. La condition d'affichage n'est écrite qu'**une**
    fois (dérivée partagée par le × et la classe, DRY). **Test et mutation** : `AccountAutocomplete.test.ts` : avec le ×, le champ porte `pr-9` ; **sans** ×
    (`allowClear` faux, ou valeur vide), il ne le porte pas — *mutations* : poser `pr-9` inconditionnellement (rougit le test « sans ×») ; ne pas le poser (rougit « avec × »).
12. **Gate frontend vert** (`check`, `lint-i18n-ownership`, `test:unit`, `build`), **gate Rust ciblé** (D7), **E2E complet** à la dernière modification de code de la
    story : « vert » = les sept échecs de la KF-029 seuls (`docs/testing.md`, *Les échecs attendus*).
13. **Recette visuelle** (D6) : captures avant/après de chaque état listé, thème clair, base de démonstration, rangées dans
    `/home/gcorbaz/devel/kesh-gate-logs/e65/58-5/captures/`, jointes au Dev Agent Record et au prompt de la revue de code.
14. **Détail d'un import bancaire (#1549).** `bank-import/[id]/+page.svelte:167-184` : `pr-4` en en-tête et en cellule sur chaque colonne sauf la dernière
    (Montant et Référence, mais aussi Date, Détails, Statut) ; capture : « Montant » et « Référence » séparés. **Test et mutation** : `bank-import-detail-page.test.ts` :
    le `th` et le `td` Montant portent `pr-4` — *mutation* : retirer `pr-4` du `td` seul.

## Tasks / Subtasks

- [ ] **Tâche 0 — Contrôles d'entrée (AC : 12, 13).** `git log origin/main..HEAD` propre ; rejouer le grep des onze numéros dans les commits depuis le tag.
  - [ ] Monter la base de démonstration (`docs/testing.md` § *Base de dev jetable*, bases `kesh_585`, `kesh_e2e_585`, port 3655) ; prendre **toutes les captures
        « avant »** de D6 **avant** de toucher au code.
- [ ] **Tâche 1 — Contraste : jeton, chips, textes sur teinte (AC : 1-4).**
  - [ ] Écrire la garde `frontend/src/lib/components/ui/jetons-contraste.test.ts` **rouge d'abord** (jeton `#d97706` : 3,19:1), puis poser `#92400e`.
  - [ ] Mutation : remettre `#d97706` dans `app.css`, `touch` le fichier, vérifier le rouge, restaurer (un binaire/cache périmé garde la mutation).
  - [ ] Chips : `email-templates/+page.svelte:253`, `:276`, `:319`, `due-dates/+page.svelte:498` → `class:bg-primary` + `class:text-primary-foreground` ;
        badge `:298` → `bg-primary-light/20 text-primary`. Tests d'AC1 (rendu Vitest des échéances, valeur calculée Playwright), mutations jouées et restaurées.
  - [ ] Retirer les deux `.disableRules(['color-contrast'])` (AC2) ; ajouter le test axe des onglets de langue et du badge.
  - [ ] Mesurer au navigateur (`getComputedStyle`) **tout** texte non `--color-text` posé sur une teinte d'avertissement (D2) avec `contrast.ts` ; corriger
        `settings/dunning/+page.svelte:295` (`text-text`) ; écrire le tableau au Dev Agent Record.
- [ ] **Tâche 2 — Colonnes collées (AC : 5, 6, 8, 14).** `pr-4` sur chaque colonne sauf la dernière des cinq tableaux de D1, en-tête et cellule ; une assertion de
      classe **par `th` et par `td` Montant/N°** (jamais le seul `th`) dans `audit-log-page.test.ts`, `ReconciliationProposals.test.ts`, `BankImportUpload.test.ts`,
      `bank-import-detail-page.test.ts` ; mutations de l'AC jouées.
- [ ] **Tâche 3 — Libellé et manuel (AC : 7).** `fr-CH/messages.ftl:1718`, repli Svelte `:710` ; `user-manual.tex:3893` (colonne) **et** `:2814` (« bordé d'orange »,
      D4) ; **ne pas régénérer les PDF** (règle des fichiers communs : seul le `.tex` change dans une branche de story).
- [ ] **Tâche 4 — Débordements (AC : 9, 10, 11).** `BankAccountsCard.svelte`, `CompanyConflictNotice.svelte`, `AccountAutocomplete.svelte` ; une assertion de
      classe pour chacun (`HomepageCards.test.ts`, `settings-page.test.ts`, `AccountAutocomplete.test.ts`) et les mutations de l'AC ; mesures de largeur de D5.
- [ ] **Tâche 5 — Recette visuelle et gates (AC : 12, 13).** Captures « après », comparaison, gate frontend, gate Rust ciblé (D7), E2E complet (une fois, au dernier
      commit de code), Dev Agent Record.

## Dev Notes

### D1 — Sites exacts des colonnes (AC 5, 6, 7, 8, 14)

**Règle (C-58-5-3)** : `pr-4` sur **chaque colonne sauf la dernière** des cinq tableaux touchés, en-tête **et** cellule. Le tableau garde ses `text-left`/`text-right` ;
un `<td>` sans classe devient `<td class="pr-4">`.

| Écran | Fichier:ligne (en-tête, cellule) | Colonnes à marge |
|---|---|---|
| Aperçu d'import, table principale (`preview-tx-table`) | `BankImportUpload.svelte:871-873`, `:879-881` | Date, Montant (3 colonnes) |
| Aperçu d'import, lignes signalées (« ressemblent à… ») | `:683-687`, `:696-700` | Ligne, Date, Montant, Détails (5 colonnes) |
| Détail d'un import (`detail-tx-table`, #1549) | `bank-import/[id]/+page.svelte:167-184` | Date, Montant, Référence, Détails, Statut (6 colonnes ; la dernière, Rapprochement, porte des boutons) |
| Réconciliation (`reconciliation-table`) | `ReconciliationProposals.svelte:706-714`, `:752-770` | case, Date, Montant, Contrepartie, Candidate/Proposition, Score (7 colonnes ; la dernière, Actions) |
| Journal d'audit (`audit-log-table`) | `audit-log/+page.svelte:358-363`, `:374-388` | Date, Auteur, Action, Type d'entité, N° (6 colonnes) |

⚠️ La marge se pose en **en-tête et en cellule** : un en-tête sans la marge de la cellule décale les colonnes l'une de l'autre. Le tableau de
`ReconciliationProposals` fusionne des cellules (`colspan="2"`, `:730-770`) **sous** Contrepartie/Candidate : ne pas toucher aux `colspan`. `table-auto` répartit la
largeur : vérifier à 1280 px que les tables ne débordent pas de leur carte avec les marges de plus. **Angle mort assumé** : #1552 (écriture, autre module) et tout
tableau de l'application que la story ne touche pas. Si la capture d'avant montre un autre couple collé **dans ces cinq tableaux** que le tableau ci-dessus ne
couvre pas, il se corrige avec ; ailleurs, il s'ouvre en issue.

### D2 — Inventaire des usages de `--color-warning` (AC 3, 4 ; effet de bord de C-58-5-1)

Relevé `grep -rn "text-warning\|bg-warning\|border-warning\|color-warning" frontend/src` au commit de départ (lignes lues) :

- **Texte `text-warning` (18 sites, ceux à mesurer)** : `contacts/+page.svelte:1100` ; `settings/+page.svelte:876` ; `settings/fiscal-years/+page.svelte:736` ;
  `invoices/due-dates/+page.svelte:481` ; `settings/CompanyConflictNotice.svelte:37`, `:56` ; `imported-supplier-invoices/DocumentPreview.svelte:99` ;
  `fiscal-years/RevaluationPanel.svelte:243`, `:249`, `:308`, `:363`, `:370`, `:377` ; `fiscal-years/FiscalYearChecks.svelte:58` ; `login/+page.svelte:112`,
  `:134` (sur `bg-warning/5`) ; `components/invoices/InvoiceForm.svelte:925` (sur `bg-warning/10`) ; `invoices/PaymentStatusBadge.svelte:63` (`var(--color-warning)`
  sur `warning` à 20 %, `:62`).
- **Fonds et bordures (une soixantaine de lignes)** : `border-warning`, `bg-warning/10`, `/5`, `/30` des encadrés (factures, fournisseurs, rapprochement,
  sauvegarde, MCP…) ; `color-mix(in srgb, var(--color-warning…) N%, transparent)` : `due-dates/+page.svelte:678` (10 %), `ReminderTerminalBadge.svelte:26`
  (22 %, texte `--color-text`), `McpSendsList.svelte:131`, `McpPromptVersion.svelte:91`, `McpPromptsList.svelte:297`, `McpPromptDetail.svelte:780` (22 %, texte
  foncé). Ceux dont le texte est `--color-text` **ne se mesurent pas** : seul l'aspect change (D6). **Tout autre texte sur une teinte d'avertissement se mesure** : `settings/dunning/+page.svelte:295` (`text-text-muted` sur `border-warning/30 bg-warning/5` : 4,51:1 aujourd'hui, **4,40:1** avec `#92400e` ; à passer en `text-text`), les lignes `.row-overdue` de `due-dates/+page.svelte:589` et `:677` (teinte à 10 % : les cellules à `text-text-muted` s'y mesurent), et tout texte imbriqué dans un encadré `bg-warning/N` au-delà de ce que `grep` voit en une ligne : relevé **au navigateur** (`getComputedStyle`, parcours des encadrés de D6), le `grep` ne suffisant pas.
- **Replis littéraux** : `#f59e0b` à `due-dates/+page.svelte:678` et `PaymentStatusBadge.svelte:62-63`, `#d97706` à `ReminderTerminalBadge.svelte:26` : le repli de `var(--color-warning, …)` n'est jamais employé (le jeton est défini) ; **les laisser**, hors périmètre.
- **Angles morts assumés (valeur `#d97706` hors de l'application)** : `docs/manual/shared/kesh-style.sty:86` (`keshWarning`, cadre et titre des encadrés d'avertissement des PDF : titre `keshWarning` sur blanc à 3,19:1), `docs/kesh-specifications.txt:2037` et `ux-design-specification.md` : hors périmètre (PDF non régénérés dans la story, spécifications historiques). Le `.sty` est suivi par une issue à ouvrir par l'orchestrateur (§ Questions).
- `bg-warning-soft` (`bank-accounts/+page.svelte:1041`, `SupplierInvoice*.svelte`, `BankImportUpload.svelte:472…`) n'existe pas au thème : **#645**, hors
  story. Le noter : ces fonds ne se voient pas, seule la bordure change de brun.
- Pas de thème sombre (CLAUDE.md) : une seule valeur à régler.
- Barème à recalculer par `contrastRatio` + `blend` : `#92400e` sur `#ffffff` **7,09** ; sur `#f8fafc` 6,78 ; sur teinte 5 % 6,56 ; 10 % 6,08 ; 20 % 5,16 ;
  22 % 4,99. Si le développement choisit une autre valeur (plus claire), elle doit tenir **toutes** ces lignes ≥ 4,5:1 et la garde l'impose.

### D3 — #253 : ce qui reste hors périmètre

- `design-system/+page.svelte:41` (swatch « P. Light », `text-white` sur #3b82f6, 3,68:1) et `:45` (swatch « Warning », `text-white` sur le jeton) : page de
  démonstration des jetons, hors des écrans d'usage ; le swatch Warning gagne en contraste avec C-58-5-1 (7,09:1 en blanc). Ne pas retoucher ; si
  `design-system` est sous axe quelque part, la recette le dit.
- `users.spec.ts:134` : `.disableRules(['color-contrast']) // tokens custom, vérifiés manuellement` : **autre motif**, non lié à #253 ; ne pas y toucher.
- `--color-primary-light` **ne change pas** (focus `app.css:84`, survols `/10`).

### D4 — Manuel (`docs/manual/`, PDF compris)

- `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` relu à l'écriture de la fiche : **aucune** occurrence du défaut visuel (les emplois de « collé » et « chevauche » du PDF parlent de copier-coller et de relevés qui se chevauchent), aucune mention d'un titre de colonne « Candidate » ni « Solde non disponible ». Les 20 captures de `docs/manual/fr/screenshots/` (21 fichiers dont `_placeholder.png`)
  (multi-devises, rapports) ne montrent **aucun** des écrans touchés (aperçu d'import, réconciliation, journal d'audit, accueil, société, champ de compte,
  email-templates) : `reconciliation-auto.png`, nommée à `user-manual.tex:3822`, **n'existe pas** (TODO de la ligne, repli `_placeholder.png`). Donc **aucune
  capture du manuel à refaire**.
- **Un texte à changer pour #1153** : `user-manual.tex:3893` dit « avec leur date, leur montant, leur contrepartie, et \textbf{une} candidate » : y nommer la
  colonne (« et \textbf{une} proposition (colonne \emph{Proposition}), la \textbf{candidate} affichée… ») sans réécrire le vocabulaire « candidate »
  (concept défini en `:3822`, `:3834`, `:3844`, qui reste juste). Les manuels DE/IT/EN n'existent pas (`docs/manual/{de,it,en}/` ne portent qu'un `README.md`) et leurs catalogues ne changent pas : rien à y toucher.
- **Un second texte à changer (effet du jeton, C-58-5-1)** : `user-manual.tex:2814` dit que le cadre du rapport d'import de factures fournisseur « est bordé d'orange » ; le cadre est `border-2 border-warning` (`supplier-invoices/import/+page.svelte:378`), qui devient brun. Reprendre la phrase (« bordé d'ambre foncé », ou « de la couleur d'avertissement »), `.tex` seul. L'écran (rapport avec un fichier refusé) entre à la recette D6.
- Les PDF **ne se commitent pas** dans la branche (règle des fichiers communs) : seul le `.tex`.

### D5 — Reproduire avant de corriger (#1008, #1015, #1184)

Un agent de fiche ne lance rien : les trois mécanismes sont **déduits du code**, à confirmer à la capture d'avant.

- **#1008** : colonne de droite sans `max-width` ⇒ sa largeur de base est la ligne la plus longue ; le nom (`min-w-0 flex-1`) se rétrécit sous sa largeur de contenu et
  déborde sur elle. Reproduire à 1280 px (largeur minimale de la page) avec un compte **non lié** (`currentBalance === null`, `:43`) au nom long (une
  description d'au moins 30 caractères, `bankAccountName`). Si le chevauchement vient d'ailleurs (par ex. la carte dans une grille à deux colonnes, `+page.svelte`
  de l'accueil), corriger **là où il se produit** et le dire au Dev Agent Record.
- **#1015** : saisir une adresse e-mail de 320 caractères sans espace à la société, provoquer un conflit de version (deux onglets), lire l'avis « Enregistré
  entre-temps » à **375 px**. Mesurer `scrollWidth`/`clientWidth` **de l'avis** (`settings-company-email-conflict-changed-email`) **et de son conteneur** (`<div class="flex-1">`, `settings/+page.svelte:908`), avant et après — non du document : #708 (ouverte) fait déjà déborder la page entière à 390 px, angle mort assumé.
- **#1184** : une ligne d'écriture (`/journal-entries/new`) ou de facture dont le compte porte un nom long, × visible (compte choisi).

### D6 — Recette visuelle (AC 13), thème clair, base de démonstration, captures rangées sous `…/e65/58-5/captures/`

Nommage : `<issue>-<ecran>-<largeur>-<avant|apres>.png`. Chaque ligne = **une** paire avant/après.

| Issue | Écran (route) | Largeur | État à capturer |
|---|---|---|---|
| #253 | Paramètres → Modèles d'e-mail (`/settings/email-templates`) | 1280 | chip *Type* sélectionné + chip *Niveau* (type « rappel ») ; onglet de langue actif ; badge « Personnalisé » (modèle personnalisé) et badge « Défaut » |
| #253 | Échéances (`/invoices/due-dates`) | 1280 | onglet de statut actif + un non actif |
| #996 | Formulaire de facture avec l'avis `invoice-config-warning` (`InvoiceForm.svelte:925`, création/édition de facture) ; une fiche facture avec encadré `border-warning`/`bg-warning/10` (`invoices/[id]/+page.svelte:1243`, `:1575`) ; Réglage des rappels (`settings/dunning`, `dunning-cgv-hint`) ; Échéances avec une ligne en retard (`.row-overdue`) ; import de factures fournisseur avec un fichier refusé (`inbox-import-report`, cadre `border-2 border-warning`) ; Connexion avec l'avis (`login`) ; Paramètres → Exercices (`fiscal-year-close-checks-count`, `settings/fiscal-years/+page.svelte:736`) ; Liste des factures avec un badge « en retard » (`PaymentStatusBadge.overdue`) | 1280 | l'état d'avertissement, avant/après le brun |
| #996 | Paramètres → Société, avis de conflit | 1280 | texte `text-warning` à 12 px (le cas de l'issue) |
| #933 | Banque → Import → aperçu d'un relevé (`bank-import-confirm` visible) | 1280 | table des transactions ; **et** un relevé qui produit des « lignes qui ressemblent à une ligne déjà importée » (`warning-uncertain-legacy-lines-row`) |
| #1016 | Réconciliation d'un compte (`reconciliation-table`) | 1280 | en-tête et une ligne avec contrepartie ; une ligne avec « Aucune correspondance » ; l'en-tête qui dit « Proposition » |
| #1153 | idem, **en-tête** de la table | 1280 | « Proposition » (fr-CH) ; contrôle rapide du même en-tête en de-CH, it-CH, en-CH : inchangé |
| #1549 | Détail d'un import bancaire (`detail-tx-table`) | 1280 | en-tête et plusieurs lignes ; Montant et Référence séparés |
| #1154 | Administration → Journal d'audit (`audit-log-table`) | 1280 | en-tête et plusieurs lignes, dont une à n° d'entité et une à « — » |
| #1008 | Accueil, tuile Comptes bancaires | **1280** et **1920** | un compte lié, un compte **non lié** au nom long (« Solde non disponible… ») |
| #1015 | Paramètres → Société, avis de conflit | **375** (téléphone), 1280 | valeur de 320 caractères sans espace ; l'avis et son conteneur ne débordent pas (`scrollWidth`/`clientWidth`, D5) ; le défilement horizontal de la page est #708, hors story |
| #1184 | Écriture manuelle ou facture, champ de compte | 1280 | compte à libellé long **avec** × ; compte vide (sans × : largeur inchangée) ; compte désactivé |
| #1096 *(confirmation)* | Fiche d'une facture validée et impayée | **1400** (barre latérale ouverte) et **1024** | la barre d'actions passe à la ligne, aucun bouton coupé ; **une capture « après » seule** (l'issue est déjà corrigée) |

Contrôles de lecture : la mesure d'un ratio se fait sur la **valeur calculée** du navigateur (`getComputedStyle`), non sur la classe ; une capture d'un chip dit
son ratio dans son nom de fichier ou le Dev Agent Record.

### D7 — Tests et gate

- **Vitest (le gate) :** `frontend/src/lib/components/ui/jetons-contraste.test.ts` (neuf) — garde **du jeton seul** (AC3, `--color-warning` et `--color-text-muted` sur ses teintes) ; elle est vraie par construction pour les chips (`bg-primary`/`text-primary-foreground` lus aux jetons) et **ne compte donc pas** comme test d'AC1, dont les tests sont nommés à l'AC (rendu de `due-dates-page.test.ts`, valeur calculée Playwright) ; `import { contrastRatio, blend } from
  '../../../../tests/e2e/helpers/contrast'` (même chemin relatif que le test voisin à recalculer) et `blocTheme`/`jetonDans` de `tests/e2e/helpers/theme`.
  Assertions de classes : une par composant (les tests de composants existent : `HomepageCards.test.ts`, `ReconciliationProposals.test.ts`,
  `audit-log-page.test.ts`, `BankImportUpload.test.ts`, `bank-import-detail-page.test.ts`, `due-dates-page.test.ts`, `settings-page.test.ts` pour l'avis de conflit — `company-conflict.test.ts` teste la logique pure et ne rend rien —, `AccountAutocomplete.test.ts`). Ce sont des gardes de **non-régression de
  classe**, pas de rendu : le rendu est celui des captures. **Chaque AC nomme son test et sa mutation** ; le développement joue chaque mutation (restaurée ensuite, `touch` du fichier : cargo/vite gardent la mutation sinon) et l'écrit au Dev Agent Record.
- **Playwright :** `email-templates.spec.ts:198-208` (axe sans `disableRules`), `reminders.spec.ts:293-305` ; nouveau test axe des onglets de langue/badge ;
  E2E complet une fois, au dernier commit de code.
- **Gates :** `cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build` (voie frontend,
  `flock …/.gate-front.lock scripts/mem-guard.sh …`) ; **gate Rust ciblé** (aucun `.rs` ne change, mais le catalogue fr-CH est lu par `kesh-i18n` et par `kesh-api/tests/textes_coherents.rs:112` et `:471`, et `reconciliation-cols-candidate` change de valeur) : `flock /tmp/station-heavy.lock flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock scripts/mem-guard.sh cargo nextest run -E 'package(kesh-i18n) | binary(textes_coherents)'` (voie base, `docs/agents-orchestration.md` § 5.1 ; `scripts/test-fast.sh` n'accepte que `--no-lint` et `--ci`, aucun filtre). Le Dev Agent Record dit ce qui a tourné ; le story file n'affirme que ce qui a tourné.
- **Jetons non définis** : `jetons-non-definis.test.ts` doit rester vert (`wrap-anywhere`, `pr-4`, `max-w-[60%]` ne sont pas des classes de couleur) ; vérifier
  qu'aucune classe de couleur neuve n'est écrite.

### Project Structure Notes

- Aucun fichier de production nouveau : balisage Svelte, `app.css`, une ligne de `fr-CH/messages.ftl`, `user-manual.tex`. Un fichier de test neuf
  (`jetons-contraste.test.ts`) à côté de `jetons-non-definis.test.ts`.
- **Fichiers partagés, à déclarer** : (1) **`crates/kesh-i18n/locales/fr-CH/messages.ftl:1718`** (`reconciliation-cols-candidate`) : une ligne **existante**, hors du bloc d'un epic, que la 58-4 (#1086, espaces insécables sur tout le catalogue fr-CH) peut réécrire à côté ; la fiche 58-4 affirme que la 58-5 n'y touche pas : **l'une des deux fiches se corrige** (la 58-4, selon l'orchestrateur) et le conflit se résout à la fusion en gardant les deux côtés ; (2) **`frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte`** (production, `:704-714`, `:752-770`), que la 58-6 modifie aussi (garde de `confirmGesture`, vers `:520`) : zones **disjointes**, fusion en gardant les deux côtés. La 58-6 touche `reconciliation/*.test.ts` et `AccountAutocomplete*` : **`ReconciliationProposals.test.ts`** et **`AccountAutocomplete.test.ts`** sont
  communs ; une assertion ajoutée ici (classe) se résout à la fusion en gardant les deux côtés. `docs/manual/fr/user-manual.tex` : la 58-4 peut y toucher.

### References

- [Source: `_bmad-output/planning-artifacts/epic-65-nettoyage-suite.md` § Story 58-5]
- [Source: `frontend/tests/e2e/helpers/contrast.ts`, `tests/e2e/helpers/theme.ts` ; `frontend/src/lib/features/journal-entries/account-autocomplete-classes.test.ts`
  (patron de garde de contraste) ; `frontend/src/lib/components/ui/jetons-non-definis.test.ts` (#645)]
- [Source: CLAUDE.md § *Recette visuelle*, § *Fichiers communs* ; `docs/testing.md` § *Les échecs attendus*]

## Questions et risques (pour l'orchestrateur)

1. **Un conflit de fusion annoncé** : `fr-CH/messages.ftl:1718` (une ligne) avec la 58-4, `user-manual.tex` avec la 58-4 éventuellement.
2. **#1096 est fermée** (le 2026-11-28, preuve en commentaire : `invoices/[id]/+page.svelte:1093`, v0.19.0) ; aucun commit de la story ne la porte en `refs` utile.
3. **#996 change l'aspect de la soixantaine d'encadrés d'avertissement** (bordure brune, teinte plus sourde) : décision C-58-5-1, à relire à la validation ; une
   solution à jeton séparé existe (alternative (b)) mais coûte 18 sites.
4. **Risque #253 / AC2** : retirer `disableRules` peut révéler d'autres contrastes sur `reminders-list` ; règle écrite à l'AC2 (issue neuve, jamais corrigée dans
   cette story).
5. **Issue à ouvrir (orchestrateur)** : `docs/manual/shared/kesh-style.sty:86` (`keshWarning` `#D97706`, titre des encadrés d'avertissement des PDF à 3,19:1) — hors story.
6. **#1552** (écriture, « Crédit » et « Compte analytique » collés) : autre module, angle mort assumé, reste ouverte.
7. **#1008, #1015, #1184 sont déduites du code**, non reproduites : la capture d'avant décide (D5).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-28 — Fiche rédigée (`bmad-create-story`, Sonnet 5.5). Neuf issues retenues, #1096 écartée (déjà corrigée). Statut : `drafted`.
- 2026-11-28 — **Validation P1 (Opus) : 0/1/6/7, remédiée.** F1 (HIGH) : AC10 mesure l'avis et son conteneur, #708 angle mort assumé. F2/F3 : tout texte non
  `--color-text` sur teinte d'avertissement mesuré (AC4, `dunning/+page.svelte:295` en `text-text`), `user-manual.tex:2814` repris, `.sty:86` inventorié. F4 : #1549
  retenue (dix issues), `pr-4` sur chaque colonne sauf la dernière des cinq tableaux touchés (C-58-5-3), #1552 angle mort. F5 : test et mutation nommés par AC. F6 :
  `fr-CH/messages.ftl:1718` et `ReconciliationProposals.svelte` déclarés partagés. F7 : gate Rust `cargo nextest run -E 'package(kesh-i18n) | binary(textes_coherents)'`.
  F8 : #1096 fermée le 2026-11-28. F9 : lignes de `BankAccountsCard.svelte` corrigées (37, 38, 42, 43). F10 : décomptes (cinq tableaux ; 20 captures). F11 :
  `fiscal-year-close-checks-count`, `invoice-config-warning` dans `InvoiceForm`. F12 : `settings-page.test.ts`. F13 : replis et `#d97706` hors application en angles
  morts. F14 : parents de l'avis (flex pour l'e-mail), chemin `reconciliation/wise/+page.svelte`.
