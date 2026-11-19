# Story 49-1c : types de lignes de la facture client — l'écran, la ligne de commentaire, l'ouverture de `comment` dans l'API (#1242)

Status: ready-for-dev

*Fiche rédigée le 2026-11-19 par l'agent de rédaction de la 49-1c (Opus 5.5), par le workflow `bmad-create-story`.
Release **0.20.0** (coupure jeudi 2026-11-26 au soir). **Aucune migration** (celle de la release est la 49-1a,
`20261118000002`). Code lu à la tête de la branche `story/49-1c-types-de-lignes-ecran` = **`5156f2b7b`** : la branche
`epic/49-types-de-lignes-facture` (49-1a et 49-1b fusionnées, `f57f6ac80`) augmentée de la 48-13 (fusion `5156f2b7b`,
`InvoiceForm.svelte` : la 48-13 d'abord, arbitrage de Guy du 2026-11-18). Sources : `epic-49-types-de-lignes-facture.md`
(§ 49-1, tableau des fichiers), `release-0.20.0.md`, `kesh-gate-logs/e491/decoupage-49-1.md`, les fiches closes
`49-1a-types-de-lignes-migration-api.md` et `49-1b-types-de-lignes-pdf.md` (décisions, dettes renvoyées ici), les issues
#1242, #1400, #1401 ; les **décisions de Guy du 2026-11-19** sur les trois questions de la fiche (portées par l'orchestrateur
au fichier d'épique, `c91a5bac4`, et au `sprint-status.yaml`, fusion `2cce7b905`). Les choix de cette fiche sont D1 à D12.
Validation P1 (Sonnet) remédiée, voir le Change Log.*

**Issues** : `refs #1242`, `refs #1400`, `refs #1401` ; **aucun mot-clé de fermeture** dans un commit. #1242 ne se ferme
pas avec cette story (réordonnancement 49-2, titres 49-3, sous-totaux #1330). #1364 (capacité du PDF contrôlée avant la
validation) et #1400 vont à la **49-4**, pas ici ; #1411 (nom et adresse du contact en NFD) est au milestone
« Plus tard ». **Règle de sortie** (décision de Guy du 2026-11-19, `epic-49-types-de-lignes-facture.md:154`) : **la 49-1c
ne sort pas sans la 49-4**, ni sans la 49-1b.

## Story

En tant que Guy, qui émettra ses vraies factures clients avant la fin novembre, je veux pouvoir **ajouter à une facture une
ligne de commentaire** — du texte seul, sans quantité, sans prix, sans TVA, hors du total et des écritures —, la **retrouver
telle quelle** quand je rouvre le brouillon, la **voir** sur la fiche de la facture et sur celle de l'avoir qui l'annule, et
la voir **imprimée** sur le PDF, **sans qu'aucune facture ni aucun avoir existants changent** à l'écran, dans les livres ou
sur le papier.

C'est la story qui **rend la fonctionnalité visible** : la 49-1a a posé le type en base et gardé l'API fermée, la 49-1b a
appris au PDF à imprimer un commentaire. La 49-1c **ouvre `comment`** dans l'API (retrait de la garde pour ce seul type),
lève le dernier obstacle du serveur (`verify_vat_rates_against_db`) et livre l'écran, les textes et le manuel.
`heading` reste fermé jusqu'à la 49-3.

## Ce qui se lit au code (`5156f2b7b`)

Chaque ligne citée est copiée du fichier, lue à la tête du worktree.

### Les prérequis sont sur la branche

- La 49-1b est fusionnée : `git merge-base --is-ancestor 57758e1b9 HEAD` rend 0 ; `crates/kesh-qrbill/src/types.rs:195`
  `pub enum LinePdfKind {` ; `crates/kesh-api/src/routes/invoice_pdf_service.rs:39` `pub(crate) fn ligne_pdf(` ; le
  chemin du commentaire, `crates/kesh-qrbill/src/pdf.rs:1148` `fn texte_de_commentaire(description: &str) -> String {`
  et `pdf.rs:1182` `fn lignes_de_commentaire(description: &str) -> Vec<String> {`. La règle de sortie « la 49-1c ne sort
  pas sans la 49-1b » est donc tenue par la branche.

### La garde de l'API

- `crates/kesh-db/src/entities/line_type.rs:56-61` :
  ```rust
      pub fn open_to_api(&self) -> bool {
          match self {
              Self::Priced => true,
              Self::Comment | Self::Heading => false,
          }
      }
  ```
  Son doc-comment (`line_type.rs:47-55`) dit « La Story 49-1c ouvre `comment`, la Story 49-3 `heading` ». Son test
  `line_type.rs:130` `    fn only_priced_is_open_to_api() {` affirme `assert!(!LineType::Comment.open_to_api());` (`:132`).
  **La garde vit dans `kesh-db`**, non dans `kesh-api` : le fichier d'épique ne range pas `kesh-db` parmi les modules de la
  49-1c (écart déclaré, § *Dev Notes*).
- `crates/kesh-api/src/routes/invoices.rs:621-626` :
  ```rust
      if !line_type.open_to_api() {
          return Err(AppError::Validation(format!(
              "Ligne {} : le type de ligne {line_type} n'est pas encore disponible",
              index + 1
          )));
      }
  ```
  dans `validate_unpriced_line` (`invoices.rs:598`), qui exige d'abord quantité, prix et taux **absents ou nuls** et
  **aucun** compte de produit (`invoices.rs:604-620`), puis rend `NewInvoiceLine { quantity: Decimal::ZERO, …, line_type }`
  (`invoices.rs:627-634`).
- `invoices.rs:507` `    let description: String = req.description.trim().nfc().collect();` : **la description de toute
  ligne, chiffrée ou non, est normalisée en NFC à la saisie** (depuis la Story 5.1, `bf8f5d145`, avril 2026), avant la
  bifurcation par type (`invoices.rs:521` `    if !req.line_type.is_priced() {`). Le texte d'un commentaire est borné
  par `invoices.rs:58` `const MAX_DESCRIPTION_LEN: usize = 1000;` (`:513-517`) ; il accepte les sauts de ligne (seul
  `trim` les retire en tête et en fin).
- Les doc-comments qui décrivent la garde : `invoices.rs:80-86` (champ `line_type` de `CreateInvoiceLineRequest` :
  « `comment` et `heading` sont reconnus mais **refusés** (`400`) tant que la garde … ») et `invoices.rs:594-597`
  (`validate_unpriced_line`).
- Les tests qui affirment la garde fermée : `invoices.rs:1913` `    /// Story 49-1a (D9) : la garde refuse `comment` et `heading`, même`
  (assertions `:1923`, `:1930`) ; `crates/kesh-api/tests/invoice_line_types_e2e.rs:349-351`
  `/// D9 — `comment` et `heading` refusés `400` au `POST` et au `PUT`, sans` …
  `async fn unpriced_types_are_refused_without_writing(pool: MySqlPool) {`, qui boucle `for t in ["comment", "heading"]`
  (`:366`). **Ces deux tests rougissent dès que la garde s'ouvre** : ils se réécrivent (D2), ils ne se suppriment pas.

### La liste blanche des taux de TVA

- `invoices.rs:771-772` (création) et `invoices.rs:864-865` (modification), identiques :
  ```rust
      let rates: Vec<Decimal> = lines.iter().map(|l| l.vat_rate).collect();
      vat::verify_vat_rates_against_db(&state.pool, current_user.company_id, &rates).await?;
  ```
- `crates/kesh-api/src/routes/vat.rs:79` `pub async fn verify_vat_rates_against_db(` : `SELECT COUNT(DISTINCT rate) FROM
  vat_rates WHERE company_id = ? AND active = TRUE AND rate IN (…)` (`vat.rs:96-97`) ; un taux absent ou désactivé rend
  `Err(AppError::Validation(VAT_REJECTED_MSG.into()))` (`vat.rs:109`). **Un commentaire porte `vat_rate = 0`
  (`invoices.rs:631` `        vat_rate: Decimal::ZERO,`) : dans une société dont le taux 0 % est désactivé ou absent, la facture qui le porte serait refusée
  « taux de TVA », au `POST` comme au `PUT`.** En 49-1a le site était inatteignable (la garde, dans `validate_lines`,
  rejette le type avant) ; il devient atteignable avec l'ouverture.
- `vat.rs` lui-même ne connaît pas les lignes (ses autres appelants, `products.rs:388` et `:416`, lui passent le taux d'un
  article) : le tri par type se fait **chez l'appelant** (D4).

### L'écran de saisie (`frontend/src/lib/components/invoices/InvoiceForm.svelte`, 1158 lignes)

- `InvoiceForm.svelte:112` `	type LineState = CreateInvoiceLineRequest & { _uiKey: string; _vatPlaceholder?: boolean };`
  et `frontend/src/lib/features/invoices/invoices.types.ts:346-361` : `CreateInvoiceLineRequest` porte `description`,
  `quantity: string`, `unitPrice: string`, `vatRate: string`, `revenueAccountId?` — **aucun type de ligne**.
  `git grep -n "lineType" -- frontend/src` est **vide** (rc 1). `InvoiceLineResponse` (`invoices.types.ts:16-38`) et
  `CreditNoteLineResponse` (`frontend/src/lib/features/credit-notes/credit-notes.types.ts:8-29`) n'ont pas non plus le
  champ, que l'API renvoie pourtant depuis la 49-1a (`invoices.rs:192` `    pub line_type: LineType,`,
  `crates/kesh-api/src/routes/credit_notes.rs:64`).
- **Deux sites relisent les lignes d'un brouillon**, sans le type : `InvoiceForm.svelte:126-137` (`initLines`, ouverture
  du brouillon : `return invoice.lines.map((l) => ({` … `description: l.description,` `quantity: readableAmount(l.quantity, 0),`
  …) et `InvoiceForm.svelte:778-791` (`reloadFromServer`, rechargement après un conflit de version :
  `lines = fresh.lines.map((l) => ({` …). Le second porte déjà l'avertissement de la Story 16-1a (`:783-788`) : *« Un oubli
  ici n'est visible qu'APRÈS un conflit de version … `stripUiKey` étant un spread, rien ne signale l'absence du champ. »*
  **C'est exactement le risque que l'épique nomme** : un `PUT` qui omet `lineType` fait redevenir chiffrée une ligne de
  commentaire (`invoices.rs:68-76`, `lineType` absent vaut `priced`), refusée alors pour sa quantité nulle.
- L'envoi : `InvoiceForm.svelte:704-713`, `payloadLines.push({ ...stripUiKey(lines[i]), unitPrice: prix.payload });` après
  `readLineUnitPrice` (`:706`) — **toute ligne passe par la lecture du prix**.
- Les contrôles du client : `validateClient` (`:578-636`) exige pour **chaque** ligne une description, un prix lisible, une
  quantité `> 0` (`:613` `				if (qty.lte(0)) return `Ligne ${i + 1} : quantité > 0`;`) et un taux actif (`:621-633`).
- Le total : `InvoiceForm.svelte:529-533` `computeInvoiceTotal(lines.map(…))`
  (`frontend/src/lib/features/invoices/invoice-helpers.ts:40` `export function computeInvoiceTotal(lines: Pick<CreateInvoiceLineRequest, 'quantity' | 'unitPrice'>[]): string {`).
- Les taux désactivés : `InvoiceForm.svelte:260-266` (`inactiveRates`, dérivé de `lines.map((l) => l.vatRate)`) ; le repli
  du taux provisoire : `:98-107` (`_vatPlaceholder`).
- Les gestes d'ajout : `InvoiceForm.svelte:483` `	function addFreeLine() {` et `:499` `	function onProductSelect(p: ProductResponse) {`
  (ligne sans produit et ligne produit : **le même enregistrement**, 49-1a D1), et leurs boutons `:1094-1101`
  (« Ligne libre », « Depuis catalogue », libellés **en dur**). Une ligne s'ajoute **en fin de tableau** (`lines.push`).
- Le tableau : `InvoiceForm.svelte:952` `		<table class="w-full border-collapse text-sm" data-testid="invoice-lines-table">`, sept
  colonnes (`:540` `	const LINE_TABLE_COLUMNS = 7;`), la description en `<Input type="text" bind:value={line.description} />`
  (`:976`) — **une seule ligne de texte**, alors qu'un commentaire peut en porter plusieurs (49-1b, D5).
- Les tests de ce composant : `frontend/src/lib/components/invoices/InvoiceForm.test.ts` (1655 lignes), qui capture déjà la
  charge d'un `PUT` (`:31` `const updateInvoiceMock = vi.fn();`, `:348` `const payload = updateInvoiceMock.mock.calls[0][1] as {`).

### Les écrans de lecture

- Fiche facture : `frontend/src/routes/(app)/invoices/[id]/+page.svelte:1386` `				{#each invoice.lines as l (l.id)}`, six
  cellules par ligne (description, quantité, prix, `{l.vatRate}%`, compte de produit, total, `:1387-1398`). La cellule du
  compte appelle `revenueAccountCell(l.revenueAccountId)` (`:321-328`), qui rend « … (défaut société) » pour un compte
  `null` sur un brouillon : **pour un commentaire (compte toujours `NULL`, contrainte de la 49-1a), ce serait faux**.
- Fiche avoir : `frontend/src/routes/(app)/credit-notes/[id]/+page.svelte:177` `			{#each creditNote.lines as line (line.position)}`,
  six cellules (`:178-190`). Un commentaire repris de la facture (snapshot de la 49-1a) s'y afficherait
  `0 · 0.00 · 0.00 · — · 0.00`.
- La fiche facture et la fiche avoir sont testées par `invoice-display-page.test.ts` et `credit-note-display-page.test.ts`.

### L'avoir, le PDF, les sommes : déjà justes

- L'avoir **reprend** le type et la position de chaque ligne (49-1a ; `docs/api-external.md:514` : « Un avoir reprend le type
  et la position de chaque ligne de sa facture »). Le PDF de la facture et de l'avoir imprime un commentaire pleine largeur
  (49-1b, AC3). Les sommes, la ventilation de TVA et l'écriture de vente écartent la ligne non chiffrée (49-1a, AC8 ;
  49-1b, D9). **Une facture faite de commentaires seuls ne se valide pas** (`invoiceTotalZero`, fiche 49-1a, l. 135) ;
  son brouillon s'enregistre. Rien de cela ne change ici.

### Les textes qui deviendront faux

- `docs/api-external.md:512` : « Les valeurs **`comment` et `heading` sont réservées** : reconnues, elles sont refusées en
  **`400 VALIDATION_ERROR`** (« Ligne N : le type de ligne comment n'est pas encore disponible ») … ».
- `docs/manual/fr/admin-manual.tex:2559` (§ « Pour les intégrations », dans la section « Passer à la 0.20.0 » que complète
  la 49-1d) : « \texttt{comment} et \texttt{heading} sont refusés pour l'instant (\texttt{400 VALIDATION\_ERROR}) ».
- `docs/manual/fr/user-manual.tex:1456` : `\item \textbf{Lignes de facture} : un ou plusieurs produits avec quantité, prix,
  TVA, et \textbf{compte de produit}.` — ne connaît que la ligne chiffrée.
- `user-manual.tex:1594` : « d'ici là, le \textbf{reste du corps} (libellés des lignes, conditions, téléphone, e-mail et
  site) n'a pas ce repli et perd encore ces lettres » — **devient faux pour le commentaire**, qui replie les 125 lettres sur
  leur base et garde ’ et — (tâche que la 49-1b laisse ici, Dev Agent Record de la 49-1b, « Manuel, E-5 »).
- `user-manual.tex:1924` : « crée un avoir reprenant toutes les lignes de la facture » — reste vrai ; se précise
  (commentaires compris).
- Manuels DE, EN, IT : n'existent pas (`docs/manual/de/README.md:3`). Les PDF des manuels ne se commitent pas.

## Inventaire par le symptôme

Commandes rejouées à `5156f2b7b`.

**S1 — « la garde refuse `comment` »** (le texte ou le test qui le dit, ou le code qui le fait) :

```
$ git grep -n -E "open_to_api|pas encore disponible" -- crates
crates/kesh-api/src/routes/invoices.rs:83:    /// [`LineType::open_to_api`] ne les ouvre pas ; une valeur inconnue est un
crates/kesh-api/src/routes/invoices.rs:595:/// compte. Puis **la garde** : un type que [`LineType::open_to_api`] ne
crates/kesh-api/src/routes/invoices.rs:621:    if !line_type.open_to_api() {
crates/kesh-api/src/routes/invoices.rs:623:            "Ligne {} : le type de ligne {line_type} n'est pas encore disponible",
crates/kesh-api/src/routes/invoices.rs:1913:    /// Story 49-1a (D9) : la garde refuse `comment` et `heading`, même
crates/kesh-api/src/routes/invoices.rs:1923:                format!("Ligne 2 : le type de ligne {nom} n'est pas encore disponible")
crates/kesh-api/src/routes/invoices.rs:1930:            assert!(message(validate_line(z, 0).unwrap_err()).contains("pas encore disponible"));
crates/kesh-api/tests/invoice_line_types_e2e.rs:382:            format!("Ligne 2 : le type de ligne {t} n'est pas encore disponible")
crates/kesh-api/tests/invoice_line_types_e2e.rs:396:            message(&err).contains("n'est pas encore disponible"),
crates/kesh-db/src/entities/line_type.rs:56:    pub fn open_to_api(&self) -> bool {
crates/kesh-db/src/entities/line_type.rs:130:    fn only_priced_is_open_to_api() {
crates/kesh-db/src/entities/line_type.rs:131:        assert!(LineType::Priced.open_to_api());
crates/kesh-db/src/entities/line_type.rs:132:        assert!(!LineType::Comment.open_to_api());
crates/kesh-db/src/entities/line_type.rs:133:        assert!(!LineType::Heading.open_to_api());
$ git grep -n -E "pas encore disponible|pour l.instant|sont réservées" -- docs/api-external.md docs/manual/fr
docs/api-external.md:512:- **Création et modification** (`POST /invoices`, `PUT /invoices/{id}`) : chaque ligne accepte un champ optionnel `
docs/api-external.md:1009:*(Section créée par la v0.14.0, …                    ← autre sujet (dossier d'import)
docs/api-external.md:1153:² **Trois opérations sur les factures sont réservées à l'interface web** …  ← autre sujet
docs/manual/fr/admin-manual.tex:2559:\paragraph{Pour les intégrations.} Les lignes d'une facture client portent désormais un \textbf{type} :
docs/manual/fr/user-manual.tex:4468:transactions à rapprocher) est prévu mais pas encore disponible.  ← autre sujet (rapprochement)
```

S'y ajoutent les doc-comments de `line_type.rs:47-55` et `invoice_line_types_e2e.rs:8-10` (en-tête du fichier de tests :
« `comment` et `heading` sont refusés `400 VALIDATION_ERROR` au `POST` **et** au `PUT` »), trouvés par
`git grep -n -i "refusés" -- crates/kesh-api/tests/invoice_line_types_e2e.rs crates/kesh-db/src/entities/line_type.rs`.
**Tous dans le périmètre** (D2, D11), hors les trois lignes marquées « autre sujet ».

**S2 — « les taux de TVA de toutes les lignes passent la liste blanche »** :

```
$ git grep -n "verify_vat_rates_against_db(" -- 'crates/*/src/*'
crates/kesh-api/src/routes/invoices.rs:772:    vat::verify_vat_rates_against_db(&state.pool, current_user.company_id, &rates).await?;
crates/kesh-api/src/routes/invoices.rs:865:    vat::verify_vat_rates_against_db(&state.pool, current_user.company_id, &rates).await?;
crates/kesh-api/src/routes/products.rs:388:    vat::verify_vat_rates_against_db(&state.pool, current_user.company_id, &[v.vat_rate]).await?;
crates/kesh-api/src/routes/products.rs:416:    vat::verify_vat_rates_against_db(&state.pool, current_user.company_id, &[v.vat_rate]).await?;
crates/kesh-api/src/routes/vat.rs:79:pub async fn verify_vat_rates_against_db(
```

Deux sites de lignes (`invoices.rs:772`, `:865`), **dans le périmètre** (D4). `products.rs` vérifie le taux d'un article,
non d'une ligne : hors du symptôme. Aucune autre route ne reçoit de lignes de facture client (`git grep -ln
"validate_lines\|CreateInvoiceLineRequest" -- crates` ne nomme que `routes/invoices.rs` côté production).

**S3 — « une ligne de facture ou d'avoir s'affiche ou se relit sans son type »** (le frontend) :

```
$ git grep -n -E "\.lines\b" -- 'frontend/src/*.svelte' | grep -E "invoices|credit-notes"
frontend/src/lib/components/invoices/InvoiceForm.svelte:127:		if (invoice && invoice.lines.length > 0) {
frontend/src/lib/components/invoices/InvoiceForm.svelte:128:			return invoice.lines.map((l) => ({
frontend/src/lib/components/invoices/InvoiceForm.svelte:778:			lines = fresh.lines.map((l) => ({
frontend/src/routes/(app)/credit-notes/[id]/+page.svelte:177:			{#each creditNote.lines as line (line.position)}
frontend/src/routes/(app)/invoices/[id]/+page.svelte:1386:				{#each invoice.lines as l (l.id)}
$ git grep -ln "InvoiceLineResponse\|CreditNoteLineResponse\|CreateInvoiceLineRequest" -- frontend/src
frontend/src/lib/components/invoices/InvoiceForm.svelte
frontend/src/lib/features/credit-notes/credit-notes.types.ts
frontend/src/lib/features/invoices/invoice-helpers.ts
frontend/src/lib/features/invoices/invoices.types.ts
frontend/src/lib/features/supplier-invoices/supplier-invoice-form.ts      ← facture fournisseur, autre type
frontend/src/lib/features/supplier-invoices/supplier-invoices.types.ts    ← idem
```

(La première commande rend aussi quatre lignes de `supplier-invoices`, écartées par le filtre : les factures fournisseur
n'ont pas de type de ligne.) **Cinq sites, tous dans le périmètre** : deux relectures du brouillon (D5), deux affichages
(D7), et les types miroirs (`invoices.types.ts`, `credit-notes.types.ts`) ; `invoice-helpers.ts` (le total, D6). Aucun
autre écran ne lit une facture client : `git grep -n "getInvoice(" -- frontend/src` (hors tests) ne nomme que la fiche
(`invoices/[id]/+page.svelte`, dix appels), l'édition (`invoices/[id]/edit/+page.svelte:21`, qui passe la facture à
`InvoiceForm`) et `InvoiceForm.svelte:771` lui-même.

**S4 — « un caractère du commentaire disparaît du PDF sans avertissement »** (#1400, #1401) : traité en D9 et D10 ;
le chemin du commentaire est `pdf.rs:1148` (`texte_de_commentaire`), celui de la ligne chiffrée `pdf.rs:668`
(`truncate_display`, 49-1b D8). Les autres champs du PDF qui pourraient perdre un accent saisi en NFD :

```
$ git grep -n "\.nfc()" -- 'crates/*/src/*'
crates/kesh-api/src/routes/invoices.rs:501:        let t: String = v.trim().nfc().collect();
crates/kesh-api/src/routes/invoices.rs:507:    let description: String = req.description.trim().nfc().collect();
crates/kesh-api/src/routes/products.rs:183:    let trimmed_name: String = name.trim().nfc().collect();
crates/kesh-api/src/routes/products.rs:195:    let description = normalize_optional(description).map(|d| d.nfc().collect::<String>());
```

Les libellés des lignes (`:507`) et les conditions de paiement (`:501`) sont normalisés ; **le nom et l'adresse du
contact ne le sont pas** (`git grep -n -i "nfc" -- crates/kesh-api/src/routes/contacts.rs` est vide), alors qu'ils
s'impriment au corps de la facture et à la section de paiement. Hors du champ de #1401 tel que son corps le borne (« les
libellés des lignes chiffrées et … les commentaires ») : porté par **#1411** (ouverte par l'orchestrateur, milestone
« Plus tard »).

## Décisions

- **D1 — Le type se choisit à l'ajout : un troisième bouton, « Commentaire ».** À côté de « Ligne libre » (ligne sans
  produit) et « Depuis catalogue » (ligne produit), qui sont déjà, à l'écran, le sélecteur des deux formes de la ligne
  chiffrée (49-1a D1 : un seul enregistrement), un bouton **« Commentaire »** ajoute une ligne de commentaire en fin de
  tableau. Le type d'une ligne **ne se change plus** une fois ajoutée : on la supprime et on en ajoute une autre.
  *Écartées* : (a) un `<select>` de type sur chaque ligne — changer une ligne chiffrée en commentaire jetterait en silence
  quantité, prix, taux et compte, et l'inverse inventerait des montants ; (b) un menu déroulant « Ajouter ▾ » — un geste de
  plus pour le cas courant, et un composant de menu neuf à rendre accessible. La 49-3 ajoutera « Titre de section » à
  côté, de la même façon.
- **D2 — L'API ouvre `comment`, et lui seul.** `open_to_api()` rend `true` pour `Priced` et `Comment`, `false` pour
  `Heading` (le `match` reste exhaustif, sans joker). `heading` garde son refus et son message (« Ligne N : le type de
  ligne heading n'est pas encore disponible »). Les tests de la garde **se réécrivent** : `only_priced_is_open_to_api`
  devient `comment_is_open_heading_is_not` ; `unpriced_types_are_refused_without_writing` ne boucle plus que sur
  `heading` (renommé `heading_is_refused_without_writing`, mêmes assertions de zéro écriture), et un test neuf joue
  l'acceptation de `comment` (T3). *Écartée* : supprimer la garde (elle reste la seule barrière de `heading` jusqu'à la
  49-3, `decoupage-49-1.md` § 3).
- **D3 — Le contrat d'une ligne de commentaire à l'API** est celui que la 49-1a a déjà écrit et testé
  (`validate_unpriced_line`) : `{"lineType": "comment", "description": "…"}`, `quantity`, `unitPrice`, `vatRate` **absents ou
  nuls**, `revenueAccountId` absent ou `null` ; tout autre montant ou un compte rend `400` nommant la ligne. La réponse
  porte la ligne avec ses valeurs neutres (`quantity` `0.0000`, `lineTotal` `0.0000`, `revenueAccountId` `null`,
  `lineType` `comment`). Rien de neuf à écrire dans `validate_line`, hors la garde.
- **D4 — `verify_vat_rates_against_db` ne reçoit que les taux des lignes chiffrées.** Un helper unique de
  `routes/invoices.rs`, `fn taux_des_lignes_chiffrees(lines: &[NewInvoiceLine]) -> Vec<Decimal>` (filtre
  `l.line_type.is_priced()`), remplace la collecte aux **deux** sites (`:771`, `:864`) — DRY, et un seul endroit à muter
  pour la preuve du rouge. `vat.rs` **ne change pas** (écart au tableau des fichiers de l'épique, qui le nomme : il ne
  connaît pas les lignes, et ses appelants d'articles n'ont pas de type). Une facture faite de commentaires seuls passe une
  liste vide, déjà acceptée (`vat.rs:85` `    if unique.is_empty() {`, `:86` `        return Ok(());`). *Écartées* : (a) filtrer dans
  `vat.rs` en lui passant les lignes (couple la liste blanche au modèle de facture) ; (b) tolérer le taux `0` dans
  `verify_vat_rates_against_db` (ouvrirait le 0 % désactivé à une ligne **chiffrée**, ce que la liste blanche interdit).
- **D5 — L'écran relit et renvoie le type de chaque ligne, par un seul passage.** Un module neuf
  `frontend/src/lib/features/invoices/invoice-lines.ts` porte **les seules** conversions entre l'API et l'état du
  formulaire : `ligneDepuisReponse(l: InvoiceLineResponse)` (utilisée par `initLines` **et** `reloadFromServer`, qui ne
  recopient plus les champs à la main — l'avertissement de `:783-788` devient sans objet, le mapper étant unique) et
  `ligneAEnvoyer(l)` (une ligne chiffrée : les champs d'aujourd'hui plus `lineType: 'priced'` ; une ligne non chiffrée :
  **`{ lineType, description }` seulement**). **L'état du formulaire a son propre type** : `LigneFormulaire = { lineType:
  InvoiceLineType; description: string; quantity: string; unitPrice: string; vatRate: string; revenueAccountId: number |
  null; _uiKey: string; _vatPlaceholder?: boolean }` remplace `LineState` (`InvoiceForm.svelte:112`, aujourd'hui
  `CreateInvoiceLineRequest & {…}`). Ses champs de montant **restent des `string`** (une ligne de commentaire y porte `''`,
  jamais envoyé) : les `bind:value={line.quantity}` (`:979`), `{line.unitPrice}` (`:985`), `{line.vatRate}` (`:991`) et
  `new Big(l.quantity)` (`:610`) gardent leur type, et **`npm run check` reste vert** sans garde ajoutée dans le gabarit.
  Seul `ligneAEnvoyer` produit un `CreateInvoiceLineRequest`. Une ligne `heading` relue (créée par import ou par SQL, l'API la refusant)
  **garde son type** au renvoi : le serveur la refuse alors en le disant (« … heading n'est pas encore disponible »), au
  lieu qu'elle redevienne chiffrée ou commentaire en silence. Cette limite n'est pas écrite au manuel : un tel brouillon
  ne se crée ni par l'interface ni par l'API ; elle est dite en Dev Notes et la 49-3 la lève. *Écartées* : (a) recopier `lineType` dans les deux
  `map` existants (deux sites à tenir identiques, le défaut même que la 16-1a a payé) ; (b) une union discriminée
  TypeScript complète (`PricedLine | TextLine`) dans tout `InvoiceForm` — juste, mais réécrit les 1158 lignes du composant
  pour un gain que le mapper unique et ses tests donnent déjà.
- **D6 — Les types miroirs, et la règle d'une ligne sans type.** `invoices.types.ts` gagne
  `export type InvoiceLineType = 'priced' | 'comment' | 'heading';` ; `InvoiceLineResponse` et `CreditNoteLineResponse`
  gagnent `lineType?: InvoiceLineType` — **optionnel**, documenté « absent = `priced` » ; dans `CreateInvoiceLineRequest`,
  `lineType: InvoiceLineType` (toujours envoyé par l'écran) et `quantity`, `unitPrice`, `vatRate` **optionnels** (le
  contrat de la 49-1a, `invoices.rs:96-100`). **Règle par défaut** (V1-2) : `estChiffree(l) = l.lineType === undefined ||
  l.lineType === 'priced'`, et `estLigneDeTexte(t) = t === 'comment' || t === 'heading'` ; `ligneDepuisReponse` pose
  `lineType: l.lineType ?? 'priced'`. Une ligne sans type est donc **chiffrée** : c'est le seul cas des données d'avant la
  0.20.0 (la migration de la 49-1a les pose toutes en `priced`) et celui des fixtures de test existantes, qui n'ont pas le
  champ (`git grep -n "lineType" -- frontend/src` vide) — elles restent **inchangées et inertes**, et l'optionnel les garde
  valides pour `npm run check`. L'API, elle, renvoie toujours le champ depuis la 49-1a. **Le filtre du total est dans
  l'assistant** : `computeInvoiceTotal(lines: Pick<CreateInvoiceLineRequest, 'lineType' | 'quantity' | 'unitPrice'>[])`
  (`lineType` facultatif dans ce `Pick`) n'additionne que les lignes `estChiffree` ; `InvoiceForm.svelte:529-533` lui
  passe `lineType` en plus. Les dérivés de `InvoiceForm` (`inactiveRates`, le repli `_vatPlaceholder`, `validateClient`,
  `invalidRevenueAccountLines`) filtrent par le même `estChiffree`. Un commentaire n'exige qu'un texte non blanc (« Ligne
  N : le texte du commentaire est obligatoire », clé i18n). *Écartée* : `lineType` obligatoire dans les types de réponse
  (exact pour l'API, mais impose de retoucher toutes les fixtures existantes et rendrait AC8 (c) invérifiable telle
  qu'écrite).
- **D7 — À l'écran, un commentaire est du texte sur toute la largeur.** Dans le formulaire : un `<textarea>` (plusieurs
  lignes, `\n` gardés, comme le PDF les imprime, 49-1b D5) qui couvre les colonnes description à total (`colspan` 5) ; la
  cellule de suppression reste. Sur la fiche facture et la fiche avoir : une cellule unique `colspan` 6 (toutes les
  colonnes), texte en `white-space: pre-line`, **aucune** cellule de compte (pas de « (défaut société) »), aucun montant.
  Une ligne `heading` s'affiche **comme un commentaire** (même règle que le PDF, 49-1b D1), jusqu'à la 49-3. Un prédicat
  unique `estLigneDeTexte(lineType)` de `invoice-lines.ts` décide pour les trois écrans. *Écartée* : afficher le
  commentaire dans la seule colonne description avec des cellules vides (un « 0.00 » ou un tiret dans les colonnes de
  montant laisserait croire à une ligne à zéro).
- **D8 — La place d'un commentaire : là où on l'ajoute.** Il s'ajoute en fin de tableau, comme les deux autres gestes ;
  « plaçable n'importe où » (#1242) s'obtient en 0.20.0 par l'ordre des ajouts, puis par le réordonnancement de la **49-2**
  (↑ / ↓). La position envoyée est l'ordre du tableau (`update` réinsère les lignes, épique § *Ce qui se lit au code*).
  *Écartée* : un bouton « insérer après cette ligne » (préjuge de l'écran de la 49-2).
- **D9 — #1401 (accents en NFD) : dans le périmètre, par la preuve, sans code.** La description de toute ligne est
  normalisée NFC à la saisie (`invoices.rs:507`) avant d'être écrite : un commentaire collé depuis macOS (`e` + U+0301)
  s'écrit `é` (U+00E9) et s'imprime avec son accent. La story le **prouve** par un test HTTP de bout en bout (T5 : saisie
  NFD → réponse NFC → PDF imprimé avec la lettre accentuée), qui protège désormais ce comportement. Le défaut décrit par
  #1401, borné par son corps aux libellés des lignes et aux commentaires, **ne se reproduit donc pas par la saisie**, au
  `POST` comme au `PUT` (`create_invoice` `:770` et `update_invoice` `:863` passent tous deux par `validate_lines`) ; il ne
  se voit qu'en écrivant au dépôt sans passer par l'API : les tests de la 49-1b, et les **écritures de reprise** (import
  d'installation, `kesh-db/src/backup.rs:2526` `invoices::create`), qui recopient le texte tel qu'il est stocké sans
  normaliser — sans effet sur des lignes nées d'une saisie par l'API, ce que T5 établit. Un test unitaire le protège déjà
  (`invoices.rs:2035` `validate_line_normalizes_description_to_nfc`) ; T5 le prouve de bout en bout. **Décision de Guy du
  2026-11-19** : #1401 se ferme par la PR de la 0.20.0 sur la foi de T5 (mot-clé dans le **corps de la PR**, jamais dans
  un commit) ; le nom et l'adresse du contact sont portés par **#1411** (milestone « Plus tard »). *Écartée* : normaliser aussi dans
  `kesh-qrbill` (un second endroit pour la même règle, et un changement du chemin chiffré, interdit par les contraintes de
  production).
- **D10 — #1400 (caractères que la police n'a pas) : hors du code de cette story, documenté au manuel ; rattachée à la
  49-4 par décision de Guy du 2026-11-19** (`epic-49-types-de-lignes-facture.md:130`). Motifs, qui ont fondé la décision : (a) un remède limité au commentaire (repli dans `texte_de_commentaire`) laisserait la ligne
  chiffrée dans le même défaut — **l'issue se résoudrait par morceaux sur deux releases**, ce que la règle de Guy du
  2026-11-19 interdit (« scinder avant ») ; (b) un remède sur la ligne chiffrée (repli au rendu) **change le rendu de
  factures existantes** dont le PDF n'est pas figé — exactement ce que la 49-4 fait déjà pour les désignations longues
  (#1363), avec sa propre preuve avant / après : c'est la story qui sait faire ce geste ; (c) un refus ou un repli **à la
  saisie** (dans `validate_line`) exigerait que l'API connaisse l'ensemble des caractères imprimables **par chemin**
  (le commentaire replie 125 lettres, la ligne chiffrée non) — une seconde source de vérité de ce que dessine `kesh-qrbill`.
  Ce que la 49-1c fait : le manuel dit, pour le commentaire, ce qu'il garde et ce qu'il perd (AC10 a), **par la règle**
  et non par une liste close ; la 49-4 reprendra la phrase quand elle traitera #1400. *Écartée* : le repli U+2212 → « - » dans le seul chemin du commentaire (sans
  risque pour les factures existantes, mais morcelle l'issue).
- **D11 — Les documents suivent l'ouverture.** `docs/api-external.md:512` et `admin-manual.tex:2559` disent que
  `comment` est **accepté** depuis la 0.20.0 (contrat de D3, valeurs neutres, aucun compte, hors total et TVA) et que
  `heading` reste refusé. Le tableau des fichiers de l'épique ne range pas ces deux documents parmi ceux de la 49-1c :
  **écart déclaré**, inévitable (ils deviendraient faux à la fusion). Le paragraphe de l'administration est dans la
  section que complète la **49-1d** en parallèle : hunk à garder des deux côtés à la fusion (§ *Dev Notes*).
- **D12 — Pas de borne de longueur propre au commentaire ; la capacité est la 49-4, et la 49-1c ne sort pas sans elle.**
  La 49-1b demandait à la 49-1c de « borner ou signaler la longueur d'un commentaire » (`49-1b-types-de-lignes-pdf.md:243`
  et `:474`) ; ce besoin est **#1364** (capacité du PDF contrôlée avant la validation), que **Guy a rattachée à la 49-4 le
  2026-11-19**, avec la règle de sortie **« la 49-1c ne sort pas sans la 49-4 »** (`epic-49-types-de-lignes-facture.md:154` ;
  commentaire de l'orchestrateur sur #1364). La consigne de la 49-1b est donc **renvoyée à la 49-4 par décision de Guy** ;
  la 49-1c n'écrit aucun contrôle de capacité. Elle garde la borne de 1000 caractères du serveur (`MAX_DESCRIPTION_LEN`)
  et le **dit au manuel** : chaque ligne imprimée d'un commentaire prend la place d'une ligne de facture ; une facture en
  tient **huit au plus, moins avec le récapitulatif de TVA** (`invoice_pdf_service.rs:81-87`). *Écartée* : un compteur côté
  écran (il dupliquerait la mesure de largeur de `kesh-qrbill`, que seul le serveur sait faire — DRY). Le risque qu'un
  commentaire long rende une facture validée impossible à imprimer (#1110) est levé par la règle de sortie, non par
  cette story.

## Critères d'acceptation

- **AC1 — L'API accepte `comment`, refuse toujours `heading`.** `POST /api/v1/invoices` avec `[chiffrée A, {lineType:
  "comment", description: "Merci de votre confiance"}, chiffrée B]` rend `201` ; la réponse porte trois lignes, positions
  1 à 3, la deuxième `lineType = "comment"`, `quantity`, `unitPrice`, `vatRate`, `lineTotal` à zéro, `revenueAccountId`
  `null` ; `totalAmount`, `totalTtc` et `vatBreakdown` sont **égaux** à ceux d'une facture `[A, B]` créée à côté. Le `PUT`
  qui renvoie les lignes avec leur type rend `200` et garde le commentaire. Le `GET` relit le type. `heading` reste refusé
  `400 VALIDATION_ERROR` « Ligne 2 : le type de ligne heading n'est pas encore disponible », au `POST` et au `PUT`, sans
  rien écrire (comptage des lignes et `version` inchangés). `open_to_api()` : `Priced` et `Comment` vrais, `Heading` faux.
- **AC2 — Un commentaire ne passe pas la liste blanche des taux.** Dans une société dont le taux 0 % est **désactivé**,
  `POST` et `PUT` d'une facture `[A à 8.10 %, commentaire]` rendent `201` / `200` ; une ligne **chiffrée** à 0 % y reste
  refusée (`400`, `VAT_REJECTED_MSG`) ; une facture de commentaires seuls s'enregistre en brouillon (liste vide).
- **AC3 — La saisie.** Le formulaire de facture offre un bouton « Commentaire » à côté de « Ligne libre » et « Depuis
  catalogue » ; il ajoute en fin de tableau une ligne faite d'un champ de texte multiligne qui couvre la largeur, sans
  quantité, prix, taux, compte ni total ; le **total affiché ne change pas** ; un commentaire vide bloque l'envoi en
  nommant la ligne ; un commentaire n'exige ni quantité, ni prix, ni taux actif, et n'apparaît jamais dans le message des
  taux désactivés ni dans celui des comptes invalides. La charge envoyée pour lui est `{lineType: "comment", description}`.
- **AC4 — L'édition d'un brouillon renvoie le type de chaque ligne.** Un brouillon `[chiffrée, commentaire]` ouvert puis
  enregistré sans changement envoie `lines[0].lineType = "priced"` (avec ses champs d'aujourd'hui) et
  `lines[1] = {lineType: "comment", description}` ; **de même après le rechargement qui suit un conflit de version**. Une
  ligne `heading` relue est renvoyée `heading` (jamais convertie). Les deux relectures passent par `ligneDepuisReponse`
  (`git grep -n "ligneDepuisReponse" -- frontend/src/lib/components/invoices/InvoiceForm.svelte` : deux appels, aucun
  `invoice.lines.map((l) => ({` ni `fresh.lines.map((l) => ({` restant).
- **AC5 — La lecture : facture et avoir.** Sur la fiche d'une facture (brouillon et validée) et sur la fiche de l'avoir qui
  l'annule, un commentaire s'affiche en **une cellule couvrant toutes les colonnes**, sauts de ligne gardés, **sans**
  montant, taux ni « (défaut société) » ; une ligne chiffrée s'affiche exactement comme avant ; une ligne `heading`
  s'affiche comme un commentaire.
- **AC6 — De bout en bout, le commentaire saisi s'imprime et se reprend.** Une facture créée **par l'API** avec un
  commentaire entre deux lignes chiffrées, validée, rend un PDF où le commentaire est dessiné à `x = 20` mm, sans texte aux
  colonnes de montant à sa hauteur (49-1b AC3) ; l'écriture de vente est celle de `[A, B]` ; l'avoir créé par
  `POST /api/v1/credit-notes` porte la ligne `comment` à la même position, et son PDF l'imprime.
- **AC7 — #1401 : un accent saisi en forme décomposée s'imprime.** Un commentaire et un libellé de ligne chiffrée envoyés
  en NFD (`"Cafe\u{0301} offert"`) sont relus en NFC (`"Café offert"`, U+00E9), **au `POST` et au `PUT`**, et le PDF de
  la facture validée porte
  `Café offert` (relecture par `kesh_qrbill::empreinte::textes`).
- **AC8 — Les factures et avoirs existants restent identiques (lignes, totaux, TVA, écritures, numéros, PDF).**
  (a) `git diff 5156f2b7b..HEAD -- crates/kesh-qrbill crates/kesh-core crates/kesh-report crates/kesh-db/migrations
  crates/kesh-db/test-schema crates/kesh-db/src/repositories crates/kesh-api/src/routes/invoice_pdf_service.rs
  crates/kesh-api/src/routes/issued_invoice_pdf.rs crates/kesh-api/src/routes/invoice_pdf.rs
  crates/kesh-api/src/routes/credit_notes.rs crates/kesh-api/src/routes/invoice_email.rs` est **vide** ; dans `kesh-db`,
  seul `src/entities/line_type.rs` change ; (b) les preuves avant / après de la 49-1b
  (`kesh-qrbill/tests/rendu_avant_apres.rs`, `kesh-api/tests/pdf_avant_apres_e2e.rs`) et `invoice_frozen_pdf_e2e.rs` sont
  **vertes sans modification** (`git diff 5156f2b7b..HEAD` vide sur ces fichiers et sur les deux dossiers
  `tests/fixtures/avant-0.20.0/`) ; (c) à l'écran, une facture et un avoir faits de lignes chiffrées s'affichent comme avant :
  les cas existants de `invoice-display-page.test.ts`, `credit-note-display-page.test.ts`, `InvoiceForm.test.ts` et
  `invoice-helpers.test.ts` sont verts **sans modification de leurs fixtures ni de leurs assertions** — leurs lignes, sans
  `lineType`, sont chiffrées par la règle de D6 ; contrôle : `git diff --numstat 5156f2b7b..HEAD -- <ces quatre
  fichiers>` montre des ajouts seulement (suppressions à 0, hors lignes d'`import` nommées au Dev Agent Record) ; (d) un
  brouillon chiffré-seulement ouvert et réenregistré envoie les mêmes champs qu'avant, plus `lineType: "priced"` sur chaque
  ligne (T8, comparaison `toEqual` à la charge attendue), que le serveur traite à l'identique (49-1a AC11).
- **AC9 — Textes.** Toute chaîne **neuve** passe par `i18nMsg`, avec sa clé dans les **quatre** catalogues
  (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`), dans le bloc de l'Epic 49 ; `npm run
  lint-i18n-ownership` vert ; `sitesTotal` de `frontend/src/lib/shared/i18n-keys.test.ts` (3495 à `5156f2b7b`) **recompté
  depuis la source**, l'écart ventilé au Dev Agent Record. Les en-têtes et boutons existants en français codé en dur
  (« Ligne libre », « Depuis catalogue », en-têtes du tableau) **restent** tels quels (même arbitrage qu'AC6-bis de la
  16-1b, cité `InvoiceForm.svelte:960-964`).
- **AC10 — Documents.** (a) `user-manual.tex` : l'item « Lignes de facture » (`:1456`) **nomme pour la première fois**
  les trois boutons — « Ligne libre », « Depuis catalogue », « Commentaire » ; aucun n'est nommé au manuel aujourd'hui —, et
  dit, avec la mention « depuis la version~0.20.0 » pour le commentaire (convention du manuel) : qu'un commentaire est
  hors du total, de la TVA et de l'écriture ; qu'il se place par l'ordre des ajouts ; qu'il s'imprime sur toute la largeur
  et sur plusieurs lignes ; que chaque ligne imprimée prend la place d'une ligne de facture, et qu'une facture en tient
  **huit au plus, moins avec le récapitulatif de TVA** ; qu'une facture faite de commentaires seuls s'enregistre en
  brouillon mais ne se valide pas (total nul). Le paragraphe de `:1594` dit que le commentaire **replie** les 125 lettres
  sur leur base et garde ’ et —, et **la règle** de ce qu'il perd : tout caractère hors du jeu Windows-1252, hors ces 125
  lettres, disparaît du PDF sans avertissement (`pdf.rs:1146-1147`, doc-comment de `texte_de_commentaire`) — avec pour
  **exemples** le signe moins U+2212, les traits d'union U+2010 / U+2011, le grec, le cyrillique, les flèches, ≤ / ≥, les
  émojis — et le conseil d'écrire le moins en trait d'union ordinaire (« Remise −50 » s'imprimerait « Remise 50 »,
  \#1400) ; `:1924` dit que l'avoir reprend aussi les commentaires ; (b) `docs/api-external.md:512` et `admin-manual.tex:2559` : `comment` accepté (D11), `heading`
  refusé ; (c) aucun PDF de manuel commité ; (d) `git grep -n -E "comment.*(réservées|pour l.instant)|refusés pour
  l.instant" -- docs` ne nomme plus `comment`.
- **AC11 — E2E et recette visuelle.** Un spec Playwright joue le parcours (T10) ; les captures de chaque état touché
  (formulaire avec commentaire, fiche facture brouillon et validée, fiche avoir, PDF de la facture saisie à l'écran) sont
  prises **sur une base de démonstration** (`kesh_e2e_e491c` peuplée par le seed de démonstration), thème clair, rangées
  dans `kesh-gate-logs/e491c/captures/`, jointes au Dev Agent Record et au prompt de revue, et **lues**.
- **AC12 — Gates.** § *Tâches*, dernière tâche ; le Dev Agent Record n'affirme que ce qui a tourné.

## Tests à écrire, et la preuve du rouge

Journaux dans `/home/gcorbaz/devel/kesh-gate-logs/e491c/` (`rouge-T<n>.log` : la mutation, la commande, la sortie rouge).
Chaque mutation s'**ajoute au code**, le test tourne, le journal se garde, puis la mutation se retire **et le fichier se
touche** (`touch`) avant la reconstruction (cargo garderait le binaire muté). Côté frontend, la mutation se retire avant le
test suivant (`git diff` vide sur le fichier muté, copié au journal).

| # | Test | Ce qu'il prouve | Preuve du rouge |
|---|---|---|---|
| T1 | `kesh-db/src/entities/line_type.rs::tests::comment_is_open_heading_is_not` (remplace `only_priced_is_open_to_api`) | AC1 : `Priced`, `Comment` ouverts ; `Heading` fermé | mutation : `Self::Comment => false` → **ROUGE** ; mutation : `Self::Heading => true` → **ROUGE** (`rouge-T1.log`) |
| T2 | `routes/invoices.rs::tests` (tests unitaires de `validate_line`, réécriture de celui de `:1913`) | `comment` accepté avec valeurs neutres ; `comment` avec quantité, prix, taux ou compte non nuls refusé `400` nommant la ligne ; `heading` refusé « pas encore disponible » | mutation T1 (garde refermée) → **ROUGE** ; mutation : `validate_unpriced_line` saute le contrôle `revenue_account_id` → **ROUGE** (`rouge-T2.log`) |
| T3 | `kesh-api/tests/invoice_line_types_e2e.rs::comment_is_accepted_at_post_and_put` (neuf) et `::heading_is_refused_without_writing` (réécriture de `unpriced_types_are_refused_without_writing`) | AC1 par le HTTP : 201/200, positions, valeurs neutres, totaux égaux à `[A, B]`, `GET` relit le type ; `heading` 400 sans écriture | mutation T1 → **ROUGE** sur le premier ; mutation `Self::Heading => true` → **ROUGE** sur le second (`rouge-T3.log`) |
| T4 | `invoice_line_types_e2e.rs::comment_ignores_the_vat_rate_whitelist` (neuf) | AC2 : 0 % désactivé (`UPDATE vat_rates SET active = FALSE WHERE company_id = ? AND rate = 0`, assertion qu'**une** ligne a changé) ; `POST`/`PUT` à commentaire acceptés ; ligne chiffrée à 0 % refusée ; facture de commentaires seuls enregistrée | mutation : `taux_des_lignes_chiffrees` rend tous les taux (le filtre retiré) → **ROUGE** au `POST` **et** au `PUT` (deux assertions, un journal) (`rouge-T4.log`) |
| T5 | `invoice_line_types_e2e.rs::nfd_comment_prints_with_its_accent` (neuf, #1401) | AC7 : réponse en NFC (U+00E9, `chars().count()` égal) **au `POST`, puis au `PUT`** (brouillon réenregistré avec un second texte en NFD, relu en NFC) ; facture validée (`POST …/validate`) ; `GET …/pdf` 200 ; `kesh_qrbill::empreinte::textes` contient `Café offert` pour le commentaire **et** pour le libellé chiffré | mutation : `.nfc()` retiré de `invoices.rs:507` → **ROUGE** (`rouge-T5.log`) ; la mutation touche du code de production **le temps du journal seulement** |
| T6 | `invoice_line_types_e2e.rs::comment_typed_at_the_api_reaches_the_pdf_and_the_credit_note` (neuf) | AC6 : PDF de la facture (commentaire à `x = 20`, rien aux colonnes 110 / 130 / 160 / 180 à sa hauteur), écriture égale à celle de `[A, B]` (comptes, montants), avoir par `POST /api/v1/credit-notes` : ligne 2 `comment`, PDF de l'avoir qui l'imprime | mutation : T1 (garde refermée) → **ROUGE** au premier `POST` ; mutation : `ligne_pdf` met `LinePdfKind::Priced` pour `Comment` → **ROUGE** sur le PDF (`rouge-T6.log`) |
| T7 | `frontend/src/lib/features/invoices/invoice-lines.test.ts` (neuf) | D5, D6 : `ligneDepuisReponse` garde `lineType` (trois types) et pose `priced` pour une ligne **sans** `lineType` ; `estChiffree` vrai pour `undefined` et `priced`, faux pour `comment` et `heading` ; `ligneAEnvoyer` : chiffrée = champs d'aujourd'hui + `lineType: 'priced'`, commentaire = `{lineType, description}` **exactement** (`toEqual`), `heading` gardé ; `estLigneDeTexte` ; `computeInvoiceTotal` (cas neufs dans `invoice-helpers.test.ts`) ignore une ligne `comment` qui porterait des montants et compte une ligne sans `lineType` | mutations : `ligneDepuisReponse` omet `lineType` → **ROUGE** ; `ligneAEnvoyer` d'un commentaire garde `quantity` → **ROUGE** ; le filtre `estChiffree` retiré de `computeInvoiceTotal` → **ROUGE** ; `estChiffree` réduit à `l.lineType === 'priced'` → **ROUGE** (cas `undefined`) (`rouge-T7.log`) |
| T8 | `frontend/src/lib/components/invoices/InvoiceForm.test.ts` (nouveau `describe` « Story 49-1c ») | AC3, AC4, AC8 d : bouton « Commentaire » → `textarea`, pas de `invoice-line-vat-rate` ni de compte pour cette ligne, total inchangé ; commentaire vide → message nommant la ligne ; brouillon `[chiffrée, commentaire]` ouvert puis enregistré → charge de l'AC4 ; **même chose après « Recharger »** sur conflit (`OPTIMISTIC_LOCK_CONFLICT`) ; brouillon chiffré-seulement → mêmes champs qu'avant + `lineType: 'priced'` ; brouillon à taux désactivé + commentaire : le message ne nomme que la ligne chiffrée | mutations : `initLines` n'appelle plus `ligneDepuisReponse` (recopie sans `lineType`) → **ROUGE** ; idem pour `reloadFromServer` → **ROUGE** sur le seul cas du conflit ; `validateClient` contrôle la quantité d'un commentaire → **ROUGE** (`rouge-T8.log`) |
| T9 | `invoice-display-page.test.ts` et `credit-note-display-page.test.ts` (cas neufs) | AC5 : la rangée du commentaire a **une** cellule (`colspan` 6), le texte avec son saut de ligne, ni « (défaut société) », ni montant ; une rangée `heading` idem ; la rangée chiffrée inchangée | mutation : le commentaire passe par la rangée chiffrée (prédicat `estLigneDeTexte` rendu `false`) → **ROUGE** sur les deux fichiers (`rouge-T9.log`) |
| T10 | `frontend/tests/e2e/invoice-comment-lines.spec.ts` (neuf) | AC3 à AC6 dans le navigateur : nouvelle facture, ligne libre, « Commentaire » (deux lignes de texte), ligne du catalogue ; enregistrer ; rouvrir (édition) : le commentaire est là, à sa place ; enregistrer de nouveau ; valider ; la fiche montre le commentaire pleine largeur ; `GET …/pdf` 200 ; créer l'avoir ; la fiche de l'avoir montre le commentaire | mutation : T1 (garde refermée), backend reconstruit → **ROUGE** à l'enregistrement (400 affiché) (`rouge-T10.log`, un seul spec lancé : `npx playwright test invoice-comment-lines`) |
| T11 | épreuve, non test neuf : `rendu_avant_apres`, `pdf_avant_apres_e2e`, `invoice_pdf_e2e`, `invoice_frozen_pdf_e2e`, `invoice_line_types` (`kesh-db`), `invoices.spec.ts`, `credit-notes.spec.ts` | AC8 : verts **sans modification** de leurs assertions | — |
| T12 | `frontend/src/lib/shared/i18n-keys.test.ts` (`sitesTotal` recompté) | AC9 | la borne rougit **d'elle-même** avant le recompte (journal de ce rouge copié : c'est sa preuve) |

## Tâches

- [ ] **Étape 0 — vérifier les prérequis et rejouer l'inventaire.** `git merge-base --is-ancestor 57758e1b9 HEAD` (49-1b) ;
  `git grep -n "pub enum LinePdfKind" crates/kesh-qrbill/src` ; rejouer les commandes S1 à S4 et comparer à la fiche (un
  écart se consigne au Dev Agent Record et se traite) ; créer les bases `kesh_e491c` et `kesh_e2e_e491c` sur la voie
  assignée (`kesh-gate-logs/prompts/e35-dev-commun.md`, tableau des voies).
- [ ] **`kesh-db`** (un fichier) : `open_to_api()` (D2), son doc-comment, T1.
- [ ] **`kesh-api`** : `taux_des_lignes_chiffrees` et ses deux appelants (D4) ; doc-comments `invoices.rs:80-86`,
  `:594-597` ; T2 ; T3 à T6 dans `tests/invoice_line_types_e2e.rs` (en-tête du fichier mis à jour, `:8-10`). **Aucune autre
  ligne de production** (AC8 a).
- [ ] **Frontend** : `invoice-lines.ts` (D5, D6, D7) et T7 ; types miroirs (`invoices.types.ts`, `credit-notes.types.ts`) ;
  `invoice-helpers.ts` (total) ; `InvoiceForm.svelte` (bouton, rangée de commentaire, deux relectures par le mapper,
  envoi par `ligneAEnvoyer`, `validateClient`, `inactiveRates`, `_vatPlaceholder`, total) et T8 ; fiche facture et fiche
  avoir (D7) et T9. `ProductPicker.svelte` **ne change pas** (écart au tableau de l'épique : la ligne produit est inchangée).
- [ ] **Fixtures et types** (V1-2) : aucune fixture existante ne se modifie (règle de D6) ; `npm run check` après le
  changement des types, sortie copiée (`kesh-gate-logs/e491c/check-types.log`) ; si un fichier ne compile plus, corriger le
  **type**, non la fixture, et le dire au Dev Agent Record.
- [ ] **i18n** : clés neuves ×4 (bouton, aide du champ, `aria-label` « Commentaire — ligne N », message du commentaire vide,
  et toute autre chaîne neuve), dans le bloc de l'Epic 49 ; `sitesTotal` recompté ; `npm run lint-i18n-ownership`.
- [ ] **E2E** : T10 ; captures (AC11) prises sur `kesh_e2e_e491c` avec le seed de démonstration, puis le PDF de la facture
  rendu en PNG (`mutool draw -r 150`) ; chaque capture **lue**.
- [ ] **Documents** (AC10) : `user-manual.tex` (`:1456`, `:1594`, `:1924`), `admin-manual.tex:2559`,
  `docs/api-external.md:512` ; régénérer le PDF du manuel **pour le relire** (`make fr` dans `docs/manual/`, sous
  `mem-guard`), sans le commiter ; aplatir le texte (`pdftotext user-manual.pdf - | tr '\n' ' ' | tr -s ' '`) et y greper
  les phrases changées.
- [ ] **Propagation** (CLAUDE.md § *Propagation post-patch*) : `git grep -n -E "pas encore disponible|open_to_api" --
  crates docs frontend` après le patch : ne restent que le refus de `heading` et la garde elle-même ; `git grep -n
  "lineType" -- frontend/src` nomme les types, le mapper, `InvoiceForm` et les deux fiches.
- [ ] **Gates** (CLAUDE.md § *Test Locally First* ; `e35-dev-commun.md` § *Quels gates, quand*). **Cette story touche
  `kesh-db`** (`line_type.rs`, ni migration, ni `post_restore.rs`, ni dépôt : l'exception « gate complet en cours de
  boucle » du CLAUDE.md ne s'applique pas, elle vise les migrations, `post_restore.rs` et les dépôts). En cours de boucle,
  gate ciblé : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo nextest run -E 'binary(kesh_db) | binary(kesh_api) | binary(invoice_line_types_e2e) | binary(invoice_pdf_e2e) |
  binary(pdf_avant_apres_e2e) | binary(invoice_frozen_pdf_e2e) | binary(vat_rates_e2e)'`, et côté frontend `npm run
  check`, `npm run lint-i18n-ownership`, `npm run test:unit`. **Au passage en `review`** : bases remises à neuf,
  `scripts/test-fast.sh --ci`, `scripts/gate-ci.sh kesh_ci_e491c`, frontend complet (`check`, `lint-i18n-ownership`,
  `test:unit`, `build`). **À la clôture** (boucle de revue terminée, au dernier commit de code) : `scripts/gate-ci.sh
  kesh_ci_e491c` et **E2E Playwright complet** sur `kesh_e2e_e491c` recréée (montage de `docs/testing.md` § *Prérequis
  Playwright local*, `KESH_COOKIE_SECURE=false`, `KESH_TEST_MODE=true` des deux côtés) ; « vert » = les seuls échecs
  attendus, jugés fichier par fichier (`docs/testing.md` § *Les échecs attendus*). Tout sous `scripts/mem-guard.sh` et
  `flock` du verrou de la voie ; **une commande lourde à la fois** ; `CARGO_BUILD_JOBS=2`.
- [ ] **Contrôle des mots-clés** : `git log origin/epic/49-types-de-lignes-facture..HEAD --format=%B | /usr/bin/grep -niE
  '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` **vide**.

## Note de version

*(Texte pour le `CHANGELOG`, assemblé à l'intégration ; la story n'écrit pas `CHANGELOG.md`.)*

- **Factures — lignes de commentaire** : le formulaire de facture offre, à côté de « Ligne libre » et « Depuis
  catalogue », un bouton **« Commentaire »** qui ajoute une ligne de texte seul, sur une ou plusieurs lignes, sans
  quantité, prix ni TVA. Un commentaire n'entre ni dans le total, ni dans la TVA, ni dans l'écriture comptable ; il se
  retrouve à sa place quand on rouvre le brouillon, s'affiche sur toute la largeur de la fiche de la facture, s'imprime sur
  toute la largeur du PDF et se reprend dans l'avoir qui annule la facture. Les factures et avoirs existants ne changent
  pas. Un commentaire collé avec des accents en forme décomposée garde ses accents à l'impression (`refs #1242`,
  `refs #1401`).
- **API** : une ligne de facture accepte désormais `"lineType": "comment"` (texte seul ; quantité, prix et taux absents ou
  nuls, aucun compte de produit) ; `"heading"` reste refusé (`400 VALIDATION_ERROR`). Le taux de TVA d'une ligne de
  commentaire n'est plus contrôlé contre les taux actifs (`refs #1242`).

## Dev Notes

- **Hors périmètre, avec la story qui le prend** : réordonnancement (↑ / ↓) → **49-2** ; titre de section, ouverture de
  `heading`, son écran → **49-3** ; désignation longue (#1363) et capacité du PDF avant la validation (#1364) → **49-4** ;
  recette de montée et « Passer à la 0.20.0 » → **49-1d** ; sous-totaux → **#1330** ; #1400 → **49-4** (décision de
  Guy, D10) ; nom et adresse du contact en NFD → **#1411** (« Plus tard »).
- **Questions de la fiche, tranchées par Guy le 2026-11-19** (portées au fichier d'épique, `c91a5bac4`) :
  1. **#1400** → la **49-4** (D10) ;
  2. **#1401** → fermée par la PR de la 0.20.0 sur la foi de T5 (D9) ; issue distincte pour le contact : **#1411** ;
  3. **règle de sortie : la 49-1c ne sort pas sans la 49-4** (D12) ; à la coupure, si la 49-4 n'est pas close, la 49-1c
     sort avec elle.
- **Écarts au fichier d'épique, déclarés** : (a) `kesh-db/src/entities/line_type.rs` change (la garde y vit), absent du
  tableau ; (b) `routes/vat.rs` **ne change pas** (D4) ; (c) `ProductPicker.svelte` ne change pas ; (d)
  `docs/api-external.md` et le manuel d'administration changent (D11).
- **Splitting** (CLAUDE.md § *Règle de splitting préventif*) : au grain de la crate, comme le découpage
  (`decoupage-49-1.md` § 1), **cinq modules** — `kesh-db` (un fichier), `kesh-api`, frontend (`components/invoices`,
  `features/invoices`, `features/credit-notes`, routes `invoices` et `credit-notes`, comptés une fois comme au découpage),
  `kesh-i18n`, manuels et `docs/`. **Au plafond, sans le dépasser** ; le découpage en comptait quatre pour la 49-1c, faute
  d'y voir `kesh-db`. La règle se compte à la crate : **cinq**, au plafond, à dire au Project Lead. Aucun recyclage de défauts constatable avant la validation. Pas de section « Dérogation ».
- **Fichiers communs avec les autres stories de la release** :
  - `InvoiceForm.svelte` : la 48-13 est déjà dans la branche (`5156f2b7b`) ; **la 49-2 et la 49-3 partent de la 49-1c**.
    Le mapper `invoice-lines.ts` est le point d'appui de la 49-2 (ordre) et de la 49-3 (`heading`).
  - `docs/manual/fr/admin-manual.tex` : le paragraphe « Pour les intégrations » (`:2559`) est dans la section
    `sec:maj-0-20-0` que la **49-1d** complète en parallèle (worktree `kesh-49-1d`) : hunks distincts attendus, le second
    arrivé garde les deux intentions.
  - Catalogues i18n ×4, `sitesTotal` : toutes les stories (48-14 en parallèle) ; blocs par épique, compteur recompté après
    fusion.
  - `docs/api-external.md` : la 49-3 y écrira l'ouverture de `heading`.
  - PR #1327 de Beat : outre `kesh-qrbill` et `kesh-report`, elle touche les **quatre catalogues i18n** et
    **`docs/manual/fr/user-manual.tex`** (`gh pr view 1327 --json files` ; hunks vers `:4082`, `:4181`, et fr-CH `:2165`),
    distincts de ceux de la 49-1c (`:1456`, `:1594`, `:1924`) : pas de conflit de texte attendu ; le second arrivé garde
    les deux.
- **Ce qui doit rester vrai** : (1) aucune facture ni aucun avoir existants ne change (AC8) ; (2) `heading` reste refusé
  par l'API ; (3) le chemin de rendu du PDF n'est pas touché (`kesh-qrbill` et les services PDF hors du diff) ; (4) un
  brouillon relu et réenregistré ne change jamais le type d'une ligne.
- **Pièges** : (a) `reloadFromServer` n'est atteint qu'après un conflit de version : T8 doit le **jouer** (mock
  `OPTIMISTIC_LOCK_CONFLICT`, puis « Recharger »), sans quoi la mutation du second site survit ; (b) `stripUiKey` est un
  spread : un champ de prix resté sur un commentaire partirait au serveur, qui refuserait une quantité non nulle — d'où
  `ligneAEnvoyer` et son `toEqual` exact ; (c) le taux provisoire `_vatPlaceholder` ne doit jamais se poser sur un
  commentaire (il ferait apparaître un taux dans sa charge) ; (d) un commentaire n'a **aucun** compte : la cellule
  « (défaut société) » serait fausse ; (e) T4 doit **désactiver** le 0 % (le seed le crée actif) et vérifier qu'une ligne
  a changé, sinon il passe à vide ; (f) la base de démonstration des captures n'est jamais celle du NAS.
- **Références** : `epic-49-types-de-lignes-facture.md` (§ 49-1, § Contraintes de production, tableau des fichiers) ;
  `decoupage-49-1.md` (§ 3, § 3 bis) ; fiche 49-1a (D1, D8, D9, l. 98 de l'inventaire) ; fiche 49-1b (D1, D5, D6, D9,
  Dev Notes, « Manuel, E-5 ») ; `docs/testing.md` ; `kesh-gate-logs/prompts/e0200-commun.md`.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Entrée |
|---|---|
| 2026-11-19 | **Création** de la fiche (Opus 5.5, `bmad-create-story`), code lu à `5156f2b7b` (49-1a, 49-1b et 48-13 dans la branche). Périmètre : ouverture de `comment` (garde), `verify_vat_rates_against_db` limité aux lignes chiffrées, écran de saisie et relecture typée du brouillon, affichage dans la facture et l'avoir, i18n ×4, E2E, manuel utilisateur, plus `docs/api-external.md` et le paragraphe « Pour les intégrations » du manuel d'administration (écart déclaré, D11). #1401 dans le périmètre par la preuve (D9) ; #1400 hors du code, documentée, arbitrage proposé (D10) ; #1364 à la 49-4 (D12). Cinq modules au grain de la crate. Validation P1 à lancer : `49-1c-types-de-lignes-ecran-validation-prompt-p1.md`. |
| 2026-11-19 | **Validation P1** (Sonnet 5.5, passe complète, contexte frais ; `kesh-gate-logs/e491c/validation-49-1c-p1.md`) : **0/0/2/6**. **Décisions de Guy du 2026-11-19** sur les trois questions de la fiche (orchestrateur : `c91a5bac4`, fusion `2cce7b905`) : #1400 → 49-4 ; #1401 fermée par la PR de la 0.20.0 sur la foi de T5, contact → #1411 ; la 49-1c ne sort pas sans la 49-4. **Remédiation** (Opus 5.5) : **V1-1** (MEDIUM) levée sur le fond par ces décisions — en-tête, D9, D10, D12 (consigne de la 49-1b `:243`, `:474` renvoyée à la 49-4 par décision de Guy), Dev Notes (questions marquées tranchées, règle de sortie, issue du contact = #1411) ; **V1-2** (MEDIUM) — D6 fixe la règle d'une ligne sans type (`estChiffree` vrai pour `undefined`, `lineType` optionnel dans les types de réponse, fixtures existantes inchangées), D5 donne au formulaire son type d'état aux montants `string` (`npm run check` vert sans garde), le filtre du total est dans `computeInvoiceTotal` ; T7 gagne le cas `undefined` et deux mutations ; AC8 (c) contrôlé par `git diff --numstat` sur quatre fichiers de test ; tâche « Fixtures et types » ; **V1-3** T5 et AC7 jouent aussi le `PUT`, D9 nomme les écritures de reprise (`backup.rs:2526`) et le test unitaire existant (`invoices.rs:2035`) ; **V1-4** AC10 (a) dit la règle de ce qu'un commentaire perd, les quatre caractères deviennent des exemples, conseil du trait d'union ; **V1-5** AC10 (a) : « huit au plus, moins avec le récapitulatif de TVA » (`invoice_pdf_service.rs:81-87`, remplace la citation inexacte `pdf.rs:629`), facture de commentaires seuls non validable, boutons nommés pour la première fois, « depuis la version~0.20.0 » ; **V1-6** PR #1327 : catalogues et `user-manual.tex` communs, hunks distincts ; **V1-7** compte des modules à la crate seulement (cinq), phrase du « six » retirée ; **V1-8** `pdf.rs:668`, `vat.rs:85-86`. Propagation grepée dans la fiche : `pdf.rs:637`, `83-86`, `pdf.rs:629`, `environ huit`, « six » (restent les seuls « six cellules », vrais), « issue à ouvrir », « arbitr ». Aucun reclassement. Passe suivante : P2 complète sur Opus, `49-1c-types-de-lignes-ecran-validation-prompt-p2.md`. |
