# Story 58-5 : Cosmétique des écrans — colonnes collées, débordements, contraste (#253, #933, #996, #1008, #1015, #1016, #1153, #1154, #1184)

Status: drafted

*Fiche rédigée le 2026-11-28 (Sonnet 5.5, en autonomie), workflow `bmad-create-story`. Worktree `/home/gcorbaz/devel/kesh-58-5`, branche
`story/58-5-cosmetique-ecrans` partie de `main` à `287deea33` (v0.20.1). Epic 65 (nettoyage, suite de l'Epic 58), release visée : vendredi 2026-12-04
(numéro attribué au tag). **Aucune migration, aucune règle métier, aucun changement d'API.** Du balisage, des classes CSS, un jeton de couleur, un libellé
fr-CH et quelques tests ; aucune compilation ni aucun gate n'a été lancé pour écrire la fiche (un agent de fiche n'exécute ni `docker` ni `curl` : ce qui
doit être vu à l'écran est prescrit au développement). Ratios de contraste calculés à la main (formule WCAG 2.x, script jetable hors dépôt) : le
développement les **recalcule** avec `tests/e2e/helpers/contrast.ts`.*

## Story

En tant qu'utilisateur de Kesh, je veux que les colonnes de montant ne touchent plus le texte qui les suit (aperçu d'import, réconciliation, journal d'audit), que les
valeurs longues se coupent dans leur cadre (avis de conflit de la société, tuile des comptes bancaires, champ de compte) et que les textes d'avertissement et les
chips sélectionnés soient lisibles au seuil AA, afin de lire les écrans sans deviner où finit un mot et où commence l'autre.

## Contrôle de couverture (règle du 2026-11-26 : on vérifie qu'une issue n'est pas déjà couverte avant de la retenir)

Chaque issue a été relue (`gh issue view`, lecture seule) puis cherchée au code de `287deea33` (`grep -n`, lignes lues). Les issues voisines ouvertes : #645
(jetons de couleur non définis, dont `bg-warning-soft`) est **hors de cette story** (une issue par problème) ; aucun commit depuis le tag précédent ne cite
l'une des dix issues (`git log --format=%B | grep -oE '#(253|933|996|1008|1015|1016|1096|1153|1154|1184)\b'` : à rejouer au développement).

| # | Constat relu au code | Verdict |
|---|---|---|
| #253 | Le fond `bg-primary-light` (#3b82f6) porte du texte foncé sur **cinq sites** : `settings/email-templates/+page.svelte:253`, `:276`, `:319` (chips type, niveau, onglets de langue), `:298` (badge « Personnalisé », texte `text-primary`), `invoices/due-dates/+page.svelte:498` (onglets de statut). Les sites que l'issue listait en plus — `(app)/+layout.svelte` (nav), `onboarding/*`, `bank-accounts/+page.svelte` — n'emploient plus le fond plein : `hover:bg-primary-light/10`, `/5`, `/20` avec `text-primary` (≥ 6,9:1), **hors défaut**. `design-system/+page.svelte:41` : étiquette `text-white` sur `bg-primary-light` (3,68:1), page de démonstration des jetons, hors des écrans de l'utilisateur (voir D3). Le contournement existe toujours : `email-templates.spec.ts:203` et `reminders.spec.ts:301` ne passent qu'avec `.disableRules(['color-contrast'])`. | **gardée** |
| #933 | `bank-import/BankImportUpload.svelte:872-873` et `:880-881` (aperçu, table `preview-tx-table`) : `<th class="text-right">Montant</th>` puis `<th class="text-left">Détails</th>`, **aucun** espacement de cellule (Tailwind en préflight remet `padding` à zéro). Même motif au tableau des lignes signalées « ressemblent à une ligne déjà importée » : `:685-686` et `:698-699` (colonne Montant à gauche, suivie de Détails). | **gardée** (deux tableaux du même écran) |
| #996 | `app.css:27` : `--color-warning: #d97706` = **3,19:1** sur blanc (recalcul : 3,19). `text-warning` : **18 sites** (le relevé de l'issue disait 11 ; inventaire en D2). Aucun test de contraste du jeton. | **gardée** |
| #1008 | `homepage/BankAccountsCard.svelte:29` : `<div class="min-w-0 flex-1">` (nom du compte) à côté de `:36` `<div class="text-right">` sans borne de largeur : sa largeur est celle de la ligne la plus longue, « Solde non disponible — lier au plan comptable » (`:58-62`, une seule ligne), et le nom, `min-w-0`, déborde dessus. | **gardée** (mécanisme à **confirmer** à la capture d'avant, D5) |
| #1015 | `settings/CompanyConflictNotice.svelte:37` (`text-warning`) et `:44` (`text-text-muted`) : `<p class="mt-1 text-xs …">` portant `{ $value }` — ni `break-words` ni `wrap-anywhere`. Les parents sont des cellules de grille (`settings/+page.svelte:827-840`, `col-span-2`) : un mot long y élargit la colonne. | **gardée** |
| #1016 | `reconciliation/ReconciliationProposals.svelte:708` (`th text-right` Montant) et `:756` (`td text-right`), suivis de Contrepartie `:709`, `:757` : aucun espacement. | **gardée** |
| #1096 | `invoices/[id]/+page.svelte:1093` (fiche validée) et `:1229` (annulée) : `<div class="flex min-w-0 flex-wrap justify-end gap-2">` ; le commentaire `:1089-1092` le dit (« passe à la ligne au lieu de couper « Dévalider » »). Introduit par `773df80da` (v0.19.0, 2026-11-18), **quatre jours après** l'ouverture de l'issue (2026-11-14, qui citait `:1030` d'avant). Le brouillon (`:1069`, `flex gap-2`) n'a que trois boutons. | **déjà corrigée** — sort de la story, preuve `invoices/[id]/+page.svelte:1093` ; confirmation visuelle à 1400 px et 1024 px à la recette (D6), non bloquante ; l'orchestrateur ferme l'issue, preuve à l'appui |
| #1153 | Deux demandes. (1) « MontantContrepartie » : **aucune** chaîne soudée au catalogue (`grep -rn MontantContrepartie` vide hors fiches) : le mot soudé est l'effet visuel de #1016, **même défaut**, corrigé avec lui. (2) « Candidate » : `crates/kesh-i18n/locales/fr-CH/messages.ftl:1718` `reconciliation-cols-candidate = Candidate` ; repli `ReconciliationProposals.svelte:710`. En français, « candidate » est un mot juste (le manuel l'emploie comme nom : `user-manual.tex:3822`, `:3893`) mais, **seul en titre de colonne**, il ne dit pas de quoi : la colonne porte la facture ou la règle proposée. La demande est celle de l'auteur ; D1. | **gardée** (la moitié (2) seule, (1) étant #1016) |
| #1154 | `audit-log/+page.svelte:362` (`th py-2 text-right`, « N° ») et `:387` (`td py-2 text-right`), suivis de Détails `:363`, `:388`. | **gardée** |
| #1184 | `journal-entries/AccountAutocomplete.svelte:508-536` : `<Input>` sans marge droite, le × (`:551`, `absolute right-2`) se pose sur la fin du texte. `Input` (`ui/input/input.svelte`) passe sa classe à `cn` (tailwind-merge) : `pr-9` l'emporte sur le `px-2.5` de base. Consommateurs de `allowClear` : `products/+page.svelte:779`, `GroupSettlementModal.svelte:962`, `InvoiceForm.svelte:1190`. | **gardée** |

**Bilan : neuf issues retenues, une sortie (#1096, déjà corrigée).** Aucune ne révèle de défaut de production ni de règle métier. L'Epic 65 prévoit cinq à dix
issues par story : neuf est dans la borne.

## Choix consignés (autonomie : aucune question à Guy)

- **C-58-5-1 — le jeton `--color-warning` s'assombrit à `#92400e`** (ambre 800 de Tailwind), au lieu d'un jeton neuf. Alternatives écartées : (a) `#b45309`
  (la piste de l'issue, 5,02:1 sur blanc) : **sous 4,5:1 sur sa propre teinte à 10 %** (4,39:1) et à 20 % (3,81:1) — or `InvoiceForm.svelte:925` pose
  `text-warning` sur `bg-warning/10` et `PaymentStatusBadge.svelte:62-63` sur `warning` à 20 % ; il aurait fallu retoucher ces sites ; (b) un jeton
  `--color-warning-text` à côté : 18 sites à reprendre, et deux couleurs d'avertissement. `#92400e` mesure **7,09:1** sur blanc, 6,78:1 sur `surface-alt`
  (#f8fafc), 6,56 / 6,08 / 5,16 / 4,99:1 sur sa propre teinte à 5 / 10 / 20 / 22 %. Contrepartie visible : les **bordures** `border-warning` (une soixantaine
  de sites, D2) et les teintes `bg-warning/N` passent à un brun plus sombre ; la recette le montre (D6). *Réversible* : une ligne de `app.css`.
- **C-58-5-2 — le chip sélectionné prend `bg-primary text-primary-foreground`** (#1e40af / blanc = **8,72:1**), le patron déjà employé par
  `accounts/groups/+page.svelte:316` et les boutons primaires. Alternative écartée : éclaircir `--color-primary-light` — il sert aussi au contour de focus
  (`app.css:84`, `outline: 2px solid var(--color-primary-light)`) et aux survols `/10`, `/20` ; le changer élargit la story pour rien. Le **badge**
  « Personnalisé » (`:298`) devient `bg-primary-light/20 text-primary` (patron du badge de `bank-accounts/+page.svelte:1447` ; **6,92:1**), non plein : un
  badge n'est pas un état sélectionné.
- **C-58-5-3 — l'espacement des colonnes se pose par `pr-4` sur la colonne de montant** (en-tête **et** cellules), jamais par une marge globale sur `th`/`td`
  (les autres tableaux de l'application ont déjà leurs `py-2 pr-2`/`pr-4`, ex. `wise/+page.svelte:133`) : seul le tableau défectueux change. Seize pixels,
  comme les tableaux voisins.
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
2. **Axe réactivé (#253).** Le `.disableRules(['color-contrast'])` de `email-templates.spec.ts:203` et celui de `reminders.spec.ts:301` sont **retirés**, et les
   deux tests passent. Si la règle remonte un autre défaut de contraste hors des chips (`reminders-list` porte des badges), il s'ouvre en **issue** (« on trace »)
   et le `disableRules` de ce seul test reste, avec son commentaire corrigé pour citer la nouvelle issue au lieu de #253 ; le commentaire de #253 disparaît dans
   les deux cas. Les onglets de langue et le badge `data-testid="email-template-badge"` (hors du `.include` actuel) passent sous axe dans un test neuf.
3. **Jeton d'avertissement (#996).** `--color-warning` vaut `#92400e` (`app.css:27`), et une garde Vitest nouvelle (`jetons-contraste.test.ts`, patron de
   `account-autocomplete-classes.test.ts`, lisant `app.css` par `blocTheme`/`jetonDans`) établit que ce jeton atteint **≥ 4,5:1** sur `--color-surface`, sur
   `--color-surface-alt` **et** sur sa propre teinte à 5, 10, 20 et 22 % posée sur `--color-surface` (les alphas des sites : `login` 5 %, `InvoiceForm` 10 %,
   `PaymentStatusBadge` 20 %, badges `color-mix` 22 %). La garde lit le jeton **dans `app.css`** (une valeur écrite en dur dans le test ne prouverait rien) et
   rougit avec `#d97706` (mutation jouée, § Tâches).
4. **Aucun site à retoucher pour #996** : l'inventaire de D2 est parcouru ; les 18 sites `text-warning` mesurent chacun **≥ 4,5:1** sur leur fond réel (blanc,
   `bg-warning/5`, `/10`, `/20`), le tableau des mesures va au Dev Agent Record. Un site qui reste sous le seuil (fond hors liste) est corrigé par le patron
   AA-safe `text-text`, et dit.
5. **Aperçu d'import (#933).** Dans `BankImportUpload.svelte`, les colonnes Montant des deux tableaux (`:872`/`:880` et `:685`/`:698`) portent `pr-4` en en-tête
   et en cellule : à 1280 px, une transaction « 100.00 CHF » et son libellé sont séparés d'au moins 16 px ; aucun test existant ne casse.
6. **Réconciliation (#1016).** `ReconciliationProposals.svelte:708` et `:756` : `pr-4` sur l'en-tête **et** sur la cellule Montant ; capture : « -1450 CHF »
   et « Régie du Lac » séparés, en-tête compris.
7. **Libellé (#1153).** `fr-CH/messages.ftl:1718` : `reconciliation-cols-candidate = Proposition` ; repli de `ReconciliationProposals.svelte:710` :
   `'Proposition'` ; les trois autres catalogues **inchangés** ; le manuel (`docs/manual/fr/user-manual.tex:3893`) nomme la colonne (§ Tâches). La garde i18n
   (`i18n-keys.test.ts`, `sitesTotal`) ne bouge pas (une valeur change, aucune clé).
8. **Journal d'audit (#1154).** `audit-log/+page.svelte:362` et `:387` : `pr-4` sur « N° » (en-tête et cellule) ; capture : « N° » et « Détails » séparés.
9. **Tuile des comptes bancaires (#1008).** À 1280 px **et** à 1920 px, le lien « Solde non disponible — lier au plan comptable » ne recouvre ni le nom ni
   l'identifiant court du compte : il passe à la ligne dans sa colonne. Correctif attendu (à confirmer à la capture d'avant, D5) : colonne de droite
   `min-w-0 max-w-[60%] text-right` (`BankAccountsCard.svelte:36`) et nom `break-words` (`:30`).
10. **Avis de conflit (#1015).** `CompanyConflictNotice.svelte:37` **et** `:44` portent `wrap-anywhere` : une adresse e-mail de 320 caractères sans espace se
    coupe dans son cadre à **375 px**, la page ne défile pas horizontalement (`document.documentElement.scrollWidth <= clientWidth`, mesuré à la capture).
11. **Champ de compte (#1184).** Quand le × est affiché (`allowClear && !disabled && (value !== null || query !== '')`), le champ reçoit `pr-9` : le texte
    d'un libellé long se coupe avant le bouton, **sans** que la largeur du champ change quand le × est absent. La condition d'affichage n'est écrite qu'**une**
    fois (dérivée partagée par le × et la classe, DRY).
12. **Gate frontend vert** (`check`, `lint-i18n-ownership`, `test:unit`, `build`), **E2E complet** à la dernière modification de code de la story : « vert » =
    les sept échecs de la KF-029 seuls (`docs/testing.md`, *Les échecs attendus*).
13. **Recette visuelle** (D6) : captures avant/après de chaque état listé, thème clair, base de démonstration, rangées dans
    `/home/gcorbaz/devel/kesh-gate-logs/e65/58-5/captures/`, jointes au Dev Agent Record et au prompt de la revue de code.

## Tasks / Subtasks

- [ ] **Tâche 0 — Contrôles d'entrée (AC : 12, 13).** `git log origin/main..HEAD` propre ; rejouer le grep des dix numéros dans les commits depuis le tag.
  - [ ] Monter la base de démonstration (`docs/testing.md` § *Base de dev jetable*, bases `kesh_585`, `kesh_e2e_585`, port 3655) ; prendre **toutes les captures
        « avant »** de D6 **avant** de toucher au code.
- [ ] **Tâche 1 — Contraste : jeton et chips (AC : 1-4).**
  - [ ] Écrire la garde `frontend/src/lib/components/ui/jetons-contraste.test.ts` **rouge d'abord** (jeton `#d97706` : 3,19:1), puis poser `#92400e`.
  - [ ] Mutation : remettre `#d97706` dans `app.css`, `touch` le fichier, vérifier le rouge, restaurer (un binaire/cache périmé garde la mutation).
  - [ ] Chips : `email-templates/+page.svelte:253`, `:276`, `:319`, `due-dates/+page.svelte:498` → `class:bg-primary` + `class:text-primary-foreground` ;
        badge `:298` → `bg-primary-light/20 text-primary`.
  - [ ] Retirer les deux `.disableRules(['color-contrast'])` (AC2) ; ajouter le test axe des onglets de langue et du badge.
  - [ ] Mesurer les 18 sites `text-warning` (D2) avec `contrast.ts`, écrire le tableau au Dev Agent Record.
- [ ] **Tâche 2 — Colonnes collées (AC : 5, 6, 8).** `pr-4` aux sept couples en-tête/cellule listés en D1 ; une assertion de classe par écran dans les tests
      existants (`audit-log-page.test.ts`, `ReconciliationProposals.test.ts`, `BankImportUpload.test.ts`) : le `th` Montant porte `pr-4`.
- [ ] **Tâche 3 — Libellé (AC : 7).** `fr-CH/messages.ftl:1718`, repli Svelte `:710` ; `user-manual.tex` (D4) ; **ne pas régénérer les PDF** (règle des fichiers
      communs : seul le `.tex` change dans une branche de story).
- [ ] **Tâche 4 — Débordements (AC : 9, 10, 11).** `BankAccountsCard.svelte`, `CompanyConflictNotice.svelte`, `AccountAutocomplete.svelte` ; une assertion de
      classe pour chacun (`HomepageCards.test.ts`, `company-conflict.test.ts` ou un test de composant voisin, `AccountAutocomplete.test.ts`).
- [ ] **Tâche 5 — Recette visuelle et gates (AC : 12, 13).** Captures « après », comparaison, gate frontend, E2E complet (une fois, au dernier commit de code),
      Dev Agent Record.

## Dev Notes

### D1 — Sites exacts des colonnes (AC 5, 6, 7, 8)

| Écran | Fichier:ligne (en-tête, cellule) | Avant → après |
|---|---|---|
| Aperçu d'import, table principale | `BankImportUpload.svelte:872`, `:880` | `class="text-right"` → `class="pr-4 text-right"` |
| Aperçu d'import, lignes signalées (« ressemblent à… ») | `:685`, `:698` | `class="text-left"` → `class="pr-4 text-left"` ; la cellule `<td>` n'a pas de classe : `<td class="pr-4">` |
| Réconciliation | `ReconciliationProposals.svelte:708`, `:756` | `class="text-right"` → `class="pr-4 text-right"` |
| Journal d'audit | `audit-log/+page.svelte:362`, `:387` | `class="py-2 text-right"` → `class="py-2 pr-4 text-right"` |

⚠️ `pr-4` ne se pose **que sur la colonne de montant**, en-tête et cellules : un en-tête sans la marge de la cellule décale les colonnes l'une de l'autre.
Le tableau de `ReconciliationProposals` fusionne des cellules (`colspan="2"`, `:730-770`) **sous** Contrepartie/Candidate : ne pas toucher aux `colspan`.
`table-auto` répartit la largeur : vérifier à 1280 px que la table ne déborde pas de sa carte avec les 16 px de plus.

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
  foncé). Ceux-là **ne se mesurent pas** (texte `--color-text`, fond clair) : seul l'aspect change (D6).
- **Repli littéral** `#f59e0b` à `due-dates/+page.svelte:678` et `PaymentStatusBadge.svelte:62-63` : le repli de `var(--color-warning, #f59e0b)` n'est jamais
  employé (le jeton est défini) ; **le laisser**, hors périmètre.
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

- `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` relu à l'écriture de la fiche : **aucune** occurrence du défaut visuel (les emplois de « collé » et « chevauche » du PDF parlent de copier-coller et de relevés qui se chevauchent), aucune mention d'un titre de colonne « Candidate » ni « Solde non disponible ». Les 21 captures de `docs/manual/fr/screenshots/`
  (multi-devises, rapports) ne montrent **aucun** des écrans touchés (aperçu d'import, réconciliation, journal d'audit, accueil, société, champ de compte,
  email-templates) : `reconciliation-auto.png`, nommée à `user-manual.tex:3822`, **n'existe pas** (TODO de la ligne, repli `_placeholder.png`). Donc **aucune
  capture du manuel à refaire**.
- **Un texte à changer pour #1153** : `user-manual.tex:3893` dit « avec leur date, leur montant, leur contrepartie, et \textbf{une} candidate » : y nommer la
  colonne (« et \textbf{une} proposition (colonne \emph{Proposition}), la \textbf{candidate} affichée… ») sans réécrire le vocabulaire « candidate »
  (concept défini en `:3822`, `:3834`, `:3844`, qui reste juste). Les manuels DE/IT/EN n'existent pas (`docs/manual/{de,it,en}/` ne portent qu'un `README.md`) et leurs catalogues ne changent pas : rien à y toucher.
- Les PDF **ne se commitent pas** dans la branche (règle des fichiers communs) : seul le `.tex`.

### D5 — Reproduire avant de corriger (#1008, #1015, #1184)

Un agent de fiche ne lance rien : les trois mécanismes sont **déduits du code**, à confirmer à la capture d'avant.

- **#1008** : colonne de droite sans `max-width` ⇒ sa largeur de base est la ligne la plus longue ; le nom (`min-w-0 flex-1`) se rétrécit sous sa largeur de contenu et
  déborde sur elle. Reproduire à 1280 px (largeur minimale de la page) avec un compte **non lié** (`currentBalance === null`, `:57`) au nom long (une
  description d'au moins 30 caractères, `bankAccountName`). Si le chevauchement vient d'ailleurs (par ex. la carte dans une grille à deux colonnes, `+page.svelte`
  de l'accueil), corriger **là où il se produit** et le dire au Dev Agent Record.
- **#1015** : saisir une adresse e-mail de 320 caractères sans espace à la société, provoquer un conflit de version (deux onglets), lire l'avis « Enregistré
  entre-temps » à **375 px**. Mesurer `scrollWidth`/`clientWidth` avant et après.
- **#1184** : une ligne d'écriture (`/journal-entries/new`) ou de facture dont le compte porte un nom long, × visible (compte choisi).

### D6 — Recette visuelle (AC 13), thème clair, base de démonstration, captures rangées sous `…/e65/58-5/captures/`

Nommage : `<issue>-<ecran>-<largeur>-<avant|apres>.png`. Chaque ligne = **une** paire avant/après.

| Issue | Écran (route) | Largeur | État à capturer |
|---|---|---|---|
| #253 | Paramètres → Modèles d'e-mail (`/settings/email-templates`) | 1280 | chip *Type* sélectionné + chip *Niveau* (type « rappel ») ; onglet de langue actif ; badge « Personnalisé » (modèle personnalisé) et badge « Défaut » |
| #253 | Échéances (`/invoices/due-dates`) | 1280 | onglet de statut actif + un non actif |
| #996 | Fiche facture avec encadré d'avertissement (`border-warning`/`bg-warning/10`, p. ex. `/invoices/[id]` d'une facture dont la config est incomplète) ; Connexion avec l'avis (`login`) ; Paramètres → Exercices (`fiscal-years-close-checks-count`) ; Liste des factures avec un badge « en retard » (`PaymentStatusBadge.overdue`) | 1280 | l'état d'avertissement, avant/après le brun |
| #996 | Paramètres → Société, avis de conflit | 1280 | texte `text-warning` à 12 px (le cas de l'issue) |
| #933 | Banque → Import → aperçu d'un relevé (`bank-import-confirm` visible) | 1280 | table des transactions ; **et** un relevé qui produit des « lignes qui ressemblent à une ligne déjà importée » (`warning-uncertain-legacy-lines-row`) |
| #1016 | Réconciliation d'un compte (`reconciliation-table`) | 1280 | en-tête et une ligne avec contrepartie ; une ligne avec « Aucune correspondance » ; l'en-tête qui dit « Proposition » |
| #1153 | idem, **en-tête** de la table | 1280 | « Proposition » (fr-CH) ; contrôle rapide du même en-tête en de-CH, it-CH, en-CH : inchangé |
| #1154 | Administration → Journal d'audit (`audit-log-table`) | 1280 | en-tête et plusieurs lignes, dont une à n° d'entité et une à « — » |
| #1008 | Accueil, tuile Comptes bancaires | **1280** et **1920** | un compte lié, un compte **non lié** au nom long (« Solde non disponible… ») |
| #1015 | Paramètres → Société, avis de conflit | **375** (téléphone), 1280 | valeur de 320 caractères sans espace ; page sans défilement horizontal |
| #1184 | Écriture manuelle ou facture, champ de compte | 1280 | compte à libellé long **avec** × ; compte vide (sans × : largeur inchangée) ; compte désactivé |
| #1096 *(confirmation)* | Fiche d'une facture validée et impayée | **1400** (barre latérale ouverte) et **1024** | la barre d'actions passe à la ligne, aucun bouton coupé ; **une capture « après » seule** (l'issue est déjà corrigée) |

Contrôles de lecture : la mesure d'un ratio se fait sur la **valeur calculée** du navigateur (`getComputedStyle`), non sur la classe ; une capture d'un chip dit
son ratio dans son nom de fichier ou le Dev Agent Record.

### D7 — Tests et gate

- **Vitest (le gate) :** `frontend/src/lib/components/ui/jetons-contraste.test.ts` (neuf) — gardes du jeton (AC3) **et** des chips (les constantes ou les chaînes de
  classes lues, patron `account-autocomplete-classes.test.ts` : `bg-primary` + `text-primary-foreground` ≥ 4,5:1) ; `import { contrastRatio, blend } from
  '../../../../tests/e2e/helpers/contrast'` (même chemin relatif que le test voisin à recalculer) et `blocTheme`/`jetonDans` de `tests/e2e/helpers/theme`.
  Assertions de classes : une par composant (les tests de composants existent : `HomepageCards.test.ts`, `ReconciliationProposals.test.ts`,
  `audit-log-page.test.ts`, `BankImportUpload.test.ts`, `company-conflict.test.ts` ou un fichier voisin, `AccountAutocomplete`). Ce sont des gardes de **non-régression de
  classe**, pas de rendu : le rendu est celui des captures.
- **Playwright :** `email-templates.spec.ts:198-208` (axe sans `disableRules`), `reminders.spec.ts:293-305` ; nouveau test axe des onglets de langue/badge ;
  E2E complet une fois, au dernier commit de code.
- **Gates :** `cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build` (voie frontend,
  `flock …/.gate-front.lock scripts/mem-guard.sh …`) ; **pas de gate Rust** (aucun `.rs`) — le catalogue fr-CH est lu par `kesh-i18n` : lancer néanmoins
  `scripts/test-fast.sh --no-lint` limité à `binary(kesh-i18n)` ou, à défaut, le test qui parse les catalogues, puisque `reconciliation-cols-candidate` change
  de valeur. Le story file n'affirme que ce qui a tourné.
- **Jetons non définis** : `jetons-non-definis.test.ts` doit rester vert (`wrap-anywhere`, `pr-4`, `max-w-[60%]` ne sont pas des classes de couleur) ; vérifier
  qu'aucune classe de couleur neuve n'est écrite.

### Project Structure Notes

- Aucun fichier de production nouveau : balisage Svelte, `app.css`, une ligne de `fr-CH/messages.ftl`, `user-manual.tex`. Un fichier de test neuf
  (`jetons-contraste.test.ts`) à côté de `jetons-non-definis.test.ts`.
- **Fichiers partagés** : la 58-4 seule écrit dans les catalogues (consignes communes de l'Epic 65) ; cette story n'y met qu'**une ligne**, dans son bloc,
  et le dit. La 58-6 touche `reconciliation/*.test.ts` et `AccountAutocomplete*` : **`ReconciliationProposals.test.ts`** et **`AccountAutocomplete.test.ts`** sont
  communs ; une assertion ajoutée ici (classe) se résout à la fusion en gardant les deux côtés. `docs/manual/fr/user-manual.tex` : la 58-4 peut y toucher.

### References

- [Source: `_bmad-output/planning-artifacts/epic-65-nettoyage-suite.md` § Story 58-5]
- [Source: `frontend/tests/e2e/helpers/contrast.ts`, `tests/e2e/helpers/theme.ts` ; `frontend/src/lib/features/journal-entries/account-autocomplete-classes.test.ts`
  (patron de garde de contraste) ; `frontend/src/lib/components/ui/jetons-non-definis.test.ts` (#645)]
- [Source: CLAUDE.md § *Recette visuelle*, § *Fichiers communs* ; `docs/testing.md` § *Les échecs attendus*]

## Questions et risques (pour l'orchestrateur)

1. **Un conflit de fusion annoncé** : `fr-CH/messages.ftl:1718` (une ligne) avec la 58-4, `user-manual.tex` avec la 58-4 éventuellement.
2. **#1096 se ferme** : déjà corrigée (`invoices/[id]/+page.svelte:1093`, v0.19.0) ; la PR l'écrit sans mot-clé de fermeture dans les commits, avec la preuve ;
   l'orchestrateur décide de sa fermeture.
3. **#996 change l'aspect de la soixantaine d'encadrés d'avertissement** (bordure brune, teinte plus sourde) : décision C-58-5-1, à relire à la validation ; une
   solution à jeton séparé existe (alternative (b)) mais coûte 18 sites.
4. **Risque #253 / AC2** : retirer `disableRules` peut révéler d'autres contrastes sur `reminders-list` ; règle écrite à l'AC2 (issue neuve, jamais corrigée dans
   cette story).
5. **#1008, #1015, #1184 sont déduites du code**, non reproduites : la capture d'avant décide (D5).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-28 — Fiche rédigée (`bmad-create-story`, Sonnet 5.5). Neuf issues retenues, #1096 écartée (déjà corrigée). Statut : `drafted`.
