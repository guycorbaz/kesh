# Story 58-4 : Textes d'interface et catalogues i18n (#324, #339, #844, #861, #873, #874, #908, #1049, #1491)

Status: drafted

*Fiche rédigée le 2026-11-28 (Sonnet 5.5), worktree `kesh-58-4`, branche `story/58-4-textes-interface-i18n` partie de `epic/65-nettoyage-suite`
(`287deea33`, v0.20.1), par le workflow `bmad-create-story`, en autonomie. Epic 65 (suite de l'Epic 58 réduit, décision du Project Lead du
2026-11-27), release visée vendredi 2026-12-04, coupure jeudi 2026-12-03 au soir. **Aucune migration, aucune règle métier, aucun changement d'API**
(un texte de message serveur change, jamais un code, un champ ni un `details`). Aucune compilation ni aucun gate lancé pour écrire la fiche, hormis
`vitest run src/lib/shared/i18n-keys.test.ts` (vert, `sitesTotal` = 3985) ; rien n'est commité hors de ce fichier et du statut dans
`sprint-status.yaml`.*

## Story

En tant qu'utilisateur non francophone (ou francophone qui lit un message d'erreur), je veux que le bouton d'une écriture, la liste des factures, la
coque de l'application et quelques messages disent ce que Kesh fait, dans ma langue, au format suisse, afin de ne plus valider une écriture
irréversible sous le mot « Speichern », ni lire « Statut » en français à côté d'un filtre traduit.

## Contrôle de couverture (règle du 2026-11-26 : on vérifie qu'une issue n'est pas déjà couverte)

Chaque issue est relue au code **tel qu'il est à `287deea33`** (les numéros de ligne de plusieurs issues ont bougé).

| # | Constat au code | Verdict |
|---|---|---|
| #324 | `journal-entry-form-submit` : `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl:330` = Valider / Speichern / Salva / Save ; un seul site : `JournalEntryForm.svelte:1267` | **gardée** |
| #339 | `routes/(app)/invoices/+page.svelte:288` (titre), `:297`, `:307`, `:314-317`, `:337`, `:344`, `:348`, `:362`, `:404-413` : français en dur, à côté de `invoice-paused-filter-*` traduits (`:322-334`) | **gardée, périmètre scindé** (voir D1) |
| #844 | `homepage-bank-no-transactions` : `messages.ftl:181` ×4 locales ; `grep -rn "homepage-bank-no-transactions" frontend/src crates --include=*.ts --include=*.svelte --include=*.rs` vide ; aucune construction dynamique (`grep "homepage-bank-"` ne rend que des `data-testid`) | **gardée** |
| #861 | placeholder : `routes/(app)/+layout.svelte:345` ; pied de page : `routes/(app)/+layout.svelte:496` et `routes/onboarding/+layout.svelte:38` (les lignes de l'issue ont bougé) | **gardée** |
| #873 | `supplier-invoices-unvalidate-blocked-later-fiscal-year-closed-named` (`messages.ftl:3384` fr, `:3340` de, `:3341` it, `:3346` en) nomme l'exercice postérieur clos **le plus proche** (`find_later_closed_in_tx`, `ORDER BY start_date ASC`, `fiscal_years.rs:1140`) puis dit « en commençant par le plus récent » | **gardée** (voisines signalées, voir D4) |
| #874 | it-CH mêle les deux formes (relevé en D5) | **gardée** |
| #908 | `crates/kesh-api/src/errors.rs:6479-6495` : `locked_through.to_string()` / `attempted.to_string()` dans les arguments de `journal-entries-period-locked` (ISO) ; fallback `:6484` idem ; test pinant l'ISO : `crates/kesh-api/tests/period_lock_e2e.rs:398-404` | **gardée** |
| #1049 | `messages.ftl:1367` (fr-CH, `error-opening-complement-retained-earnings-line`) et `errors.rs:2581` : « d'ouverture » à apostrophe droite (la ligne de l'issue, 1165, a bougé) | **gardée** |
| #1086 | 159 lignes fr-CH portent des guillemets « » à espace ordinaire, aucune insécable (`grep -cP '«\x{00A0}' = 0`) ; **ces mêmes chaînes sont répétées comme replis** : 2968 lignes dans 505 fichiers de `frontend/src`, 3004 lignes dans 424 fichiers `.rs`, 489 lignes d'E2E, et la garde G9 (`textes_coherents.rs`) comme `i18n-un-repli-par-cle` exigent l'égalité repli = catalogue | **sortie de la story** (voir D2) |
| #1491 | `messages.ftl:3808` (fr), `:3729` (de), `:3730` (it), `:3735` (en) : « Doublon de facture fournisseur confirmé » ; aucun autre site (ni code, ni test, ni manuel : `grep` vide hors catalogues) | **gardée** |

Aucune de ces issues n'est déjà corrigée. `git log --grep` n'a pas été relu issue par issue : aucune n'a été touchée par les epics 55, 58 (58-1 à 58-3) ni 64
au vu des lignes ci-dessus (tous les textes sont tels que l'issue les décrit). **Dix issues annoncées, neuf retenues** : #1086 sort.

## Ce qui se lit au code

### #324 — le bouton d'une écriture

`JournalEntryForm.svelte:1266-1268` : `data-testid="journal-entry-submit"`, libellé `i18nMsg('journal-entry-form-submit', 'Valider')`. La clé n'a **aucun
autre site** (Rust, TS, Svelte). Les E2E sélectionnent le bouton par `getByRole("button", { name: "Valider" })` (`journal-entries.spec.ts:129,190,234,261,295`,
`fiscal-years.spec.ts:248`) ou par `data-testid` (`fx-entries.spec.ts:146,486,493`) : **le français ne change pas**, donc aucun E2E ne bouge. Le manuel
(`docs/manual/fr/user-manual.tex:713`, « corrigez, puis \emph{Valider} ») nomme ce bouton : il reste vrai.

Collisions contrôlées au catalogue (`grep -nE "= (…)$"` par locale) : `Buchen` (de) et `Contabilizza` (it) **n'existent nulle part** comme valeur ; `Post entry` (en)
est déjà la valeur de `reconciliation-manual-submit` (`en-CH:1664`), même sens (comptabiliser une écriture), autre écran — pas de collision au sens de
la 23-3. `Registra` (it) est écarté : déjà `supplier-invoices-new` (`it-CH:3256`) et neuf boutons « Enregistrer ».

### #339 — français en dur dans le balisage de la liste des factures

Le **catalogue possède déjà presque toutes les clés** de cette page, que la page n'appelle pas : `invoices-page-title` (628), `invoice-new-button` (688),
`invoice-filter-search` (677, « Rechercher… »), `invoice-filter-status-all` (678), `invoice-filter-contact-all` (679), `invoice-filter-date-from` (680),
`invoice-filter-date-to` (681), `invoice-col-date|contact|number|status|total|actions` (668-673), `invoice-status-*` (674-676), `common-loading|previous|next`
(1071-1073), `invoice-delete-button` (690). `invoice-empty-list` (693) dit **plus** que le texte actuel (« Aucune facture. » seul) : choix D1.

Textes en dur de la page à couvrir (le dev **relit le fichier en entier**, l'heuristique ci-dessous n'est qu'un plancher) : `<svelte:head><title>` (`Factures — Kesh`),
`h1`, bouton « Nouvelle facture », labels « Recherche », « Statut », « Contact », « Depuis », « Jusqu'à », options « Tous les statuts », « Brouillon »,
« Validée », « Annulée », placeholder « Rechercher… », placeholder « Tous les contacts » (`:344`), bouton « Réinitialiser », « Chargement… », « Aucune facture. »,
`<th>` Date / Contact / N° / Statut / Total / Actions, les trois `aria-label` `Voir|Modifier|Supprimer la facture {n}` (`:435,439,447`), le pluriel `{total} facture{s}`
(`:458`), « Précédent », « Suivant », le message `dateRangeError` (`:158`) et `deleteError = 'Erreur lors de la suppression'` (`:253`).

Heuristique de relevé (celle de l'issue, ne voit ni attributs ni multi-lignes) : `grep -rnE '<(option|label|th|h[1-3]|button)[^>]*>[^<{]*[A-Za-zéèà]{3,}[^<{]*</' --include=*.svelte frontend/src | grep -v 'i18nMsg\|design-system'`
rend aujourd'hui, par fichier : `invoices/+page.svelte` 13, `bank-import/BankProfileForm.svelte` 12, `users/+page.svelte` 9, `invoices/InvoiceForm.svelte` 9,
`invoices/[id]/+page.svelte` 7, `journal-entries/[id]/+page.svelte` 4, `contacts/+page.svelte` 2, `invoices/new/+page.svelte` 1, `invoices/due-dates` 1,
`invoices/[id]/edit` 1, `payment-batches/[id]` 1, `login` 1 — **soit 61 sites dans 12 fichiers**, contre 48 dans l'issue (le plancher a monté).

### #844 — clé morte

Quatre lignes `messages.ftl:181`. Le garde « aucune clé orpheline » (`i18n-keys.test.ts:1761`) ne couvre que des préfixes à couverture close : le dev vérifie si
`homepage-` en fait partie ; sinon le retrait ne change aucun compteur.

### #861 — coque

`+layout.svelte:345` `placeholder="Rechercher..."` ; `:496` pied de page `Kesh v{appVersion.value} &mdash; Logiciel libre (EUPL 1.2). Les données ne remplacent pas un fiduciaire.` ;
`onboarding/+layout.svelte:38` même texte (entités HTML `&eacute;`), `i18nMsg` déjà importé. Les messages se chargent via `loadI18nMessages()`
(`/api/v1/i18n/messages`) ; **avant ce chargement le repli fr s'affiche** (cas de l'onboarding avant la première société : le repli sert, c'est voulu).
`invoice-filter-search` (« Rechercher… », points de suspension typographiques) **ne se réutilise pas** : namespace de la facture, et le placeholder de la coque s'écrit avec
trois points (« Rechercher... ») — un repli égal à fr-CH exige la valeur du catalogue, donc le **même texte** : voir D3.

### #873 — nom de l'exercice et consigne

Le message nomme le **premier** exercice postérieur clos (le plus proche) et prescrit de rouvrir « en commençant par le plus récent » : deux exercices clos
(2027 et 2028) font nommer 2027 et rouvrir 2028 d'abord. Le correctif (D4) dit **explicitement** que le nom est celui du plus proche ; la consigne reste vraie. Sites :
quatre valeurs de catalogue + le repli de `supplier-invoice-unvalidate.ts:142-146` (garde repli = fr-CH) ; le test
`supplier-invoice-unvalidate.test.ts:152` lit le catalogue (`rempli(...)`) et suit sans retouche.

### #908 — dates ISO

Une seule clé de message serveur : `journal-entries-period-locked` (4 locales, `:486`/`:492`). `errors.rs:6479-6495` : args `lockedThrough` et `attempted` = `NaiveDate::to_string()`
(AAAA-MM-JJ) ; `details` en ISO **pour le client** (contrat inchangé, #869). Il existe déjà `date.format("%d.%m.%Y")` à `errors.rs:3412` (`RATE_UNAVAILABLE`) : même forme. Le fallback Rust
(`:6484`) interpole les mêmes dates et doit suivre. La clé de l'écran (`settings-books-lock-current`, etc.) est déjà au format suisse côté TS (`swissDate`) : hors sujet.
Le batch (`period_locked_failed_proposal`, `reconciliation.rs:609`) ne porte pas de message : rien à faire.

### #1049 — apostrophe

`messages.ftl:1367` fr-CH : « d'ouverture » → « d’ouverture » ; même changement au repli `errors.rs:2581` (la garde G9 exige l'égalité). Il n'y a **que** ce site pour
cette clé (les trois autres locales n'ont pas d'apostrophe droite ici : `it-CH:1298` a déjà `’`).

### #1491 — libellé d'audit

Quatre valeurs `audit-log-action-supplier-invoice-duplicate-confirmed`. La trace naît de « Ce n'est pas un doublon, enregistrer » (saisie) et du repassage d'une annulée
(manuel : `user-manual.tex:2238`, `admin-manual.tex` « duplicate\_confirmed » = **nom de code**, non le libellé : à laisser).

## Inventaire par le symptôme (propagation post-patch)

- `Valider|Speichern|Salva|Save` : seule `journal-entry-form-submit` est en cause ; les autres valeurs « Speichern »/« Salva »/« Save » sont des boutons d'enregistrement ordinaires.
- Texte du pied de page : `grep -rn "ne remplacent pas un fiduciaire" frontend crates docs website` (2 sites Svelte aujourd'hui ; le dev greppe aussi `website/` et les manuels).
- `Rechercher\.\.\.` : un site (`+layout.svelte:345`) ; `grep -rn "Rechercher\.\.\." frontend/tests` vide.
- Dates ISO dans les messages serveur : `grep -n "to_string()" crates/kesh-api/src/errors.rs` autour des `args.set` pour les autres clés à date — **hors périmètre** (une issue par problème) ; le dev ne les corrige pas, il signale.

## Décisions (choix autonomes, `C-58-4-k`)

- **D1 (`C-58-4-a`) — #339 : la liste des factures seule, le reste en issue neuve.** L'issue est une **classe** (61 sites dans 12 fichiers, plancher) ; la règle « pas d'issue
  résolue par morceaux sur plusieurs releases : scinder avant » appelle un découpage au lieu d'un demi-traitement. La story couvre **`routes/(app)/invoices/+page.svelte` en entier**
  (balisage, attributs, `<title>`, messages d'erreur de la page) **et `routes/(app)/invoices/new/+page.svelte` (`h1`)**, soit le « cas d'école » de l'issue. **Les autres fichiers
  du relevé (11) sortent** (choix `C-58-4-k` : #339 est **réduite** à la liste des factures et au `h1` de `invoices/new` ; l'orchestrateur la découpe avant le développement) : l'orchestrateur ouvre **une issue neuve par famille** (écrans de facture `[id]`, `[id]/edit`, `InvoiceForm`, `due-dates` ; `BankProfileForm` ;
  `users` ; `journal-entries/[id]` ; `contacts`, `payment-batches/[id]`, `login`) en citant le relevé ci-dessus, et commente #339 (« traitée pour la liste des factures ; reste : #…
  »). **Garde** : l'inventaire exige l'ensemble clos des sites qui ne résolvent pas (voir AC4), pas l'énumération des formes traitées. Pas de garde générale (même calcul que la 23-3b : la proscrire).
  Pour « Aucune facture. », garder `invoice-empty-list` **n'est pas** un choix neutre (le texte du catalogue est plus long) : on **garde le texte fr de l'écran** par une clé
  neuve `invoice-list-empty` (valeur : « Aucune facture. », traduite), et `invoice-empty-list` n'est pas touchée (elle a peut-être un autre site : le dev le vérifie avant toute décision).
  Toute autre clé du catalogue dont le repli fr serait **différent** du texte actuel de l'écran se traite de même (clé neuve) : l'écran ne change pas en français.
- **D2 (`C-58-4-b`) — #1086 sort de la story.** Le remède demandé (remplacer l'espace après « et avant » dans 159 lignes fr-CH) **ne peut pas se limiter au catalogue** : la garde G9 et
  `i18n-un-repli-par-cle` exigent le même texte au repli (≈ 5972 lignes à lire, 489 lignes d'E2E qui sélectionnent par texte), et un test qui rejoue du texte fr contre le catalogue rougit sur tout désaccord. Ce n'est
  pas « peu coûteux » (coût réel L, centaines de sites répartis sur plusieurs modules : la règle de splitting préventif s'applique). Piste à chiffrer dans une story à part : **un traitement unique au rendu**
  (`kesh-i18n::I18nBundle::format`/`all_messages` côté serveur ; `i18nMsg` côté frontend) qui pose l'espace insécable après « et avant », sans toucher ni catalogue ni replis, avec une garde sur la **sortie**
  rendue. L'orchestrateur la garde ouverte (#1086 reste ouverte), à replanifier ; *aucune écriture* n'est faite ici.
- **D3 (`C-58-4-c`) — clés de la coque.** Deux clés neuves hors des namespaces de feature : `layout-search-placeholder` (fr-CH : `Rechercher...`, **trois points ASCII, valeur actuelle** ; les trois autres langues avec
  leurs points de suspension habituels) et `layout-footer-disclaimer` (fr-CH : `Kesh v{ $version } — Logiciel libre (EUPL 1.2). Les données ne remplacent pas un fiduciaire.`, le tiret cadratin en
  caractère, plus `&mdash;`). Un seul texte pour les **deux** layouts (app et onboarding), donc deux sites `i18nMsg` du même repli (garde `i18n-un-repli-par-cle` : un repli par clé, **identique** aux deux sites).
  La variable `$version` passe par `{ version: appVersion.value }` (garde `i18n-entrees-a-variables.test.ts` : le dev l'applique comme pour les autres clés à variable). Traductions : de « Kesh v{ $version } — Freie
  Software (EUPL 1.2). Die Daten ersetzen keine Treuhänderin oder keinen Treuhänder. » (le dev relit la forme sur les catalogues voisins : `Treuhand`/`Fiduciaria`/`fiduciary` déjà employés ? `grep`), it « … I dati non sostituiscono
  una fiduciaria. », en « … The data does not replace a fiduciary. » : **le dev reprend le terme du catalogue** (`grep -in "fiduciar\|treuh" messages.ftl`) et le consigne.
- **D4 (`C-58-4-d`) — #873 : une clé, le nom dit « le plus proche ».** fr : « L’exercice postérieur « { $name } » (le plus proche) est clôturé : pour pouvoir repasser cette facture en brouillon, un administrateur doit
  rouvrir les exercices postérieurs clôturés, en commençant par le plus récent. » ; de/it/en traduits sur le même modèle (« nächstgelegene », « il più vicino », « the nearest »). Le repli de
  `supplier-invoice-unvalidate.ts` s'aligne. **Les deux voisines entrent dans la story** (décision de l'orchestrateur du 2026-11-28 : même module, même défaut, un LOW se corrige dans la story ; rattachées à #873) : `error-later-fiscal-year-closed` (`messages.ftl:482` fr ; repli `errors.rs:5695`, via `later_fiscal_year_closed_response`, `errors.rs:2123`, qui fournit `details.fiscalYearName`) et `journal-entries-modify-blocked-later-fiscal-year-closed` (`messages.ftl:481` fr ; repli `blocker-messages.ts:95`). Pour **chacune**, le dev lit au code que le nom servi est bien celui du premier exercice clos (`find_later_closed` / `find_later_closed_in_tx`, `ORDER BY start_date ASC`, `fiscal_years.rs:1140`) et consigne la ligne lue ; si l'une des deux nomme autre chose, le texte dit ce que le code nomme. Même correctif de forme (« le plus proche » / « nächstgelegene » / « il più vicino » / « the nearest », la consigne « en commençant par le plus récent » inchangée), quatre locales, repli égal au fr-CH. **Tests qui épinglent le texte** à mettre à jour avec la nouvelle valeur : `invoice-validate-page.test.ts:164`, `fiscal-years-page.test.ts:334`, `fiscal_years_e2e.rs:2340` (`msg.contains("« Exercice 2026 », postérieur, est clôturé")` — vérifier si elle porte sur `error-later-…` ou sur la clé de création `error-fiscal-year-create-later-closed`, `errors.rs:2919`, **qui n'est pas concernée** : elle n'a pas la consigne), G9, `loader.rs:1373` (liste des clés à variable), `textes_coherents.rs:825`. Le grep du symptôme (`postérieur`, `plus récent`) sur le dépôt, manuels compris, s'exécute avant la passe.
- **`C-58-4-k` — #339 réduite** (voir D1) : le relevé par famille, avec sites `fichier:ligne`, nombres et titres d'issue proposés, est dans `kesh-gate-logs/e65/58-4/suite-339.md`.
- **D5 (`C-58-4-e`) — #874 : une seule forme, le « voi ».** Mesure (script de relevé, voir AC8) sur les 4646 lignes de `it-CH` limitées aux messages de plus de 40 caractères : **124 messages au voi (pluriel :
  « annullate », « correggetela », « riattivatelo », « createlo »…), 64 au tu (« annulla », « riportala », « considera »…)** ; le voi est la majorité **et** le registre de politesse parallèle du « vous » fr-CH et du « Sie »
  de-CH. Les messages au tu sont **ramenés au voi** (impératif pluriel, pronoms `vi`/`vostro`). Les boutons à verbe nu à l'infinitif/impératif neutre (« Salva », « Annulla »,
  « Registra ») ne sont **pas** touchés : ils sont des étiquettes, non une adresse. Le relevé exact (lignes, formes) est la première tâche du dev et se consigne dans le Dev Agent Record ; **toute ligne ambiguë est listée
  comme angle mort assumé**, non devinée (CLAUDE.md § *Inventorier les sites NON RÉSOLUS*). Les replis TS/Rust sont en français : **aucun repli ne change**.
- **D6 (`C-58-4-f`) — #324 : le français ne change pas.** « Valider » (fr) reste : il est cité par le manuel (`user-manual.tex:713`) et par dix E2E, et il porte déjà l'idée de l'acte. de **Buchen**, it **Contabilizza**, en **Post entry**.
  Le repli `Valider` du Svelte est déjà égal au fr.
- **D7 (`C-58-4-g`) — #908 : une fonction unique de mise en forme.** Dans `errors.rs`, une fonction `fn swiss_date(d: chrono::NaiveDate) -> String { d.format("%d.%m.%Y").to_string() }` (ou l'existante si
  `grep -n 'fn .*swiss' crates` en trouve une réutilisable : à chercher d'abord, DRY) sert les deux arguments **et** le repli ; **`details` reste en ISO**. Exemple attendu : « verrouillées jusqu'au 31.03.2026 ; celle-ci est datée du 15.01.2026. » **L'apostrophe droite de « jusqu'au »
  n'est pas touchée** (hors périmètre : un symptôme, une issue).
- **D8 (`C-58-4-h`) — #1491 : libellé neutre, pas « probable ».** fr « Facture fournisseur enregistrée malgré un doublon signalé » ; de « Lieferantenrechnung trotz gemeldetem Duplikat erfasst » ; it « Fattura fornitore
  registrata nonostante un doppione segnalato » ; en « Supplier invoice recorded despite a flagged duplicate ». *Motif* : la trace couvre aussi le doublon **certain** (même numéro) confirmé au repassage d'une annulée, que
  « probable » (suggestion de l'issue) fausserait.

## Critères d'acceptation

1. **AC1 (#324)** — `journal-entry-form-submit` = « Valider » / « Buchen » / « Contabilizza » / « Post entry » (fr-CH inchangé). Un test Vitest (dans un fichier de la feature journal-entries ou celui de ses libellés) pose les **quatre** valeurs lues dans les
   catalogues **et** vérifie qu'aucune des trois cibles n'est égale à `common-save` de sa locale (la parité ne le voit pas, l'issue le dit). Aucun E2E modifié.
2. **AC2 (#844)** — `homepage-bank-no-transactions` n'existe plus dans aucun des quatre catalogues ; aucune garde n'est cassée ; le dev consigne si `homepage-` est un préfixe à couverture close.
3. **AC3 (#861)** — le placeholder de recherche et le pied de page de `(app)/+layout.svelte` et de `onboarding/+layout.svelte` passent par `i18nMsg` avec les clés de D3, présentes dans les quatre catalogues ; en français, **rendu identique caractère pour caractère**
   (le tiret cadratin et « données » rendus à l'identique) ; avec `KESH_LANG=de` les deux textes sont en allemand (capture).
4. **AC4 (#339, liste des factures)** — `routes/(app)/invoices/+page.svelte` ne contient plus de texte français hors `i18nMsg` (balisage, `placeholder`, `aria-label`, `<title>`, messages) ; `invoices/new/+page.svelte` idem pour son `h1`. **Inventaire clos** : le dev
   produit la liste des sites **résolus** et celle de ce qui **reste** dans les deux fichiers (angle mort assumé, ex. une valeur d'API affichée telle quelle), chacune lue au code. Chaque repli = valeur fr-CH de sa clé
   (gardes `i18n-un-repli-par-cle`, `i18n-repli-divergent-actif`). Les clés neuves existent dans les **quatre** catalogues, sans collision de libellé contrôlée (méthode de la 23-3, § AC7 de `23-3-supplier-invoices.md`). Un test Vitest monte la page
   avec un catalogue **de** (messages de test) et vérifie que les libellés « Statut », les trois options du filtre de statut, les en-têtes de colonnes sont allemands. **L'écran en français ne change pas** (E2E de la liste inchangés).
5. **AC5 (#873)** — les **trois** valeurs `supplier-invoices-unvalidate-blocked-later-fiscal-year-closed-named`, `error-later-fiscal-year-closed` et `journal-entries-modify-blocked-later-fiscal-year-closed` disent « le plus proche » dans les quatre langues (D4), chaque repli (TS ou Rust) égale le fr-CH ; le contrôle `find_later_closed_in_tx` (tri `ASC`) est lu et cité au Dev Agent Record **pour chacune des trois clés** ; `supplier-invoice-unvalidate.test.ts:152` reste vert sans modification ; les tests qui épinglent le texte (D4) sont mis à jour et verts.
6. **AC6 (#908)** — le message `journal-entries-period-locked` rend les deux dates au format `JJ.MM.AAAA` en fr-CH, de-CH, it-CH, en-CH ; **`details.lockedThrough` et `details.attempted` restent en ISO** ; le repli Rust égale le catalogue (G9) ;
   `period_lock_e2e.rs:398-404` est mis à jour **avec la même date formatée** (le test, et non son attente, prouve la forme : une assertion qui reconstruit la date avec la même fonction que le code ne prouve rien — écrire la date attendue **en littéral** `31.03.2026` pour une borne connue).
7. **AC7 (#1049)** — `error-opening-complement-retained-earnings-line` (fr-CH) et son repli `errors.rs:2581` portent « d’ouverture » (U+2019) ; G9 vert. `grep -n "d'ouverture" crates/kesh-i18n/locales/fr-CH/messages.ftl crates/kesh-api/src/errors.rs` ne rend plus ce site (les autres occurrences éventuelles sont des symptômes **différents** : listées, non traitées).
8. **AC8 (#874)** — `it-CH` : une seule forme d'adresse dans **tout** le catalogue. Le relevé (script de lecture posé en `kesh-gate-logs/e65/58-4/`, formes de tu et de voi, **liste fermée** des formes cherchées) est joint au Dev Agent Record avec : lignes converties, lignes ambiguës laissées (justifiées).
   Un test Rust de `kesh-i18n` (module `tests` de `loader.rs`, à côté de `parity_between_locales`) lit `it-CH` et échoue si un message porte une des formes de tu **de la liste fermée du relevé** (ex. impératifs en `-a/-i` isolés : « Annulla prima », « riportala », « considera ») ; **la liste des formes est explicite et l'écart assumé** (le test ne prouve pas l'absence de toute forme de tu, il garde celles relevées).
   Les clés citées par l'issue (`supplier-invoices-cancel-*`, `error-*-must-be-unvalidated-first`, `invoices-settlement-cancel-blocked-written-off`, `reconciliation-cancel-blocked-written-off`, `bank-import-manual-delete-blocked`) sont **déjà au voi** (relevé au code) : elles ne bougent pas — c'est le tu qui est ramené au voi.
9. **AC9 (#1491)** — les quatre valeurs de `audit-log-action-supplier-invoice-duplicate-confirmed` suivent D8. Le journal d'audit les affiche (capture).
10. **AC10 (compteur)** — `sitesTotal` de `frontend/src/lib/shared/i18n-keys.test.ts` (ligne 1300, **3985** à `287deea33`) se **recompte depuis la garde** (« expected N to be 3985 »), et le commentaire de la ligne s'enrichit d'une entrée datée avec le détail (sites de la coque +3, liste des factures +≈ 20 à relever, 0 pour #324/#873/#874/#908/#1049/#1491/#844). `sitesNonResolus` (31), `relais` (6), `sitesGabarit` (12) **ne bougent pas** : le dev le contrôle ; un écart se **recompte**, il ne s'ajuste pas.
11. **AC11 (parité et gardes)** — `parity_between_locales` (`kesh-i18n`), `loader.rs:1373` (liste de clés à variables `error-later-…`), `textes_coherents.rs` (G9 et ses voisines), `lint-i18n-ownership`, `i18n-entrees-a-variables`, `i18n-un-repli-par-cle`, `i18n-repli-divergent-actif` verts.
12. **AC12 (manuel)** — voir *Manuel* ci-dessous : aucun passage du manuel ne cite un texte changé ; ce contrôle (texte **et** PDF aplati) est rejoué par le dev avant la clôture et consigné.

## Manuel (`docs/manual/`, texte et PDF aplati)

Relevé à la rédaction : `grep -rn` des chaînes changées dans `docs/manual/*/*.tex` et dans `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '` (fr user + admin) :

- « Valider » (#324) : `user-manual.tex:713` et `:2374` (autre bouton, la facture fournisseur) : **inchangés** (D6), toujours vrais ; les manuels de/it/en citent-ils le bouton ? le dev greppe `docs/manual/{de,it,en}/*.tex` pour « Speichern »/« Salva »/« Save » dans le passage de la saisie d'écriture et corrige le `.tex` si le texte **dit** le libellé de ce bouton (seul le `.tex`, jamais le PDF : règle des fichiers communs).
- « Rechercher... », « ne remplacent pas un fiduciaire », « Doublon de facture fournisseur confirmé », « verrouillées jusqu'au » : **aucune occurrence** dans les `.tex` ni dans le PDF aplati fr (user/admin) ; `d'ouverture` : sans objet pour les manuels (message d'erreur) — à regreper par le dev sur les quatre langues.
- #873 : le manuel décrit la dévalidation avec le verrou et l'exercice clos (`user-manual.tex:719-723`) **sans** citer le message ; rien à corriger.
- **Notes de version** (pas de CHANGELOG dans la story) : trois lignes pour l'administrateur et l'intégrateur — (1) **message `PERIOD_LOCKED`** : dates désormais `JJ.MM.AAAA` dans le texte, `details` inchangés (ISO) : un intégrateur qui **parse le texte du message** verra le changement ; (2) libellés (bouton d'écriture en de/it/en, libellé d'audit) ; (3) catalogue it-CH harmonisé au « voi ».

## Preuve du rouge

Tests rédigés **avant** le correctif (rouge nommé dans le Dev Agent Record) : AC1 (valeurs de/it/en = `common-save` avant correctif), AC4 (page montée avec catalogue allemand : libellés fr avant), AC6 (`period_lock_e2e`, date littérale), AC8 (garde du tu sur `it-CH`), et la garde `sitesTotal`.

## Tâches

- [ ] **T0** Brancher, relire les prémisses (lignes ci-dessus à `287deea33` ; ne pas se fier aux numéros d'issue) ; relevé tu/voi de D5 (AC8) ; relevé du markup de `invoices/+page.svelte` (AC4).
- [ ] **T1 (AC1, AC2, AC9)** catalogues : `journal-entry-form-submit`, retrait de `homepage-bank-no-transactions`, libellé d'audit ; test AC1.
- [ ] **T2 (AC3)** coque : deux clés, deux layouts.
- [ ] **T3 (AC4)** liste des factures : clés neuves (quatre locales) et réutilisées ; test du rendu allemand.
- [ ] **T4 (AC5)** clé `-named` (quatre locales) et repli TS.
- [ ] **T5 (AC6, AC7)** `errors.rs` : mise en forme des dates, fallback, apostrophe ; `period_lock_e2e.rs` ; catalogue fr-CH.
- [ ] **T6 (AC8)** it-CH : conversion au voi ; test de la liste fermée.
- [ ] **T7 (AC10, AC11, AC12)** compteur, gardes, manuels ; recette visuelle ; E2E.
- [ ] **T8** Notes de version dans la fiche ; Dev Agent Record.

## Gates

(Fichier d'epic § Story 58-4 ; `docs/agents-orchestration.md` § 5.1 pour les verrous.)

- **Entre deux étapes** : gate ciblé — `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run -E 'binary(textes_coherents) | binary(period_lock_e2e)'` + `-p kesh-i18n` ; frontend : `npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit` sur les fichiers `i18n-*.test.ts` et la feature touchée.
- **Gate complet** avant tout push et au dernier commit de code : Rust (`scripts/test-fast.sh`, puis `gate-ci.sh` sur idefix) **et** frontend (`check`, `lint-i18n-ownership`, `test:unit`, `build`) ; le compteur `sitesTotal` recompté à ce moment. Exception `kesh-db` : non touchée (aucune migration).
- **E2E complet à la clôture** (libellés ; « vert » = les sept échecs KF-029 seuls) ; ports et bases : `3654`, `kesh_584`, `kesh_e2e_584`.
- **Recette visuelle** (`kesh-gate-logs/e65/58-4/captures/`, thème clair, base de démonstration, **jointe au Dev Agent Record et au prompt de revue**) : (a) l'écran d'écriture (bouton, en **de**, **it**, **en**, et fr pour contraste) ; (b) la liste des factures, fr **et** de ; (c) l'en-tête et le pied de page de la coque en fr et de, et le pied de page de l'onboarding ; (d) un refus `PERIOD_LOCKED` (message daté) ; (e) le journal d'audit sur une ligne « doublon confirmé » ; (f) le refus de dévalidation d'une facture fournisseur avec exercice postérieur clos.
- Le story file n'affirme que ce qui a tourné.

## Questions (pour l'orchestrateur)

- **Q1** #1086 sortie : à replanifier (story dédiée, piste D2 au rendu) ; la laisser ouverte.
- **Q2** Issues à ouvrir (une par problème, `gh issue list --state all --search` avant, ≥ 5 s entre deux créations) : (a) la suite de #339 par famille de fichiers (D1, relevé de `suite-339.md`) ; (b) *(annulé : les deux clés voisines entrent dans la story, D4)*.
- **Q3** Le choix du « voi » (D5), gardé en attendant la réponse du Project Lead, est une décision de ton : si le Project Lead préfère le tu, le sens de la conversion s'inverse et le coût passe de 64 à 124 messages.
- **Q4** Dix issues annoncées pour l'epic, **neuf** ici : le total de l'Epic 65 passe de 30 à 29 si #1086 n'est pas remplacée.

## Dev Notes

### Modules touchés (règle de splitting) — **trois**

`kesh-i18n` (catalogues, un test), `kesh-api` (`errors.rs` et un E2E : #908, #1049), frontend (`routes/(app)/+layout.svelte`, `onboarding/+layout.svelte`, `invoices/+page.svelte`, `invoices/new/+page.svelte`, `JournalEntryForm`-test, `supplier-invoice-unvalidate.ts`, `i18n-keys.test.ts`). Sous la borne de cinq ; pas de dérogation.

### Fichiers communs (kesh-gate-logs/e65/e65-commun.md)

Seule la 58-4 touche les catalogues i18n ; la 58-5 et la 58-6 n'y touchent pas. `sitesTotal` est un compteur commun : **se recompte après la fusion**. Aucun PDF, aucun CHANGELOG dans la story.

### Prémisses à confirmer à l'étape 0

- `ORDER BY start_date ASC` de `find_later_closed_in_tx` (`fiscal_years.rs:1140`, doc-comment) et que **c'est bien ce nom** que `fiscalYearName` porte dans `details` (un `grep` du chemin du refus de la dévalidation fournisseur, `LATER_FISCAL_YEAR_CLOSED` de `supplier_invoices`).
- Que `invoice-empty-list` a (ou non) un autre site que la liste (D1).
- Que le journal d'écriture est bien **immuable à la saisie** (c'est l'hypothèse de l'issue #324 ; le dev la relit au code avant de choisir « Buchen »).

### Ce qui doit rester vrai

Les E2E sont en français : aucun libellé fr n'est modifié, sauf le libellé d'audit (D8) — le dev greppe `frontend/tests` pour l'ancien texte avant de le changer (vide à la rédaction). Aucun repli de Svelte ou de Rust ne diffère du catalogue fr-CH.

### Charge estimée

Moyenne (M) : #339 et #874 portent le travail ; les sept autres sont S. Un jour de développement, E2E et recette compris.

### Références

- `_bmad-output/planning-artifacts/epic-65-nettoyage-suite.md` § Story 58-4 ; `kesh-gate-logs/e65/e65-commun.md`.
- `crates/kesh-api/tests/textes_coherents.rs` (G9) ; `frontend/src/lib/shared/i18n-keys.test.ts` ; `crates/kesh-i18n/src/loader.rs` (tests).
- `23-3-supplier-invoices.md` § AC7 (contrôle de collision à deux portées).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-28 — Fiche rédigée (Sonnet 5.5, `bmad-create-story`) ; statut `drafted`, validation à lancer par l'orchestrateur.
