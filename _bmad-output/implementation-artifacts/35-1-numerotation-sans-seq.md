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
`invoice-number-format.test.ts` (`:28-64` le format client, `:66-…` l'option `numbering`).

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
  i18n neuve.
- **D2 — Un format n'est jugé que s'il change** (C-e350-43) : la route valide le format client et le format d'avoir **quand
  la requête le porte et qu'il diffère de la valeur enregistrée** ; un format déjà enregistré, même sans compteur, ne bloque
  pas l'enregistrement d'autre chose (le défaut que #932 nommait : « durcir la validation bloquerait l'enregistrement sans
  moyen de corriger »). Aucune migration ne réécrit un format enregistré.
- **D3 — Le format des avoirs devient visible et modifiable** (C-e350-44) : un champ *Format des avoirs* dans la section
  *Numérotation*, avec son aperçu, lu (`creditNoteNumberFormat`) et envoyé ; clés i18n ×4 (`settings-invoicing-credit-note-
  format-label`, `-help`), à côté de `settings-invoicing-supplier-format-*`. La phrase `user-manual.tex:1561` devient vraie.
- **D4 — Un numéro impossible à produire est un refus typé** (C-e350-45) : à la validation d'une facture
  (`invoices.rs:2334-2340`) et à la création d'un avoir (`credit_notes.rs:572-577`), une erreur de `render` devient
  `DbError::NumberFormatUnrenderable { document: Invoice | CreditNote, cause }` (`cause` : rendu trop long, accolade dans le
  nom d'exercice), mappée comme `DesignatedAccountsNotPostable` (même famille : une configuration qui refuse à l'usage ;
  même statut), code `NUMBER_FORMAT_UNRENDERABLE`, message i18n ×4 qui nomme le format, l'exercice et la cause, et renvoie
  à *Administration → Facturation* (ou au nom de l'exercice). Rien n'est écrit : la transaction est annulée, aucun numéro
  consommé (à vérifier : `next_number_for` est dans la même transaction, `:2333`).
- **D5 — Un format sans compteur déjà enregistré** (D2) : la deuxième validation de l'exercice heurte
  `uq_invoices_number` (`uq_credit_notes_number` pour un avoir). L'étape 0 relit le chemin de la 1062 à la validation ; si
  elle sort en 500 ou en refus générique, elle est mappée sur `NUMBER_FORMAT_UNRENDERABLE`, cause `duplicate` (« ce format
  produit un numéro déjà attribué : ajoutez-y un compteur »). Sinon, la limite s'écrit au Change Log et au manuel.
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
   le même `PUT` qui le change en `AV-{YEAR}-{SEQ}` aboutit.
3. **AC3 — Le 500 devenu refus** (test d'API) : un exercice nommé de 50 caractères et le format `{FY}-{SEQ:10}` enregistré par
   SQL direct ; `POST /invoices/{id}/validate` → statut de D4, code `NUMBER_FORMAT_UNRENDERABLE`, `details` ou message qui
   nomme la cause ; la facture reste brouillon, sans numéro ; le compteur n'a pas avancé. Même chose pour un avoir. Un nom
   d'exercice à accolade, s'il peut être enregistré (sinon : test unitaire de la conversion).
4. **AC4 — D5** : le constat de l'étape 0 et sa suite, au Change Log ; si mappé, un test d'API (deux validations sous
   `F-{YEAR}`) rend le refus typé à la seconde.
5. **AC5 — L'écran** (Vitest, `settings-invoicing-page.test.ts`, mock local d'`i18nMsg`, sans état i18n global) : le champ
   *Format des avoirs* affiche la valeur lue et son aperçu, l'envoie à l'enregistrement ; `F-{YEAR}` en format client ou
   d'avoir → refus d'écran (clé de D1), aucun appel ; `validateFormatTemplate('F-{YEAR}')` refusé
   (`invoice-number-format.test.ts`).
6. **AC6 — Quatre langues et gardes** : les clés de D1, D3 et D4 dans `fr-CH`, `de-CH`, `it-CH`, `en-CH`, sans sélecteur,
   replis littéraux égaux au fr-CH ; `textes_coherents.rs` (G9) — la clé de D4 côté `errors.rs`, à la règle d'union ;
   `i18n-keys.test.ts` (`sitesTotal`, 3026 sur `main`) recompté à la source (C-e350-18).
7. **AC7 — Docs** : `user-manual.tex:1226-1227` (le compteur obligatoire ; un format enregistré sans compteur n'est pas
   réécrit, et sa correction) ; `:1561` (vrai désormais : le champ existe) ; une phrase sur le refus de D4 ;
   `docs/api-external.md` : `NUMBER_FORMAT_UNRENDERABLE` (ligne à la règle d'union) et le refus de `NoSeq` ; PDF régénéré,
   contrôlé aplati (sondes positives : « Format des avoirs », le code ; négative : aucune). CHANGELOG `## [0.17.7] — Non
   publié`, *Corrigé*, deux points `refs #932`, `refs #948`.
8. **AC8 — Captures** (thème clair, base jetable, `kesh-gate-logs/e351/captures/`) : la section *Numérotation* avec le champ
   des avoirs ; le refus d'écran de `F-{YEAR}` ; le refus de D4 sur la fiche d'une facture.

## Tests à écrire, et la preuve du rouge

Écrits d'abord (journal `kesh-gate-logs/e351/rouges.log`) : AC1 (`F-{YEAR}` accepté), AC2 (400 absent), AC3 (500), AC5
(champ absent, `F-{YEAR}` accepté) rouges sur `main`. Le cas « valeur enregistrée inchangée » d'AC2 est vert au départ, dit
comme tel. Pour AC5, le test d'écran cherche le champ par son libellé : rouge par absence, comportement prouvé ensuite par
l'envoi.

**Mutations** : M1 `NoSeq` retiré de `validate_template` (AC1, AC2) ; M2 la route juge le format même inchangé (AC2, le cas
de la base) ; M3 la conversion de D4 rendue à `Invariant` (AC3) ; M4 la conversion faite **après** l'avance du compteur sans
annulation (AC3, compteur) ; M5 le champ des avoirs lu mais non envoyé (AC5) ; M6 le miroir d'écran sans `NoSeq` (AC5).

## Tâches

- [ ] Étape 0 : relire les lignes citées ; constater D5 ; relire `next_number_for` et la transaction de validation.
- [ ] Tests, rouges consignés ; `invoice_format.rs` ; route ; `DbError`, `errors.rs` ; écran et miroir ; clés ×4, gardes i18n ;
      manuel, `api-external.md`, CHANGELOG ; captures.
- [ ] Gates : backend complet (`kesh-db` touché) ; frontend complet ; E2E complet au dernier commit de code
      (`settings-invoicing*`, `invoices*` en particulier).

## Dev Notes

### Ordre et fichiers communs — règle d'union C-e350-8

- **Après la 35-3** (`errors.rs` : bras `DbError` juste après `DesignatedAccountsNotPostable` ; ici le nôtre juste après le
  sien ; `messages.ftl` ×4 ; G9) et la **35-2a** (G9 : table de `textes_coherents.rs`, sa longueur recomptée à l'union ;
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
