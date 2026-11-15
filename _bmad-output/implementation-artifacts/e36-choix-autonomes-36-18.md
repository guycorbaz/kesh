# Registre des choix autonomes — story 36-18 (les autres montants « 1.250 »)

*Ouvert le 2026-11-15 à la rédaction de la fiche (`36-18-montants-ambigus-autres-ecrans.md`), worktree `kesh-36-18`,
branche `story/36-18-montants-ambigus-autres-ecrans`, tête `fed34a9c7`. Décision de Guy qui fait foi : D6 du
2026-11-15 (fichier d'epic, « Phase 2 — décisions de Guy du 2026-11-15 », rang 18). Chaque choix est prudent et
réversible (règles de l'epic) ; aucun n'écrit de donnée ni ne touche le serveur.*

## C-36-18-1 — Une seule lecture : `readJournalAmount`, et ses deux aides rendues génériques

- **Contexte** : sept écrans et deux filtres lisent un montant tapé chacun à sa façon (`isValidAmount`, `PRICE_RE`,
  `Number`, rien). La 36-3 a posé le lecteur (`readJournalAmount`, `journal-entries/balance.ts:110`) et la 36-3b deux aides
  au nom du prix unitaire (`unitPricePayload`, `unitPriceProblem`, `invoices/unit-price.ts:25`, `:34`), sans rien qui
  soit propre au prix.
- **Retenu** : les deux aides passent dans `balance.ts`, à côté du lecteur, sous les noms `amountEntryPayload` et
  `amountEntryProblem` ; `unit-price.ts` les ré-exporte sous leurs noms d'aujourd'hui (comportement de la 36-3b
  inchangé, `unit-price.test.ts` vert sans modification). Chaque écran lit par elles ; aucune grammaire neuve.
- **Écartées** : (a) importer `unitPricePayload` depuis les frais de rappel ou les soldes de départ — un nom faux à
  sept endroits ; (b) un module neuf sous `shared/utils/` — il importerait `features/journal-entries`, inversion de
  dépendance que `amount-ambiguity.ts` évite ; (c) une grammaire par écran (le prix d'article garde `PRICE_RE`, etc.) —
  « aucune seconde règle » (prompt de Guy).
- **Réversible** : oui.

## C-36-18-2 — La forme rouverte d'un montant du serveur : une fonction à côté de `readableAmount`

- **Contexte** : le prix unitaire et le montant attendu d'une facture fournisseur se rouvrent par
  `readableAmount(…, 2)` (`supplier-invoice-form.ts:248`, `SupplierInvoiceForm.svelte:185`) : « 1.2340 » → `1.234`,
  refusé à tort par la règle neuve. La 36-3b a résolu le même cas par `unitPriceFieldValue` (`unit-price.ts:46`), qui
  importe `readableAmount` de `supplier-invoice-form.ts` : ce dernier ne peut pas l'importer en retour (cycle, V1-5 de
  la 36-3b).
- **Retenu** : `readableUnambiguousAmount(raw, minDecimals)` dans `supplier-invoice-form.ts`, à côté de
  `readableAmount` ; `unitPriceFieldValue` lui délègue (`readableUnambiguousAmount(raw, 2)`), sans changement de sortie.
- **Écartées** : (a) recopier la logique de `unitPriceFieldValue` (DRY) ; (b) déplacer `readableAmount` — six
  importateurs à changer pour rien.
- **Réversible** : oui.

## C-36-18-3 — Deux messages communs, dont le nombre de décimales est un paramètre

- **Contexte** : les messages de la 36-3 (`journal-entry-form-amount-*`) conseillent `1.2340` et « quatre décimales »,
  faux pour le règlement, les frais de rappel et le montant minimal (au centime) ; ils appartiennent à `journal-entries`
  (lint de propriété).
- **Retenu** : `common-amount-ambiguous` (« … Écrivez 1'250 pour mille deux cent cinquante, ou 1.25 pour un franc
  vingt-cinq. » — juste pour tout champ) et `common-amount-unreadable` avec `{ $max }` (2 ou 4), espace de noms global
  (`lint-i18n-ownership.js:16`). Les clés de la 36-3 et de la 36-3b ne changent pas ; leur convergence vers les clés
  communes est une issue demandée (`technical-debt`).
- **Écartées** : (a) réemployer `journal-entry-form-amount-*` — texte faux au centime, et sept entrées
  `KNOWN_VIOLATIONS` ; (b) une paire de clés par écran — quatorze textes pour deux messages.
- **Réversible** : oui.

## C-36-18-4 — Chaque écran rogne là où il rogne aujourd'hui

- **Contexte** : `readJournalAmount` ne rogne pas (36-3) ; le règlement, le prix d'article, le prix et le montant
  attendu fournisseur, le montant minimal et les filtres rognent aujourd'hui ; `Number(" 1.25 ")` des frais de rappel
  admet les blancs ; les soldes de départ ne rognent pas (`isValidAmount(" 12")` est faux).
- **Retenu** : la saisie rognée est lue partout où un blanc de tête ou de fin est admis aujourd'hui (frais de rappel
  compris) ; les soldes de départ restent sans rognage. Aucun montant accepté aujourd'hui n'est refusé pour un blanc.
- **Réversible** : oui.

## C-36-18-5 — « 10.000 » au règlement devient ambigu, contre un test existant

- **Contexte** : `SettleInvoiceDialog.test.ts:83-89` fixe « 10.000 est un montant au centime », et le serveur le lit
  ainsi (`invoices.rs:1139-1147`). Sous la règle de D6, « 10.000 » est ambigu (dix francs, ou dix mille ?).
- **Retenu** : la règle de Guy prime ; le test est réécrit pour constater le refus comme ambigu, et le Dev Agent
  Record dit pourquoi. Même chose pour « 10.008 » (refus d'échelle → ambigu ; le refus d'échelle se constate sur
  « 10.0081 ») et « 1e-3 » (illisible).
- **Réversible** : oui (une ligne).

## C-36-18-6 — Les frais de rappel : l'issue dit plus que le code

- **Contexte** : #1175 dit « "1.250" vaut 1,25 » ; le serveur refuse pourtant « 1.250 », d'échelle 3, sans normaliser
  (`dunning_levels.rs:82-86`, `scale_within(fee, 2)` ; `limits.rs:31-32`). Le défaut réel : un message (« pas plus de
  2 décimales ») qui invite à écrire `1.25` pour mille deux cent cinquante, et l'apostrophe refusée (`Number("1'250")`).
- **Retenu** : l'écran reçoit la même règle que les autres (refus « ambigu » avant le serveur, apostrophe admise) ;
  l'écart à l'issue s'écrit à la fiche et au rapport. Le constat se rejoue à l'étape 0 par un `curl` ou un test.
- **Réversible** : oui.

## C-36-18-7 — Un filtre ambigu ne s'applique pas, et l'écran le dit

- **Contexte** : les filtres de montant partent tels quels (`journal-entries/+page.svelte:80-81`) ; le serveur lit
  « 1.250 » 1,25 et refuse « 1'250 » ou « 1,25 » en 400 (`services/ecritures.rs:96-101`), que la page tait (liste
  vide, `:111-113`).
- **Retenu** : une borne claire part normalisée (dans la requête et l'URL) ; une borne ambiguë ou illisible ne part
  pas, avec le message commun et une phrase `journal-entries-filter-amount-not-applied` sous le champ.
- **Écartées** : (a) ne pas recharger la liste — l'écran resterait sur un filtre périmé sans le dire ; (b) envoyer
  quand même — le défaut.
- **Réversible** : oui.

## C-36-18-8 — Le prix d'article suit la grammaire du journal ; `isValidPrice` et `PRICE_RE` partent

- **Contexte** : `PRICE_RE` (`product-helpers.ts:53`) refuse « 007.50 » et « 01 », que la grammaire du journal
  admet ; `isValidPrice` n'a plus d'appelant de production (`git grep -n "isValidPrice" -- frontend/src` : le test et le
  doc-comment seuls).
- **Retenu** : `classifyPriceInput` lit par `readJournalAmount` (saisie rognée), garde son cas `negative` avant la
  lecture et gagne `ambiguous` ; `normalizePriceInput` rend la charge de la lecture ; `isValidPrice` et `PRICE_RE`
  sont retirés avec leurs tests (code mort). « 007.50 » devient accepté (7,50) : un refus en moins, aucun à tort.
- **Réversible** : oui.

## C-36-18-9 — Découpage proposé, non écrit

- **Contexte** : sept modules de code (règle du `CLAUDE.md`), plus le manuel ; la règle de l'epic et D6 prévoient deux
  lots au-delà de cinq.
- **Retenu** : la fiche est écrite entière, chaque critère étiqueté lot A ou lot B ; le découpage est proposé au rapport,
  à l'orchestrateur de l'écrire (consigne `e36-fiche.md`).
- **Réversible** : oui.
