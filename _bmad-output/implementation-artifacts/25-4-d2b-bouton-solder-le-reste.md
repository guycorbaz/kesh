# Story 25.4-d2b : Solder le reste — le bouton, le dialogue, la liste

Status: ready-for-dev

**Issues : [#490]** (fermée par cette story — la PR porte `closes #490`), **[#384]** (`refs` — la 25-4-d2c, le rapport TVA,
la fermera).

**Deuxième des trois stories de la 25-4-d2.** Empilée sur la 25-4-d2a (branche `story/25-4-d2b-bouton-solder-le-reste`).

## Arbitrages (Project Lead, 2026-10-01)

Ceux de la d2a, dont : l'utilisateur **choisit la nature** (escompte accordé, frais bancaires, perte sur débiteur,
reste d'arrondi) ; un bouton **« Solder le reste »** sur la fiche, annulable par contre-passation ; toute facture
**validée non soldée**. Guy, le 2026-10-02 : « enchaîne ».

## Story

En tant que comptable,
je veux solder le reste d'une facture depuis sa fiche, en choisissant la nature de l'écart,
afin de clore une facture sans passer par l'API ni par une écriture manuelle.

## Les faits, vérifiés dans le code

- **L'API existe** (d2a) : `POST /api/v1/invoices/{id}/write-off` `{ nature, settledOn, version }`, réponse
  `{ invoice, journalEntryId, amount }` ; natures `discount`, `bank_fees`, `bad_debt`, `rounding` (cette dernière
  refusée à partir de 0.05) ; refus `WRITE_OFF_ACCOUNT_NOT_CONFIGURED`, `INVALID_INPUT` (`invoiceAlreadyPaid`,
  `nothingToWriteOff`, `writeOffRoundingTooLarge`, `settledOnBeforeInvoiceDate`), `OPTIMISTIC_LOCK_CONFLICT` (409),
  `CONFIGURATION_REQUIRED`, `FISCAL_YEAR_INVALID`, `PERIOD_LOCKED` — chacun avec son message traduit côté serveur.
- **La liste** (`GET …/settlements`) porte `settlementType = "write_off"` et `writeOffNature` ; le motif
  `INVOICE_WRITTEN_OFF` est déjà traduit (`features/invoices/settlement-cancel.ts`, d2a).
- **Le composant de liste** `InvoiceSettlements.svelte` : `typeLabel` binaire (`bank_transfer` → « Virement bancaire »,
  tout le reste → « Espèces ou autre compte ») ; un solde s'y affiche donc à tort.
- **La fiche** `routes/(app)/invoices/[id]/+page.svelte` : bouton « Enregistrer un règlement » `{#if !invoice.paidAt}`
  (`:781-785`, **non** protégé par `canManage` — le serveur refuse) ; `canManage` (`:80`) = Admin ou Comptable ; le
  récapitulatif « Déjà réglé / Reste dû » (`:1068-1088`) lit `invoice.amountSettled`, qui **inclut les soldes** (somme
  de toutes les lignes) ; `invoiceSettings` est chargé (`:248`, déclaré `:255`) — les comptes de nature sont connus du client.
- **Le dialogue de règlement** `SettleInvoiceDialog.svelte` est le patron (shadcn `Dialog`, erreur affichée dans le
  dialogue, `submitting`).
- **Types** `invoices.types.ts` : `InvoiceSettlementResponse.settlementType` est une union fermée de deux valeurs, sans
  `writeOffNature`.
- **Manuel** `docs/manual/fr/user-manual.tex` § *Enregistrer et annuler un règlement* (`:960-1006`) : ne connaît ni le
  solde, ni le motif « un solde existe », ni le reste insoldable de #490.

## Acceptance Criteria

**AC 1 — Le bouton.** Sur la fiche d'une facture **validée**, **non payée** (`!invoice.paidAt`), dont le reste dû est
**connu et positif** (`invoice.amountDue !== null && new Big(invoice.amountDue).gt(0)` — `amountDue` vaut `null` quand
la réponse ne l'a pas calculé, p. ex. après l'envoi d'un e-mail, `invoice_email.rs:885` : la fiche **relit** alors la
facture, et le bouton reste masqué tant que le reste est inconnu), un bouton **« Solder le reste »** (`data-testid="write-off-open"`), visible des **seuls** Comptable et
Administrateur (`canManage`), à côté de « Enregistrer un règlement ».

**AC 2 — Le dialogue** (`features/invoices/WriteOffDialog.svelte`, patron `SettleInvoiceDialog`) :
- le **montant soldé** affiché, en lecture seule : le reste dû (`invoice.amountDue`), au centime — **mais aux quatre
  décimales quand il porte une fraction de centime** (0.0040 ne s'affiche pas « 0.00 », cas même de #490), avec une
  phrase qui dit que la fraction de centime va au compte de différences d'arrondi ;
- **toutes les comparaisons portent sur le reste exact, en `big.js`** (`Big.lt`, `Big.eq`), jamais sur la valeur
  affichée ni sur des chaînes : `amountDue` arrive à l'échelle SQL (« 68.1000 ») ; « fraction de centime » se teste par
  `!new Big(amountDue).eq(dueToCentime(amountDue))`, avec `dueToCentime` **sorti** de `SettleInvoiceDialog.svelte`
  vers `invoice-helpers.ts` et partagé (DRY) ;
- la **nature**, choix obligatoire parmi les quatre, avec une phrase d'aide par nature (« la TVA est corrigée au
  prorata » pour l'escompte et la perte ; « sans TVA » pour les frais et l'arrondi) ; **reste d'arrondi** proposé
  seulement si le reste dû **exact** est **inférieur à 0.05** (même borne que le serveur) ; les intitulés de nature
  viennent d'**un seul** `writeOffNatureLabel` partagé avec la liste (AC 3) ;
- une nature dont le compte n'est **pas désigné** (`invoiceSettings.default*AccountId` à `null` ; le compte d'arrondi
  pour la nature `rounding`) est **signalée** dans le dialogue avec un renvoi à *Paramètres → Facturation* — et le
  bouton de confirmation est **désactivé** pour elle (le serveur refuserait de toute façon ; le signaler avant évite un
  aller-retour). ⚠️ **Le compte d'arrondi est aussi exigé pour les trois autres natures** dès que le reste dû a une
  **fraction de centime** (`amountDue` ≠ son arrondi au centime) : le serveur y impute la fraction
  (`invoice_settlements_write.rs:471-483`, d2a). Le pré-contrôle le vérifie donc pour **toute** nature dans ce cas ;
- si les réglages n'ont **pas pu être chargés** (`invoiceSettings === null`, échec toléré `:247-251`), **aucun
  pré-contrôle** : les quatre natures restent proposables et le serveur tranche — sans quoi toutes seraient
  désactivées sans explication ;
- le renvoi aux paramètres dit « Paramètres → Facturation (un administrateur) » : le `PUT` des réglages est réservé à
  l'Admin (`lib.rs:207-211`), le Comptable doit le demander ;
- un compte désigné mais **archivé ou non imputable** n'est pas détectable côté client (`InvoiceSettingsResponse` ne
  porte que l'identifiant) : le serveur le refuse au clic, et son message s'affiche dans le dialogue — limite assumée
  (cf. *Limites assumées*) ;
- la **date**, par défaut aujourd'hui ;
- confirmation → `POST …/write-off` avec la `version` de la facture affichée ; succès : la facture et la liste sont
  relues, notification ; refus : le **message du serveur** dans le dialogue. Les cas se distinguent sur `err.code`, **jamais sur
  le statut HTTP** (un 409 peut aussi être `ILLEGAL_STATE_TRANSITION`, `errors.rs:2841-2847`) :
  - `OPTIMISTIC_LOCK_CONFLICT` : la facture est relue et le dialogue le dit (« la facture a changé, vérifiez le reste
    dû ») ;
  - `ILLEGAL_STATE_TRANSITION` (facture dévalidée entre-temps), et `INVALID_INPUT` « déjà payée » ou « rien à solder » :
    la facture est relue et le dialogue **se ferme** ;
  - après **toute** relecture : le dialogue se ferme si la facture est payée ou son reste nul ; la nature choisie est
    réinitialisée si elle n'est plus proposée (reste d'arrondi devenu ≥ 0.05). ⚠️ **Sinon** — facture encore validée,
  non payée, reste positif, c'est le cas d'`OPTIMISTIC_LOCK_CONFLICT` — le dialogue **reste ouvert** et le
  **montant affiché comme la `version` envoyée suivent la facture relue** : ils se lisent dans la facture passée au
  dialogue **au moment de la confirmation**, pas dans une copie faite à l'ouverture (le patron `SettleInvoiceDialog` ne
  se resynchronise qu'à l'ouverture, `:91-101` — ne pas le reproduire ici, sans quoi une nouvelle tentative renverrait
  l'ancienne version et bouclerait en 409).

**AC 3 — La liste.** `typeLabel` connaît le solde : « Solde — escompte accordé » / « — frais bancaires » / « — perte
sur débiteur » / « — reste d'arrondi » selon `writeOffNature`. Types TS : `settlementType` ajoute `'write_off'`,
`writeOffNature` ajouté. Le bouton « Annuler le règlement » d'un solde dit « **Annuler le solde** » — et, sur la fiche,
le **dialogue de confirmation** qu'il ouvre (`+page.svelte`, titre `:1339`, description `:1343`, bouton `:1364`) et la
**notification de succès** (`:193`) disent « solde » quand la cible est un solde (`cancelTarget.settlementType ===
'write_off'`) : « Annuler ce solde ? … », « Solde annulé : l'écriture inverse a été passée. » La nature de la cible est **figée à
l'ouverture** du dialogue (sinon, `cancelTarget = null` le fermant, le titre repasserait à « règlement » pendant
l'animation) et lue **avant** l'`await` pour la notification. ⛔ **Des clés neuves** pour les variantes « solde »
(`invoices-write-off-cancel-*`) : changer le repli d'une clé existante selon le cas violerait « une clé, un repli »
(`i18n-un-repli-par-cle.test.ts`) ; et **pas** de clé construite par gabarit sur la nature (elle ferait bouger
`sitesNonResolus`).

**AC 4 — Le récapitulatif.** Sur la fiche, « Déjà réglé » ne compte plus les soldes : une ligne **« Soldé »** (la somme
des lignes `write_off` de la liste) s'ajoute quand il y en a, et « Déjà réglé » = `amountSettled − soldé`. Les
montants se calculent avec `big.js`, jamais en `Number`. ⚠️ Si la liste des règlements **n'a pas pu être chargée**
(échec toléré, `loadSettlements` → `[]`), la séparation est impossible : le récapitulatif n'affiche alors **que le
« Reste dû »** (juste, il vient du serveur), sans « Déjà réglé » ni « Soldé » — plutôt qu'un « Déjà réglé » qui
inclurait silencieusement le solde. Un drapeau « liste chargée » le distingue d'une liste vide : il passe à **faux au début de chaque**
`loadSettlements` et en cas d'échec, à **vrai seulement en cas de succès** — sans quoi, entre `invoice = res.invoice`
et la relecture de la liste (`+page.svelte:190-198`), le récapitulatif combinerait un `amountSettled` neuf et une
liste ancienne. Pendant le chargement, même affichage qu'en cas d'échec (« Reste dû » seul). Conséquence assumée : sans
liste, une facture **sans** solde perd elle aussi sa ligne « Déjà réglé ».

**AC 5 — i18n** : toutes les chaînes dans les 4 locales, `sitesTotal` recompté, compteur de libellés en dur
(`i18n-libelle-en-dur.test.ts`) recompté si une déclaration `*Label` est ajoutée.

**AC 6 — Le manuel utilisateur** (`user-manual.tex` § règlements) : un paragraphe **« Solder le reste »** (natures, TVA
au prorata pour escompte et perte, compte par nature dans les paramètres, reste d'arrondi sous 5 centimes — le cas
d'un reste trop petit pour être payé, #490 —, annulation comme un règlement) ; le motif « **un solde existe** —
annulez d'abord le solde » ajouté à la liste des refus d'annulation. PDF régénéré, contrôlé **aplati**. CHANGELOG :
l'entrée d2a (« le bouton arrive avec la suite ») mise à jour — le bouton est là ; #490 cité comme fermée. ⚠️ **Tous les
sites du symptôme** (grep `arrive avec la suite\|fonction prévue\|\b490\b` sur `CHANGELOG.md`, `docs/`, `website/`,
`README.md`, fait à la validation P2) : `CHANGELOG.md:49` (« ⚠️ Reste ouvert … #490 » → refermé, renvoi au solde),
`CHANGELOG.md:79` (entrée d1, « il arrive avec la suite de cette version »), `user-manual.tex:1063-1064` (l'encadré de
l'avoir refusé : « passer le reste en perte --- fonction prévue » → renvoi au § *Solder le reste*) ; et le manuel
**admin** (`admin-manual.tex`, § compte de différences d'arrondi, `:2009`) : il reçoit aussi la fraction de centime
d'un solde. Les deux PDF régénérés et contrôlés aplatis.

**AC 7 — Tests.** Vitest : le dialogue (natures proposées selon le reste, nature sans compte signalée et confirmation
désactivée, **compte d'arrondi exigé pour toute nature sur un reste à fraction de centime**, payload envoyé avec la
`version`, message serveur affiché, 409 puis **nouvelle tentative avec la version relue**) ; le dialogue d'annulation et
sa notification (« solde » / « règlement » selon la cible) ; le récapitulatif quand la liste n'a pas pu être chargée ; la liste (libellé par nature, « Annuler
le solde ») ; le récapitulatif (« Soldé » séparé, `big.js`). Vitest encore : `ILLEGAL_STATE_TRANSITION` ferme le dialogue ; « 68.1000 » n'exige pas le compte d'arrondi ;
0.0040 s'affiche aux quatre décimales ; réglages inconnus → aucun pré-contrôle ; `amountDue` à `null` → pas de bouton ;
relecture de la liste en échec après un solde → « Reste dû » seul. **E2E** `invoice-write-off.spec.ts` — **montage** :
désigner un compte d'escompte imputable (le 4000 *Charges CI* du seed) **par le formulaire** *Paramètres → Facturation*
(connecté en Admin), comme le patron — un `PUT` brut exigerait aussi `invoiceNumberFormat`, `defaultSalesJournal` et
`journalEntryDescriptionTemplate` (`company_invoice_settings.rs:83-91`) —, une
facture au TTC multiple de 0.05 à TVA 8.10 (le prorata s'exerce, sans compte d'arrondi requis) ; **nettoyage** : les
réglages remis à vide (patron `invoice-settings-write-off-accounts.spec.ts`). Scénario : une facture réglée
en partie, soldée en escompte depuis la fiche → reste dû nul, ligne « Solde — escompte accordé », le règlement
antérieur montre le motif « annulez d'abord le solde », le solde s'annule et le reste réapparaît ; le test remet son
état (base partagée).

## Tasks / Subtasks

- [ ] **T1 — types et API client** (`invoices.types.ts`, `invoices.api.ts` : `writeOffInvoice`).
- [ ] **T2 — le dialogue** (AC 2).
- [ ] **T3 — la fiche : bouton, récapitulatif** (AC 1, AC 4).
- [ ] **T4 — la liste** (AC 3).
- [ ] **T5 — i18n** (AC 5).
- [ ] **T6 — manuel, CHANGELOG** (AC 6).
- [ ] **T7 — tests** (AC 7).
- [ ] **T8 — gates** : frontend complet, backend (non touché : `fmt`, `clippy`, et le gate complet pour la forme),
  **E2E complet**.

## Dev Notes

### Ce qu'il ne faut pas faire

- ⛔ Un montant saisi : le serveur solde le reste exact.
- ⛔ Traduire les refus côté client : le message du serveur est déjà traduit (sauf le 409, qui appelle une relecture).
- ⛔ Toucher le serveur : tout ce dont l'écran a besoin existe depuis la d2a.

### Limites assumées

- L'**échéancier** (liste : Total, Reste dû, Statut, Payée le ; export : mêmes colonnes) ne distingue pas l'encaissé du
  soldé : une facture soldée en perte y paraît **« Payée »**, reste nul. *(Une première rédaction disait que
  l'échéancier affichait `amountSettled` — faux, corrigé en validation P2, et dans #496.)* Distinguer exigerait un champ
  serveur
  (`amountWrittenOff`) — hors périmètre : **[#496]** (CR ouverte le 2026-10-02, à la demande de Guy).
- Un compte de nature désigné mais **archivé ou non imputable** passe le pré-contrôle du dialogue : le serveur le
  refuse au clic (`WRITE_OFF_ACCOUNT_NOT_CONFIGURED`), message affiché dans le dialogue.
- Le compte de **TVA due**, exigé pour l'escompte et la perte d'une facture taxée, n'est pas pré-contrôlé : le serveur
  refuse (`CONFIGURATION_REQUIRED`), message dans le dialogue.
- Le nouveau bouton est réservé à Comptable et Administrateur (`canManage`), comme le patron le plus récent (« Annuler
  le règlement ») ; le bouton voisin « Enregistrer un règlement » ne l'est pas — incohérence **antérieure**, laissée
  telle quelle (le serveur refuse de toute façon).
- Le statut d'une facture soldée reste **« Payée »** (`paymentStatusOf`) ; un statut « Soldée » est une autre story :
  **[#497]** (CR ouverte le 2026-10-02).

### Modules

`frontend`, `kesh-i18n`, `docs` (+ `CHANGELOG`) — deux modules de code.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-10-02** — Validation P3 ciblée (Sonnet) : 1 MED, 1 LOW, retenus. **M1** : la règle de fermeture « après toute
  relecture » et la phrase « le dialogue reste ouvert » n'étaient pas reliées — le second cas est écrit comme le
  complémentaire du premier (« Sinon — … `OPTIMISTIC_LOCK_CONFLICT` »). **L1** : le montage de l'E2E passe par le
  formulaire des paramètres (un `PUT` brut exigerait trois champs de plus). La lentille avait aussi soupçonné un site du
  symptôme manquant à l'AC 6 — `CHANGELOG.md:81`, l'entrée d2a (« arrive avec la suite ») — et l'a trouvé couvert par la
  phrase d'introduction de l'AC 6 (« l'entrée d2a … mise à jour ») : soupçon réfuté, aucune correction requise.
- **2026-10-02** — Validation P4 ciblée (Haiku) : 1 MED, 1 LOW. **Le MED est réfuté** (erreur de catégorie) : il
  reprochait à l'entrée P3 d'affirmer une vérification « sans trace dans le diff » — or un soupçon vérifié et réfuté
  n'appelle pas de modification ; reclassé LOW, et l'entrée P3 nomme désormais le site (`CHANGELOG.md:81`) et la raison.
  **L1** : les corrections de P3 sont étiquetées (M1, L1). La partition des refus de l'AC 2 et la référence
  `company_invoice_settings.rs:83-91` sont confirmées. **Boucle close** : P1 2H/3M/2L (Sonnet) → P2 9M/8L (Opus) → P3
  1M/1L (Sonnet) → P4 0 > LOW (Haiku) ; remédiation de P4 sur la fiche seule.
- **2026-10-02** — Validation P2 (Opus) : 9 MED, 8 LOW, tous retenus. Un 409 n'est pas toujours une version périmée →
  cas distingués sur `err.code`, dialogue fermé sur facture dévalidée ou payée (M1) ; pré-contrôle « fraction de centime »
  en `Big.eq`, `dueToCentime` partagé (M2) ; reste à fraction de centime affiché aux quatre décimales — 0.0040 n'est
  plus « 0.00 » (M3) ; drapeau « liste chargée » défini (M4) ; `amountDue` à `null` (M5) ; réglages inconnus → aucun
  pré-contrôle (M6) ; tous les sites du symptôme au CHANGELOG et aux deux manuels (M7) ; **la limite sur l'échéancier
  reposait sur un fait faux, que j'avais recopié dans #496** — les deux corrigés (M8) ; montage et nettoyage de l'E2E
  (M9) ; et les LOW (ligne citée, nature figée à l'ouverture, clés neuves, renvoi à l'administrateur, compte de TVA en
  limite, perte de « Déjà réglé » sans liste, libellé de nature partagé, manuel admin).
- **2026-10-02** — Validation P1 (Sonnet) : 2 HIGH, 3 MED, 2 LOW, tous retenus. Compte d'arrondi exigé pour **toute**
  nature sur un reste à fraction de centime (H1) ; le dialogue de confirmation d'annulation et sa notification disent
  « solde » pour un solde (H2) ; récapitulatif réduit au « Reste dû » si la liste des règlements n'a pas pu être chargée
  (M1) ; compte archivé non détectable côté client, écrit en limite assumée (M2) ; montant et `version` lus au moment de
  la confirmation, dialogue ouvert après un 409 (M3) ; plage du manuel corrigée, asymétrie `canManage` écrite (L1, L2).
- **2026-10-02** — Créée (Guy : « enchaîne »). Les deux limites assumées sont tracées en CR à la demande de Guy :
  #496 (encaissé et soldé à l'échéancier), #497 (statut « Soldée »).

[#384]: https://github.com/guycorbaz/kesh/issues/384
[#490]: https://github.com/guycorbaz/kesh/issues/490
[#496]: https://github.com/guycorbaz/kesh/issues/496
[#497]: https://github.com/guycorbaz/kesh/issues/497
