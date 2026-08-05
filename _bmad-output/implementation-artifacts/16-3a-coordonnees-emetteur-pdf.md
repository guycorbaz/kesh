# Story 16.3a : Coordonnées de l'émetteur sur le PDF de facture

## Status

ready-for-dev

## Story

**As a** indépendant ou fiduciaire qui envoie ses factures à des clients,
**I want** que mes **coordonnées de contact** — téléphone, e-mail, site web — figurent sur le PDF sous mon adresse,
**so that** un client qui a une question sur la facture puisse me joindre **depuis la facture elle-même**, sans avoir à chercher ailleurs.

Issue : **#151** (moitié « émetteur »). Sous-story de l'Epic 16 « Facturation avancée », cible **v0.9.0**.

## Contexte

L'issue #151 constate que le PDF « semble incomplet comparé à des factures commerciales reçues, **particulièrement sur le bloc gauche (émetteur)** ». Le relevé du code confirme : le bloc émetteur ne rend que **le nom, l'adresse et le numéro IDE** (`pdf.rs`, `draw_invoice_section`).

**Trois des quatre champs demandés par #151 n'existent nulle part**, ce qui fait de cette story bien plus qu'une retouche de mise en page. Relevé sur le schéma réel :

| Champ | État |
|---|---|
| Récapitulatif TVA | ✅ **déjà livré** (PR #267) — `vat_lines` + `recap_reserve` dans `pdf.rs`. **Hors périmètre.** |
| E-mail émetteur | ⚠️ colonne `companies.email` **existe**, le PDF **ne la rend pas** |
| Téléphone émetteur | ❌ **aucune colonne** sur `companies` |
| Site web émetteur | ❌ **aucune colonne** sur `companies` |
| Numéro de client | ❌ aucune colonne sur `contacts` — **story 16-3b** |

**Périmètre arbitré par Guy le 2026-08-05** : découpage **par entité**, imposé par la § *Règle de splitting préventif* — 7 modules recensés pour un seuil de 5. **16-3a** traite l'émetteur (`companies`), **16-3b** le numéro de client (`contacts`). Les deux sont **indépendantes** et peuvent partir séparément — contrairement à 16-2a/16-2b, aucune ne livre de donnée que l'autre seule afficherait.

## Décisions

### D1 — Le budget vertical du haut de page est de 110 mm, et **rien ne le surveille**

C'est le risque central de cette story, et il est **silencieux**. La chaîne verticale de `draw_invoice_section` :

```
y = PAGE_H - 20                     départ du bloc émetteur
  - 5                               nom (14 pt gras)
  - 4 × N                           N lignes d'adresse (9 pt)
  - 6                               IDE, si présent
y = y.min(PAGE_H - 55.0)            ⚠️ PLAFOND, pas plancher : si y est DÉJÀ plus bas, il y reste
  - 5                               libellé « Destinataire »
  - 4.5                             nom du débiteur
  - 4 × M                           M lignes d'adresse du débiteur
                                    puis : ty = PAGE_H - 130.0   ← CONSTANTE indépendante
```

**Le tableau des lignes démarre à une ordonnée fixe.** Il n'est jamais repoussé par ce qui précède. Allonger le bloc émetteur de 3 lignes (~12 mm) rapproche donc le bloc destinataire du tableau **sans qu'aucune garde ne le détecte** : la seule garde existante (`TooManyLines`) surveille le **plancher QR**, en bas de page, et ne voit rien de ce qui se passe en haut.

**Décision** : la story **doit** ajouter une garde symétrique en haut — le bloc destinataire ne doit pas descendre sous le début du tableau — **et** la prouver par un test de pire cas. Il est interdit de se contenter de constater que le cas nominal tient.

⚠️ **Ne PAS « corriger » en déplaçant `ty`** : cette constante aligne le tableau avec la zone QR plus bas. La toucher déplacerait tout le document.

### D2 — Les trois champs sont **facultatifs**, et une valeur absente ne laisse **aucune ligne vide**

Aucune donnée existante ne les porte : toute société déjà créée aura `NULL` sur les trois. Le rendu doit donc suivre le patron **déjà employé pour l'IDE** (`if let Some(ide) = &inv.creditor_ide`) — une ligne n'est dessinée que si la valeur est présente, et le curseur ne descend pas sinon.

**Conséquence à ne pas manquer** : c'est aussi ce qui **limite** le risque de D1 — une société sans coordonnées produit exactement le PDF d'aujourd'hui, à l'octet près pour le bloc haut.

### D3 — Le rendu se fait aux **DEUX** sites de construction, avoir compris

`InvoicePdfData` est construit à **deux endroits**, et les oublier serait invisible en revue de la facture :

- `crates/kesh-api/src/routes/invoice_pdf_service.rs:279` — la facture ;
- `crates/kesh-api/src/routes/credit_notes.rs:277` — l'**avoir**.

Les deux lisent déjà `company.ide_number`. **Un avoir sans coordonnées alors que la facture en porte serait un défaut visible par le client**, et aucun test de facture ne l'attraperait. Le grep de propagation à exécuter est `grep -rn "creditor_ide" crates/kesh-api/src/`.

### D4 — Saisie sur le patron **20-3b2** : édition inline dans les réglages + route dédiée

⚠️ **Il n'existe AUCUNE route de mise à jour générale de la société.** `crates/kesh-api/src/routes/companies.rs` n'expose que **deux** fonctions publiques : `update_company_email` (`:86`) et `get_current` (`:149`). Le seul autre chemin d'écriture est `update_company_coordinates` (`onboarding.rs:983`), qui **n'a qu'un seul appelant** — l'étape 5→6 de l'onboarding — et dont le commentaire prévient qu'un second appelant imposerait d'extraire un paramètre `reset_stub`, puisqu'elle pose `is_stub = FALSE` **inconditionnellement**.

**Et l'écran de réglages n'édite pas ces champs** : nom, type, adresse et IDE y sont rendus en **lecture seule** (`<dd>`). Un seul champ y est éditable — **l'e-mail**, en édition *inline* avec un bouton « Modifier » (`settings/+page.svelte:175`), servi par sa route dédiée. C'est la **Story 20-3b2**.

**Décision : reproduire ce patron.** Téléphone et site web deviennent éditables *inline* dans les réglages, servis par une route dédiée, exactement comme l'e-mail. Ne **pas** créer de route de mise à jour générale — ce serait une refonte hors périmètre — et ne **pas** réutiliser `update_company_coordinates`, dont le `is_stub = FALSE` inconditionnel et l'appelant unique sont des invariants documentés.

⚠️ **Sans cette décision, la story serait inutilisable par son premier utilisateur.** L'onboarding ne se rejoue pas : sur une instance **déjà installée** — celle de Guy en dogfooding — les champs existeraient en base et sur le PDF sans qu'aucun écran ne permette de les renseigner. Une fonctionnalité livrée que personne ne peut activer.

### D5 — Validation : longueur bornée, et **rien de plus**

Les trois champs sont du texte libre affiché sur un document. La validation se limite à une **longueur maximale** cohérente avec la largeur disponible du bloc gauche (le bloc droit des métadonnées commence à `meta_x = 120.0`, la marge gauche est à `20.0` — soit **100 mm** de large).

**Ne PAS valider le format** du téléphone (les formats internationaux sont innombrables) ni du site web (un utilisateur écrira aussi bien `example.ch` que `https://example.ch`). Un rejet de saisie sur un champ purement décoratif coûterait plus qu'il ne rapporte.

⚠️ **L'e-mail, lui, est déjà validé** par la route dédiée existante — ne pas dupliquer cette validation.

## Acceptance Criteria

**AC1 — Migration.** Deux colonnes nullables sur `companies` (téléphone, site web). DDL pur, **aucune écriture de données**. Non-breaking : **pas** de bump de `kesh_version_min_required` ni de version Cargo.

**AC2 — Garde-fous de migration.** Ligne ajoutée à `docs/migrations-idempotence-audit.md` **à l'intérieur du tableau**, et les compteurs **recomptés depuis la source** — les deux sites du total et les trois compteurs de partition, dont la somme doit égaler le total. Le total passe de **58** à **59**. Garde-fou **P6** : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` et inspection de chaque site — l'assertion `assert_eq!(total, 58)` de `migrations_upgrade_path.rs` **devra être portée à 59**. Garde-fou **P7** : la migration n'écrivant aucune donnée, elle relève de l'exemption, avec sa justification écrite.

**AC3 — API et les cinq listes de colonnes.** Les deux champs sont lus et écrits par une **route dédiée**, sur le patron de `update_company_email` (D4), avec verrou optimiste. `PUT /companies/current/email` **n'est pas modifiée**, `update_company_coordinates` non plus. ⚠️ **Les CINQ listes de colonnes `companies` écrites à la main sont mises à jour** (cf. T2) — une omission produit un `ColumnNotFound` **à l'exécution**, invisible à la compilation et au type-check.

**AC4 — Rendu PDF, les trois champs.** Téléphone, e-mail et site web sont rendus sous l'IDE dans le bloc émetteur, **chacun précédé de son libellé traduit**, et **uniquement s'ils sont renseignés** (D2). L'e-mail vient de la colonne existante.

**AC5 — L'avoir aussi.** Le rendu est identique sur le PDF d'avoir (D3). **Un test doit le prouver sur l'avoir spécifiquement** — un test de facture ne dit rien du second site de construction.

**AC6 — La garde de capacité haute existe et elle est prouvée.** Le bloc destinataire ne peut pas chevaucher le tableau. Un test monte le **pire cas** — adresse émetteur longue **et** les trois coordonnées renseignées **et** adresse destinataire longue — et vérifie que le document est soit correctement rendu, soit refusé par une erreur explicite. ⚠️ **Un test qui se contente du cas nominal ne satisfait pas cet AC** : le cas nominal tient déjà aujourd'hui, il ne mesure rien.

**AC7 — i18n.** Trois nouvelles clés déclarées dans `I18N_KEYS` (`crates/kesh-qrbill/src/types.rs`) et traduites dans **les 4 locales**. ⚠️ Le dépôt a une KF ouverte sur des clés absentes des locales non françaises (**#283**) — ne pas l'aggraver.

**AC8 — Frontend, ÉDITABLE sur une instance déjà installée.** Les deux champs sont éditables *inline* dans les réglages, sur le patron de l'e-mail (`settings/+page.svelte:175`), avec un texte d'aide disant qu'ils apparaîtront sur les factures. ⚠️ **L'onboarding ne se rejoue pas** : une saisie qui n'existerait qu'à l'onboarding rendrait la fonctionnalité inaccessible à toute instance existante — dont celle en production. C'est le volet qui conditionne l'utilité de toute la story.

**AC9 — Documentation.** Manuel utilisateur mis à jour et **PDF régénéré** (`make fr`). CHANGELOG. README vérifié — l'Epic 16 reste « 🚧 En cours » tant que 16-3b n'est pas livrée, donc **probablement aucun changement**, mais la vérification se **trace** même si la conclusion est « rien à changer ».

**AC10 — Gate.** Backend + frontend + suite E2E, sur l'**état final**, exit 0, **verdict lu dans le log**. La story touchant `crates/kesh-db/migrations/`, le ciblage est **interdit**, y compris entre deux passes de revue.

## Tasks / Subtasks

- [ ] **T1 — Migration** (AC1, AC2) — deux colonnes nullables ; ligne d'audit **dans le tableau** ; compteurs recomptés `58 → 59` aux deux sites du total ; partition recomptée ; `assert_eq!(total, 58)` de `migrations_upgrade_path.rs` porté à **59** ; triage P7 en exemption avec justification.
- [ ] **T2 — Entité et les CINQ listes de colonnes** (AC3) — champs sur `Company`, **puis les cinq listes de colonnes écrites à la main**, relevées et vérifiées :
  1. `crates/kesh-db/src/repositories/companies.rs:17` — `FIND_BY_ID_SQL`
  2. `crates/kesh-db/src/repositories/companies.rs:22` — `LIST_SQL`
  3. `crates/kesh-api/src/routes/onboarding.rs:688` — `SELECT` en ligne
  4. `crates/kesh-api/src/routes/onboarding.rs:853` — `SELECT` en ligne
  5. `crates/kesh-api/src/routes/onboarding.rs:908` — `COMPANY_SELECT_FOR_UPDATE`

  ⚠️ **Une omission ne casse pas la compilation : elle produit un `ColumnNotFound` À L'EXÉCUTION**, et met en 500 la route concernée. C'est le piège de `FIND_BY_ID_SCOPED_SQL` de la story 16-2a, ici **multiplié par cinq**. Commande de contrôle : `grep -rn "query_as::<_, Company>\|query_as::<_, kesh_db::entities::Company>" crates/` rend **10** sites — chacun doit être rattaché à l'une des cinq listes.
- [ ] **T3 — Route dédiée** (AC3, D4) — sur le patron de `update_company_email` (`companies.rs:86`) : verrou optimiste, validation de longueur (D5). **Ne pas** toucher `PUT /companies/current/email`, **ne pas** réutiliser `update_company_coordinates` (son `is_stub = FALSE` inconditionnel et son appelant unique sont des invariants documentés).
- [ ] **T4 — Rendu PDF** (AC4, AC5) — champs sur `InvoicePdfData`, rendu conditionnel sur le patron de l'IDE, **aux deux sites de construction** (D3).
- [ ] **T5 — Garde de capacité haute** (AC6) — garde + test de pire cas. C'est la tâche à risque de la story ; la traiter **avant** le frontend.
- [ ] **T6 — i18n** (AC7) — 3 clés × 4 locales, déclarées dans `I18N_KEYS`.
- [ ] **T7 — Frontend** (AC8) — champs et texte d'aide dans les réglages société.
- [ ] **T8 — Documentation** (AC9) — manuel + PDF régénéré, CHANGELOG, vérification README **tracée**.
- [ ] **T9 — Gate complet** (AC10) — état final, exit 0, verdict lu dans le log.

## Dev Notes

### Ce que cette story ne doit PAS faire

- **Ne pas réimplémenter le récapitulatif TVA** — il est livré (PR #267). L'issue #151 le demandait à une époque où il n'existait pas.
- **Ne pas traiter le numéro de client** — c'est 16-3b.
- **Ne pas déplacer `ty = PAGE_H - 130.0`** (D1) — cette constante aligne le tableau avec la zone QR.
- **Ne pas modifier `PUT /companies/current/email`** (D4) — elle sert le flux d'envoi d'e-mail de l'Epic 20.
- **Ne pas valider le format** du téléphone ni du site web (D5).
- **Ne pas ajouter de route dédiée par champ.**

### Le piège qui coûterait le plus cher

**Oublier le second site de construction.** Une revue qui ne regarde que la facture ne verra jamais que l'avoir sort sans coordonnées. Le grep de propagation est `grep -rn "creditor_ide" crates/kesh-api/src/` — il rend **deux** sites, et c'est ce nombre qui doit être retrouvé après le patch.

### Conventions de test

Le dépôt attend qu'une story démontre que ses tests **discriminent**, pas seulement qu'ils passent : une **campagne de mutation** est la norme (cf. 16-2a et 16-2b). Pour cette story, au minimum :

- retirer le rendu d'un des trois champs → le test correspondant doit rougir, **et lui seul** ;
- retirer la garde de capacité haute → le test de pire cas d'AC6 doit rougir ;
- retirer le rendu sur l'**avoir** en le laissant sur la facture → le test d'AC5 doit rougir, et **aucun test de facture** ne doit bouger.

⚠️ **Commiter AVANT toute campagne de mutation.** `git checkout` ne restaure que ce que l'index connaît ; sur du travail non commité il **détruit** — quatre tests ont été perdus ainsi en 16-2a. À défaut, sauvegarder par copie explicite.

### References

- Issue **#151** — la source. Sa liste de champs date d'avant la livraison du récap TVA.
- **16-3b** — numéro de client, l'autre moitié. Indépendante : les deux peuvent partir séparément.
- `crates/kesh-qrbill/src/pdf.rs` — `draw_invoice_section`, bloc émetteur et chaîne verticale (D1).
- `crates/kesh-qrbill/src/types.rs` — `InvoicePdfData` et `I18N_KEYS`.
- `crates/kesh-api/src/routes/invoice_pdf_service.rs:279` et `credit_notes.rs:277` — les **deux** sites de construction (D3).
- `crates/kesh-api/src/routes/companies.rs` — route générale et route dédiée à l'e-mail (D4).
- **KF #283** — clés i18n absentes des locales non françaises ; ne pas aggraver (AC7).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

**2026-08-05 — Story créée** (`bmad-create-story`, Opus 5), née du découpage de **16-3** arbitré par Guy le même jour : 7 modules recensés contre un seuil de 5, la § *Règle de splitting préventif* imposait de scinder **avant** de lancer la spécification. Découpage **par entité** — 16-3a l'émetteur (`companies`), 16-3b le numéro de client (`contacts`) — préféré à un découpage par couche, qui aurait fait livrer à la première des colonnes que rien n'afficherait, défaut exact qui avait imposé la PR groupée de 16-2a/16-2b.

**Le relevé du schéma réel a changé la nature de la story.** #151 se lit comme une correction de mise en page ; en fait **trois des quatre champs n'existent nulle part en base**, et le quatrième — le récapitulatif TVA — **est déjà livré** depuis la PR #267. La story porte donc une migration, une route, un formulaire et un rendu, non une retouche.

**Le risque central est une garde manquante, pas un champ manquant** (D1) : le tableau des lignes démarre à une **ordonnée constante** que rien ne repousse, et la seule garde de capacité du fichier surveille le **plancher QR**, en bas. Allonger le bloc émetteur rapproche le destinataire du tableau **en silence**. D'où l'AC6, qui exige une garde symétrique **et** un test de pire cas — le cas nominal tenant déjà, il ne mesurerait rien.
