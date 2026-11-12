# Story 35-1 : un format de numéro sans compteur est refusé aux réglages, et un numéro impossible à produire ne finit plus en erreur 500 à la validation (#932, #948)

Status: ready-for-dev

*Fiche rédigée le 2026-11-12 par l'agent de rédaction des fiches de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, par le
workflow `bmad-create-story`. Release **0.17.7**. **Priorité basse** dans l'epic (décision de Guy du 2026-11-12 : il garde
le format par défaut `F-{YEAR}-{SEQ:04}`, que le défaut ne touche pas) ; voisine de **#1046** (hors périmètre, D7). **Aucune
migration** : un format déjà enregistré n'est pas réécrit. Changement serveur (un refus typé neuf) et d'écran (le format des
avoirs devient visible). Code lu sur **`main`** (`4d39f377`). Registres : `e35-choix-autonomes-e350.md` (C-e350-8, -18, -42 à
-45) ; `e35-choix-autonomes-e351.md` pour la story, à créer au premier choix. Validation non lancée.*

## Story

En tant qu'administrateur qui règle la numérotation des factures et des avoirs, je veux que Kesh refuse un format qui ne
numérote pas (sans compteur), que je puisse voir et corriger le format des avoirs à l'écran, et qu'une validation de facture
ou d'avoir dont le numéro ne peut pas être produit me dise pourquoi et quoi corriger, au lieu d'une erreur interne.

## Ce qui se lit au code (`main`)

**La grammaire** — `crates/kesh-core/src/invoice_format.rs` (613 lignes) : `validate_template` (`:159-206`) exige un gabarit
non vide, ≤ 64 octets, des caractères `[A-Za-z0-9-_/.#{}: \t]` (`is_allowed_char`, `:89-95`), **au moins un** placeholder
parmi `{YEAR}`, `{FY}`, `{SEQ}`, `{SEQ:NN}` (`:199-201`, `NoPlaceholder`) et un rendu au pire ≤ 64, en comptant `{FY}` pour
**16** caractères (`:180-184`). `F-{YEAR}` passe : tous ses numéros d'un exercice sont identiques (#932). Le format
fournisseur, lui, est strict (`validate_numbering_template`, `:221-238`, Story 33-1a). `render` (`:285-322`) refuse un nom
d'exercice portant une accolade (`:294-298`) et un rendu > 64 (`:318-320`) — or un nom d'exercice va jusqu'à **50**
caractères (`error-fiscal-year-name-too-long`, fr-CH `messages.ftl:1091`), au-delà des 16 comptés.

**La route des réglages** — `crates/kesh-api/src/routes/company_invoice_settings.rs` : `update_invoice_settings` (`:307-…`)
valide **toujours** le format client et le format d'avoir (`:328-331`), ce dernier repris de la base quand le client ne
l'envoie pas (`:317-321`, #216) ; refus `AppError::Validation(e.to_string())`, texte français en dur (limite C3 de la 15-5b,
écrite au doc-comment `:296-302`).

**L'écran** — `frontend/src/routes/(app)/settings/invoicing/+page.svelte` (627 lignes) : le format client
(`format`, `:176`, `:232`, champ `:319-330`, aperçu) et le format fournisseur (`supplierFormat`, `:50`, `:151-155`, `:177`,
`:233`, champ `:338-350`) ; **le format d'avoir n'y est pas**, ni lu ni envoyé. La validation d'écran :
`frontend/src/lib/features/invoices/invoice-number-format.ts` (`validateFormatTemplate`, option `numbering`) et ses tests
`invoice-number-format.test.ts` (`:28-64` le format client, `:66-…` l'option `numbering` ; **`:94-97`** « ne change pas la
règle des formats client et avoir » asserte `validateFormatTemplate('F-{YEAR}').ok === true` — écrit par la 33-1b pour figer
la règle que cette story change). L'écran **bloque tout l'enregistrement** dès que son validateur refuse le format client :
`formatValidation = $derived(validateFormatTemplate(format))` (`:146`), `save()` sort sur `!formatValidation.ok`
(`:209-214`). Aide du champ : `settings-invoicing-format-help` (fr-CH `messages.ftl:685`, « Placeholders : {YEAR}, {FY},
{SEQ}, {SEQ:NN} ») ; message `invoices-format-error-no-placeholder` (`:603`, « au moins un placeholder reconnu »).

**Le 500** — à la validation d'une facture, `crates/kesh-db/src/repositories/invoices.rs:2334-2340` :
`invoice_format::render(...)` → `DbError::Invariant("rendu numéro facture échoué (config invalide ?) …")` ; même chose pour
l'avoir, `crates/kesh-db/src/repositories/credit_notes.rs:572-577`. `Invariant` sort en 500 `INTERNAL_ERROR`
(`crates/kesh-api/src/errors.rs`). Le numéro interne fournisseur (`supplier_invoices.rs:1017`) ne passe pas `{FY}` et son
format est strict : hors du défaut.

**Les docs** — `docs/manual/fr/user-manual.tex:1226-1227` (le pattern et ses placeholders) ; `:1561` (« avec son propre
numéro issu d'une séquence dédiée (par défaut AV-2026-0001, configurable dans *Administration → Facturation*) » — **faux
aujourd'hui** à l'écran, vrai par l'API seule) ; `docs/api-external.md` (`PUT /company/invoice-settings`, route fermée aux
clés : `:969`).

## Décisions

- **D1 — Un format de facture ou d'avoir doit porter un compteur** (C-e350-42) : `validate_template` refuse un gabarit sans
  `{SEQ}` ni `{SEQ:NN}` — variante neuve `FormatError::NoSeq`, message « Le format doit contenir un compteur ({SEQ} ou
  {SEQ:NN}) : sans lui, toutes les factures d'un exercice auraient le même numéro ». `NoPlaceholder` reste pour un gabarit sans
  aucun placeholder (ordre : `NoPlaceholder` avant `NoSeq`). Le miroir d'écran (`validateFormatTemplate`) suit, avec une clé
  i18n neuve. L'aide du champ (`settings-invoicing-format-help`, quatre locales, texte modifié, clé inchangée) dit que le
  compteur est **obligatoire** (validation P1, V1-6). **D1 n'exige pas `{YEAR}` ni `{FY}`** (décision de l'orchestrateur,
  validation P2, C-e350-51) : le compteur repart à chaque exercice (`invoice_number_sequences`, clé `(company_id,
  fiscal_year_id)`) alors que l'unicité vaut pour la société (`uq_invoices_number UNIQUE (company_id, invoice_number)`) — un
  format sans année (`FACT{SEQ}`) heurte donc l'unicité au deuxième exercice ; ce cas est nommé par D5 et suivi par une issue
  (texte rédigé ci-dessous, *Issue à ouvrir*).
- **D2 — Un format n'est jugé que s'il change, au serveur comme à l'écran** (C-e350-43) : la route valide le format client
  et le format d'avoir **quand la requête le porte et qu'il diffère de la valeur enregistrée** ; un format déjà enregistré,
  même sans compteur, ne bloque pas l'enregistrement d'autre chose (le défaut que #932 nommait : « durcir la validation
  bloquerait l'enregistrement sans moyen de corriger »). **Le miroir d'écran suit la même règle** (validation P1, V1-1,
  HIGH) : **une seule valeur dérivée par champ** (validation P2, V2-3), `formatBlocks` (« ce format bloque-t-il
  l'enregistrement ? » : refus de `validateFormatTemplate`, sauf `NoSeq` sur une valeur égale à celle chargée,
  `invoiceNumberFormat` / `creditNoteNumberFormat`), lue aux **trois** sites qui lisent aujourd'hui `formatValidation.ok`
  (`+page.svelte:146`) : `save()` (`:209-214`), le bouton d'enregistrement (`:616-619`, `disabled`), le champ (`:326-332` :
  aperçu, message en `text-destructive`). Un format chargé sans compteur, laissé tel quel : bouton actif, **aperçu affiché**,
  et sous le champ une aide **non bloquante** (la clé de D1, en texte d'avertissement, non destructif). Les autres refus
  d'écran (caractère, accolade, longueur) restent inconditionnels, comme aujourd'hui. Aucune migration ne réécrit un format
  enregistré.
- **D3 — Le format des avoirs devient visible et modifiable** (C-e350-44) : un champ *Format des avoirs* dans la section
  *Numérotation*, avec son aperçu, lu (`creditNoteNumberFormat`) et envoyé ; clés i18n ×4 (`settings-invoicing-credit-note-
  format-label`, `-help`), à côté de `settings-invoicing-supplier-format-*`. La phrase `user-manual.tex:1561` devient vraie.
- **D4 — Un numéro impossible à produire est un refus typé** (C-e350-45) : à la validation d'une facture
  (`invoices.rs:2334-2340`) et à la création d'un avoir (`credit_notes.rs:572-577`), une erreur de `render` devient
  `DbError::NumberFormatUnrenderable { document: Invoice | CreditNote, cause }` (`cause` : rendu trop long, accolade dans le
  nom d'exercice, numéro déjà attribué — D5), mappée comme `DesignatedAccountsNotPostable` (même famille : une configuration
  qui refuse à l'usage), **`400`**, code `NUMBER_FORMAT_UNRENDERABLE`, `details` `{ document, cause }`, message i18n ×4 qui
  nomme le format, l'exercice et la cause, et renvoie à *Administration → Facturation* (ou au nom de l'exercice). Rien n'est
  écrit : la transaction est annulée, aucun numéro consommé (`next_number_for` est dans la même transaction :
  `invoice_number_sequences.rs:30-55`, appelé à `invoices.rs:2333`). La variante neuve prend aussi son bras dans
  `DbError::error_code` (`crates/kesh-db/src/errors.rs:1601-1625`). **Changement de contrat** pour un client d'API : la
  validation d'une facture (`POST /invoices/{id}/validate`) et la création d'un avoir (`POST /credit-notes`) sont ouvertes aux
  clés `read-write` (`docs/api-external.md:497`, `:600`) ; **deux passages** (validation P2, V2-2) : un rendu impossible
  rendait `500 INTERNAL_ERROR`, un numéro déjà attribué `409 RESOURCE_CONFLICT` (D5) ; les deux rendent désormais
  `400 NUMBER_FORMAT_UNRENDERABLE` — un client qui retentait sur 5xx, ou relisait et retentait sur `409`, cesse de le faire
  (AC7). **Autres erreurs de `render`** (V2-6) : une erreur de `parse` d'un gabarit enregistré hors de la route (SQL direct,
  version antérieure) prend la cause `invalid_format` ; `InvalidSeq` (`invoice_format.rs:290-292`) reste `Invariant` (500) :
  le `CHECK next_number >= 1` de la base l'interdit.
- **D5 — Un format sans compteur déjà enregistré : tranché** (validation P1, V1-5). Lecture du code : la deuxième
  validation de l'exercice sous `F-{YEAR}` heurte `uq_invoices_number`
  (`migrations/20260417000001_invoice_validation.sql:57`) à l'`UPDATE invoices SET status = 'validated', invoice_number = ?`
  (`invoices.rs:2418-2430`, `map_err(map_db_error)`), qui rend `DbError::UniqueConstraintViolation` (message MariaDB) ;
  `errors.rs:3673` le mappe sur le générique `409 RESOURCE_CONFLICT` (aucune occurrence de `uq_invoices_number` ni de
  `uq_credit_notes_number` dans `crates/**/*.rs`) ; la transaction est annulée, et **toute** validation suivante de
  l'exercice échoue de même, sans cause nommée. **Retenu** : à ce site (et à l'`INSERT` de l'avoir dans `credit_notes.rs`,
  pour `uq_credit_notes_number`), une `UniqueConstraintViolation` qui nomme la contrainte devient
  `NumberFormatUnrenderable { cause: Duplicate }` — le format et le nom d'exercice sont à la main. La collision n'est **pas**
  propre aux formats sans compteur (validation P2, V2-1) : `FACT{SEQ}` la heurte au deuxième exercice, `F-{YEAR}-{SEQ:04}`
  quand deux exercices commencent la même année civile. Le message ne suppose donc pas la cause : « ce format produit un
  numéro déjà attribué : il doit distinguer chaque facture — un compteur, et l'exercice ({FY}) ou l'année ({YEAR}) si le
  compteur repart à chaque exercice ». La correction est possible à l'écran grâce à D2 et D3.
- **D6 — `{FY}` compté à sa vraie borne ?** Non : porter `{FY}` à 50 dans le pire cas refuserait des formats valides
  aujourd'hui (`{FY}/{SEQ:06}`) ; le refus de D4 couvre l'exercice au nom trop long, à l'usage.
- **D7 — Hors périmètre** : **#1046** (jeu de caractères de la QR-facture). Constat pour son commentaire : `is_allowed_char`
  (`:89-95`) ne laisse passer que de l'ASCII, si bien qu'un « – » ne peut entrer que par le **nom d'exercice** (`{FY}`) ;
  à poster par l'orchestrateur, non traité ici.

## Critères d'acceptation

1. **AC1 — La grammaire** (`invoice_format.rs`, tests unitaires) : `F-{YEAR}` → `NoSeq` ; `{YEAR}` → `NoSeq` ; `FACTURE` →
   `NoPlaceholder` ; `F-{YEAR}-{SEQ:04}`, `FACT{SEQ}`, `{FY}/{SEQ:06}` acceptés ; `validate_numbering_template` inchangée
   (ses tests verts).
2. **AC2 — La route** (`crates/kesh-api/tests/`, test d'API sous session Administrateur) : envoyer `F-{YEAR}` en format client
   → 400 `VALIDATION_ERROR`, rien d'enregistré ; envoyer `AV-{YEAR}` en format d'avoir → 400 ; une base où le format d'avoir
   enregistré est `AV-{YEAR}` (montage SQL direct), un `PUT` qui ne l'envoie pas, ou le renvoie à l'identique, **aboutit** ;
   le même `PUT` qui le change en `AV-{YEAR}-{SEQ}` aboutit. **De même pour le format client**, que l'écran envoie toujours :
   un format client enregistré `F-{YEAR}`, renvoyé à l'identique avec un autre réglage modifié, aboutit.
3. **AC3 — Le 500 devenu refus** (test d'API) : un exercice nommé de **50** caractères et le format `FACT-{FY}-{SEQ:10}`
   enregistré par SQL direct — il passe `validate_template` (pire cas 5 + 16 + 1 + 19 = 41 ≤ 64) et son rendu dépasse
   (5 + 50 + 1 + 10 = **66** > 64) ; `POST /invoices/{id}/validate` → `400`, code `NUMBER_FORMAT_UNRENDERABLE`, `details`
   `{ document: "invoice", cause: … }` ; la facture reste brouillon, sans numéro ; le compteur n'a pas avancé. Même chose pour
   un avoir (`POST /credit-notes`). Un nom d'exercice à accolade : l'étape 0 constate s'il peut être enregistré
   (`routes/fiscal_years.rs` ne contrôle pas l'accolade, à vérifier à l'exécution) ; s'il le peut, un test d'API, sinon un
   test unitaire de la conversion.
4. **AC4 — D5** (test d'API, inconditionnel) : format client `F-{YEAR}` enregistré par SQL direct ; deux factures validées
   dans le même exercice → la première aboutit (`F-2026`), la seconde rend `400 NUMBER_FORMAT_UNRENDERABLE`, cause
   `duplicate`, et reste brouillon ; un avoir sous `AV-{YEAR}`, de même ; **`FACT{SEQ}`** (accepté par D1) sur deux exercices
   → la première facture du deuxième exercice rend `400`, cause `duplicate`. Rouge au départ : `409 RESOURCE_CONFLICT`.
5. **AC5 — L'écran** (Vitest, `settings-invoicing-page.test.ts`, mock local d'`i18nMsg`, sans état i18n global) : le champ
   *Format des avoirs* affiche la valeur lue et son aperçu, l'envoie à l'enregistrement ; un format client ou d'avoir
   **changé** en `F-{YEAR}` → refus d'écran (clé de D1), aucun appel ; un format client **chargé** `F-{YEAR}`, laissé tel
   quel, un autre réglage modifié → bouton d'enregistrement **actif**, **cliqué** (non `fireEvent.submit`), `PUT` envoyé,
   aperçu présent, aucun message destructif, l'aide non bloquante visible (D2) ; de même pour un format d'avoir chargé sans
   compteur. Dans
   `invoice-number-format.test.ts`, le test **`:94-97`** (« ne change pas la règle des formats client et avoir ») est un
   **rouge attendu** : réécrit en `F-{YEAR}` refusé (`NoSeq`), `{FY}{SEQ}` accepté — le Change Log le dit.
6. **AC6 — Quatre langues et gardes** : les clés de D1, D3 et D4, et le texte modifié de `settings-invoicing-format-help`
   (D1), dans `fr-CH`, `de-CH`, `it-CH`, `en-CH`, sans sélecteur,
   replis littéraux égaux au fr-CH ; `textes_coherents.rs` (G9) — la clé de D4 côté `errors.rs`, à la règle d'union ;
   `i18n-keys.test.ts` (`sitesTotal`, 3026 sur `main`) recompté à la source (C-e350-18).
7. **AC7 — Docs** : `user-manual.tex:1226-1227` (le compteur obligatoire ; un format enregistré sans compteur n'est pas
   réécrit, et sa correction) ; `:1561` (vrai désormais : le champ existe) ; une phrase sur le refus de D4 ; PDF régénéré,
   contrôlé aplati (sondes positives : « Format des avoirs », le code ; négative : aucune).
   `docs/api-external.md` : (a) une ligne au tableau des erreurs (`:1040-1052`, à la règle d'union) :
   `400 NUMBER_FORMAT_UNRENDERABLE`, ses `details` `{ document, cause }` et ses causes *(depuis la v0.17.7 ; jusqu'à la
   v0.17.6 : `500`, ou `409 RESOURCE_CONFLICT` pour un numéro déjà attribué)* ; (b) les
   descriptions de `POST /invoices/{id}/validate` et de `POST /credit-notes` renvoient à ce code ; (c) le refus `NoSeq` de
   `PUT /company/invoice-settings` n'y est pas décrit pour les clés : route d'administration fermée aux clés (`:79`).
   CHANGELOG `## [0.17.7] — Non publié`, *Corrigé*, deux points `refs #932`, `refs #948`, dont l'un dit les **deux
   passages** — `500` et `409 RESOURCE_CONFLICT` → `400 NUMBER_FORMAT_UNRENDERABLE` — pour un client d'API (validation d'une
   facture, création d'un avoir).
8. **AC8 — Captures** (thème clair, base jetable, `kesh-gate-logs/e351/captures/`) : la section *Numérotation* avec le champ
   des avoirs ; le refus d'écran de `F-{YEAR}` ; le refus de D4 sur la fiche d'une facture.

## Tests à écrire, et la preuve du rouge

Écrits d'abord (journal `kesh-gate-logs/e351/rouges.log`) : AC1 (`F-{YEAR}` accepté), AC2 (400 absent), AC3 (500), AC4
(`409 RESOURCE_CONFLICT`), AC5 (champ absent, `F-{YEAR}` accepté ; le test `:94-97` rouge attendu, réécrit) rouges sur
`main`. Le cas d'AC5 « format chargé sans compteur, inchangé » est vert au départ (aucun `NoSeq`), dit comme tel : sa preuve
est M7. Le cas « valeur enregistrée inchangée » d'AC2 est vert au départ, dit
comme tel. Pour AC5, le test d'écran cherche le champ par son libellé : rouge par absence, comportement prouvé ensuite par
l'envoi.

**Mutations** : M1 `NoSeq` retiré de `validate_template` (AC1, AC2) ; M2 la route juge le format même inchangé (AC2, le cas
de la base) ; M3 la conversion de D4 rendue à `Invariant` (AC3) ; M4 l'avance du compteur validée dans une transaction
à part, avant le rendu (AC3, compteur — la transaction unique d'aujourd'hui l'annule, M4 vise à la garder) ; M5 le champ des
avoirs lu mais non envoyé (AC5) ; M6 le miroir d'écran sans `NoSeq` (AC5) ; **M7 le miroir juge même le format inchangé**
(AC5, le format chargé sans compteur — tuée par le bouton désactivé comme par l'envoi absent) ; M8 la conversion
`duplicate` retirée (AC4, `409` revient).

## Tâches

- [ ] Étape 0 : relire les lignes citées ; relire `next_number_for` et la transaction de validation ; constater si un nom
      d'exercice à accolade s'enregistre (AC3).
- [ ] Tests, rouges consignés ; `invoice_format.rs` ; route — avec ses textes rendus faux par D3 (V2-5 :
      `company_invoice_settings.rs:105` et `:314-316`, « cas du formulaire frontend actuel » ; `idor_multi_tenant_e2e.rs:747`,
      `:773`, « comme le fait le formulaire frontend » → « un client qui ne l'envoie pas » ; #216 reste testé) ; `DbError`,
      `errors.rs` ; écran et miroir ; clés ×4, gardes i18n ; manuel, `api-external.md`, CHANGELOG ; captures.
- [ ] Gates : backend complet (`kesh-db` touché) ; frontend complet ; E2E complet au dernier commit de code
      (`settings-invoicing*`, `invoices*` en particulier).

## Dev Notes

### Issue à ouvrir (C-e350-51, « on trace toujours »)

- **Titre** : « Numérotation : un format sans année ni exercice heurte l'unicité au deuxième exercice »
- **Corps** : « Le compteur de numérotation des factures et des avoirs repart à 1 à chaque exercice
  (`invoice_number_sequences`, `credit_note_number_sequences`, clé `(company_id, fiscal_year_id)`), alors que l'unicité du
  numéro vaut pour la société, tous exercices confondus (`uq_invoices_number`, `uq_credit_notes_number`). Un format accepté
  aux réglages sans `{YEAR}` ni `{FY}` (`FACT{SEQ}`) produit donc au deuxième exercice un numéro déjà attribué ; de même
  `F-{YEAR}-{SEQ:04}` quand deux exercices commencent la même année civile. Depuis la 0.17.7 (story 35-1, #932, #948), la
  validation le refuse en `400 NUMBER_FORMAT_UNRENDERABLE`, cause `duplicate`, avec un message qui dit quoi corriger ; mais
  les réglages n'empêchent pas d'enregistrer un tel format. Piste : exiger `{YEAR}` ou `{FY}` aux réglages (décision de
  périmètre écartée de la 35-1), ou un compteur par société pour un format sans année. Priorité proposée : P3 (le format par
  défaut `F-{YEAR}-{SEQ:04}` n'est touché que par deux exercices commençant la même année). »

### Ordre et fichiers communs — règle d'union C-e350-8

- **Avec la 35-3, deux fichiers de plus** (validation P2, V2-4) : `crates/kesh-api/src/routes/company_invoice_settings.rs` (la
  35-3 `:345-414`, validation des comptes ; ici `:317-331`, D2 : hunks distincts) ;
  `frontend/src/routes/(app)/invoices/[id]/+page.svelte:449-456` — la liste des codes **non rejouables** qui ferment le
  dialogue de validation : `NUMBER_FORMAT_UNRENDERABLE` y entre (l'administrateur doit corriger le format ou l'exercice
  avant de réessayer) ; la 35-3 édite la même liste : **même hunk**, la seconde fusionnée ajoute son code à la liste de la
  première ; AC8 capture le refus, dialogue fermé.

- **Après la 35-3** (`errors.rs` : bras `DbError` juste après `DesignatedAccountsNotPostable` ; ici le nôtre juste après **le
  bras** de la 35-3 — la 35-3 place une dizaine de clés à `errors.rs` et G9, la clé de D4 suit son bloc, non une clé
  isolée ; `messages.ftl` ×4 ; G9), la **35-2b** (`errors.rs` : deux fonctions de message extraites à côté des bras
  `:2144-2168`, C-e350-46 ; `api-external.md`, `user-manual.tex`, `CHANGELOG.md` — hunks distincts ; la fiche 35-2b nomme déjà
  la 35-1) et la **35-2a** (G9 : table de `textes_coherents.rs`, sa longueur recomptée à l'union ;
  `api-external.md`) — l'epic place la 35-1 en dernier ; la règle de l'epic (§ *Ordre et dépendances*, point 4) la met
  après la 35-3 dès qu'elle touche `invoices.rs:2334-2340`, ce que fait D4.
- `crates/kesh-db/src/repositories/invoices.rs` avec la **35-5b** (`35-5b-date-du-jour-reste.md`, § *Fichiers communs* :
  « avec la 35-1 si son correctif touche `:2334-2340` (hunks distincts) ») et la **35-8b** (traces d'audit de `invoices.rs`,
  hunks distincts) ; `credit_notes.rs` avec la 35-8b.
- `messages.ftl` ×4 : clés de D1 et D3 à côté de `settings-invoicing-supplier-format-*` (fr-CH `:686-687`) ; clé de D4 juste
  après celle de la 35-3 (`error-designated-account-…`). `user-manual.tex` : `:1226-1227`, `:1561` (paragraphes que la
  35-2a, `:1251`, ne touche pas). `i18n-keys.test.ts`, `CHANGELOG.md`.

### Hors périmètre

- #1046 (D7). La renumérotation de factures déjà validées : jamais.

### Ce qui doit rester vrai

- Un numéro attribué ne change jamais ; aucun numéro n'est réattribué ; un refus n'écrit rien. Fixtures synthétiques
  seulement.

### Références

- Issues #932, #948 (et #926, C-e331-16, C-e331-17 de la 33-1a), #1046, #216 ; Story 5.2, Story 33-1a.
- `CLAUDE.md` : § *Recette visuelle* ; § *Le prompt d'une passe doit NOMMER le manuel*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

- Fiche rédigée par le workflow `bmad-create-story` (analyse du code de `main`, `4d39f377`).

### File List

## Change Log

- 2026-11-12 — Fiche rédigée (Epic 35). Validation non lancée.
  **Écart à la consigne Edit** : deux références de ligne (`company_invoice_settings.rs:307`, `:317-321`) corrigées dans la
  fiche neuve par un court script Python qui contrôlait l'unicité de chaque chaîne avant de la remplacer, au lieu de
  l'outil Edit. Rien d'autre.
- 2026-11-13 — **Validation P1** (Sonnet) : 0 CRITICAL, **1 HIGH**, 4 MEDIUM, 3 LOW (`kesh-gate-logs/e35-1-validation-p1.md`).
  Remédiée par l'agent de rédaction des fiches (Opus 5.5). Prémisses relues sur `main` : `+page.svelte:146` (le validateur
  dérivé) et `:209-214` (`save()` bloqué ; le rapport citait `:176-178`, `:216-221`, décalés) ; `invoice-number-format.test.ts:94-97` ;
  `api-external.md:79`, `:497`, `:600`, `:1048` ; `messages.ftl:603`, `:685` ; `invoices.rs:2333`, `:2418-2430` ;
  `kesh-api/src/errors.rs:3673` (`UniqueConstraintViolation` → `409 RESOURCE_CONFLICT`, aucune contrainte `uq_invoices_number`
  ni `uq_credit_notes_number` nommée) ; `kesh-db/src/errors.rs:1601`. Toutes confirmées.
  - V1-1 (HIGH) : D2 étendue au miroir d'écran (`NoSeq` ne bloque que si le champ a changé ; aide non bloquante) ; AC5 et M7.
  - V1-2 (MEDIUM) : AC3, `FACT-{FY}-{SEQ:10}` (pire cas 41, rendu 66 > 64) ; l'exemple `{FY}-{SEQ:10}` rendait 61.
  - V1-3 (MEDIUM) : le test `invoice-number-format.test.ts:94-97`, rouge attendu, réécrit (AC5, rouges).
  - V1-4 (MEDIUM) : D4 et AC7 — `400`, `details`, changement de contrat 500 → 400 pour les clés, (a) à (d) d'`api-external.md`,
    CHANGELOG.
  - V1-5 (MEDIUM) : D5 **tranchée** par la lecture du code (C-e350-47) : site nommé (`invoices.rs:2418-2430`, `INSERT` de
    l'avoir), AC4 inconditionnel, M8.
  - LOW : V1-6 (aide du champ ×4, compteur obligatoire) ; V1-7 (union : 35-2b, bras de la 35-3) ; V1-8 (accolade dans le nom
    d'exercice constatée à l'étape 0, AC3).
  - Symptômes greppés (fiche, fichier d'epic, registre) : `{FY}-{SEQ:10}`, « constater D5 », « si mappé », « même statut »
    (statut désormais écrit, `400`) — remplacés ; registre : C-e350-43 et C-e350-45 annotés ; fichier d'epic : ligne 35-1.
- 2026-11-13 — **Validation P2** (Opus) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 3 LOW (`kesh-gate-logs/e35-1-validation-p2.md`) ; les
  trois MEDIUM naissent de la remédiation P1. Remédiée au plus court. Prémisses relues sur `main` : la clé
  `(company_id, fiscal_year_id)` des compteurs et l'unicité par société (`20260417000001_invoice_validation.sql:32`, `:57` ;
  `20260627000001_credit_notes.sql:55`, `:97`) ; `+page.svelte:146`, `:209-214`, `:326-332`, `:616-619` ;
  `invoices/[id]/+page.svelte:449-456` ; `company_invoice_settings.rs:105`, `:314-316` ; `idor_multi_tenant_e2e.rs:747`.
  Confirmées.
  - V2-1 (MEDIUM) : **décision de l'orchestrateur** — D1 n'exige ni `{YEAR}` ni `{FY}` (C-e350-51) ; message de la cause
    `duplicate` qui ne suppose pas l'absence de compteur ; AC4 gagne `FACT{SEQ}` sur deux exercices ; issue rédigée
    (*Issue à ouvrir*).
  - V2-2 (MEDIUM) : D4, AC7 (`api-external.md`, CHANGELOG) nomment les deux passages, `500` et `409 RESOURCE_CONFLICT` →
    `400`.
  - V2-3 (MEDIUM) : une valeur dérivée unique, `formatBlocks`, lue aux trois sites ; AC5 : bouton actif et cliqué, aperçu
    présent ; M7 tuée par le bouton comme par l'envoi.
  - LOW : V2-4 (union avec la 35-3 : `company_invoice_settings.rs`, et la liste des codes non rejouables
    d'`invoices/[id]/+page.svelte:449-456`, même hunk, `NUMBER_FORMAT_UNRENDERABLE` y entre) ; V2-5 (textes rendus faux par
    D3, inventaire à la tâche « route ») ; V2-6 (`parse` → cause `invalid_format`, `InvalidSeq` reste 500).
  - **Numérotation du registre** : C-e350-48 à -50 avaient été pris en parallèle par la remédiation de la 35-8a dans le même
    worktree ; le choix de cette passe est **C-e350-51**.
  - Symptômes greppés : « ajoutez-y un compteur », « passage 500 → 400 », `save()` seul — remplacés ; registre : C-e350-47
    annoté.
