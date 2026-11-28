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
coque de l'application et quelques messages disent ce que Kesh fait, dans ma langue, au format suisse, afin de ne plus lire, sur le bouton qui
comptabilise une écriture, le même mot que sur dix boutons d'enregistrement (« Speichern »), ni lire « Statut » en français à côté d'un filtre traduit.

## Contrôle de couverture (règle du 2026-11-26 : on vérifie qu'une issue n'est pas déjà couverte)

Chaque issue est relue au code **tel qu'il est à `287deea33`** (les numéros de ligne de plusieurs issues ont bougé).

| # | Constat au code | Verdict |
|---|---|---|
| #324 | `journal-entry-form-submit` : `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl:330` = Valider / Speichern / Salva / Save ; **un seul bouton** (`JournalEntryForm.svelte:1267`) pour la création **et** la modification (`isEdit`, `:141`) | **gardée, deux clés** (D6) |
| #339 | `routes/(app)/invoices/+page.svelte:288` (titre), `:297`, `:307`, `:314-317`, `:337`, `:344`, `:348`, `:362`, `:404-413` : français en dur, à côté de `invoice-paused-filter-*` traduits (`:322-334`) | **gardée, périmètre scindé** (voir D1) |
| #844 | `homepage-bank-no-transactions` : `messages.ftl:181` ×4 locales ; `grep -rn "homepage-bank-no-transactions" frontend/src crates --include=*.ts --include=*.svelte --include=*.rs` vide ; aucune construction dynamique (`grep "homepage-bank-"` ne rend que des `data-testid`) | **gardée** |
| #861 | placeholder : `routes/(app)/+layout.svelte:345` ; pied de page : `routes/(app)/+layout.svelte:496` et `routes/onboarding/+layout.svelte:38` (les lignes de l'issue ont bougé) | **gardée** |
| #873 | quatre clés nomment l'exercice postérieur clos **le plus proche** (SQL `FIND_LATER_CLOSED_SQL`, `fiscal_years.rs:1147-1150`, `ORDER BY start_date ASC LIMIT 1`) puis disent « en commençant par le plus récent » : `supplier-invoices-unvalidate-blocked-later-fiscal-year-closed-named` (fr `messages.ftl:3384`), `journal-entries-modify-blocked-later-fiscal-year-closed` (`:481`), `error-later-fiscal-year-closed` (`:482`), `error-fiscal-year-create-later-closed` (`:1236`) | **gardée, quatre clés** (D4) |
| #874 | it-CH mêle tu, voi et Lei (relevé rejouable : `kesh-gate-logs/e65/58-4/releve-it-adresse.py`, voir D5) : **68 refus à convertir** (45 au tu, 23 au Lei), sous le seuil de 80 | **gardée** (décision du Project Lead du 2026-11-28 : le « voi » partout dans les refus) |
| #908 | `crates/kesh-api/src/errors.rs:6479-6495` : `locked_through.to_string()` / `attempted.to_string()` dans les arguments de `journal-entries-period-locked` (ISO) ; fallback `:6484` idem ; test pinant l'ISO : `crates/kesh-api/tests/period_lock_e2e.rs:398-404` | **gardée** |
| #1049 | `messages.ftl:1367` (fr-CH, `error-opening-complement-retained-earnings-line`) et `errors.rs:2581` : « d'ouverture » à apostrophe droite (la ligne de l'issue, 1165, a bougé) | **gardée** |
| #1086 | 159 lignes fr-CH portent des guillemets « » à espace ordinaire, aucune insécable (`grep -cP '«\x{00A0}' = 0`) ; **ces mêmes chaînes sont répétées comme replis** : 2841 lignes de `frontend/src` portant un « » (505 fichiers ; toutes les citations, non les seuls replis des 159 clés : le coût est surestimé dans sa forme), 3004 lignes dans 424 fichiers `.rs`, 489 lignes d'E2E, et la garde G9 (`textes_coherents.rs`) comme `i18n-un-repli-par-cle` exigent l'égalité repli = catalogue | **sortie de la story** (voir D2) |
| #1491 | `messages.ftl:3808` (fr), `:3729` (de), `:3730` (it), `:3735` (en) : « Doublon de facture fournisseur confirmé » ; aucun autre site (ni code, ni test, ni manuel : `grep` vide hors catalogues) | **gardée** |

Aucune de ces issues n'est déjà corrigée. `git log --grep` n'a pas été relu issue par issue : aucune n'a été touchée par les epics 55, 58 (58-1 à 58-3) ni 64
au vu des lignes ci-dessus (tous les textes sont tels que l'issue les décrit). **Dix issues annoncées, neuf retenues** : #1086 sort.

## Ce qui se lit au code

### #324 — le bouton d'une écriture

`JournalEntryForm.svelte:86-89` : `initialEntry` fourni = **mode édition** (Story 15-8a, #532), `:141` `const isEdit = $derived(initialEntry !== null)` ; `:1266-1268` : **un seul** bouton
`data-testid="journal-entry-submit"`, `i18nMsg('journal-entry-form-submit', 'Valider')`, quel que soit `isEdit`. **La prémisse « écriture irréversible » de l'issue est donc périmée** (une écriture se modifie et se supprime tant que
son exercice est ouvert ; `user-manual.tex:708-713` : « le bouton Modifier ouvre le formulaire de saisie, pré-rempli : corrigez, puis *Valider* »). Le défaut réel reste : un bouton qui **comptabilise** une écriture reçoit en de/it/en le mot
des boutons d'enregistrement ordinaires. Mais « Buchen » sur le bouton d'une **modification** serait faux là où « Speichern » était juste : d'où **deux clés selon le mode** (D6). La clé n'a aucun autre site (Rust, TS, Svelte). Les E2E
sélectionnent par `name: "Valider"` ou `data-testid` : le français ne change pas dans les deux modes, aucun E2E ne bouge ; le manuel (`:713`, `:2374` fournisseur) reste vrai.

Collisions contrôlées au catalogue (`grep -nE "= (…)$"` par locale) : `Buchen`, `Contabilizza`, `Änderungen speichern`, `Salva modifiche`, `Save changes` **n'existent nulle part** comme valeur ; `Post entry` (en) est déjà la valeur de `reconciliation-manual-submit`
(`en-CH:1664`), même sens, autre écran : pas de collision au sens de la 23-3. `Registra` (it) est écarté : déjà `supplier-invoices-new` (`it-CH:3256`) et neuf boutons « Enregistrer ». Le français « Valider » est aujourd'hui le seul `Valider` de ce sens ; la clé du mode modification
porte aussi « Valider » en fr (D6) : deux clés, **même valeur fr**, volontairement (le repli Svelte est le même).

### #339 — français en dur dans le balisage de la liste des factures

Le **catalogue possède déjà presque toutes les clés** de cette page, que la page n'appelle pas : `invoices-page-title` (628), `invoice-new-button` (688),
`invoice-filter-search` (677, « Rechercher… »), `invoice-filter-status-all` (678), `invoice-filter-contact-all` (679), `invoice-filter-date-from` (680),
`invoice-filter-date-to` (681), `invoice-col-date|contact|number|status|total|actions` (668-673), `invoice-status-*` (674-676), `common-loading|previous|next`
(1071-1073), `invoice-delete-button` (690). `invoice-empty-list` (693) dit **plus** que le texte actuel (« Aucune facture. » seul), et **n'a aucun site** (`grep -rn "invoice-empty-list" crates frontend/src frontend/tests --include=*.rs --include=*.ts --include=*.svelte` vide) : choix D1. Plusieurs clés réutilisables (`invoices-page-title`, `invoice-col-*`, `invoice-filter-*`) n'ont, de même, aucun site aujourd'hui : le dev le relève avant de les appeler.

Textes en dur de la page à couvrir (le dev **relit le fichier en entier**, l'heuristique ci-dessous n'est qu'un plancher) : `<svelte:head><title>` (`Factures — Kesh`),
`h1`, bouton « Nouvelle facture », labels « Recherche », « Statut », « Contact », « Depuis », « Jusqu'à », options « Tous les statuts », « Brouillon »,
« Validée », « Annulée », placeholder « Rechercher… », placeholder « Tous les contacts » (`:344`), bouton « Réinitialiser », « Chargement… », « Aucune facture. »,
`<th>` Date / Contact / N° / Statut / Total / Actions, les trois `aria-label` `Voir|Modifier|Supprimer la facture {n}` (`:435,439,447`), le pluriel `{total} facture{s}`
(`:458` ; **deux clés choisies en TS**, D1), « Précédent », « Suivant », le message `dateRangeError` (`:158`) et `deleteError = 'Erreur lors de la suppression'` (`:253`).

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

Le message nomme le **premier** exercice postérieur clos (le plus proche : `FIND_LATER_CLOSED_SQL`, `fiscal_years.rs:1147-1150`, `ORDER BY start_date ASC LIMIT 1`, appelé par `find_later_closed_in_tx` à la dévalidation fournisseur, à la création d'une écriture et à la création d'un exercice, `fiscal_years.rs:402-407`) et prescrit de rouvrir « en commençant par le plus récent » :
deux exercices clos (2027 et 2028) font nommer 2027 et rouvrir 2028 d'abord. **Inventaire clos des clés à `{ $name }` + « en commençant par le plus récent »**, lu au catalogue fr-CH : (1) `supplier-invoices-unvalidate-blocked-later-fiscal-year-closed-named` (`:3384`), (2) `journal-entries-modify-blocked-later-fiscal-year-closed` (`:481`), (3) `error-later-fiscal-year-closed` (`:482`), (4) `error-fiscal-year-create-later-closed` (`:1236`) : **à traiter**.
**Sans défaut** (lus, écartés) : `fiscal-year-out-of-order-warning` (`:1090`, nomme `$closed` **et** `$latest`), `reconciliation-failed-later-fiscal-year-closed` (`:1688`, sans consigne), `error-fiscal-year-reopen-blocked` (`:1231`, sans nom). Repli de chacune : (1) `supplier-invoice-unvalidate.ts:142-146`, (2) `blocker-messages.ts:95`, (3) `errors.rs:5695`, (4) `errors.rs:2919`.

### #908 — dates ISO

Une seule clé de message serveur : `journal-entries-period-locked` (4 locales, `:486`/`:492`). `errors.rs:6479-6495` : args `lockedThrough` et `attempted` = `NaiveDate::to_string()` (AAAA-MM-JJ) ; `details` en ISO **pour le client** (contrat inchangé, #869). **La fonction existe déjà** :
`kesh_i18n::format_date(&NaiveDate)` (`crates/kesh-i18n/src/formatting.rs:38`, `%d.%m.%Y`), déjà importée par `routes/invoices.rs:37` et `invoice_email.rs:31` : **aucune fonction neuve** (D7). Le fallback Rust (`:6484`) interpole les mêmes dates et suit. La clé de l'écran
(`settings-books-lock-current`) est déjà au format suisse côté TS (`swissDate`) : hors sujet. Le batch (`period_locked_failed_proposal`, `reconciliation.rs:609`) ne porte pas de message : rien à faire. **Tests qui épinglent l'ISO** : `crates/kesh-api/tests/period_lock_e2e.rs:360-364` (`message.contains(&borne.to_string())`) et `:398-404` (message complet) ;
les deux calculent la borne depuis l'horloge (`aujourd_hui_test() - Duration::days(10)`, `:344`, `:380`) : une date attendue « en littéral » y est impossible sans changer le montage (AC6).

### #1049 — apostrophe

`messages.ftl:1367` fr-CH : « d'ouverture » → « d’ouverture » ; même changement au repli `errors.rs:2581` (la garde G9 exige l'égalité). Il n'y a **que** ce site pour
cette clé (les trois autres locales n'ont pas d'apostrophe droite ici : `it-CH:1298` a déjà `’`).

### #1491 — libellé d'audit

Quatre valeurs `audit-log-action-supplier-invoice-duplicate-confirmed`. La trace naît de « Ce n'est pas un doublon, enregistrer » (saisie) et du repassage d'une annulée
(manuel : `user-manual.tex:2238`, `admin-manual.tex` « duplicate\_confirmed » = **nom de code**, non le libellé : à laisser).

## Inventaire par le symptôme (propagation post-patch)

- `Valider|Speichern|Salva|Save` : seule `journal-entry-form-submit` (et sa jumelle de modification, D6) est en cause ; les autres valeurs « Speichern »/« Salva »/« Save » sont des boutons d'enregistrement ordinaires.
- Texte du pied de page : `grep -rn "ne remplacent pas un fiduciaire" frontend crates docs website` (2 sites Svelte aujourd'hui ; le dev greppe aussi `website/` et les manuels).
- `Rechercher\.\.\.` : un site (`+layout.svelte:345`) ; `grep -rn "Rechercher\.\.\." frontend/tests` vide.
- Dates ISO dans les messages serveur : `grep -n "to_string()" crates/kesh-api/src/errors.rs` autour des `args.set` pour les autres clés à date — **hors périmètre** (une issue par problème) ; le dev ne les corrige pas, il signale. Site non résolu connu : `DbError::PeriodLocked` `Display` (`kesh-db/src/errors.rs:2274-2276`, D7).

## Décisions (choix autonomes, `C-58-4-k`)

- **D1 (`C-58-4-a`) — #339 : la liste des factures et `invoices/new`.** L'issue est une **classe** (61 sites dans 12 fichiers, plancher) ; la règle « pas d'issue résolue par morceaux sur plusieurs releases : scinder avant » a été appliquée par l'orchestrateur le 2026-11-28 (commentaire de #339).
  **#339 réduite** (choix `C-58-4-k`) : la story couvre **`routes/(app)/invoices/+page.svelte` en entier** (balisage, attributs, `<title>`, messages d'erreur de la page) **et `routes/(app)/invoices/new/+page.svelte` en entier** : `<title>Nouvelle facture — Kesh</title>` (`:6`) **et** `h1` (`:9`), repli de `invoice-new-button` / `invoice-new-title` = « Nouvelle facture » (identique).
  La suite est **déjà ouverte** : #1775 (écrans de facture : `[id]`, `[id]/edit`, `InvoiceForm`, `due-dates`), #1776 (`BankProfileForm`), #1777 (Utilisateurs), #1778 (fiche d'écriture), #1779 (contacts, sites isolés) ; `suite-339.md` en donne le relevé. **Angle mort assumé** : les fichiers que #339 citait à l'origine et que l'heuristique ne voit plus
  (`BankImportUpload.svelte`, `bank-import/[id]/+page.svelte`, `BankImportList.svelte`), et la classe « attributs, multi-lignes, texte hors balise » hors des fichiers découpés : aucune issue ne les porte ; l'orchestrateur décide d'en ouvrir une (le dev ne les traite pas).
  **Garde** : pas de garde générale (même calcul que la 23-3b : la proscrire) ; l'AC4 pose un test à **liste fermée** sur la page.
  **« Aucune facture. »** : la clé `invoice-empty-list` (`:693`, texte plus long, **sans aucun site**) **change de valeur** (« Aucune facture. », quatre locales) et la page l'appelle : pas de clé neuve, pas de clé morte de plus. Le texte qui l'accompagnait (« Créez votre première facture… ») n'est
  pas repris (l'écran ne le disait pas). Toute autre clé réutilisée dont le repli fr diffère du texte actuel de l'écran se traite de même (valeur du catalogue **alignée sur l'écran**, jamais l'inverse) : l'écran ne change pas en français.
  **Pluriel `{total} facture{s}`** : le frontend reçoit les messages pré-résolus **sans arguments** (`loader.rs:158`, `format_pattern(pattern, None, …)`) et `i18nMsg` ne fait qu'un remplacement de `{ $x }` (`i18n.svelte.ts:20-27`) : un sélecteur Fluent s'afficherait toujours sous sa variante par défaut. **Deux clés**
  (`invoice-list-count-one`, `invoice-list-count-other`, avec `{ $count }`) choisies en TS selon `total === 1` (patron existant : `contact-duplicate-others-count-one`, `fr-CH:522`).
- **D2 (`C-58-4-b`) — #1086 sort de la story.** Le remède demandé (remplacer l'espace après « et avant » dans 159 lignes fr-CH) **ne peut pas se limiter au catalogue** : la garde G9 et
  `i18n-un-repli-par-cle` exigent le même texte au repli (≈ 5800 lignes à lire, 489 lignes d'E2E qui sélectionnent par texte), et un test qui rejoue du texte fr contre le catalogue rougit sur tout désaccord. Ce n'est
  pas « peu coûteux » (coût réel L, centaines de sites répartis sur plusieurs modules : la règle de splitting préventif s'applique). Piste à chiffrer dans une story à part : **un traitement unique au rendu**
  (`kesh-i18n::I18nBundle::format`/`all_messages` côté serveur ; `i18nMsg` côté frontend) qui pose l'espace insécable après « et avant », sans toucher ni catalogue ni replis, avec une garde sur la **sortie**
  rendue. L'orchestrateur la garde ouverte (#1086 reste ouverte), à replanifier ; *aucune écriture* n'est faite ici.
- **D3 (`C-58-4-c`) — clés de la coque.** Deux clés neuves hors des namespaces de feature : `layout-search-placeholder` (fr-CH : `Rechercher...`, **trois points ASCII, valeur actuelle** ; les trois autres langues avec
  leurs points de suspension habituels) et `layout-footer-disclaimer` (fr-CH : `Kesh v{ $version } — Logiciel libre (EUPL 1.2). Les données ne remplacent pas un fiduciaire.`, le tiret cadratin en
  caractère, plus `&mdash;`). Un seul texte pour les **deux** layouts (app et onboarding), donc deux sites `i18nMsg` du même repli (garde `i18n-un-repli-par-cle` : un repli par clé, **identique** aux deux sites).
  La variable `$version` passe par `{ version: appVersion.value }` (garde `i18n-entrees-a-variables.test.ts` : le dev l'applique comme pour les autres clés à variable). Traductions : de « Kesh v{ $version } — Freie
  Software (EUPL 1.2). Die Daten ersetzen keine Treuhänderin oder keinen Treuhänder. » (le dev relit la forme sur les catalogues voisins : `Treuhand`/`Fiduciaria`/`fiduciary` déjà employés ? `grep`), it « … I dati non sostituiscono
  una fiduciaria. », en « … The data does not replace a fiduciary. » : **le dev reprend le terme du catalogue** (`grep -in "fiduciar\|treuh" messages.ftl`) et le consigne.
- **D4 (`C-58-4-d`) — #873 : quatre clés, le nom dit « le plus proche ».** fr (clé 1) : « L’exercice postérieur « { $name } » (le plus proche) est clôturé : pour pouvoir repasser cette facture en brouillon, un administrateur doit rouvrir les exercices postérieurs clôturés, en commençant par le plus récent. » ;
  les clés 2, 3 et 4 reçoivent la **même insertion** dans leur phrase (« le plus proche » / « nächstgelegene » / « il più vicino » / « the nearest » après le nom), la consigne finale **inchangée**. **Marqueurs à préserver** (garde `kesh-i18n`, `loader.rs`) : `CLES_POSTERIEURS` (`:1361-1373`) exige par locale
  « les exercices postérieurs clôturés » / « gli esercizi successivi chiusi »… et `loader.rs:1243-1255` exige « en commençant par le plus récent » / « cominciando dal più recente » (et équivalents) : le nouveau texte **les garde** ; le dev relit ces deux gardes avant d'écrire. Quatre locales par clé, repli égal au fr-CH.
  **Tests qui rougissent** (relevé `grep -rnE "postérieur, est clôturé|exercice postérieur « " frontend/src frontend/tests crates`) : `frontend/src/routes/(app)/supplier-invoices/[id]/supplier-invoice-draft-page.test.ts:680-682` (épingle en littéral la valeur de la clé 1) ; `crates/kesh-api/tests/fiscal_years_e2e.rs:2340` (`msg.contains("« Exercice 2026 », postérieur, est clôturé")`,
  porte sur la clé 4 : la sous-chaîne subsiste si l'insertion vient **après** `postérieur,` — sinon à mettre à jour). **Ne rougissent pas** (messages du serveur **simulés**, injectés puis affichés tels quels) : `invoice-validate-page.test.ts:164`, `fiscal-years-page.test.ts:334`. `textes_coherents.rs:825` (G9 + variable `NOM` pour la clé 3) vérifie le repli.
  **Prémisse lue au code, par clé** : les quatre noms viennent de `find_later_closed_in_tx` (tri `ASC`, `LIMIT 1`) ; le dev cite la ligne lue pour chacune (AC5).
- **`C-58-4-k` — #339 réduite** (voir D1) : la liste des factures et `invoices/new` restent à #339 ; la suite est portée par #1775 (écrans de facture), #1776 (`BankProfileForm`), #1777 (Utilisateurs), #1778 (fiche d'écriture), #1779 (contacts). Relevé : `kesh-gate-logs/e65/58-4/suite-339.md` (14 sites restent à #339, 13 + 1 ; 45 retenus passés aux cinq issues).
- **D5 (`C-58-4-e`) — #874 : le « voi » dans les refus** (décision du Project Lead du 2026-11-28). **Méthode rejouable** : `python3 /home/gcorbaz/devel/kesh-gate-logs/e65/58-4/releve-it-adresse.py crates/kesh-i18n/locales/it-CH/messages.ftl [liste]` : messages `cle = valeur` de plus de 40 caractères, formes cherchées en
  **début de phrase** (début de valeur, ou après `: `, `; `, `. `, `, `, `? `, `« `), par **liste fermée de 60 lemmes** (impératif tu, voi, subjonctif de politesse « Lei », clitiques facultatifs) ; « refus » = clés `error-*`, `*-blocked*`, `*-failed*`, `*-refus*`, `*-cannot*`, `*-conflict*`, `*-unavailable*`, `*-invalid*`, `*-denied*`.
  **Résultat à `287deea33`** (4283 messages lus) : tous messages — tu **103**, voi **199**, Lei **45** ; **refus — tu 45, voi 87, Lei 23** : **68 refus à convertir** (45 + 23, sans recouvrement), **sous le seuil de 80** : #874 **reste dans la story**. (Le décompte « 124 / 64 » d'une première estimation, sans les Lei, est abandonné.)
  Périmètre : les **refus**, conformément à l'issue (« dans les refus ») et à la décision ; les **76 autres messages** au tu ou au Lei hors refus (144 − 68) restent **angle mort assumé**, listés dans le relevé du dev ; l'orchestrateur décide d'une issue pour eux. Les étiquettes de bouton (« Salva », « Annulla », « Registra ») ne sont pas des refus et ne sont pas touchées.
  **Conversion vers le voi** : impératif pluriel (« annullate », « correggetela », « riprovate »), pronoms `vi`/`vostro`. La première tâche du dev est de **rejouer le script**, de joindre sa sortie et la liste des 68 clés au Dev Agent Record, et de marquer chaque ligne ambiguë ; **le script est une heuristique à liste fermée** : tout refus qui s'adresse au lecteur par une forme absente de la liste est un site non résolu, à relire à la main
  (les 87 + 68 refus relevés ne sont pas « tous les refus »). Les replis TS/Rust sont en français : **aucun repli ne change**.
- **D6 (`C-58-4-f`) — #324 : deux clés selon le mode (arbitrage de l'orchestrateur du 2026-11-28).** `journal-entry-form-submit` (**création**) = fr « Valider » (inchangé) / de **Buchen** / it **Contabilizza** / en **Post entry** ; clé neuve `journal-entry-form-submit-edit` (**modification**, `isEdit`) = fr « Valider » (inchangé : le manuel `:713` et les E2E le citent dans les deux modes) /
  de **Änderungen speichern** / it **Salva modifiche** / en **Save changes**. Le Svelte choisit la clé par `isEdit` (un seul bouton, `data-testid` inchangé). Le repli est « Valider » dans les deux cas. **Motif** : « Buchen » sur une modification serait faux ; l'irréversibilité de l'issue est périmée (F1 de la validation P1), reste l'homonymie avec « Speichern ».
- **D7 (`C-58-4-g`) — #908 : `kesh_i18n::format_date`, aucune fonction neuve.** Dans `errors.rs`, les deux arguments **et** le repli passent par `kesh_i18n::format_date(&locked_through)` / `(&attempted)` ; **`details` reste en ISO**. Exemple attendu : « verrouillées jusqu'au 31.03.2026 ; celle-ci est datée du 15.01.2026. ».
  **L'apostrophe droite de « jusqu'au » n'est pas touchée** (un symptôme, une issue). Le site inline voisin `errors.rs:3412` (`date.format("%d.%m.%Y")`, `RATE_UNAVAILABLE`) est **hors périmètre** : produit déjà du format suisse, non aligné sur la fonction (à signaler, non traité). `DbError::PeriodLocked` a aussi un `Display` à dates ISO
  (`crates/kesh-db/src/errors.rs:2274-2276`, `#[error("Les écritures sont verrouillées jusqu'au {locked_through} …")]`) : lu pour les journaux, **angle mort assumé** (jamais renvoyé au client : le mappage HTTP passe par `journal-entries-period-locked`).
- **D8 (`C-58-4-h`) — #1491 : libellé neutre, pas « probable ».** fr « Facture fournisseur enregistrée malgré un doublon signalé » ; de « Lieferantenrechnung trotz gemeldetem Duplikat erfasst » ; it « Fattura fornitore
  registrata nonostante un doppione segnalato » ; en « Supplier invoice recorded despite a flagged duplicate ». *Motif* : la trace couvre aussi le doublon **certain** (même numéro) confirmé au repassage d'une annulée, que
  « probable » (suggestion de l'issue) fausserait.

## Critères d'acceptation

1. **AC1 (#324)** — création : `journal-entry-form-submit` = « Valider » / « Buchen » / « Contabilizza » / « Post entry » ; modification : `journal-entry-form-submit-edit` = « Valider » / « Änderungen speichern » / « Salva modifiche » / « Save changes » (fr-CH inchangé dans les deux). Test Vitest (fichier de la feature journal-entries) : (a) les valeurs des deux clés lues dans
   les catalogues ; (b) **la création n'égale `common-save` dans aucune des trois cibles** ; (c) **la clé du mode création et celle du mode modification diffèrent dans de/it/en** (rougit si les deux modes affichent la même clé ou la même valeur) ; (d) le composant monté en mode création puis en mode édition (`initialEntry` fourni) demande la **bonne clé** (mutation nommée :
   le bouton revient à une clé unique). Aucun E2E modifié ; la recette visuelle montre **les deux modes**.
2. **AC2 (#844)** — `homepage-bank-no-transactions` n'existe plus dans aucun des quatre catalogues ; aucune garde n'est cassée. `homepage-` **n'est pas** un préfixe à couverture close (`PREFIXES_A_COUVERTURE_CLOSE = ['contact-duplicate-', 'settings-invoicing-']`, `i18n-keys.test.ts:1540`) : aucun test ne rougirait si la clé revenait — **écart assumé** (texte pur, vérifié par `grep`, AC-grep consigné).
3. **AC3 (#861)** — le placeholder de recherche et le pied de page de `(app)/+layout.svelte` et de `onboarding/+layout.svelte` passent par `i18nMsg` avec les clés de D3, présentes dans les quatre catalogues ; en français, **rendu identique caractère pour caractère** ; avec `KESH_LANG=de` les deux textes sont en allemand (capture).
   **Test qui rougit** : un test Vitest **par layout** monte la coque avec un `i18nMsg` simulé renvoyant une marque allemande (`⟦clé⟧` ou une valeur de test) et asserte le placeholder et le pied de page (avec `{ version }` substitué) ; mutation nommée : retour au littéral français dans l'un des deux layouts.
4. **AC4 (#339, liste des factures et `invoices/new`)** — `routes/(app)/invoices/+page.svelte` **et** `routes/(app)/invoices/new/+page.svelte` (`<title>` et `h1`) ne contiennent plus de texte français hors `i18nMsg` (balisage, `placeholder`, `aria-label`, `<title>`, messages). **Inventaire clos** : le dev produit, par fichier, la liste des sites **résolus** et celle de ce qui **reste** (angle mort assumé, ex. une valeur d'API affichée telle quelle),
   chacune lue au code. Chaque repli = valeur fr-CH de sa clé (gardes `i18n-un-repli-par-cle`, `i18n-repli-divergent-actif`). Les clés neuves existent dans les **quatre** catalogues, sans collision de libellé (méthode de la 23-3, § AC7 de `23-3-supplier-invoices.md`).
   **Test qui rougit** : un test Vitest monte la page avec `i18nMsg` simulé par une **marque** (`⟦clé⟧`) et asserte que le HTML rendu **y compris les attributs** (`container.innerHTML`) ne contient **aucune chaîne d'une liste fermée** tirée du relevé T0 (« Statut », « Recherche », « Depuis », « Réinitialiser », « Chargement… », les trois options, les en-têtes, les `aria-label`…) ; cas supplémentaires : `dateRangeError` (dates inversées), liste vide,
   **pluriel `total = 1` et `total = 2`** (deux clés différentes). Mutation nommée : un seul site rendu à son littéral. **L'écran en français ne change pas** (E2E de la liste inchangés).
5. **AC5 (#873)** — les **quatre** clés de D4 disent « le plus proche » dans les quatre langues, chaque repli (TS ou Rust) égale le fr-CH ; les marqueurs de `loader.rs` (`CLES_POSTERIEURS`, `:1243-1255`) sont préservés ; le contrôle `find_later_closed_in_tx` / `FIND_LATER_CLOSED_SQL` (`ORDER BY start_date ASC LIMIT 1`) est lu et cité au Dev Agent Record **pour chacune des quatre clés** ; les tests qui épinglent le texte (D4) sont mis à jour et verts, `supplier-invoice-unvalidate.test.ts:152` reste vert sans modification.
6. **AC6 (#908)** — le message `journal-entries-period-locked` rend les deux dates au format `JJ.MM.AAAA` en fr-CH, de-CH, it-CH, en-CH, par `kesh_i18n::format_date` ; **`details.lockedThrough` et `details.attempted` restent en ISO** ; le repli Rust égale le catalogue (G9). **Tests** : `period_lock_e2e.rs:360-364` et `:398-404` sont mis à jour, et assertent la forme suisse par une **construction indépendante du code**
   (`format!("{:02}.{:02}.{}", d.day(), d.month(), d.year())`, pas `format_date`) ; **la preuve en littéral** (`31.03.2026` / `15.01.2026`) passe par un test qui **fixe ses dates** : test unitaire de la réponse de `DbError::PeriodLocked` dans `errors.rs` avec des `NaiveDate` littérales (les E2E dérivent leur borne de l'horloge, `:344`, `:380`). Mutation nommée : retour à `to_string()` rougit les trois.
7. **AC7 (#1049)** — `error-opening-complement-retained-earnings-line` (fr-CH) et son repli `errors.rs:2581` portent « d’ouverture » (U+2019) ; G9 vert. `grep -n "d'ouverture" crates/kesh-i18n/locales/fr-CH/messages.ftl crates/kesh-api/src/errors.rs` ne rend plus ce site (les autres occurrences éventuelles sont des symptômes **différents** : listées, non traitées).
8. **AC8 (#874)** — les **68 refus** relevés (D5) sont au voi ; aucun refus du catalogue it-CH ne porte une forme de tu ou de Lei **de la liste fermée du relevé**. Le relevé (script rejoué, sortie, liste des 68 clés, lignes ambiguës laissées et justifiées, **les 76 messages hors refus laissés en angle mort**) est joint au Dev Agent Record. Un test Rust de `kesh-i18n` (module `tests` de `loader.rs`, à côté de `parity_between_locales`) lit it-CH
   et échoue si un **refus** (même motif de clé que le script) porte une forme de tu ou de Lei **de la liste fermée** : l'écart est assumé (le test ne prouve pas l'absence de toute forme absente de la liste). Les clés citées par l'issue (`supplier-invoices-cancel-*`, `error-*-must-be-unvalidated-first`, `invoices-settlement-cancel-blocked-written-off`, `reconciliation-cancel-blocked-written-off`, `bank-import-manual-delete-blocked`) sont **déjà au voi** : elles ne bougent pas.
   `loader.rs:1454` (`("it-CH", "Chiudi l’esercizio")`, étiquette de bouton, hors refus) n'est pas touchée.
9. **AC9 (#1491)** — les quatre valeurs de `audit-log-action-supplier-invoice-duplicate-confirmed` suivent D8 ; le test d'AC1 (ou un voisin) **assert les quatre valeurs** (écart assumé pour le reste : texte pur). Le journal d'audit les affiche (capture).
10. **AC10 (compteur)** — `sitesTotal` de `frontend/src/lib/shared/i18n-keys.test.ts` (ligne 1300, **3985** à `287deea33`) se **recompte depuis la garde** (« expected N to be 3985 »), et le commentaire de la ligne s'enrichit d'une entrée datée avec le détail (sites de la coque +3 ; `JournalEntryForm` : le site unique devient une expression à deux clés, à relever selon la façon dont `findCallSites` la lit ; liste des factures et `invoices/new` +≈ 20 à relever ; 0 pour #873/#874/#908/#1049/#1491/#844). `sitesNonResolus` (31), `relais` (6), `sitesGabarit` (12) **ne bougent pas** : le dev le contrôle ; un écart se **recompte**, il ne s'ajuste pas.
11. **AC11 (parité et gardes)** — `parity_between_locales` (`kesh-i18n`), `loader.rs:1373` (liste de clés à variables `error-later-…`), `textes_coherents.rs` (G9 et ses voisines), `lint-i18n-ownership`, `i18n-entrees-a-variables`, `i18n-un-repli-par-cle`, `i18n-repli-divergent-actif` verts.
12. **AC12 (manuel)** — voir *Manuel* ci-dessous : aucun passage du manuel ne cite un texte changé ; ce contrôle (texte **et** PDF aplati) est rejoué par le dev avant la clôture et consigné.

## Manuel (`docs/manual/`, texte et PDF aplati)

Relevé à la rédaction : `grep -rn` des chaînes changées dans `docs/manual/*/*.tex` et dans `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '` (fr user + admin) :

- « Valider » (#324) : `user-manual.tex:713` et `:2374` (autre bouton, la facture fournisseur) : **inchangés** (D6), toujours vrais ; `docs/manual/{de,it,en}/` ne contiennent que `README.md` : **aucun `.tex`, sans objet** (un grep y serait vide par construction). Contrôle rejoué à la rédaction sur les `.tex` **et** les PDF aplatis fr (user, admin, brochure) : « Rechercher... », « ne remplacent pas », « Doublon de facture », « verrouillées jusqu », « Tous les statuts », « bénéfice reporté » : 0 occurrence ; aucune capture du manuel ne montre un texte fr modifié.
- « Rechercher... », « ne remplacent pas un fiduciaire », « Doublon de facture fournisseur confirmé », « verrouillées jusqu'au » : **aucune occurrence** dans les `.tex` ni dans le PDF aplati fr (user/admin) ; `d'ouverture` : sans objet (message d'erreur) ; le dev rejoue le grep (texte et PDF aplati) avant la clôture.
- #873 : le manuel décrit la dévalidation avec le verrou et l'exercice clos (`user-manual.tex:719-723`) **sans** citer le message ; rien à corriger.
- **Notes de version** (pas de CHANGELOG dans la story) : trois lignes pour l'administrateur et l'intégrateur — (1) **message `PERIOD_LOCKED`** : dates désormais `JJ.MM.AAAA` dans le texte, `details` inchangés (ISO) : un intégrateur qui **parse le texte du message** verra le changement ; (2) libellés (bouton d'écriture en de/it/en, libellé d'audit) ; (3) catalogue it-CH : refus harmonisés au « voi » ; (4) trois autres refus d'exercice clos disent désormais « le plus proche ».

## Preuve du rouge

Tests rédigés **avant** le correctif (rouge nommé dans le Dev Agent Record) : AC1 (valeurs de/it/en = `common-save` avant correctif), AC4 (page montée avec catalogue allemand : libellés fr avant), AC6 (`period_lock_e2e`, date littérale), AC8 (garde du tu sur `it-CH`), et la garde `sitesTotal`.

## Tâches

- [ ] **T0** Brancher, relire les prémisses (lignes ci-dessus à `287deea33` ; ne pas se fier aux numéros d'issue) ; rejeu du script de D5 (AC8) ; relevé du markup de `invoices/+page.svelte` (AC4).
- [ ] **T1 (AC1, AC2, AC9)** catalogues : `journal-entry-form-submit` et `journal-entry-form-submit-edit` (D6), retrait de `homepage-bank-no-transactions`, libellé d'audit ; test AC1.
- [ ] **T2 (AC3)** coque : deux clés, deux layouts.
- [ ] **T3 (AC4)** liste des factures : `invoices/+page.svelte` et `invoices/new/+page.svelte` ; clés neuves (quatre locales), valeur de `invoice-empty-list`, deux clés de pluriel ; test à marques.
- [ ] **T4 (AC5)** quatre clés de D4 (quatre locales), replis TS et Rust, tests épinglés.
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
- **Q2** Plus d'issue à ouvrir pour la suite de #339 : #1775 à #1779 existent (D1). Restent à décider par l'orchestrateur : une issue pour les fichiers hors heuristique de #339 (`BankImportUpload`, `bank-import/[id]`, `BankImportList`, D1) et une pour les **76 messages it-CH hors refus** au tu ou au Lei (D5) ; `gh issue list --state all --search` avant, ≥ 5 s entre deux créations.
- **Q3** Le « voi » est **décidé** (Project Lead, 2026-11-28) ; le seuil de 80 n'est pas atteint (68 refus), #874 reste.
- **Q4** Dix issues annoncées pour l'epic, **neuf** ici : le total de l'Epic 65 passe de 30 à 29 si #1086 n'est pas remplacée.

## Dev Notes

### Modules touchés (règle de splitting) — **trois**

`kesh-i18n` (catalogues, un test), `kesh-api` (`errors.rs` et deux E2E : #873, #908, #1049), frontend (`routes/(app)/+layout.svelte`, `onboarding/+layout.svelte`, `invoices/+page.svelte`, `invoices/new/+page.svelte`, `JournalEntryForm.svelte`, `blocker-messages.ts`, `supplier-invoice-unvalidate.ts`, `i18n-keys.test.ts`). Sous la borne de cinq ; pas de dérogation.

### Fichiers communs (kesh-gate-logs/e65/e65-commun.md)

La 58-4 est la seule à toucher les catalogues, **sauf une ligne de la 58-5** : `fr-CH/messages.ftl:1718` (`reconciliation-cols-candidate = Proposition`, #1153, bloc réconciliation) ; conflit de fusion annoncé par la 58-5, **résolu en gardant les deux côtés** ; le contrôle de collision de libellé (AC1, AC4) se **rejoue après la fusion** si « Proposition » entre en fr. La 58-6 ne touche aucun fichier de la 58-4. `sitesTotal` est un compteur commun : **se recompte après la fusion**. Aucun PDF, aucun CHANGELOG dans la story.

### Prémisses à confirmer à l'étape 0

- `ORDER BY start_date ASC` de `find_later_closed_in_tx` (`fiscal_years.rs:1147-1150`, doc-comment) et que **c'est bien ce nom** que `fiscalYearName` porte dans `details` (un `grep` du chemin du refus de la dévalidation fournisseur, `LATER_FISCAL_YEAR_CLOSED` de `supplier_invoices`).
- Que `invoice-empty-list` n'a toujours aucun site (D1 : la fiche l'a vérifié par `grep`).
- Que le bouton unique de `JournalEntryForm.svelte:1266` sert bien la création et la modification (`isEdit`, `:141`) : prémisse de D6 relue à la rédaction.

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
- 2026-11-28 — **Validation P1 (Opus) : 0/1/10/7, remédiée** (rapport `kesh-gate-logs/e65/58-4/58-4-validation-p1.md`, 18 findings, patch unique). F1 deux clés selon le mode (D6, AC1) ; F2 quatrième clé `error-fiscal-year-create-later-closed` (D4, AC5, inventaire clos) ; F3 tests qui rougissent / ne rougissent pas, marqueurs de `loader.rs` ; F4 deux tests ISO et preuve en littéral par test à dates fixes (AC6) ; F5 `kesh_i18n::format_date` (D7) ; F6 pluriel à deux clés choisies en TS (D1) ; F7 `invoices/new` en entier (D1, AC4) ; F8 découpage #1775 à #1779 (D1, Q2, `C-58-4-k`) ; F9 ligne `fr-CH:1718` de la 58-5 déclarée ; F10 tests des deux layouts (AC3) ; F11 test à liste fermée et marques (AC4) ; F12 `invoice-empty-list` change de valeur (D1) ; F13 angle mort déclaré (D1) ; F14 `Display` de kesh-db déclaré (D7) ; F15 manuels de/it/en inexistants ; F16 AC8 limité aux refus, relevé rejouable, **décompte italien : 68 refus (45 tu + 23 Lei) sous le seuil de 80, #874 reste** ; F17 références corrigées (`FIND_LATER_CLOSED_SQL`, 2841 lignes, `homepage-` non clos) ; F18 écarts assumés pour AC2 et AC9.
