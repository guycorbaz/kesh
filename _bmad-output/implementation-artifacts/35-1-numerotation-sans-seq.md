# Story 35-1 : un format de numéro sans compteur est refusé aux réglages, et un numéro impossible à produire ne finit plus en erreur 500 à la validation (#932, #948)

Status: done

*Fiche rédigée le 2026-11-12 par l'agent de rédaction des fiches de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, par le
workflow `bmad-create-story`. Release **0.17.7**. **Priorité basse** dans l'epic (décision de Guy du 2026-11-12 : il garde
le format par défaut `F-{YEAR}-{SEQ:04}`, que le défaut ne touche pas) ; voisine de **#1046** (hors périmètre, D7). **Aucune
migration** : un format déjà enregistré n'est pas réécrit. Changement serveur (un refus typé neuf) et d'écran (le format des
avoirs devient visible). Code lu sur **`main`** (`4d39f377`). Registres : `e35-choix-autonomes-e350.md` (C-e350-8, -18, -42 à
-45) ; `e35-choix-autonomes-e351.md` pour la story, à créer au premier choix. Validation non lancée.*

*(Annoté le 2026-11-13 à l'étape 0 du développement : « Validation non lancée » est périmé — la validation est terminée en P4,
cf. Change Log. Les lignes citées ci-dessous valent pour `main` ; leurs écarts sur la branche sont au Dev Agent Record.)*

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
  format sans année (`FACT{SEQ}`) heurte donc l'unicité au deuxième exercice ; ce cas est nommé par D5 et suivi par
  **#1058** (texte ci-dessous, *Issue ouverte*).
- **D2 — Un format n'est jugé que s'il change, au serveur comme à l'écran** (C-e350-43) : la route valide le format client
  et le format d'avoir **quand la requête le porte et qu'il diffère de la valeur enregistrée** ; un format déjà enregistré,
  même sans compteur, ne bloque pas l'enregistrement d'autre chose (le défaut que #932 nommait : « durcir la validation
  bloquerait l'enregistrement sans moyen de corriger »). **Le miroir d'écran suit la même règle** (validation P1, V1-1,
  HIGH) : **une seule valeur dérivée par champ** (validation P2, V2-3), `formatBlocks` (« ce format bloque-t-il
  l'enregistrement ? » : refus de `validateFormatTemplate`, sauf `NoSeq` sur une valeur égale à celle chargée,
  `invoiceNumberFormat` / `creditNoteNumberFormat`), lue aux **quatre** sites qui lisent aujourd'hui `formatValidation.ok`
  (`+page.svelte:146`) : le dérivé de l'aperçu `formatPreview` (`:147-148`), `save()` (`:209-214`), le bouton
  d'enregistrement (`:616-619`, `disabled`), le champ (`:326-332` : aperçu, message en `text-destructive`). Pour reconnaître
  `NoSeq` sans comparer de textes traduits, `FormatValidationResult` (`invoice-number-format.ts:17-20`, aujourd'hui
  `{ ok, error? }`) gagne `code?: 'NO_SEQ'` (validation P3, V3-5 ; précisé en P4, V4-1) : la branche `NoSeq` du miroir,
  neuve (elle n'existe pas sur `main`), s'écrit dans `validateTemplateRules`, après la boucle des placeholders,
  `NoPlaceholder` testé d'abord (ordre du Rust) ; elle seule pose `code: 'NO_SEQ'` ; `formatBlocks` ne teste que
  `code === 'NO_SEQ'`. Un format chargé sans compteur, laissé tel quel : bouton actif, **aperçu affiché**,
  et sous le champ une aide **non bloquante** (la clé de D1, en texte d'avertissement, non destructif). Les autres refus
  d'écran (caractère, accolade, longueur) restent inconditionnels, comme aujourd'hui. Aucune migration ne réécrit un format
  enregistré.
- **D3 — Le format des avoirs devient visible et modifiable** (C-e350-44) : un champ *Format des avoirs* dans la section
  *Numérotation*, avec son aperçu, lu (`creditNoteNumberFormat`) et envoyé ; clés i18n ×4 (`settings-invoicing-credit-note-
  format-label`, `-help`), à côté de `settings-invoicing-supplier-format-*`. La phrase `user-manual.tex:1561` devient vraie.
- **D4 — Un numéro impossible à produire est un refus typé** (C-e350-45) : à la validation d'une facture
  (`invoices.rs:2334-2340`) et à la création d'un avoir (`credit_notes.rs:572-577`), une erreur de `render` devient
  `DbError::NumberFormatUnrenderable { document: Invoice | CreditNote, cause }` — **quatre causes**, écrites ici seulement
  (validation P3, V3-3) : `too_long` (rendu > 64), `brace_in_fiscal_year_name`, `duplicate` (D5), `invalid_format` (ci-dessous)
  —, mappée comme `DesignatedAccountsNotPostable` (même famille : une configuration
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
  numéro déjà attribué : il doit distinguer chaque facture — un compteur, qui repart à chaque exercice, et l'exercice ({FY}) ;
  l'année ({YEAR}) ne suffit pas si deux exercices commencent la même année » (validation P3, V3-4 : le compteur repart
  toujours, `uq_ins_company_fy`). La correction est possible à l'écran grâce à D2 et D3.
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
   `400 NUMBER_FORMAT_UNRENDERABLE`, ses `details` `{ document, cause }` et ses quatre causes (D4) *(depuis la v0.17.7 ; jusqu'à la
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

- [x] Étape 0 : relire les lignes citées ; relire `next_number_for` et la transaction de validation ; constater si un nom
      d'exercice à accolade s'enregistre (AC3).
- [x] Tests, rouges consignés ; `invoice_format.rs` ; route — avec ses textes rendus faux par D3 (V2-5 :
      `company_invoice_settings.rs:105` et `:314-316`, « cas du formulaire frontend actuel » ; `idor_multi_tenant_e2e.rs:747`,
      `:773`, « comme le fait le formulaire frontend » → « un client qui ne l'envoie pas » ; #216 reste testé) ; `DbError`,
      `errors.rs` ; écran et miroir ; clés ×4, gardes i18n ; manuel, `api-external.md`, CHANGELOG ; captures.
- [x] Gates du passage en `review` et de la remédiation : backend complet (`kesh-db` touché) ; frontend complet.
- [x] E2E complet au dernier commit de code (`settings-invoicing*`, `invoices*` en particulier) — **à la clôture**, après la
      revue (consigne de l'Epic 35, `e35-dev-commun.md`). *(Case scindée le 2026-11-13, revue P1, A7 : elle était cochée avec
      l'E2E, qui n'a pas tourné.)*

## Dev Notes

### Issue ouverte : #1058 (C-e350-51, « on trace toujours »)

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

Claude Opus 5.5 (agent de développement de l'Epic 35, clé `e351`), worktree `/home/gcorbaz/devel/kesh-e351`, branche
`story/35-1-numerotation-sans-seq`, partie de `569d7bd41` (35-2b terminée + lignée 35-5a0, qui porte la 35-3 et la 35-2a).

### Debug Log References

Journaux hors dépôt, `/home/gcorbaz/devel/kesh-gate-logs/e351/` : `rouges.log` (synthèse), `rouges-api.log`, `rouges-core.log`,
`rouges-vitest.log` ; `cible-1.log` ; `verts-vitest.log`, `i18n-keys.log` ; `mut/` (`campagne-rust.log`, `campagne-m4b.log`,
`M1.log` … `M8.log`, `M4b.log`, `M5.log`, `M6.log`, `M7.log`, `muter-e351.py`, scripts) ; `test-fast-ci.log`, `build-workspace.log`,
`gate-ci.log` (+ `gate-ci.journal`), `front-check.log`, `front-lint-i18n-ownership.log`, `front-test:unit.log`, `front-build.log` ;
`make-user-fusion.log`, `make-user.log`, `user-manual-plat-fusion.txt`, `user-manual-plat.txt` ; `captures.log`, `captures/`.

### Completion Notes List

- Fiche rédigée par le workflow `bmad-create-story` (analyse du code de `main`, `4d39f377`).
- **Étape 0** (lignes relues sur la branche, base `569d7bd41`) — écarts avec la fiche, lue sur `main` :
  - `invoice_format.rs`, `credit_notes.rs`, `settings/invoicing/+page.svelte`, `invoice-number-format.ts` : **inchangés** entre
    `4d39f3774` et `569d7bd41` (`git diff --stat` vide) ; les lignes citées valent.
  - `company_invoice_settings.rs` (route) : décalée par la 35-3 — `update_invoice_settings` en `:322` (fiche `:307`), le format
    d'avoir repris de la base en `:336` (`.unwrap_or(current.credit_note_number_format);`), sa validation en `:345`
    (`invoice_format::validate_template(&credit_note_number_format)`).
  - `invoices.rs` : le rendu en `:2339-2344` (`"rendu numéro facture échoué (config invalide ?) : {e}"` à `:2342` ; fiche
    `:2334-2340`), l'`UPDATE` en `:2424` (`"UPDATE invoices SET status = 'validated', invoice_number = ?, \"` ; fiche `:2418`).
    `next_number_for` (`invoice_number_sequences.rs:30-…`) relu : `SELECT … FOR UPDATE` dans la transaction de l'appelant —
    l'annulation de la transaction annule l'avance du compteur.
  - `credit_notes.rs` : le rendu en `:571-577` (`"rendu numéro avoir échoué (config invalide ?) : {e}"` à `:575`), l'`INSERT` en
    `:731-747`.
  - `kesh-api/src/errors.rs` : `DbError::UniqueConstraintViolation(m) => {` en `:3772` (fiche `:3673`, décalée par la 35-3) ;
    `kesh-db/src/errors.rs:1753` `pub fn error_code` (fiche `:1601`).
  - `invoices/[id]/+page.svelte` : la liste des codes non rejouables en `:452-458`, avec le code de la 35-3 déjà présent
    (`err.code === 'DESIGNATED_ACCOUNT_WRONG_TYPE' ||` à `:456`) ; la 35-1 y ajoute le sien après lui.
  - `user-manual.tex` : les lignes citées `:1226-1227` et `:1561` sont en `:1235-1236` et `:1570` sur la branche.
  - **Accolade dans le nom d'exercice** : `PUT /fiscal-years/{id}` l'enregistre (`fiscal_years::update_name` ne contrôle que vide
    et longueur) ; constaté à l'exécution (`rouges-api.log`, AC3 : 200 au renommage « Ex {2026} »). D'où un test d'API pour AC3.
    Relu aussi : `render` refusait l'accolade **avant** d'analyser le format, donc sous tout format — limite voisine, C-e351-2.
  - L'en-tête de la fiche dit « Validation non lancée » : **périmé**, la validation est terminée en P4 (Change Log, 2026-11-13).
  - Le PDF du manuel, pris côté 35-2b à la fusion `569d7bd41`, ne portait pas les paragraphes de la 35-3 du `.tex` : régénéré au
    premier commit (`21deafde5`), contrôlé aplati (« ne se retype pas sans reprendre la désignation » : 1 ; 0 dans l'ancien PDF).
- **Rouges d'abord** (`rouges.log`) : API, 10 tests, **8 rouges** (AC2 : 200 au lieu de 400 ×2 ; AC3 : 500 ×3 ; AC4 :
  `409 RESOURCE_CONFLICT` ×3) et **2 verts attendus** (formats enregistrés sans compteur, inchangés — preuve par M2, M2b) ;
  `kesh-core` : rouge de compilation (9 erreurs : `NoSeq`, `FiscalYearNameBrace`, `validate_template_change` absents) ; Vitest :
  63 tests, **10 rouges** (le test réécrit de l'ancien `:94-97`, rouge attendu ; 4 `NO_SEQ` ; 5 de la page : champ des avoirs absent,
  refus absent, testids neufs) — le cas « format client chargé sans compteur » rougit par son testid et l'aide neufs, son comportement
  (bouton actif, envoi) étant déjà celui de `main` : sa preuve est M7.
- **Implémentation** (`1cc78ffb1`) : D1 à D5 comme la fiche, avec les choix C-e351-1 à -6. Écart à la lettre de la fiche (V4-1) :
  la branche `NO_SEQ` du miroir est dans `validateFormatTemplate` (formats client et avoir seuls), non dans `validateTemplateRules`,
  pour garder le refus de forme du numéro interne (parité avec `validate_numbering_template`, inchangée ; C-e351-4).
- **Mutations** (`mut/`), restaurées chacune par `git checkout -- <f>` puis `touch` ; script de pose `muter-e351.py` (chaîne unique) :
  - M1 (`NoSeq` retiré de `validate_template`) — tuée : `template_without_counter_is_refused`,
    `unchanged_template_without_counter_is_tolerated`, `ac2_changed_invoice_format_without_counter_is_refused`,
    `ac2_changed_credit_note_format_without_counter_is_refused` (`M1.log`, 4 rouges / 341).
  - M2 (la route juge le format client même inchangé) — tuée : `ac2_stored_invoice_format_without_counter_does_not_block` (`M2.log`).
  - M2b (même chose, format d'avoir) — tuée : `ac2_stored_credit_note_format_without_counter_does_not_block` (`M2b.log`).
  - M3 (la conversion de D4 rendue à `Invariant`, facture) — tuée : `ac3_invoice_number_too_long_is_a_typed_refusal`,
    `ac3_brace_in_fiscal_year_name_is_a_typed_refusal` (500 au lieu de 400, `M3.log`).
  - M3b (même chose, avoir) — tuée : `ac3_credit_note_number_too_long_is_a_typed_refusal` (`M3b.log`).
  - M4 (l'avance du compteur dans une transaction à part, avant le rendu) — tuée, **mais pas par l'assertion du compteur** : la
    seconde transaction attend le verrou que tient la première, et six tests rendent 500 au bout de ~50 s (`M4.log`). D'où **M4b**
    (sur un rendu impossible, `COMMIT` de la transaction avant le refus : le compteur avance) — tuée par l'assertion visée :
    `ac3_invoice_number_too_long_is_a_typed_refusal`, « le compteur n'a pas avancé », `left: Some(3)`, `right: Some(2)`
    (`M4b.log`, voie 2).
  - M5 (le format des avoirs lu mais non envoyé) — tuée : « le champ Format des avoirs … (M5) », « Format des avoirs chargé sans
    compteur … (D2, M7) » (`M5.log`).
  - M6 (le miroir sans `NO_SEQ`) — tuée : 9 tests (`M6.log`).
  - M7 (le miroir juge même le format inchangé : `formatBlocks` rend toujours vrai) — tuée : les deux « chargé sans compteur, laissé
    tel quel » (`M7.log`).
  - M8 (la conversion `duplicate` retirée : contrainte renommée) — tuée : `ac4_invoice_duplicate_number_is_a_typed_refusal`,
    `ac4_format_without_fiscal_year_collides_on_second_fiscal_year` (`M8.log`). Le pendant avoir (`uq_credit_notes_number`) n'a pas
    été muté : `ac4_credit_note_duplicate_number_is_a_typed_refusal` le couvre, sans preuve de mutation.
- **Gate ciblé** (`cible-1.log`, sources de `1cc78ffb1` avant son commit, Rust identique hors un doc-comment de `invoices.rs`) :
  `number_format_unrenderable_e2e`, `textes_coherents`, `idor_multi_tenant_e2e`, `designated_account_type_e2e`,
  `company_invoice_settings_postable_e2e`, `credit_notes_repository`, `invoices_validate_vat`, lib `kesh-core` et `kesh-db`,
  `kesh-i18n` : **982 exécutés, 982 passés**.
- **Gates au dernier commit de code `1cc78ffb1`** (bases de voie 2 remises à neuf avant) :
  - `scripts/test-fast.sh --ci` (voie 2, 3307) : **5386 exécutés, 5386 passés, 13 ignorés** (`test-fast-ci.log`) ;
  - `cargo build --workspace --all-targets` : vert (`build-workspace.log`) ;
  - `scripts/gate-ci.sh kesh_ci_e351` (voie 1) : **5393 passés, 0 échoués, 17 ignorés**, code 0 (`gate-ci.log`, `gate-ci.journal`) ;
  - frontend complet : `npm run check` 0 erreur, 29 avertissements préexistants, aucun dans les fichiers touchés
    (`front-check.log`) ; `lint-i18n-ownership` vert ; `test:unit` **3211 passés, 233 fichiers** (`front-test:unit.log`) ;
    `npm run build` vert (`front-build.log`) ;
  - **E2E complet : non lancé** (à la clôture, consigne de l'Epic 35).
- **Manuel** : `.tex` (cinq items de *Numérotation des factures*, l'item de l'avoir) et PDF régénéré (`make-user.log`), contrôlé
  aplati (`user-manual-plat.txt`) : « Format des avoirs » 2, « champ Format des avoirs » 2, « Le compteur est obligatoire » 1,
  « Un numéro impossible à produire est refusé » 1, « FACT{SEQ} donnerait » 1. Le code `NUMBER_FORMAT_UNRENDERABLE` n'est **pas**
  au manuel utilisateur, qui ne cite aucun code d'erreur (la 35-3 non plus) : il est à `docs/api-external.md` (§ 10, la section
  neuve *Valider une facture — un numéro impossible à produire*, la ligne du tableau de l'avoir). Écart assumé à la lettre d'AC7.
  *(Annoté le 2026-11-13, revue P1, A1 : la justification était **fausse** — le manuel utilisateur cite des codes d'erreur
  (`user-manual.tex:464` `\texttt{ACCOUNT\_LINKED\_\allowbreak TO\_BANK\_ACCOUNT}`, `:2506`
  `SETTLEMENT\_ACCOUNT\_\allowbreak IS\_CREDIT\_CARD`) ; seule « la 35-3 non plus » était vraie. Le code est ajouté au manuel
  à la remédiation, `84a6a0334`, sur le même modèle ; l'écart à AC7 est levé.)*
- **Captures** (thème clair, `kesh_e2e_e351` de voie 2, preset `with-data`, `captures.log`) : `01-numerotation-champ-avoirs.png`,
  `02-refus-ecran-f-year.png` (bouton désactivé : `true`), `02b-refus-ecran-bouton-desactive.png`,
  `03-formats-charges-sans-compteur.png` (aperçu et aide ambre ; l'enregistrement d'un autre réglage rend 200),
  `04-refus-validation-numero-deja-attribue.png` (toast du refus `duplicate`, la facture reste brouillon ; la même requête par
  l'API rend `400 NUMBER_FORMAT_UNRENDERABLE`, `details {"cause":"duplicate","document":"invoice"}`).
- **Voies de base** : jusqu'au message de l'orchestrateur, bases et mutations M1 à M8 sur la voie 1 (3306) ; ensuite voie 2 (3307) :
  M4b, captures, `test-fast`. Les bases `kesh_e351` et `kesh_e2e_e351` de la **voie 1** restent (à supprimer par l'orchestrateur).
- **AC4, l'année du numéro** (revue P1, A2) : la fiche attend `F-2026`, le test asserte `F-2020` — l'exercice du montage
  (`seed_accounting_company`) va de 2020 à 2030, et `{YEAR}` est l'année de **début** de l'exercice. Sans effet sur le
  comportement prouvé.
- **Écart à la consigne Edit** : aucun sur le code et les tests ; les mutations ont été posées par `muter-e351.py` (Python, chaîne
  unique contrôlée), permis pour les mutations. Les scripts hors dépôt (`kesh-gate-logs/e351/`) ont été retouchés par `sed -i`
  (bascule de voie, citation d'une mutation) : ce ne sont ni du code ni des tests du dépôt.

### File List

- `crates/kesh-core/src/invoice_format.rs`
- `crates/kesh-db/src/errors.rs`
- `crates/kesh-db/src/repositories/invoices.rs`
- `crates/kesh-db/src/repositories/credit_notes.rs`
- `crates/kesh-api/src/errors.rs`
- `crates/kesh-api/src/routes/company_invoice_settings.rs`
- `crates/kesh-api/tests/number_format_unrenderable_e2e.rs` (neuf)
- `crates/kesh-api/tests/textes_coherents.rs`
- `crates/kesh-api/tests/idor_multi_tenant_e2e.rs`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`
- `frontend/src/lib/features/invoices/invoice-number-format.ts`
- `frontend/src/lib/features/invoices/invoice-number-format.test.ts`
- `frontend/src/lib/features/invoices/invoices.types.ts`
- `frontend/src/lib/shared/i18n-keys.test.ts`
- `frontend/src/routes/(app)/settings/invoicing/+page.svelte`
- `frontend/src/routes/(app)/settings/invoicing/settings-invoicing-page.test.ts`
- `frontend/src/routes/(app)/invoices/[id]/+page.svelte`
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/user-manual.pdf`
- `docs/api-external.md`
- `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/e35-choix-autonomes-e351.md` (neuf)
- `_bmad-output/implementation-artifacts/35-1-numerotation-sans-seq-review-prompt-p1.md` (neuf)
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

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
- 2026-11-13 — **Validation P3, passe ciblée** (Sonnet) sur la remédiation P2 : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 3 LOW
  (`kesh-gate-logs/e35-1-validation-p3-ciblee.md`). Remédiée au plus court. Prémisses relues sur `main` : `+page.svelte:147-148`
  (`formatPreview` lit `formatValidation.ok`) ; `invoice-number-format.ts:17-20` (`{ ok, error? }`) ;
  `20260417000001_invoice_validation.sql:31` (`uq_ins_company_fy`) ; fichier d'epic `:109`. Confirmées.
  - V3-1 (MEDIUM) : **quatre** sites de `formatBlocks`, `formatPreview` (`:147-148`) nommé.
  - V3-2 (MEDIUM) : fichier d'epic (`invoices/[id]/+page.svelte` : 35-3 puis 35-1, même hunk ; `company_invoice_settings.rs`
    ajouté) et registre (C-e350-8, extension annotée).
  - LOW : V3-3 (les quatre causes écrites à D4 seul, AC7 y renvoie) ; V3-4 (message de `duplicate` juste : le compteur
    repart toujours, `{FY}` seul distingue deux exercices de même année) ; V3-5 (`code?: 'NO_SEQ'` à `FormatValidationResult`).
  - Symptômes greppés : « trois sites », « ou l'année ({YEAR}) si le », « 35-3 seule » — remplacés hors des entrées passées.
- 2026-11-13 — **Validation P4, passe ciblée** (Haiku) sur la remédiation P3 : 0 finding au-dessus de LOW, 3 LOW
  (`kesh-gate-logs/e35-1-validation-p4-ciblee.md`). Appliqués : V4-1 (D2 : `code?: 'NO_SEQ'` seul, posé par la seule branche
  `NoSeq` neuve de `validateTemplateRules`, `NoPlaceholder` d'abord) ; V4-2 (union du fichier d'epic : la 35-1 nommée sous
  `messages.ftl`, `api-external.md` et `user-manual.tex`) ; V4-3 (titre de l'extension de C-e350-8 qui annonce l'annotation
  35-1 / 35-3). Rédaction seule.
- 2026-11-13 — **Validation terminée.** Série des findings au-dessus de LOW : P1 1 HIGH + 4 MEDIUM → P2 3 MEDIUM → P3 2 →
  P4 0 (CRITICAL : 0). Modèles : Sonnet (P1), Opus (P2), Sonnet en passe ciblée (P3), Haiku en passe ciblée (P4). La boucle
  se clôt sur une passe sans finding au-dessus de LOW ; la remédiation de P4 ne touche que la rédaction. Choix retenus :
  **C-e350-42** (compteur obligatoire), **-43** (format jugé s'il change, serveur et écran), **-44** (format des avoirs à
  l'écran), **-45** (refus typé `400 NUMBER_FORMAT_UNRENDERABLE`), **-47** (numéro déjà attribué, même refus), **-51** (ni
  année ni exercice exigés) ; issue **#1058**. Statut : `ready-for-dev`.
- 2026-11-13 — **Développement** (Opus 5.5, clé `e351`), base `569d7bd41`. Commits : `21deafde5` (PDF du manuel régénéré après
  la fusion de la 35-3), `1cc78ffb1` (implémentation, tests, i18n ×4, manuel `.tex` et PDF, `api-external.md`, CHANGELOG,
  registre `e351` — **dernier commit de code**), puis ce commit-ci (comptes rendus, prompt de la revue P1, `Status: review`).
  Étape 0 : écarts de lignes `main` → branche et en-tête périmé, au Dev Agent Record. Rouges : API 8/10 (2 verts attendus),
  `kesh-core` rouge de compilation, Vitest 10/63. Mutations M1 à M8, M2b, M3b, M4b : toutes tuées (M4 par un délai de verrou, d'où
  M4b, tuée par l'assertion du compteur). Gates au commit `1cc78ffb1` : `test-fast --ci` 5386/5386, 13 ignorés (voie 2) ; `cargo
  build --workspace --all-targets` vert ; `gate-ci` 5393 passés, 0 échoué, 17 ignorés (voie 1) ; frontend complet vert (check 0
  erreur ; Vitest 3211, 233 fichiers ; build). **E2E complet non lancé** (à la clôture). Choix C-e351-1 à -6. Écarts : branche
  `NO_SEQ` du miroir hors de `validateTemplateRules` (C-e351-4) ; le code d'erreur au manuel API et non au manuel utilisateur (AC7) ;
  aucune entorse à la consigne Edit sur le code et les tests (mutations par script Python, permis). Statut : `review`.
- 2026-11-13 — **Revue de code P1** (Opus, trois lentilles ; `kesh-gate-logs/e351-review-p1-{B,E,A}.md`) : B 0 MEDIUM + 5 LOW,
  E 2 MEDIUM + 4 LOW, A 1 MEDIUM + 7 LOW — **3 MEDIUM** (A1, E1, E2, vérifiés au code par l'orchestrateur), 16 LOW.
  **Remédiation** (Opus 5.5, `e351`, voie 2), un seul commit de code : **`84a6a0334`**.
  - **Rouges d'abord** : Vitest `rem-p1-rouges-vitest.log` (base `c8725b4bb` + tests) — 7 rouges / 71 (les cinq formats d'E2
    rendaient `NO_SEQ`, et les deux formats mal formés chargés laissaient le bouton actif) ; Rust `rem-p1-rouges-rust.log` (voie 2)
    — 3 rouges / 15 : la garde `into_number_duplicate` (B2), le message `duplicate` sans « Un administrateur » (E4), le nom
    multi-octets (message « 64 caractères », E1). Verts au départ, dits comme tels : les compteurs ajoutés aux tests `duplicate` et
    `brace` (le code les tenait déjà ; preuve par M4c), l'accolade côté avoir, `invalid_format` (API et unitaire), le message de
    l'avoir `too_long` (A4).
  - **A1** : `(\texttt{NUMBER\_FORMAT\_\allowbreak UNRENDERABLE})` à l'item du manuel, sur le modèle de `:464` et `:2506` ; PDF
    régénéré (`rem-p1-make-user.log`), sondes aplaties (`rem-p1-sondes-pdf.log`) : « NUMBER_FORMAT_ UNRENDERABLE » 1 (coupé en
    fin de ligne au point d'`\allowbreak`, l'espace vient de l'aplatissement), « plus de 64 octets » 1, « un administrateur doit
    corriger » 1, « plus de 64 caractères (un nom » 0. Justification fausse du Dev Agent Record annotée.
  - **E1** (C-e351-7) : la borne reste en **octets** (la charge QR est bornée en octets, `generator.rs:73`) ; message ×4, manuel,
    `api-external.md` (deux lieux), CHANGELOG et doc-comment de `render` le disent ; test `ac3_multibyte_fiscal_year_name_counts_bytes`.
  - **E2** : le miroir découpe tout le gabarit (`/\{([^}]*)\}|([^{}]+)|([{}])/g`) — une accolade isolée rend
    `invoices-format-error-unmatched-brace` (clé neuve ×4) — et n'accepte qu'un padding décimal (`/^\d+$/`) ; tests des cinq
    formats et de deux formats mal formés chargés (bouton désactivé). `sitesTotal` 3031 → 3032, relevé par la garde.
  - **LOW traités** : B1 (« clé d'API » retirée de quatre commentaires de la route, dont deux antérieurs à la story, même
    symptôme) ; B5/E6 (doc-comment de `render`) ; B2 (la clé se lit en fin de message, `for key '<clé>'` ou `.<clé>'` ; test
    unitaire) ; B3/A3/E5 (compteur asserté dans les deux `duplicate` et les deux `brace`, accolade côté avoir, `invalid_format` à
    l'API et `from_render_error` en unitaire) ; A2, A7 (Dev Agent Record ; case « Gates » scindée) ; A4 ; A5 (ci-dessous) ; A6
    (journaux frontend annotés de leur commit, et les nouveaux l'écrivent en tête) ; E4 (« un administrateur doit… » dans les
    quatre causes, ×4, G9).
  - **A5** (complément de l'entrée *Développement*) : le test `invoice-number-format.test.ts:94-97` de la 33-1b (« F-{YEAR}
    accepté ») a été **réécrit**, rouge attendu d'AC5 : `F-{YEAR}` refusé en `NO_SEQ`, `{FY}{SEQ}` accepté.
  - **LOW non traités**, à tracer : **B4** (les phrases `too_long` et `duplicate` présument leur cas typique — nom d'exercice cité
    sans `{FY}` ; doublon venu d'un format antérieur) ; **E3** (course entre deux administrateurs : `NoSeq` jugé contre la valeur
    enregistrée passe avant le `409` de version, l'écran ne se recharge pas) ; **E5, partie** (aucun test du format par défaut
    `F-{YEAR}-{SEQ:04}` sur deux exercices commençant la même année). **A8** : porté par l'orchestrateur à la rétrospective.
  - **Mutations** (`mut/`, restaurées par `git checkout -- <f>` puis `touch`) : **M8b** (contrainte de l'avoir renommée) — tuée
    par `ac4_credit_note_duplicate_number_is_a_typed_refusal` (409 au lieu de 400, `M8b.log`) ; **M4c** (`COMMIT` juste après
    l'avance du compteur de l'avoir) — tuée par l'assertion « le compteur n'a pas avancé » dans
    `ac4_credit_note_duplicate_number_is_a_typed_refusal` (branche `duplicate`), `ac3_credit_note_number_too_long_…` et
    `ac3_brace_in_fiscal_year_name_refuses_the_credit_note` (`M4c.log`) ; **M9** (garde de `into_number_duplicate` rendue au
    `contains` libre) — tuée par `into_number_duplicate_converts_only_the_named_key` (`M9.log`). M4 (entrée *Développement*)
    reste déclarée tuée par un délai de verrou, non par le compteur.
  - **Gates de remédiation au commit `84a6a0334`** : `clippy -D warnings` vert (`rem-p1-clippy.log`) ; ciblé
    (`number_format_unrenderable_e2e`, `textes_coherents`, lib `kesh-db` et `kesh-core`, `kesh-i18n`) **884/884**
    (`rem-p1-verts-rust.log`, sources de `84a6a0334` avant son commit) ; `scripts/test-fast.sh --ci` (voie 2, bases remises à
    neuf) : **5391 exécutés, 5391 passés, 13 ignorés** (`rem-p1-test-fast-ci.log`) ; frontend complet : `check` 0 erreur (29 avertissements anciens), `lint-i18n-ownership` vert, `test:unit`
    **3219 passés, 233 fichiers**, `build` vert (`rem-p1-front-*.log`, commit écrit en tête). **Ni `gate-ci` ni E2E** (consigne de
    la remédiation ; à la clôture). Statut : `review`, passe P2 à lancer.
- 2026-11-13 — **Revue de code P2, passe ciblée** (Sonnet, une lentille, sur `84a6a0334` ; prompt
  `35-1-numerotation-sans-seq-review-prompt-p2-ciblee.md`, rapport `kesh-gate-logs/e351-review-p2-ciblee.md`) : **0 CRITICAL,
  0 HIGH, 0 MEDIUM, 7 LOW** (C1 à C7 : résidus « caractères » autour de la borne en octets, `{SEQ:+4}` au miroir, liste blanche du
  miroir, `{FY}` compté 16, justification de C-e351-7, « une lettre accentuée en compte deux », compteur de deux causes). Aucun
  correctif de code : les sept LOW sont tracés par l'orchestrateur en **#1118 à #1124**.
- 2026-11-13 — **Clôture.** Gates au dernier commit de code **`84a6a0334`** (HEAD `c33f7e2ac`, comptes rendus seuls depuis) :
  - `scripts/gate-ci.sh kesh_ci_e351` (voie 1) : **5398 passés, 0 échoué, 17 ignorés**, code 0 (`e351/cloture-gate-ci.log`,
    `cloture-gate-ci.journal`) ;
  - **E2E complet** (voie 2, `kesh_e2e_e351` recréée à neuf, migrée et semée, backend sur 3411, frontend rebâti
    `cloture-front-build.log`) : **405 passés, 7 échecs, 18 ignorés** (`e351/cloture-e2e.log`, console `cloture-e2e-console.log`,
    backend `backend-cloture.log`). Les sept échecs, un à un, sont ceux de la KF-029 (#97) au tableau de `docs/testing.md`
    § « Les échecs attendus » : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`,
    `:77`, `:150`. Aucun échec changeant, aucun imputable à la story ; `backup-restore-new-install` (#931) n'a pas rougi.
  - **Synthèse finale de la boucle de revue** — findings au-dessus de LOW : **P1** (Opus, trois lentilles B/E/A) **3 MEDIUM**
    (A1, E1, E2) + 16 LOW → **P2 ciblée** (Sonnet) **0** + 7 LOW. Close sur une passe sans rien au-dessus de LOW ; la remédiation P1
    touchait la production, d'où la passe ciblée. CRITICAL et HIGH : 0 sur toute la boucle. Validation de la fiche : P1 1 HIGH +
    4 MEDIUM → P2 3 → P3 2 → P4 0.
  - **Issues ouvertes autour de la story** : #1114 à #1124 (LOW de la P1 non traités — B4, E3, E5 en partie — et autres demandes
    de l'orchestrateur ; les sept LOW de la P2, #1118 à #1124) ; #1058 (format sans année ni exercice), de la validation.
  - Statut : **`done`**. Bases de voie 2 gardées (suppression par l'orchestrateur) ; aucun backend vivant (PID du backend d'E2E
    tué en fin de script, port 3411 libre).
