# Story 58-5 : Cosmétique des écrans — colonnes collées, débordements, contraste (#253, #933, #1008, #1015, #1016, #1153, #1154, #1184, #1549)

Status: drafted

*Fiche rédigée le 2026-11-28 (Sonnet 5.5, en autonomie), workflow `bmad-create-story`. Worktree `/home/gcorbaz/devel/kesh-58-5`, branche
`story/58-5-cosmetique-ecrans` partie de `main` à `287deea33` (v0.20.1). Epic 65 (nettoyage, suite de l'Epic 58), release visée : vendredi 2026-12-04
(numéro attribué au tag). **Aucune migration, aucune règle métier, aucun changement d'API.** Du balisage, des classes CSS, un libellé
fr-CH, quelques tests et une phrase du manuel ; aucune compilation ni aucun gate n'a été lancé pour écrire la fiche (un agent de fiche n'exécute ni `docker` ni `curl` : ce qui
doit être vu à l'écran est prescrit au développement). Ratios de contraste des chips calculés à la main (formule WCAG 2.x, script jetable hors dépôt) : le
développement les **recalcule** avec `tests/e2e/helpers/contrast.ts`. **#996 (`--color-warning`) est sortie de la story le 2026-11-28** (C-58-5-1).*

## Story

En tant qu'utilisateur de Kesh, je veux que les colonnes de montant ne touchent plus le texte qui les suit (aperçu d'import, réconciliation, journal d'audit), que les
valeurs longues se coupent dans leur cadre (avis de conflit de la société, tuile des comptes bancaires, champ de compte) et que les
chips sélectionnés soient lisibles au seuil AA, afin de lire les écrans sans deviner où finit un mot et où commence l'autre.

## Contrôle de couverture (règle du 2026-11-26 : on vérifie qu'une issue n'est pas déjà couverte avant de la retenir)

Chaque issue a été relue (`gh issue view`, lecture seule) puis cherchée au code de `287deea33` (`grep -n`, lignes lues). Les issues voisines ouvertes : #645
(jetons de couleur non définis, dont `bg-warning-soft`) est **hors de cette story** (une issue par problème) ; aucun commit depuis le tag précédent ne cite
l'une des issues (`git log 287deea33..origin/main --format=%B | grep -oE '#(253|933|1008|1015|1016|1096|1153|1154|1184|1549)\b'` : vide au 2026-11-28, à rejouer au développement). Les doublons #1548, #1550 et #1551 sont déjà fermés vers #1154, #933 et #1016.

| # | Constat relu au code | Verdict |
|---|---|---|
| #253 | Le fond `bg-primary-light` (#3b82f6) porte du texte foncé sur **cinq sites** : `settings/email-templates/+page.svelte:253`, `:276`, `:319` (chips type, niveau, onglets de langue), `invoices/due-dates/+page.svelte:498` (onglets de statut) — quatre états sélectionnés — et `:298` (badge « Personnalisé », texte `text-primary`). Les sites que l'issue listait en plus — `(app)/+layout.svelte` (nav), `onboarding/*`, `bank-accounts/+page.svelte` — n'emploient plus le fond plein : `hover:bg-primary-light/10`, `/5`, `/20` avec `text-primary` (≥ 6,9:1), **hors défaut**. `design-system/+page.svelte:41` : étiquette `text-white` sur `bg-primary-light` (3,68:1), page de démonstration des jetons, hors des écrans de l'utilisateur (voir D3). Le contournement existe toujours : `email-templates.spec.ts:203` et `reminders.spec.ts:301` ne passent qu'avec `.disableRules(['color-contrast'])`. | **gardée** |
| #933 | `bank-import/BankImportUpload.svelte:872-873` et `:880-881` (aperçu, table `preview-tx-table`) : `<th class="text-right">Montant</th>` puis `<th class="text-left">Détails</th>`, **aucun** espacement de cellule (Tailwind en préflight remet `padding` à zéro). Même motif au tableau des lignes signalées « ressemblent à une ligne déjà importée » : `:685-686` et `:698-699` (colonne Montant à gauche, suivie de Détails). | **gardée** (deux tableaux du même écran) |
| #1008 | `homepage/BankAccountsCard.svelte:37` : `<div class="min-w-0 flex-1">` (nom du compte, `<p class="font-medium">` à `:38`) à côté de `:42` `<div class="text-right">` sans borne de largeur : sa largeur est celle de la ligne la plus longue, « Solde non disponible — lier au plan comptable » (`:58-62`, une seule ligne ; branche `currentBalance === null` à `:43`), et le nom, `min-w-0`, déborde dessus. | **gardée** (mécanisme à **confirmer** à la capture d'avant, D5) |
| #1015 | `settings/CompanyConflictNotice.svelte:37` (`text-warning`) et `:44` (`text-text-muted`) : `<p class="mt-1 text-xs …">` portant `{ $value }` — ni `break-words` ni `wrap-anywhere`. Les parents sont, pour l'adresse, des cellules de grille (`settings/+page.svelte:827-840`, dont la seule rue est en `col-span-2`) et, pour l'e-mail (le cas de l'AC10), un `<div class="flex-1">` dans un `flex` (`:907-908`) : dans les deux, un mot long élargit le conteneur. | **gardée** |
| #1016 | `reconciliation/ReconciliationProposals.svelte:708` (`th text-right` Montant) et `:756` (`td text-right`), suivis de Contrepartie `:709`, `:757` : aucun espacement. | **gardée** |
| #1096 | `invoices/[id]/+page.svelte:1093` (fiche validée) et `:1229` (annulée) : `<div class="flex min-w-0 flex-wrap justify-end gap-2">` ; le commentaire `:1089-1092` le dit (« passe à la ligne au lieu de couper « Dévalider » »). Introduit par `773df80da` (v0.19.0, 2026-11-18), **quatre jours après** l'ouverture de l'issue (2026-11-14, qui citait `:1030` d'avant). Le brouillon (`:1069`, `flex gap-2`) n'a que trois boutons. | **déjà corrigée** — sort de la story, preuve `invoices/[id]/+page.svelte:1093` ; **fermée le 2026-11-28, preuve en commentaire** ; capture de confirmation à 1400 px et 1024 px à la recette (D6), non bloquante |
| #1153 | Deux demandes. (1) « MontantContrepartie » : **aucune** chaîne soudée au catalogue (`grep -rn MontantContrepartie` vide hors fiches) : le mot soudé est l'effet visuel de #1016, **même défaut**, corrigé avec lui. (2) « Candidate » : `crates/kesh-i18n/locales/fr-CH/messages.ftl:1718` `reconciliation-cols-candidate = Candidate` ; repli `ReconciliationProposals.svelte:710`. En français, « candidate » est un mot juste (le manuel l'emploie comme nom : `user-manual.tex:3822`, `:3893`) mais, **seul en titre de colonne**, il ne dit pas de quoi : la colonne porte la facture ou la règle proposée. La demande est celle de l'auteur ; D1. | **gardée** (la moitié (2) seule, (1) étant #1016) |
| #1154 | `audit-log/+page.svelte:362` (`th py-2 text-right`, « N° ») et `:387` (`td py-2 text-right`), suivis de Détails `:363`, `:388`. | **gardée** |
| #1184 | `journal-entries/AccountAutocomplete.svelte:508-536` : `<Input>` sans marge droite, le × (`:551`, `absolute right-2`) se pose sur la fin du texte. `Input` (`ui/input/input.svelte`) passe sa classe à `cn` (tailwind-merge) : `pr-9` l'emporte sur le `px-2.5` de base. Consommateurs de `allowClear` : `products/+page.svelte:779`, `GroupSettlementModal.svelte:962`, `InvoiceForm.svelte:1190`. | **gardée** |

| #1549 | `bank-import/[id]/+page.svelte:170-171` (`th` Montant `text-right` puis Référence) et `:183-184` (`td`) : aucun espacement ; issue ouverte (`bug`, P4), **même module** (`bank-import`) et **même correctif** que #933 ; décision de l'orchestrateur du 2026-11-28 (validation P1, F4) : retenue. | **gardée**, ajoutée à la story |

**Bilan : neuf issues retenues** (les dix de l'epic, moins #1096 déjà corrigée et fermée, moins #996 sortie par décision du Project Lead, plus #1549), **deux sorties : #1096 (déjà corrigée et fermée), #996 (autre story, C-58-5-1)**. Aucune ne révèle de défaut de production ni de règle métier. Dans la borne de cinq à dix issues par story. **#1552** (écriture, « Crédit » et « Compte analytique » collés) est du même genre mais d'un **autre module** (`journal-entries`) : **angle mort assumé**, elle reste ouverte, hors de cette story.

## Choix consignés (autonomie : aucune question à Guy)

- **C-58-5-1 — #996 (`--color-warning` à 3,19:1) sort de la story** (décision du Project Lead du 2026-11-28, après les validations P1 et P2 : la remédiation du jeton recyclait ses défauts — signal de splitting préventif). Elle aura **sa propre story dans l'Epic 65**, qui reprend : le choix de la valeur du jeton, l'inventaire des usages (`text-warning`, `bg-warning/N`, `border-warning`, `color-mix`), la garde de contraste du jeton, les textes sur teinte d'avertissement (dont `settings/dunning/+page.svelte:295`), la phrase du manuel `user-manual.tex:2814` (« bordé d'orange »), les captures de recette de la couleur et la ligne de note de version. **Cette story ne touche ni `--color-warning` ni un site qui n'en dépend que par lui.** `settings/dunning/+page.svelte:295` **part avec #996** : son défaut (4,40:1) n'existe que si le jeton change (4,51:1 aujourd'hui) ; sans changement de jeton, rien à corriger ici. *Réversible* : l'autre story.
- **C-58-5-2 — le chip sélectionné prend `bg-primary text-primary-foreground`** (#1e40af / blanc = **8,72:1**), le patron déjà employé par
  `accounts/groups/+page.svelte:316` et les boutons primaires. Alternative écartée : éclaircir `--color-primary-light` — il sert aussi au contour de focus
  (`app.css:84`, `outline: 2px solid var(--color-primary-light)`) et aux survols `/10`, `/20` ; le changer élargit la story pour rien. Le **badge**
  « Personnalisé » (`:298`) devient `bg-primary-light/20 text-primary` (patron du badge de `bank-accounts/+page.svelte:1447` ; **6,92:1**), non plein : un
  badge n'est pas un état sélectionné.
- **C-58-5-3 — l'espacement des colonnes se pose par `pr-4` sur chaque colonne sauf la dernière** (en-tête **et** cellules), **dans les huit tableaux que la
  story touche** (les cinq de `BankImportUpload.svelte` — `:503`, `:604`, `:680`, `:730`, `:868` —, le détail d'import, la réconciliation, le journal d'audit) — jamais par une marge globale sur `th`/`td` (les autres tableaux de
  l'application ont déjà leurs `py-2 pr-2`/`pr-4`, ex. `reconciliation/wise/+page.svelte:133`). Même module, même correctif (décision de l'orchestrateur du
  2026-11-28) : les couples voisins du couple nommé par l'issue — journal d'audit, **toutes** les colonnes (`audit-log/+page.svelte:358-363`, `py-2` seul) ; lignes
  « ressemblent à… » (`BankImportUpload.svelte:683-687`) ; réconciliation, Contrepartie → Candidate → Score (`:709-711`) — se corrigent avec lui, sinon le même défaut
  reste à côté. **Pas au-delà de ces huit tableaux** (décision de l'orchestrateur du 2026-11-28 : les trois tables d'erreurs, de doublons et de lignes refusées de `BankImportUpload.svelte`, même fichier, même correctif, entrent) : #1552 et les autres tableaux collés de l'application restent hors de la story. Seize pixels, comme les tableaux voisins.
- **C-58-5-4 — « Candidate » devient « Proposition » en fr-CH seulement** (`Kandidat`, `Candidato`, `Candidate` restent justes dans leurs langues, et le
  français ne les imite pas). Le mot est celui que l'écran emploie ailleurs : l'astuce `reconciliation-labels-priority-hint` (« Une seule **proposition** est
  affichée par transaction », `ReconciliationProposals.svelte:697`). Cette story est la **seule** de l'epic hors 58-4 à écrire dans un catalogue : elle ne touche
  qu'**une ligne** de `fr-CH/messages.ftl` (1718), à la fusion un conflit se résout en gardant les deux côtés ; l'orchestrateur le sait (§ Questions).
- **C-58-5-5 — la valeur longue se coupe par `wrap-anywhere`** (Tailwind 4.2.2, présent dans `node_modules/tailwindcss/dist/lib.js`), non par `break-words`
  seul : `overflow-wrap: break-word` ne réduit pas la largeur minimale du contenu, donc un mot long continue d'élargir une cellule de grille ; `anywhere` le fait.

## Acceptance Criteria

1. **Chips (#253).** Les quatre sites de `bg-primary-light` en fond plein d'un état sélectionné (`email-templates/+page.svelte:253`, `:276`, `:319`, `due-dates/+page.svelte:498`) n'en
   portent plus : l'état sélectionné est `bg-primary text-primary-foreground` ; le badge `:298` est `bg-primary-light/20 text-primary`. Un état non sélectionné
   ne change pas. Ratio mesuré par `contrastRatio` : **≥ 4,5:1** (attendu 8,72 et 6,92), écrit au Dev Agent Record.
   **Test et mutation** : (a) `due-dates-page.test.ts` (rendu Vitest) : l'onglet sélectionné porte `bg-primary` et `text-primary-foreground` et **non**
   `bg-primary-light` — *mutation* : remettre `class:bg-primary-light` à `due-dates/+page.svelte:498` ; (b) Playwright, `email-templates.spec.ts` : le chip *Type*
   sélectionné a pour `getComputedStyle` fond `rgb(30, 64, 175)` et texte `rgb(255, 255, 255)` (valeurs asserties, l'E2E ne recalcule aucun contraste) — *mutation* :
   remettre `class:bg-primary-light` à `email-templates/+page.svelte:253` ; (c) aucune garde de jeton ne compte comme test d'AC1 : le ratio des jetons est vrai par construction, seul le site se mute.
2. **Axe réactivé (#253).** Le `.disableRules(['color-contrast'])` de `email-templates.spec.ts:203` et celui de `reminders.spec.ts:301` sont **retirés**, et les
   deux tests passent. Si la règle remonte un autre défaut de contraste hors des chips (`reminders-list` porte des badges), il s'ouvre en **issue** (« on trace »)
   et le `disableRules` de ce seul test reste, avec son commentaire corrigé pour citer la nouvelle issue au lieu de #253 — **si la violation vient de `--color-warning`** (p. ex. `PaymentStatusBadge .overdue`, `text-warning` sur teinte), il cite la story de #996 et ne s'ouvre pas en issue ; le commentaire de #253 disparaît dans
   les deux cas. Les onglets de langue et le badge `data-testid="email-template-badge"` (hors du `.include` actuel) passent sous axe dans un test neuf.
   **Troisième site (`invoices.spec.ts:440-462`)** : le test « axe-core sans violations sur la fiche (badge suspendu + historique) » cite #253 et #256 et neutralise `.disableRules(['color-contrast', 'button-name'])` ; #256 est close et le test voisin (`:219-224`) enforce déjà les deux règles : retirer `color-contrast` et `button-name` si le test passe ; sinon, issue neuve citée au commentaire (qui, dans tous les cas, ne renvoie plus à #253 ni à #256 comme dettes ouvertes). Au Dev Agent Record : `grep -rn "#253" frontend/tests frontend/src` avant et après (trois fichiers de tests aujourd'hui : `email-templates.spec.ts:200`, `reminders.spec.ts:298`, `invoices.spec.ts:440`). **Mutation** : remettre `class:bg-primary-light` au chip sélectionné : l'axe (3,98:1) rougit (AC1 le prouve déjà par la valeur calculée ; l'axe le double).
3. **Vacant** : #996 est sortie de la story (C-58-5-1). Les numéros d'AC ne sont pas renumérotés (les renvois restent lisibles).
4. **Vacant** : idem (les « textes sur teinte d'avertissement » partent avec #996).
5. **Aperçu d'import (#933).** Dans `BankImportUpload.svelte`, **chaque colonne sauf la dernière** des deux tableaux (principal `:871-881` ; lignes signalées
   `:683-700`, C-58-5-3) porte `pr-4` en en-tête **et** en cellule : à 1280 px, « 100.00 CHF » et son libellé sont séparés d'au moins 16 px ; aucun test existant
   ne casse. **Test et mutation** : `BankImportUpload.test.ts` : **pour chacun des cinq tableaux** (principal, lignes signalées, erreurs, doublons, lignes refusées), toute cellule `th` et `td` de chaque rangée **sauf la dernière** porte `pr-4` — *mutations* : retirer `pr-4` du `td` Montant seul, du `th` Montant seul, de la **deuxième colonne** d'un des trois petits tableaux (chacune rougit).
6. **Réconciliation (#1016).** `ReconciliationProposals.svelte:704-711` et `:752-770` : `pr-4` en en-tête **et** en cellule sur chaque colonne sauf la dernière
   (Montant, Contrepartie, Candidate, Score compris) ; capture : « -1450 CHF » et « Régie du Lac » séparés, en-tête compris. **Test et mutation** : `ReconciliationProposals.test.ts` : toute cellule `th`/`td` d'une rangée sauf la dernière (Actions) porte `pr-4`, **y compris les `td colspan="2"`** (notice et « Aucune correspondance », qui couvrent Candidate+Score et précèdent Actions : **tranché, ils portent `pr-4`**, étant suivis d'une cellule) — *mutations* : retirer `pr-4` du `td` Montant seul ; de la **deuxième colonne** ; d'un `td colspan`.
7. **Libellé (#1153).** `fr-CH/messages.ftl:1718` : `reconciliation-cols-candidate = Proposition` ; repli de `ReconciliationProposals.svelte:710` :
   `'Proposition'` ; les trois autres catalogues **inchangés** ; le manuel (`docs/manual/fr/user-manual.tex:3893`) nomme la colonne (§ Tâches). La garde i18n
   (`i18n-keys.test.ts`, `sitesTotal`) ne bouge pas (une valeur change, aucune clé). **Test et mutation** : `ReconciliationProposals.test.ts` asserte que l'en-tête de
   la colonne se lit « Proposition » (rendu avec le repli) ; un test Rust **neuf** — `la_colonne_de_la_reconciliation_se_dit_proposition_en_fr_ch`, dans `crates/kesh-api/tests/textes_coherents.rs`, sur `catalogue_fr()` (`:101`) — asserte que `reconciliation-cols-candidate` vaut « Proposition » (aucun test existant ne lit cette clé : `grep -rnF reconciliation-cols-candidate crates frontend --include=*.rs --include=*.ts` est vide) ; **tranché** : test de catalogue Rust, non assertion Vitest sur un catalogue chargé (Vitest mocke `i18nMsg` sur le repli) ; la valeur — *mutations* : remettre « Candidate » au seul repli Svelte, puis au seul catalogue (chacune rougit un des deux).
8. **Journal d'audit (#1154).** `audit-log/+page.svelte:358-363` et `:374-388` : `pr-4` en en-tête et en cellule sur chaque colonne sauf la dernière (N° compris) ;
   capture : « N° » et « Détails » séparés. **Test et mutation** : `audit-log-page.test.ts` : toute cellule `th`/`td` d'une rangée sauf la dernière porte `pr-4` — *mutations* : retirer `pr-4` du `td` « N° » seul ; de la **deuxième colonne**.
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
    (Montant et Référence, mais aussi Date, Détails, Statut) ; capture : « Montant » et « Référence » séparés. **Test et mutation** : `bank-import-detail-page.test.ts` : toute cellule `th`/`td` d'une rangée sauf la dernière porte `pr-4` — *mutations* : retirer `pr-4` du `td` Montant seul ; de la **deuxième colonne**.

## Tasks / Subtasks

- [ ] **Tâche 0 — Contrôles d'entrée (AC : 12, 13).** `git log origin/main..HEAD` propre ; rejouer le grep des dix numéros dans les commits depuis le tag.
  - [ ] Monter la base de démonstration (`docs/testing.md` § *Base de dev jetable*, bases `kesh_585`, `kesh_e2e_585`, port 3655) ; prendre **toutes les captures
        « avant »** de D6 **avant** de toucher au code.
- [ ] **Tâche 1 — Contraste des chips (AC : 1, 2).**
  - [ ] Chips : `email-templates/+page.svelte:253`, `:276`, `:319`, `due-dates/+page.svelte:498` → `class:bg-primary` + `class:text-primary-foreground` ;
        badge `:298` → `bg-primary-light/20 text-primary`. Tests d'AC1 (rendu Vitest des échéances, valeur calculée Playwright), mutations jouées et restaurées.
  - [ ] Retirer les `.disableRules(['color-contrast'])` d'AC2 (trois fichiers) ; ajouter le test axe des onglets de langue et du badge.
- [ ] **Tâche 2 — Colonnes collées (AC : 5, 6, 8, 14).** `pr-4` sur chaque colonne sauf la dernière des huit tableaux de D1, en-tête et cellule (`td colspan` compris) ; **une assertion générique par tableau** (toute cellule `th`/`td` d'une rangée sauf la dernière porte `pr-4`) dans `audit-log-page.test.ts`, `ReconciliationProposals.test.ts`, `BankImportUpload.test.ts` (cinq tableaux), `bank-import-detail-page.test.ts` ; mutations de l'AC jouées.
- [ ] **Tâche 3 — Libellé et manuel (AC : 7).** `fr-CH/messages.ftl:1718`, repli Svelte `:710` ; `user-manual.tex:3893` (colonne) ; **le test Rust ci-dessus** (`textes_coherents.rs`, seul `.rs` de la story : un fichier de test) ; **ne pas régénérer les PDF** (règle des fichiers communs : seul le `.tex` change dans une branche de story).
- [ ] **Tâche 4 — Débordements (AC : 9, 10, 11).** `BankAccountsCard.svelte`, `CompanyConflictNotice.svelte`, `AccountAutocomplete.svelte` ; une assertion de
      classe pour chacun (`HomepageCards.test.ts`, `settings-page.test.ts`, `AccountAutocomplete.test.ts`) et les mutations de l'AC ; mesures de largeur de D5.
- [ ] **Tâche 5bis — Notes de version.** Vérifier que la section *Notes de version* dit ce qui a été livré (une ligne par changement visible) ; la reporter au Dev Agent Record.
- [ ] **Tâche 5 — Recette visuelle et gates (AC : 12, 13).** Captures « après », comparaison, gate frontend, gate Rust ciblé (D7), E2E complet (une fois, au dernier
      commit de code), Dev Agent Record.

## Dev Notes

### D1 — Sites exacts des colonnes (AC 5, 6, 7, 8, 14)

**Règle (C-58-5-3)** : `pr-4` sur **chaque colonne sauf la dernière** des huit tableaux touchés, en-tête **et** cellule. Le tableau garde ses `text-left`/`text-right` ;
un `<td>` sans classe devient `<td class="pr-4">`.

| Écran | Fichier:ligne (en-tête, cellule) | Colonnes à marge |
|---|---|---|
| Aperçu d'import, table principale (`preview-tx-table`) | `BankImportUpload.svelte:871-873`, `:879-881` | Date, Montant (3 colonnes) |
| Aperçu d'import, erreurs de lecture (Ligne, Code, Valeur) | `BankImportUpload.svelte:503-520` | Ligne, Code (3 colonnes) |
| Aperçu d'import, doublons (Ligne, ID existant, Clé) | `:604-618` | Ligne, ID existant (3 colonnes) |
| Aperçu d'import, lignes refusées (Ligne, Code, Valeur) | `:730-747` | Ligne, Code (3 colonnes) |
| Aperçu d'import, lignes signalées (« ressemblent à… ») | `:683-687`, `:696-700` | Ligne, Date, Montant, Détails (5 colonnes) |
| Détail d'un import (`detail-tx-table`, #1549) | `bank-import/[id]/+page.svelte:167-184` | Date, Montant, Référence, Détails, Statut (6 colonnes ; la dernière, Rapprochement, porte des boutons) |
| Réconciliation (`reconciliation-table`) | `ReconciliationProposals.svelte:706-714`, `:752-770` | case, Date, Montant, Contrepartie, Candidate/Proposition, Score (7 colonnes ; la dernière, Actions) |
| Journal d'audit (`audit-log-table`) | `audit-log/+page.svelte:358-363`, `:374-388` | Date, Auteur, Action, Type d'entité, N° (6 colonnes) |

⚠️ La marge se pose en **en-tête et en cellule** : un en-tête sans la marge de la cellule décale les colonnes l'une de l'autre. Le tableau de
`ReconciliationProposals` fusionne des cellules (`colspan="2"`, `:730-770`) **sous** Contrepartie/Candidate : ne pas toucher aux `colspan`. `table-auto` répartit la
largeur : vérifier à 1280 px que les tables ne débordent pas de leur carte avec les marges de plus. **Angle mort assumé** : #1552 (écriture, autre module) et tout
tableau de l'application que la story ne touche pas. Si la capture d'avant montre un autre couple collé **dans ces huit tableaux** que le tableau ci-dessus ne
couvre pas, il se corrige avec ; ailleurs, il s'ouvre en issue.

### D2 — vacant (#996 est sortie de la story, C-58-5-1)

L'inventaire des usages de `--color-warning` (18 sites `text-warning`, soixantaine de fonds et bordures, replis littéraux, `bg-warning-soft` #645, `kesh-style.sty:86` #1782) part avec #996 ; le relevé `grep` de la validation P1/P2 se refait dans sa fiche.

### D3 — #253 : ce qui reste hors périmètre

- `design-system/+page.svelte:41` (swatch « P. Light », `text-white` sur #3b82f6, 3,68:1) et `:45` (swatch « Warning », `text-white` sur le jeton, relève de #996) : page de
  démonstration des jetons, hors des écrans d'usage   Ne pas retoucher ; si
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
| #933 | Banque → Import → aperçu d'un relevé (`bank-import-confirm` visible) | 1280 | table des transactions ; **et** un relevé qui produit des « lignes qui ressemblent à une ligne déjà importée » (`warning-uncertain-legacy-lines-row`) |
| #1016 | Réconciliation d'un compte (`reconciliation-table`) | 1280 | en-tête et une ligne avec contrepartie ; une ligne avec « Aucune correspondance » ; l'en-tête qui dit « Proposition » |
| #1153 | idem, **en-tête** de la table | 1280 | « Proposition » (fr-CH) ; contrôle rapide du même en-tête en de-CH, it-CH, en-CH : inchangé |
| #933 | Aperçu d'import avec **erreurs de lecture, doublons et lignes refusées** (`BankImportUpload.svelte:503`, `:604`, `:730`) | 1280 | les trois petites tables ; Ligne/Code/Valeur et Ligne/ID existant/Clé séparés |
| #1549 | Détail d'un import bancaire (`detail-tx-table`) | 1280 | en-tête et plusieurs lignes ; Montant et Référence séparés |
| #1154 | Administration → Journal d'audit (`audit-log-table`) | 1280 | en-tête et plusieurs lignes, dont une à n° d'entité et une à « — » |
| #1008 | Accueil, tuile Comptes bancaires | **1280** et **1920** | un compte lié, un compte **non lié** au nom long (« Solde non disponible… ») |
| #1015 | Paramètres → Société, avis de conflit | **375** (téléphone), 1280 | valeur de 320 caractères sans espace ; l'avis et son conteneur ne débordent pas (`scrollWidth`/`clientWidth`, D5) ; le défilement horizontal de la page est #708, hors story |
| #1184 | Écriture manuelle ou facture, champ de compte | 1280 | compte à libellé long **avec** × ; compte vide (sans × : largeur inchangée) ; compte désactivé |
| #1096 *(confirmation)* | Fiche d'une facture validée et impayée | **1400** (barre latérale ouverte) et **1024** | la barre d'actions passe à la ligne, aucun bouton coupé ; **une capture « après » seule** (l'issue est déjà corrigée) |

Contrôles de lecture : la mesure d'un ratio se fait sur la **valeur calculée** du navigateur (`getComputedStyle`), non sur la classe ; une capture d'un chip dit
son ratio dans son nom de fichier ou le Dev Agent Record.

### D7 — Tests et gate

- **Vitest (le gate) :** **aucune garde de jeton dans cette story** (celle de `--color-warning` part avec #996) ; les tests d'AC1 sont nommés à l'AC (rendu de `due-dates-page.test.ts`, valeur calculée Playwright).
  Assertions de classes : une par composant (les tests de composants existent : `HomepageCards.test.ts`, `ReconciliationProposals.test.ts`,
  `audit-log-page.test.ts`, `BankImportUpload.test.ts`, `bank-import-detail-page.test.ts`, `due-dates-page.test.ts`, `settings-page.test.ts` pour l'avis de conflit — `company-conflict.test.ts` teste la logique pure et ne rend rien —, `AccountAutocomplete.test.ts`). Ce sont des gardes de **non-régression de
  classe**, pas de rendu : le rendu est celui des captures. **Chaque AC nomme son test et sa mutation** ; le développement joue chaque mutation (restaurée ensuite, `touch` du fichier : cargo/vite gardent la mutation sinon) et l'écrit au Dev Agent Record.
- **Playwright :** `email-templates.spec.ts:198-208` (axe sans `disableRules`), `reminders.spec.ts:293-305` ; nouveau test axe des onglets de langue/badge ;
  E2E complet une fois, au dernier commit de code.
- **Gates :** `cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build` (voie frontend,
  `flock …/.gate-front.lock scripts/mem-guard.sh …`) ; **gate Rust ciblé** (un seul `.rs` change, un fichier de test — `textes_coherents.rs`, Tâche 3 —, donc `cargo fmt --all -- --check` et `cargo clippy -p kesh-api --all-targets -- -D warnings` en plus du nextest ;  le catalogue fr-CH est lu par `kesh-i18n` et par `kesh-api/tests/textes_coherents.rs:112` et `:471`, et `reconciliation-cols-candidate` change de valeur) : `flock /tmp/station-heavy.lock flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock scripts/mem-guard.sh cargo nextest run -E 'package(kesh-i18n) | binary(textes_coherents)'` (voie base, `docs/agents-orchestration.md` § 5.1 ; `scripts/test-fast.sh` n'accepte que `--no-lint` et `--ci`, aucun filtre). Le Dev Agent Record dit ce qui a tourné ; le story file n'affirme que ce qui a tourné.
- **Jetons non définis** : `jetons-non-definis.test.ts` doit rester vert (`wrap-anywhere`, `pr-4`, `max-w-[60%]` ne sont pas des classes de couleur) ; vérifier
  qu'aucune classe de couleur neuve n'est écrite.

### Project Structure Notes

- Aucun fichier de production nouveau : balisage Svelte, une ligne de `fr-CH/messages.ftl`, `user-manual.tex`. Aucun fichier de test frontend neuf ; un test Rust neuf dans `textes_coherents.rs` (AC7). `app.css` **n'est pas touché** (#996 sortie).
- **Fichiers partagés, à déclarer** : (1) **`crates/kesh-i18n/locales/fr-CH/messages.ftl:1718`** (`reconciliation-cols-candidate`) : une ligne **existante**, hors du bloc d'un epic, que la 58-4 (#1086, espaces insécables sur tout le catalogue fr-CH) peut réécrire à côté ; la fiche 58-4 affirme que la 58-5 n'y touche pas : **l'une des deux fiches se corrige** (la 58-4, selon l'orchestrateur) et le conflit se résout à la fusion en gardant les deux côtés ; (2) **`frontend/src/lib/features/journal-entries/AccountAutocomplete.svelte`** (production, `Input class` et dérivée de l'AC11, vers `:508-551`), que la 58-6 modifie aussi (branche `Tab` de `handleKeydown`) : régions distinctes, fusion en gardant les deux côtés ; (3) **`frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte`** (production, `:704-714`, `:752-770`), que la 58-6 modifie aussi (garde de `confirmGesture`, vers `:520`) : zones **disjointes**, fusion en gardant les deux côtés. La 58-6 touche `reconciliation/*.test.ts` et `AccountAutocomplete*` : **`ReconciliationProposals.test.ts`** et **`AccountAutocomplete.test.ts`** sont
  communs ; une assertion ajoutée ici (classe) se résout à la fusion en gardant les deux côtés. `docs/manual/fr/user-manual.tex` : la 58-4 peut y toucher.

### References

- [Source: `_bmad-output/planning-artifacts/epic-65-nettoyage-suite.md` § Story 58-5]
- [Source: `frontend/tests/e2e/helpers/contrast.ts`, `tests/e2e/helpers/theme.ts` ; `frontend/src/lib/features/journal-entries/account-autocomplete-classes.test.ts`
  (patron de garde de contraste) ; `frontend/src/lib/components/ui/jetons-non-definis.test.ts` (#645)]
- [Source: CLAUDE.md § *Recette visuelle*, § *Fichiers communs* ; `docs/testing.md` § *Les échecs attendus*]

## Questions et risques (pour l'orchestrateur)

1. **Un conflit de fusion annoncé** : `fr-CH/messages.ftl:1718` (une ligne) avec la 58-4, `user-manual.tex` avec la 58-4 éventuellement.
2. **#1096 est fermée** (le 2026-11-28, preuve en commentaire : `invoices/[id]/+page.svelte:1093`, v0.19.0) ; aucun commit de la story ne la porte en `refs` utile.
3. **#996 est sortie** (décision du Project Lead du 2026-11-28, C-58-5-1) : sa story est à écrire par l'orchestrateur ; la liste de ce qu'elle reprend est au Change Log.
4. **Risque #253 / AC2** : retirer `disableRules` peut révéler d'autres contrastes sur `reminders-list` ; règle écrite à l'AC2 (issue neuve, jamais corrigée dans
   cette story).
5. **#1782** (ouverte, `kesh-style.sty:86`) part avec #996.
6. **#1552** (écriture, « Crédit » et « Compte analytique » collés) : autre module, angle mort assumé, reste ouverte.
7. **#1008, #1015, #1184 sont déduites du code**, non reproduites : la capture d'avant décide (D5).

## Notes de version

*(Matière du CHANGELOG, assemblée à l'intégration ; ni numéro d'epic ni numéro de version.)* Les colonnes de montant ne touchent plus le texte qui les suit dans les écrans de l'import bancaire (aperçu, détail), la réconciliation et le journal d'audit ; la colonne « Candidate » de la réconciliation se lit « Proposition » en français ; les valeurs longues se coupent dans leur cadre (avis de conflit de la société, tuile des comptes bancaires) et le champ de compte réserve la place du bouton d'effacement ; les chips sélectionnés des modèles d'e-mail et des échéances passent au contraste AA. Aucune migration, aucun changement d'API.

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
- 2026-11-28 — **Validation P2 (Sonnet) : 0/1/5/6, remédiée hors #996 (en attente du Project Lead).** P2-2 : test Rust neuf `textes_coherents.rs` (D7, fmt et clippy ajoutés). P2-3 : `invoices.spec.ts:440-462` à l'AC2. P2-4 : les trois tables de `BankImportUpload.svelte` (`:503`, `:604`, `:730`) entrent, huit tableaux. P2-5 : assertion générique par tableau, `td colspan` tranchés (`pr-4`). P2-7 : #1782. P2-8 : `AccountAutocomplete.svelte` partagé avec la 58-6. P2-9 : grep borné à `origin/main`. P2-10 : bilan et AC1 recomptés, `sprint-status.yaml` mis à jour (les ratios arrondis de #996 restent à reprendre avec elle). P2-11 : *Notes de version*. **Non touchés, en attente de la décision sur #996 : P2-1, P2-6, P2-12 (calculé/mesuré, AC4).**
- 2026-11-28 — **#996 sort de la story** (décision du Project Lead, C-58-5-1) ; la story compte **neuf issues** (#1549 comprise), `sprint-status.yaml` suivi. Partent avec #996 : C-58-5-1 d'origine (valeur `#92400e`, alternatives, barème de ratios), AC3 et AC4 (vacants), la garde `jetons-contraste.test.ts`, D2 (inventaire des 18 sites `text-warning` et des fonds/bordures), `settings/dunning/+page.svelte:295` (le défaut n'existe que si le jeton change), `user-manual.tex:2814` « bordé d'orange », la capture du manuel `md-afc-corrige.png` (bandeau `ExchangeRateNoticesBanner`), les lignes de recette D6 de la couleur d'avertissement, `kesh-style.sty:86` (#1782), le repli littéral `#d97706`, la note de version de la couleur. **Findings sans objet : P2-1, P2-6, P2-12 et les ratios de P2-10.** Restent dans la 58-5 : les chips de #253 (aucune dépendance au jeton), avec la clause d'AC2 (une violation de `reminders-list` due à `--color-warning` cite la story de #996).
