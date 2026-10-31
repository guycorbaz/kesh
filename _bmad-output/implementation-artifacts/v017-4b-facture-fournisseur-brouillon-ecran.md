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
