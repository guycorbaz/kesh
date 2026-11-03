# Story v017-4b : factures fournisseur — brouillon, validée, payée — l'écran

Status: review

*Sous-fiche issue du découpage de la Story v017-4 à sa rédaction (2026-10-31, C-v017-4-1). Même worktree et même
branche que la `v017-4a` ; **dépend de la 4a** (routes, codes, K8) : développement après le commit de son code.
Registre **commun** : `v017-4-choix-autonomes.md`. Issues : **#849**, #853, #830, #835, #831 — en `refs` seulement.
**Le texte des AC et tout le contexte sont dans la fiche parente `v017-4-facture-fournisseur-brouillon.md`.** Les
numéros de ligne cités sont ceux de `8e126cfd`.*

## Story

En tant que Guy, je veux voir et conduire à l'écran le cycle d'une facture fournisseur — un brouillon que je corrige,
que je valide, que je dévalide au besoin —, et que le formulaire ne me laisse plus enregistrer une échéance avant la
date, une date du jour qui n'est pas celle du document, une virgule que le serveur ne lit pas, ni ne m'affiche un
total arrondi qui contredit son propre message.

## Périmètre

Le frontend (liste et son banc neuf, fiche, page d'édition `/supplier-invoices/[id]/edit`, `SupplierInvoiceForm` et
ses helpers — #853, #830, #835 —, liste — #831 —, écran de clôture, écran des lots), les clés de l'écran aux quatre
catalogues, le manuel utilisateur et son PDF, l'E2E, les captures, le CHANGELOG (*Ajouté*, *Corrigé*) et le README.

## Critères d'acceptation

**AC-21 à AC-32** de la fiche parente, sous leur numéro d'origine — *(validation P4, F8)* l'AC-30 **hors** de ce que la
4a livre (helper `createSupplierInvoiceViaApi` qui valide, helper du relevé saisi par l'API, étapes API de la spec neuve). Mutations **M21 à M35**, M21-bis compris (M35 ajoutée à l'étape 0 de la 4a, validation P5, P1 : `hasDocument`, défaut `false`, scan offert ssi `document === null && !hasDocument`, banc et en-tête du composant — fiche parente, AC-23) (M31 ajoutée par la décision
Q4, C-v017-4-24 : l'état validé s'affiche « Validée » aux quatre locales, et aucun texte de l'écran, des manuels ni de
l'API ne le nomme plus « ouverte » — AC-21, AC-29).

*(Ajouté le 2026-10-31, revue de code P1 de la 4a, A-7.)* **L'inventaire des specs E2E de l'AC-30 s'étend à deux specs
qu'il omettait**, relevées à l'E2E complet de la 4a (huitième ligne du panneau de clôture, K8 ; `expected 7, received
8`) : `fiscal-year-closing-journey.spec.ts:162` (helper `openChecks`, appelé `:212`) et `fiscal-years.spec.ts:498`
(test `:474`, clair et sombre) et `:543` (test `:519`). Les autres échecs contractuels de cet E2E sont déjà à
l'inventaire : `payment-batches.spec.ts:122`, `supplier-invoices.spec.ts:135`, `:168`, `:197`,
`supplier-documents.spec.ts:401`, `:442` (Dev Agent Record de la 4a, § *E2E complet*).

## Tâches

- [x] Étape 0 — vérifier au code commité de la 4a les routes, codes et formes de réponse que l'écran consomme ; tout
      écart au Change Log.
- [x] Bancs rouges d'abord (commit séparé) : liste (neuf), fiche, formulaire, helpers, messages, clôture, lots.
- [x] Liste (AC-21) ; fiche (AC-22) ; édition (AC-23) ; formulaire (AC-24) ; messages (AC-25) ; clôture (AC-26) ;
      lots (AC-27) ; i18n (AC-28).
- [x] Manuel utilisateur et PDF (AC-29).
- [x] E2E : étapes par l'écran de la spec neuve et specs adaptées (AC-30, sa part 4b) ; captures (AC-31) ; CHANGELOG et README (AC-32).
- [x] Mutations M21 à M35 (M21-bis compris).
- [x] Gates frontend, gate complet et **E2E complet** au dernier commit de code de la 4b — il couvre aussi la 4a
      (dérogation, C-v017-4-31).

## Dev Agent Record

*Développement du 2026-10-31, Opus 5.5, en autonomie, worktree `kesh-0175`, sur mandat
`kesh-gate-logs/prompts/v0174b-dev.md`. Journaux sous `/home/gcorbaz/devel/kesh-gate-logs/v0174b/`. Port 3179, bases
`kesh_v0174b` et `kesh_e2e_v0174b` (migrations du worktree, seed `scripts/seed-dev-db.sql`).*

### Commits (de `811d8489` exclu à `HEAD`)

`1d9f81a5` statut `in-progress` ; `87eccff4` bancs rouges ; `a4b93220` code (écran, catalogues, glossaire, specs E2E,
gardes i18n recomptées) — **dernier commit de code** ; `d23b1438` manuel utilisateur et brochure (PDF), CHANGELOG,
README ; le commit de ce compte rendu (registre, prompt de revue, statut). Tous en `refs`, aucun mot-clé de fermeture.

### Étape 0

- **Contrat de la 4a relu au code** : routes (`lib.rs:314`, `:626-648` — `PUT`, `DELETE` Administrateur fermé aux clés,
  `validate`, `unvalidate` avec `{version}`), `SourceDocumentInfo` (`importedSupplierInvoiceId`, `linkOrigin`, `qr` aux
  treize champs, `routes/supplier_invoices.rs:149-176`), réponse « relue » de `PUT`/`validate`/`unvalidate`
  (`load_with_settlement_cancellation`, `sourceDocument` calculé), refus de la dévalidation
  (`errors.rs:4137-4157` : `code`, message suffixé, `details { documentId, documentNumber }`), filtre `status` de la liste
  (`statuts_du_filtre`). Aucun écart.
- Points laissés par la 4a : la condition `document === null && !hasDocument` (M35) et les deux specs de clôture
  ajoutées à l'AC-30 — traitées. Aucun § *À recontrôler à l'étape 0* dans les fiches.
- **E2E complet avant tout patch** (`e2e-etape0.log`, tête `1d9f81a5`, base neuve) : **321 réussis, 18 échecs, 18
  ignorés** — les 7 KF-029, les 10 du contrat changé relevés à la 4a (`fiscal-year-closing-journey.spec.ts:166`,
  `fiscal-years.spec.ts:474` ×2, `:519`, `payment-batches.spec.ts:122`, `supplier-invoices.spec.ts:135`, `:168`,
  `:197`, `supplier-documents.spec.ts:401`, `:442`), et `sidebar-navigation.spec.ts:75` (KF-052 ou pollution, liste de
  `docs/testing.md`).
- **Bancs rouges** (`rouges-vitest.log`, tests commités seuls, code remisé par `git stash -u`) : 23 fichiers, **59
  rouges, 270 verts** ; deux fichiers rougissent à l'import (`supplier-invoice-unvalidate.test.ts`,
  `edit/edit-page.test.ts`, modules absents). Restés verts sans le code : les décomptes de lignes de la clôture (la
  fixture porte K8 : le composant rend une ligne par contrôle reçu) — le contenu de K8, lui, était rouge.

### Ce qui a été fait (AC-21 à AC-32)

- **Liste** (`supplier-invoices/+page.svelte`) : badge de statut, « Validée » pour `open`, TTC par `formatExactAmount`
  (#831), annulées masquées (`?status=draft,open,paid`) et case *Afficher les factures annulées*.
- **Fiche** (`[id]/+page.svelte`) : brouillon — *Modifier*, *Valider*, *Supprimer le brouillon* (Administrateur), ni
  *Payer* ni *Annuler* ; validée — *Dévalider* (Administrateur, Comptable) ; lien *Voir l'écriture d'achat* ; trois
  dialogues de confirmation ; refus sous le bouton (C-v017-4-52, 53).
- **Édition** (`[id]/edit/+page.svelte`, neuve) : `initial`, `PUT` avec la version, mode QR seulement si
  `linkOrigin === 'completion' && qr !== null`, `hasDocument` (C-v017-4-57).
- **Formulaire** : `initial`, `hasDocument` (défaut `false`), scan ssi `document === null && !hasDocument`, `Pick` des
  douze champs ; #853 (date vide avec un document, aide, message propre ; échéance antérieure sous son champ, refus
  serveur au même endroit, C-v017-4-54, 55) ; #830 (`decimalForServer`, `linesForServer`, quantité vide = 1, prix vide
  refusé, C-v017-4-56) ; #835 (`formatLiveTotal`, montant du QR) ; « Enregistrer le brouillon » / « Enregistrer » ;
  « Brouillon enregistré. » (C-v017-4-59).
- **Messages** : `supplierInvoiceUnvalidateMessage` (neuf), `SUPPLIER_INVOICE_DRAFT` à `supplierInvoiceCancelMessage`
  (clé de la 4a).
- **Clôture** : K8 (types, libellés, rendu, lien vers la liste, C-v017-4-58). **Lots** : `status: ['open']`, filtre
  client gardé.
- **i18n** : 34 clés neuves `supplier-invoices-` (dont `-cancel-blocked-draft`, déjà au catalogue, appelée) et quatre
  `fiscal-years-checks-k8-*`, aux quatre catalogues ; règle Q4 sur les catalogues entiers (sept clés réécrites, plus `imported-supplier-invoices-completed` (« Brouillon enregistré. ») :
  `-status-open`, `-settlement-cancel-confirm`, `-settlement-cancelled`, `payment-batches-failed-supplier-invoice-not-open`,
  `payment-batches-no-open`, `reconciliation-failed-rule-masked-invoice-candidate`, `reconciliation-cancel-confirm-supplier-invoice`)
  et sur les replis, tests et commentaires de l'inventaire ; glossaire : « validée », « valider », « dévalider »
  ajoutés, « ouverte » et « créé(e) » annotés. Ce que l'inventaire laisse : C-v017-4-61.
- **Manuel utilisateur** : § *Saisir* (brouillon, dates, montants), § neuve *Le brouillon : modifier, valider, supprimer*
  (`sec:brouillon-fournisseur`) et *Dévalider une facture validée* (empêchements, trou, L4, L7, L8, scan), § *sous le
  centime* (dévalider/corriger/revalider ; liste exacte), § *Annuler* (annuler ou dévalider, masquage, L6), § *Numérotation*
  (trois chemins), keshnote « deux voies », § *Préparer la clôture* (huit, K8), *Documents à traiter*, réconciliation,
  lots, dépannage ; brochure (« Huit contrôles », « validée », le cycle).
- **CHANGELOG** (*Ajouté* : cycle, masquage L6, K8 ; *Corrigé* : #853, #830, #835, #831 ; bullet #687 « la corriger »)
  et **README** (`:42`, `:43`, `:47` ; `:232` historique laissé).
- **E2E** : spec neuve complétée de trois tests par l'écran (cycle complet payé par la page *Réconciliation* ; liste,
  refus de dévalidation, K8, suppression ; complétion d'un document à QR, édition en mode QR, Comptable sans
  *Supprimer*) ; specs adaptées : `supplier-invoices.spec.ts` (valide après créer, « Validée »), `payment-batches.spec.ts`,
  `supplier-documents.spec.ts` (E4 valide par la fiche, E6 par l'API), `reconciliation-supplier.spec.ts`
  (« redeviendra validée »), `fiscal-year-closing-journey.spec.ts:162` et `fiscal-years.spec.ts` (huit lignes),
  `helpers/inbox.ts` (`fillFirstInvoiceLine` pose la date, C-v017-4-60) — `inbox-import.spec.ts:101` et
  `supplier-documents.spec.ts:213`, `:249`, `:302`, `:353` passent par lui ; `helpers/api-fixtures.ts:243`
  (commentaire).

### Gates au dernier commit de code (`a4b93220` ; `d23b1438` ne porte que des textes)

| Gate | Résultat |
|---|---|
| Base `kesh_v0174b` remise à zéro, `scripts/test-fast.sh --ci` (fmt, clippy `-D warnings`, nextest) | vert — **4638 lancés, 4638 réussis, 8 ignorés** (`nextest-final.log`) |
| `npm run check` | 0 erreur, 29 avertissements dans 5 fichiers (les mêmes qu'à la 4a ; aucun dans un fichier touché) |
| `npm run lint-i18n-ownership` | vert |
| `npm run test:unit` | vert — **194 fichiers, 2369 tests** (4a : 190 et 2291 ; +4 fichiers neufs, +78 cas) |
| `npm run build` | vert (`frontend-gate-final.log`) |
| E2E complet (base `kesh_e2e_v0174b` neuve, `/tmp/kesh-e2e-v0174b-2/` passé au backend et au runner) | **335 réussis, 7 échecs, 18 ignorés** (13,7 min, `e2e-full-final.log`) |

Échecs de l'E2E complet, nominatifs, contre `docs/testing.md` § *Les échecs attendus* : les **7 KF-029 (#97)** seuls —
`mode-expert.spec.ts:26`, `:41` ; `onboarding-path-b.spec.ts:65`, `:92` ; `onboarding.spec.ts:57`, `:77`, `:150`. Ni
KF-052, ni pollution ; les dix échecs du contrat changé sont verts. L'E2E complet couvre aussi la 4a (C-v017-4-31).

Gates intermédiaires : Vitest ciblé (23 fichiers, 349/349) ; `kesh-i18n` (41/41, après correction de deux textes de
réouverture, gardes G8 et bornes) ; `i18n_e2e` + `textes_coherents` (29/29), puis `textes_coherents` +
`configuration_transmise` après le manuel (53/53) ; E2E ciblé des neuf specs touchées (37/37, `e2e-cible-1.log`) ;
captures (5/5, `e2e-captures.log`).

### Mutations (16, toutes tuées — `mut/mutations.log`, sortie de chacune `mut/M*.out`)

| Mutation | Tuée par |
|---|---|
| M21, M21-bis | `supplier-invoice-draft-page.test.ts` — « brouillon (Admin / Comptable / Consultation) : ni bloc Payer (M21) ni bloc Annuler la facture (M21-bis)… » |
| M22 | `supplier-invoice-draft-page.test.ts` — « brouillon (Comptable) : … Supprimer le brouillon à l'Administrateur seul (M22) » |
| M23 | `SupplierInvoiceForm.test.ts` — « avec un document, la date démarre VIDE… (M23) » |
| M24 | `SupplierInvoiceForm.test.ts` — « une échéance antérieure à la date se signale sous le champ et bloque l'envoi (M24) » |
| M25 | `SupplierInvoiceForm.test.ts` — « quantité, prix et montant attendu partent avec un point (M25, M32) » |
| M26 | `supplier-invoice-form.test.ts` — « formatLiveTotal … (M26) » et `SupplierInvoiceForm.test.ts` — « une ligne à 99.9999 se lit 99.9999 (M26) » |
| M27 | `supplier-invoices-list-page.test.ts` — « le TTC s'affiche aux décimales exactes (#831, M27) » |
| M28 | `supplier-invoices-list-page.test.ts` — « demande par défaut … jamais les annulées (M28) » |
| M29 | `supplier-invoice-unvalidate.test.ts` — « chaque code de la précédence a son propre message, jamais le repli (M29) » |
| M30 | `payment-batches-page.test.ts` — « demande les factures validées au serveur (M30) » |
| M31 | `supplier-invoice-helpers.test.ts` — « mappe les statuts sur leur repli français (M31) » |
| M32 | `supplier-invoice-form.test.ts` — « le montant attendu part avec un point (M32) » |
| M33 | `supplier-invoice-form.test.ts` — « un prix vide est une ligne invalide (M33) » ; `SupplierInvoiceForm.test.ts` — « un prix vide est refusé … (M33) » |
| M34 | `edit-page.test.ts` — « document joint après coup, même avec QR : saisie libre, sans mode QR (M34) ni scan (M35) » |
| M35 | `SupplierInvoiceForm.test.ts` — « scan offert ssi `document === null && !hasDocument` (M35) » ; `edit-page.test.ts` |

Chaque mutation : motif vérifié unique, banc rejoué, restauration `git checkout -- <fichier> && touch <fichier>` ;
`git status` vide en fin de campagne.

### Compteurs (ancien → nouveau, recomptés à la source)

`i18n-keys.test.ts` `sitesTotal` 2735 → **2790** (`grep -o 'i18nMsg('` aux deux bornes, dix fichiers) ;
`i18n-un-repli-par-cle.test.ts` `CLES_RELEVEES` 350 → **384** (34 clés neuves, énumérées) ;
`i18n-libelle-en-dur.test.ts` `CANDIDATES_ATTENDUES` 76 → **77**, `conforme` 62 → **63** (`withLabel`) ; contrôles de
clôture à l'écran 7 → **8** (bancs Vitest et deux specs E2E) ; chemins de trou de numérotation au manuel 2 → **3** ;
Vitest 190 → 194 fichiers, 2291 → 2369 tests ; tests Playwright de la spec neuve 2 → **5**.

### Captures (thème clair, `kesh-gate-logs/v0174b/captures/`, 14)

Liste (annulées masquées, affichées) ; fiche d'un brouillon (Administrateur, Comptable) ; édition sans document et en
mode QR ; confirmations *Valider*, *Dévalider*, *Supprimer* ; fiche validée avec *Dévalider* ; refus de dévalidation
(lot en cours) ; formulaire (échéance refusée, total à 99.9999) ; complétion à date vide ; clôture avec K8.
Non capturés : la liste avec un total sous le centime (une facture neuve ne peut plus en porter ; couvert par Vitest),
et le « refus d'annulation d'un brouillon » du mandat, que l'écran n'affiche pas (C-v017-4-62).

### Contrôle du PDF aplati (`user-flat.txt`, `grep -oF … | wc -l` sous `timeout 60`)

« facture fournisseur ouverte » 0, « factures fournisseur ouvertes » 0, « facture ouverte » 0, « ouverte, à payer » 0,
« elle est ouverte » 0, « n’est plus ouverte » 0, « sept contrôles » 0, « passe l'écriture d'achat » (ASCII) 0 ;
« passe l’écriture d’achat » 2 — toutes deux dites de la **validation** ; « Kesh passe l » 2 — règlement client et
paiement fournisseur. Gardées : « redevient ouverte » 1 (la dette, poste ouvert), « factures ouvertes » 1 (accueil,
factures client). Présents : « huit contrôles » 1, « Enregistrer le brouillon » 1, « saisissez, puis validez » 2,
« Factures fournisseur en brouillon » 2, « restent au journal » 1, « Dévalider une facture validée » 2 ; « Afficher les
factures annulées » se lit « Aﬀicher… » (ligature). Brochure : « Sept contrôles » 0, « Huit contrôles » 1,
« fournisseur ouverte » 0. Aucune référence indéfinie (« ?? » : 0).

### Choix ajoutés au registre

C-v017-4-52 à C-v017-4-62 (`v017-4-choix-autonomes.md`).

### Écarts

1. **Méthode** : une première édition des types a été faite par un script `python3` de remplacement, contraire au § 7 ;
   annulée aussitôt (`git checkout`) et refaite à l'outil d'édition — aucun reste.
2. **Capture « refus d'annulation d'un brouillon »** : non faite, l'écran ne l'affiche pas par construction (C-v017-4-62).
3. **AC-25** : la fiche passe tout refus de la dévalidation par la fonction ; les dates d'un `PERIOD_LOCKED` et le nom
   de l'exercice d'un `LATER_FISCAL_YEAR_CLOSED`, dits par le serveur, ne sont pas repris (C-v017-4-53).
4. **Textes de réouverture** : la première rédaction de deux clés neuves a rougi les gardes `kesh-i18n`
   (`les_prescriptions_de_reouverture_disent_l_ordre`, `…_sont_bornees`) ; récrites aux quatre locales avant commit.

### Défauts observés (hors périmètre, à tracer)

- La fiche d'une facture fournisseur affiche la quantité brute du serveur (`1.0000`) dans le tableau des lignes
  (capture `v0174b-fiche-validee-devalider.png`) — cosmétique, antérieur à la story.
- Les doc-comments et commentaires de tests Rust de la 4a nomment encore « ouverte » l'état `open` (inventaire
  C-v017-4-61) — à aligner au passage d'une story backend.

### Remédiation de la revue de code P1 (2026-10-31 interrompue, reprise le 2026-11-03)

*Opus 5.5, autonomie, mandat `kesh-gate-logs/prompts/v0174b-remediation-p1.md`. Revue P1 (Opus) :
`kesh-gate-logs/v0174b-review-p1-{B,E,A}.md`, 5 MEDIUM (B-1, E-1, E-2, A-1, A-2) et 23 LOW. Journaux `r-*.log` de
`kesh-gate-logs/v0174b/`.*

**Commits** : `165d9559` (WIP du 2026-10-31, jamais vérifié à sa date ; relu en entier le 2026-11-03 : aucune édition
incomplète trouvée, ses bancs ciblés verts au premier passage, `r-vitest-cible-1.log`, 20 fichiers, 240/240) ;
`79c2211e` (suite : jeton de fond, manuel et PDF, CHANGELOG, spec E2E, bancs, gardes recomptées) et `eed15c18` (banc
qui relie le repli du refus des lots au catalogue) — **dernier commit de code** ; le commit de ce compte rendu.

**Table finding → sort** (P = WIP `165d9559`, S = suite du 2026-11-03 ; « traité » = correctif et banc qui rougit
sans lui, sauf mention).

| Finding | Sév. | Sort | Où, et le banc |
|---|---|---|---|
| B-1 | MED | traité (P, S) | « Facture qui n'est pas à payer (brouillon, payée ou annulée) », 4 catalogues + repli ; `payment-batch-helpers.test.ts` (repli = catalogue fr-CH, ancien libellé absent des 4 locales — S) ; R1, R1-ftl |
| B-2 | LOW | traité (P) | `clearCycleErrors()` au début de chaque geste ; « un refus périmé s'efface… (B-2) » ; R10 |
| B-3 | LOW | traité (P) | `SUPPLIER_INVOICE_PAID` relit ; « un refus de Dévalider « payée » relit la fiche… (B-3) » ; R8 |
| B-4 | LOW | traité (P) | jeton `requestSeq` ; « une réponse périmée ne remplace pas… » ; R11 |
| B-5 | LOW | traité (S) | facture annulée réelle au décor E2E, absente / présente / absente selon la case |
| B-6 | LOW | traité (P) | de-CH « ist danach wieder validiert und zu bezahlen » (×2), en-CH « back to validated » (×3) — textes, sans banc |
| B-7 | LOW | traité (P) | projets archivés filtrés sauf celui du brouillon ; `edit-page.test.ts` |
| B-8 | LOW | traité (P) | `readableAmount` exporté, montant attendu lisible ; `SupplierInvoiceForm.test.ts` |
| B-9 = E-5 | LOW | traité (P) | la date de facture efface `dueServerError` ; `SupplierInvoiceForm.test.ts` |
| E-1 = A-6 | MED / LOW | traité (P) | refus rendus hors des blocs de statut ; deux bancs (Valider, Supprimer) ; R2, R3, R4 |
| E-2 = A-1 | MED | traité (P, S) | lot confirmé nommé au-dessus de *Dévalider*, à tous les rôles, et dans la confirmation ; manuel et CHANGELOG (S) ; deux bancs + Consultation (S) ; E2E par HTTP (S) ; R5, R6, R7 |
| E-3 | LOW | traité (P) | `SUPPLIER_INVOICE_MUST_BE_UNVALIDATED_FIRST` relit ; R9 |
| E-4 | LOW | partiel (P, S) | `LATER_FISCAL_YEAR_CLOSED` nomme l'exercice (`details.fiscalYearName`), bancs helper et fiche (S) ; `PERIOD_LOCKED` : le serveur ne rend pas de bornes — **issue demandée** (C-v017-4-67) |
| E-6 | LOW | traité (P) | message sans « document » sans document ; `SupplierInvoiceForm.test.ts` |
| E-7 | LOW | traité (P) | `extraRates` ; `SupplierInvoiceForm.test.ts` |
| E-8 | LOW | = B-4 | |
| E-9 | LOW | = B-5 | |
| E-10 | LOW | partiel (P) | « … et N autre(s) » ; `FiscalYearChecks.test.ts` ; la liste limitée à 100 sans pagination ni filtre d'URL — **issue demandée** (C-v017-4-66) |
| A-2 | MED | traité (S) | captures refaites dans `captures-r1/` : K8 amené à la vue (`v0174b-cloture-k8.png`, `-cloture-k8-ligne.png`), liste avec une annulée (`-liste-annulees-affichees.png`, badge « Annulée ») ; plus `-fiche-validee-lot-confirme.png`, `-confirmation-devalider-lot-confirme.png` |
| A-3 | LOW | traité (S) | manuel : « Une facture payée ne se dévalide pas, et sa fiche ne le propose pas » ; le refus « payée » n'arrive qu'entre-temps ; CHANGELOG non publié aligné |
| A-4 | LOW | traité (S) | manuel : « La liste dit son état par un badge, la fiche à côté de son numéro » |
| A-5 | LOW | traité (P) | un seul site `loadOpenInvoices()` pour les deux lectures des lots (le banc M30 couvre donc les deux) |
| A-7 | LOW | traité (P) | payée et annulée × trois rôles (`it.each` de 2 à 6 cas) |
| A-8 | LOW | traité (P) | it-CH `-unvalidate-blocked-lettered` à la forme « tu » ; les autres clés de la 4b relues : aucune forme « voi » |
| A-9 | LOW | traité (S) | inventaire ci-dessous |

Hors table, trouvé en remédiant : le badge « Brouillon » de la liste et l'avertissement du lot employaient
`bg-warning-soft`, jeton **non défini** (#645) — passés à `bg-warning/10` (C-v017-4-69).

**Inventaire « ouverte » (A-9)**, rejoué par la valeur (`grep -rniE "(factur|fournisseur|supplier).{0,40}ouvert|ouvert.{0,30}(factur|fournisseur)"`
sur `frontend/src`, `frontend/tests/e2e`, le catalogue fr-CH, `README.md`, `website/`, `docs/api-external.md`, le
manuel utilisateur) : **43 lignes**, relues une à une. Aucune ne nomme l'état `open` d'une facture fournisseur dans un
texte de l'écran, d'un catalogue, du manuel ou de l'API. Elles désignent : les factures **client** de l'accueil (tuile
« Factures ouvertes », ses bancs, specs et clés, le manuel `:224`, `:229`) ; le **poste ouvert** et le lettrage ;
l'**exercice ouvert** ; un dialogue ou une fiche **ouvert(e)** à l'écran, l'« ouverture » d'une page ; une route
« **ouverte aux clés** » (`docs/api-external.md`) ; l'**historique** (`i18n-libelle-en-dur.test.ts:11`,
`README.md:232`). Un seul site désigne une facture fournisseur `open` : l'identifiant `rouverte` d'un test E2E
(`reconciliation-supplier.spec.ts:301`), du code de test, laissé (C-v017-4-61).

**Gates au dernier commit de code (`eed15c18`)**

| Gate | Résultat |
|---|---|
| Bases `kesh_v0174b` et `kesh_e2e_v0174b` recréées (migrations du worktree, seed), `scripts/test-fast.sh --ci` | vert — **4638 lancés, 4638 réussis, 8 ignorés** (`r-nextest-final.log`) |
| `npm run check` (à `79c2211e` ; `eed15c18` ne touche qu'un banc) | 0 erreur, 29 avertissements dans 5 fichiers, aucun dans un fichier touché (`r-check-1.log`) |
| `npm run lint-i18n-ownership` | vert (`r-lint-1.log`) |
| `npm run test:unit` | vert — **194 fichiers, 2389 tests** (`r-vitest-complet-2.log`) |
| `npm run build` | vert (`r-build-1.log`) |
| E2E complet (`/tmp/kesh-e2e-v0174b-r1/` au backend et au runner, port 3179) | **336 réussis, 7 échecs, 17 ignorés** (14,0 min, `r-e2e-full-final.log`) ; backend arrêté par son PID |

Échecs de l'E2E, nominatifs, contre `docs/testing.md` § *Les échecs attendus* : les **7 KF-029 (#97)** seuls —
`mode-expert.spec.ts:26`, `:41` ; `onboarding-path-b.spec.ts:65`, `:92` ; `onboarding.spec.ts:57`, `:77`, `:150`. Ni
KF-052 ni pollution. Le test « liste, refus de dévalidation, K8… » est vert avec ses étapes neuves.

Gates intermédiaires : Vitest ciblé (`r-vitest-cible-2.log`, 11 fichiers, 132/132) ; `kesh-i18n` + `textes_coherents` +
`i18n_e2e` (`r-nextest-i18n-1.log`, 70/70) ; `make user` (`r-make-user-1.log`, 2 passes, stable). Un premier Vitest
complet (`r-vitest-complet-1.log`) a rougi sur trois gardes — `sitesTotal`, `CLES_RELEVEES` (décomptes du WIP non
recalés) et la garde KF-043 (un `getByRole({ name: 'Annuler' })` neuf, remplacé par Échap).

**Mutations de la remédiation** (`kesh-gate-logs/v0174b/mut-r1/mutations.log`, `mutations-2.log`, sorties `R*.out`) :
12 appliquées, **12 tuées** au final ; `git status` vide après chaque restauration.

| Mutation | Finding | Tuée par |
|---|---|---|
| R1 (repli « Facture non validée ») | B-1 | `payment-batch-helpers.test.ts` — « mappe les huit codes sur leur repli français » |
| R1-ftl (catalogue fr-CH seul) | B-1 | **survivait** à la 1re campagne ; tuée après `eed15c18` par le même banc (`mutations-2.log`) |
| R2 (`validateError` dans le bloc brouillon) | E-1 | « un refus de Valider qui relit une facture devenue validée reste lisible (E-1) » |
| R3 (`deleteError` idem) | E-1 | « un refus de Supprimer « déjà validée » relit la fiche… (E-1, E-3) » |
| R4 (`unvalidateError` masqué si payée) | E-1 | « un refus de Dévalider « payée » relit la fiche… (B-3) » |
| R5 (avertissement de la fiche retiré) | E-2 | « … la fiche et la confirmation nomment le lot (E-2 = A-1) » et « le rôle Consultation voit le lot… » |
| R6 (avertissement du dialogue retiré) | E-2 | « … la fiche et la confirmation nomment le lot (E-2 = A-1) » |
| R7 (bloc réservé à `canManage`) | E-2 | « le rôle Consultation voit le lot confirmé, sans Dévalider » |
| R8, R9 (`isStaleState` amputé) | B-3, E-3 | les bancs B-3 et E-1/E-3 |
| R10 (`unvalidateError` non effacé) | B-2 | « un refus périmé s'efface au geste réussi suivant (B-2) » |
| R11 (jeton de lecture retiré) | B-4 | « une réponse périmée ne remplace pas celle de la dernière bascule » |

A-2 ne porte aucun code de production : le filtre qu'il voulait voir prouvé reste gardé par M28 (Vitest), et l'E2E
l'éprouve désormais à travers HTTP. Aucune mutation de l'E2E n'a été jouée.

**Compteurs recalés** : `sitesTotal` 2790 → **2794** ; `CLES_RELEVEES` 384 → **387** ; Vitest 2369 → **2389** tests
(+16 déclarations, +4 cas de l'`it.each` payée/annulée) ; captures de la 4b : K8 refaite, plus trois neuves
(`-cloture-k8-ligne`, `-fiche-validee-lot-confirme`, `-confirmation-devalider-lot-confirme`).

**Contrôle du PDF aplati** (`r-user-flat.txt`, `grep -oF … | wc -l` sous `timeout 60`) : « la fiche nomme ce lot » 0,
« nomment ce lot et sa date de confirmation » 1, « Une facture payée ne se dévalide pas » 1, « La liste dit son état par
un badge, la fiche à côté de son numéro » 1, « La liste et la fiche disent son état par un badge » 0, « la fiche se
relit, la montre payée et garde le motif » 1, « que le refus nomme » 1, « ?? » 0.

**Choix ajoutés** : C-v017-4-63 à C-v017-4-70.

**Écarts** : (1) le WIP `165d9559` reste tel quel, corrigé par des commits suivants ; (2) les captures neuves vont dans
`captures-r1/`, celles de la P1 restent (pas d'écrasement) — l'E2E complet y a aussi déposé les captures des autres
specs ; (3) le statut de la sous-fiche n'est pas repassé par `in-progress` (remédiation dans le flux de la revue).

**Défauts observés, à tracer** : `PERIOD_LOCKED` sans bornes au corps de la réponse (backend) ; la liste des factures
fournisseur limitée à 100 sans pagination ni filtre de statut par l'URL ; `bg-warning-soft` employé dans
`BankImportUpload.svelte`, `bank-accounts/+page.svelte`, `SupplierInvoiceForm.svelte`, `SupplierInvoiceTotal.svelte`,
`SupplierCreatePanel.svelte` (déjà sous #645) ; la date du lot dans la confirmation d'**annulation du règlement**
(`cancelSettlement`, code antérieur) s'affiche en ISO, non au format suisse ; la quantité brute `1.0000` du tableau des
lignes de la fiche (déjà relevée ci-dessus).

*(Annotation du 2026-11-03, revue P2 : la ligne E-2 = A-1 de la table ci-dessus disait le lot « nommé » par un texte
qui affirmait « payée par le lot … puis son règlement a été annulé » — causalité que le dépôt interdit ; reprise
ci-dessous, B2-1. Le relevé de la P1 n'est pas réécrit.)*

### Remédiation de la revue de code P2 (2026-11-03)

*Revue P2 (Sonnet) : `kesh-gate-logs/v0174b-review-p2-{B,E,A}.md` — B 0/0/2/5, E 0/0/0/7, A 0/0/2/4. Journaux `p2-*.log`
de `kesh-gate-logs/v0174b/`.*

**Commits** : `fa71e7bc` (correctifs, catalogues, manuel et PDF, CHANGELOG, bancs, E2E, gardes) et `48b39fbd` (banc
des catalogues) — **dernier commit de code** ; le commit de ce compte rendu.

| Finding | Sév. | Sort | Où, et le banc / la mutation |
|---|---|---|---|
| B2-1 = A2-1 | MED | traité | le fait historique : « Cette facture figure dans le lot de paiement n° X, confirmé le D. », calqué sur `supplier-invoices-settlement-cancel-batch-warning`, sans « payée par », sans « puis son règlement a été annulé » ; 4 locales, manuel et PDF, CHANGELOG, E2E (`not.toContainText('payée par')`) ; bancs fiche + catalogues ; S1, S1-ftl |
| B2-2 = A2-2 = E2-1 | MED / LOW | traité | `clearCycleErrors()` dans `pay` (début), `cancelSettlement` et `cancel` (après confirmation) ; trois bancs dont le parcours *Dévalider* refusé « payée » → *Annuler le règlement* → plus de refus ; S5, S6, S7 |
| B2-3 = E2-2 = A2-3 | LOW | traité | `confirmedAt` nul : clé `-undated` ; banc ; S2 |
| B2-4 | LOW | traité | date suisse dans la confirmation d'annulation du règlement ; `supplier-settlement-page.test.ts` ; S8 |
| B2-5 | LOW | traité | le fait à tous les rôles, la consigne (clé `-action`) aux seuls rôles qui dévalident et dans la confirmation ; banc Consultation ; S3 |
| B2-6 | LOW | partiel | it-CH : les trois `error-supplier-invoice-unvalidate-blocked-*` au « tu » ; les autres clés italiennes au « voi » (`supplier-invoices-cancel-*`, `error-*-must-be-unvalidated-first`, `invoices-settlement-cancel-blocked-written-off`, `reconciliation-cancel-blocked-written-off`, `bank-import-manual-delete-blocked`…) restent — **issue demandée** |
| B2-7 = A2-5 | LOW | traité | de-CH « Zahlungsauftrag », comme la clé voisine |
| A2-4 = E2-3 | LOW | traité | `{:else if invoice.status === 'open' && (canManage \|\| invoice.lastConfirmedBatch)}` ; banc ; S4 |
| A2-6 | LOW | traité | manuel : « suivi d'un exercice clôturé (le refus nomme alors celui-ci) » |
| E2-5 | LOW | traité | étiquette `readableAmount(r, 2)` ; banc ; S9 |
| E2-4 | LOW | non traité | projet archivé enfant d'une racine archivée, invisible à l'édition — **issue demandée** |
| E2-6 | LOW | non traité | l'exercice nommé est le plus proche clos, la consigne dit « le plus récent » — **issue demandée** |
| E2-7 | LOW | non traité | = #870 (liste à 100) |

**Gates au dernier commit de code (`48b39fbd`)** : bases recréées ; `scripts/test-fast.sh --ci` **4638 / 4638**, 8
ignorés (`p2-nextest-final.log`) ; `npm run check` 0 erreur, 29 avertissements, aucun dans un fichier touché ;
`lint-i18n-ownership` vert ; Vitest **194 fichiers, 2396 tests** (`p2-vitest-complet-2.log`) ; `npm run build` vert ;
E2E complet (`/tmp/kesh-e2e-v0174b-r2/`) **335 réussis, 8 échecs, 17 ignorés** (14,4 min, `p2-e2e-full-final.log`) :
les 7 KF-029 (`mode-expert.spec.ts:26`, `:41` ; `onboarding-path-b.spec.ts:65`, `:92` ; `onboarding.spec.ts:57`, `:77`,
`:150`) et **`vat-rates.spec.ts:55`** (délai de 30 s dépassé en attendant `#username` sur `/login`), **rejoué seul vert**
(`/tmp/kesh-e2e-v0174b-r3/`, 2/2, `p2-e2e-vat-rates-isole.log`) : pollution au sens de `docs/testing.md`, ni KF-052 ni
lien avec la story (la spec ne touche pas aux factures fournisseur, et elle était verte à l'E2E de la P1). Backends
arrêtés par leur PID. Intermédiaires : Vitest ciblé 40 fichiers 519/519 ; `kesh-i18n` + `textes_coherents` + `i18n_e2e`
70/70 ; `make user` stable en 2 passes.

**Mutations** (`mut-r2/mutations.log`, `mutations-2.log`) : 10 appliquées, **10 tuées** au final — S1 à S9 dès la
campagne ; S1-ftl (le seul catalogue fr-CH) **survivait**, tuée après `48b39fbd` par « les catalogues disent le fait,
jamais « payée par ce lot », aux quatre locales (B2-1) ».

**Compteurs** : `sitesTotal` 2794 → **2796** (fiche 74 → 76) ; `CLES_RELEVEES` 387 → **389** (`-undated`, `-action`) ;
Vitest 2389 → **2396** (+6 à la fiche, +1 au formulaire).

**PDF aplati** (`p2-user-flat.txt`) : « payée par un lot confirmé, dont le règlement a été annulé » 0 ; « Une facture
validée qui figure dans un lot confirmé se dévalide » 1 ; « Kesh ne dit pas que ce lot l’a payée » 1 ; « le refus
nomme alors celui-ci » 1 ; « que le refus nomme » 0 ; « ?? » 0. Grep de la valeur sur le dépôt (« payée par le lot »,
« pagata dall », « mit dem Zahlungsstapel », « paid by payment batch », « puis son règlement a été annulé », « dont le
règlement a été annulé ») : plus aucun site dans le code, les catalogues, le manuel ou le CHANGELOG neufs ; restent
les fiches et prompts historiques, et des phrases d'autres stories qui décrivent une facture **payée** par un lot
(vraies).

**Choix** : C-v017-4-63 et C-v017-4-64 amendés par annotations datées (registre). Captures : `captures-r2/`
(`v0174b-fiche-validee-lot-confirme.png` montre le fait et la consigne).

## Change Log

- 2026-10-31 — Sous-fiche créée au découpage de la v017-4 (C-v017-4-1). Prochaine étape : validation P1 de la fiche
  parente et des deux sous-fiches (`v017-4-validate-prompt-p1.md`).
- 2026-10-31 — Décisions sur Q1 à Q5 (C-v017-4-21 à 25) : libellé « Validée » et inventaire des « ouverte » (AC-21,
  AC-29), M31 ; masquage des annulées confirmé (AC-21) ; pas de « Enregistrer et valider ».
- 2026-10-31 — Validation P1 remédiée dans la fiche parente (C-v017-4-26 à 33) : inventaire des « ouverte » par la valeur
  sur tout le dépôt, phrases du manuel réécrites (« saisissez, puis validez »), specs et helper E2E nommés, montant attendu
  et prix vide (#830), données du QR lues dans `sourceDocument`, M32 à M34. Prochaine : validation P2.
- 2026-10-31 — Validation P2 remédiée dans la fiche parente (C-v017-4-34 à 36) : prix vide refusé par le formulaire,
  specs de complétion et `fillFirstInvoiceLine` (date), règle du QR verrouillé et L7 au manuel, « deux voies » et clés
  `read-write`, lien d'écriture d'achat sur la fiche, union `SupplierInvoiceCancelCode`. Prochaine : validation P3.
- 2026-10-31 — Validation P3 remédiée dans la fiche parente (C-v017-4-37 à 40) : `case "SUPPLIER_INVOICE_DRAFT"` et son
  texte, `Pick` de treize champs pour le document du formulaire, `fiscal-years-page.test.ts:460`, `README.md:47`, limite L8
  au manuel ; l'AC-30 commence dans la 4a. Prochaine : validation P4.
- 2026-10-31 — Validation P4 remédiée dans la fiche parente (C-v017-4-41, 42) : la clé du motif brouillon est posée par
  la 4a (AC-13), l'AC-25 la réutilise ; ce que l'édition passe au formulaire selon le cas, scan caché avec un document ;
  exception de l'AC-30 nommée. Prochaine : validation P5 (ciblée).
- 2026-10-31 — Développement (Opus 5.5, autonomie) : écran du cycle (liste, fiche, édition, formulaire #853 #830 #835,
  liste #831, K8, lots), quatre locales, manuel utilisateur et brochure (PDF), CHANGELOG, README, specs E2E ; 16 mutations
  tuées ; gate complet 4638/4638, Vitest 2369, E2E complet 335 / 7 (KF-029 seuls). Choix C-v017-4-52 à 62. Statut
  `review`. Prochaine : revue de code P1 (`v017-4b-review-prompt-p1.md`, Opus, lentilles B, E, A).
- 2026-11-03 — Revue de code P1 (Opus, B, E, A) : 0 CRITICAL, 0 HIGH, 5 MEDIUM (B-1, E-1, E-2, A-1, A-2), 23 LOW.
  Remédiation (WIP `165d9559` du 2026-10-31, puis `79c2211e`, `eed15c18`) : tous les MEDIUM traités, LOW traités sauf
  deux parties tracées (E-4 `PERIOD_LOCKED`, E-10 liste à 100) ; 12 mutations tuées ; gate 4638/4638, Vitest 2389, E2E
  336 / 7 (KF-029 seuls). Choix C-v017-4-63 à 70. Statut `review`. Prochaine : revue de code P2 complète
  (`v017-4b-review-prompt-p2.md`, Sonnet, lentilles B, E, A).
- 2026-11-03 — Revue de code P2 (Sonnet, B, E, A) : 0 CRITICAL, 0 HIGH, 4 MEDIUM (deux convergents : B2-1 = A2-1,
  B2-2 = A2-2), 16 LOW. Remédiation (`fa71e7bc`, `48b39fbd`) : les deux MEDIUM traités, LOW traités sauf B2-6 (partiel),
  E2-4, E2-6, E2-7 (issues) ; 10 mutations tuées ; gate 4638/4638, Vitest 2396, E2E 335 / 8 (KF-029, plus
  `vat-rates.spec.ts:55` vert rejoué seul). Prochaine : passe P3 **ciblée** (`v017-4b-review-prompt-p3.md`, Opus,
  lentille R).
