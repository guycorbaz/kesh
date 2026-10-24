# Story 31.5 : quatre défauts courts avant la release (#707, #589, #504, #469)

Status: ready-for-dev

*Fiche rédigée le 2026-10-24, en autonomie (Opus 5.5), worktree `kesh-31-5`, branche `story/31-5` sur `origin/main`
`e71cd351`. Registre : `epic-31-choix-autonomes-315.md` (C-31-5-0 à C-31-5-12). Clé courte : `315`.*

## Story

En tant que Guy, qui tient désormais sa comptabilité réelle dans Kesh, je veux que quatre défauts courts, à cause
établie, soient **réellement corrigés** avant le tag v0.15.0, afin que la démonstration valide une facture avec TVA,
que l'infobulle « Réouvrir » désigne l'exercice qu'il faut effectivement rouvrir, que la communication de la
QR-facture suive la langue du document, et que les refus du journal d'audit parlent la langue de l'installation.

## Origine

La règle de release de Guy (`CLAUDE.md` § *Priorités des défauts*) exige **moins de 136 issues ouvertes au tag
v0.15.0**. Au 2026-10-24, `gh issue list --state open --limit 500 --json number --jq length` rend **140** ; #503 se
fermera avec la 31-1c (139). Il faut donc **corriger** quatre défauts de plus (135) — ⛔ **jamais fermer un défaut réel
pour tenir le chiffre**. Six candidates ont été proposées par l'orchestrateur ; chacune a été établie au code
(§ *Sélection*), quatre sont retenues.

## Sélection — établie au code le 2026-10-24

| Issue | Prio | Cause au code | Verdict |
|---|---|---|---|
| **#707** démo : facture avec TVA refusée | P3 | `insert_with_defaults_in_tx` (`crates/kesh-db/src/repositories/company_invoice_settings.rs:1069`) n'écrit **aucun** des trois comptes de TVA ; `seed_demo` (`crates/kesh-seed/src/lib.rs:391-392`) n'appelle que lui ; la validation exige `default_vat_payable_account_id` dès que la TVA > 0 (`vat_payable_account_for_write`, même fichier, l. 612-635) | **Retenue** |
| **#589** infobulle « Réouvrir » | P3 | `nearestLaterClosed` (`frontend/src/routes/(app)/settings/fiscal-years/+page.svelte:123-133`) prend le **min** `startDate` des postérieurs clos, alors que la garde LIFO du serveur (`fiscal_years.rs:1294`, `reopen`) refuse tout exercice suivi d'un clos : seul le **plus récent** clos est rouvrable | **Retenue** |
| **#504** communication QR en français | P4 | `format!("Facture {n}")` en dur (`crates/kesh-api/src/routes/invoice_pdf_service.rs:372-376`), alors que le reste du PDF est résolu dans `locale` (`build_i18n`, l. 449) | **Retenue** |
| **#469** refus du journal d'audit non traduits | P4 | six `AppError::Validation("…français…")` dans `crates/kesh-api/src/routes/audit_log.rs` (l. 179, 194, 201, 222, 224, 239), rendus tels quels par le bras `VALIDATION_ERROR` (`crates/kesh-api/src/errors.rs:1530-1532`) | **Retenue** |
| #687 TTC fournisseur à plus de deux décimales | P3 | réelle (`supplier_invoices.rs:389`, `:415`, aucun arrondi ni borne d'échelle) | **Écartée** : l'issue elle-même n'offre « aucune [piste] mécanique » — arrondir à la création change HT et TVA saisis et doit couvrir la complétion d'une pièce importée, ou bien refondre `pay_in_tx` avec une ligne d'arrondi comme côté client (#476). Ni courte ni sans risque comptable, à trois semaines d'une comptabilité réelle (C-31-5-2) |
| #645 jetons de couleur non définis | P3 | réelle (`frontend/src/app.css`, bloc `@theme`) | **Écartée** : l'issue établit qu'elle « change l'apparence de tout l'écran » (39 fichiers `border-input`, anneaux, infobulles), réactive 19 règles `dark:` et régresse l'accessibilité avec des valeurs par défaut ; elle demande une story propre avec mesures de contraste et captures avant/après (C-31-5-2) |

**Prémisses corrigées** (à commenter sur les issues par l'orchestrateur) :

- **#707** — « démonstration seulement » est vrai du **symptôme**, pas de la cause : le chemin **production**
  (`routes/onboarding.rs:830`, finalisation) appelle la même `insert_with_defaults_in_tx` et laisse lui aussi les
  trois comptes de TVA à `NULL`. C'est **voulu** depuis la Story 18-1a (colonnes nullables, désignation par
  l'administrateur dans *Administration → Facturation*) et c'est un garde-fou pour un non-assujetti — Guy l'est : une
  facture saisie par erreur avec un taux est refusée au lieu d'écrire de la TVA due. Seule la démonstration, qui
  promet un essai « sans réglage préalable », est corrigée (C-31-5-3) ; le chemin production reçoit une phrase au
  manuel (AC3).
- **#589** — la reproduction de l'issue (« A ouvert, B clos, C clos : l'infobulle de A dit B ») est **inexacte** : le
  bouton « Réouvrir » n'existe que pour un exercice **clos** (`+page.svelte:514`, `fy.status === 'Closed'`), et avec
  A ouvert l'infobulle de B nomme C, ce qui est juste. Le défaut exige **trois exercices clos** A < B < C :
  l'infobulle de A nomme B, que le serveur refusera à son tour.
- **#504** — l'avertissement de l'issue (« vérifier qu'aucun rapprochement ne s'appuie sur le texte « Facture » ») est
  levé : le score de référence (`crates/kesh-reconciliation/src/matching.rs:231-256`) teste la **contenance** du numéro
  de facture normalisé dans la référence de la transaction ; le préfixe, quel qu'il soit, n'y entre pas
  (C-31-5-7).

**Découpage préventif** : cinq modules de premier niveau — `kesh-db` (une fonction), `kesh-seed`, `kesh-api` (deux
fichiers de route et `errors.rs`), `kesh-i18n` (catalogues), `frontend` (une page) — plus le manuel utilisateur FR.
Le critère « plus de cinq modules » n'est pas franchi ; les quatre correctifs sont **indépendants** (aucun fichier de
code commun), si bien qu'une passe de revue peut les lire un par un (C-31-5-1). Aucune migration.

## Critères d'acceptation

### A — #707 : la démonstration valide une facture avec TVA

1. **AC1 — `seed_demo` désigne les comptes de TVA du plan.** Une fonction de `kesh-db`,
   `company_invoice_settings::designate_vat_accounts_by_role_in_tx(tx, company_id)`, pose
   `default_vat_payable_account_id`, `default_vat_recoverable_account_id` et `default_vat_decompte_account_id` sur les
   comptes de la société qui portent respectivement les rôles `VatPayable`, `VatRecoverable` et `VatSettlement`, **à la
   condition** que le compte soit actif, imputable et du type que la route exige au même réglage
   (`routes/company_invoice_settings.rs:337-364` : `Liability`, `Asset`, `Liability`), lu `FOR UPDATE` comme les rôles de
   `insert_with_defaults_in_tx`. Un rôle sans compte utilisable laisse son réglage à `NULL`, sans erreur (la démonstration
   se charge quand même). Elle n'écrase **jamais** une valeur déjà posée (`COALESCE(col, ?)` ou équivalent). `final_body`
   (`crates/kesh-seed/src/lib.rs:370`) l'appelle **seulement si** `invoice_settings_created` est vrai (la ligne vient
   d'être créée par la même transaction, personne ne l'a lue : aucun `version` à bumper, C-31-5-4), juste après
   `insert_with_defaults_in_tx`. Le chemin production (`onboarding.rs`) **ne l'appelle pas**.
2. **AC2 — rouge d'abord, test HTTP.** Dans `crates/kesh-api/tests/onboarding_e2e.rs`, un test (fixtures synthétiques)
   suit le chemin démonstration jusqu'à `seed-demo`, crée un contact et une facture d'une ligne à 8,1 %, la **valide** :
   `200`, et l'écriture produite crédite le compte de rôle `VatPayable` (2200 du plan PME) du montant de TVA. Le test est
   écrit et **constaté rouge** (`400 CONFIGURATION_REQUIRED`) avant le correctif. Un second test (ou une assertion du
   même) lit `GET` des réglages de facturation après `seed-demo` et vérifie les **trois** comptes (2200, 1171, 2206 —
   rôles relus en base, pas numéros codés en dur). Un test du chemin **production** vérifie que les trois réglages
   restent `NULL` après la finalisation (garde de C-31-5-3 ; vert avant comme après).
3. **AC3 — manuel utilisateur FR.** `docs/manual/fr/user-manual.tex` l. 184 : « Les quatre taux de TVA suisses et les
   réglages de facturation par défaut, **dont les comptes de TVA** (TVA due, impôt préalable, décompte) ». Au chemin B
   (production, après l. 204) : une phrase — *les comptes de TVA ne sont pas désignés d'office ; un assujetti les désigne
   dans Administration → Facturation avant sa première facture avec TVA, sans quoi la validation est refusée (« Configuration
   incomplète : configurez les paramètres de facturation avant de valider. »)* — texte actuel de
   `error-configuration-required`, qui ne nomme ni le compte ni le menu : le manuel le dit à sa place, le message ne
   change pas (périmètre minimal). PDF régénéré (`make fr`, sous verrou) et contrôlé aplati.

### B — #589 : l'infobulle nomme l'exercice qu'il faut rouvrir

4. **AC4 — le plus récent exercice clos.** La fonction de la page devient `latestLaterClosed` (renommée : son nom
   actuel décrit le défaut) et rend, parmi les exercices clos postérieurs à `fy`, celui de **plus grande** `startDate` —
   le seul que la garde LIFO accepte de rouvrir. Son doc-comment cesse de se dire « aligné sur `find_later_closed_in_tx
   ORDER BY start_date ASC` » (le serveur n'en tire qu'une **existence**) et dit pourquoi le plus récent. Le texte de la
   clé `fiscal-year-reopen-blocked-later-closed` (« … plus récent et encore clôturé ») reste juste : **aucune clé
   i18n ne change**.
5. **AC5 — rouge d'abord, Vitest.** Dans `fiscal-years-page.test.ts`, bloc « garde LIFO client » : trois exercices clos
   2024 < 2025 < 2026 ; le bouton `fiscal-year-reopen-<2024>` est désactivé et son `title` **et** celui de son enveloppe
   contiennent « Exercice 2026 » et **pas** « Exercice 2025 » ; celui de 2025 nomme aussi 2026 ; celui de 2026 est actif,
   sans `title`. Constaté rouge avant le correctif. L'intitulé du test existant (« nommant le plus proche postérieur
   clos », l. 112) est renommé.
6. **AC6 — inventaire clos des sites qui nomment un exercice à rouvrir** (établi le 2026-10-24) : (a) l'infobulle de la
   page — **corrigée** ; (b) `error-fiscal-year-reopen-blocked` (serveur, `routes/fiscal_years.rs:190-193`) — ne nomme
   aucun exercice, dit « en commençant par le plus récent » : juste ; (c) `fiscal-year-out-of-order-warning` — nomme
   `{ $latest }` : juste ; (d) `error-later-fiscal-year-closed` et
   `journal-entries-modify-blocked-later-fiscal-year-closed` — nomment l'exercice clos qui **bloque l'écriture** (le plus
   proche, à juste titre) et prescrivent « en commençant par le plus récent » : justes. Le commentaire
   `fiscal-years.helpers.ts:84` qui cite `nearestLaterClosed` suit le renommage. Commande de contrôle :
   `grep -rn "nearestLaterClosed" frontend/src` → **zéro** ligne.
7. **AC7 — captures.** Thème clair, base de démonstration (synthétique) à trois exercices clos : l'écran *Administration
   → Exercices comptables* montrant les deux boutons désactivés et le bouton actif, jointe au Dev Agent Record. Une
   infobulle native (`title`) ne figure pas dans une capture : sa valeur se lit dans le DOM (`getAttribute('title')`) et
   se recopie au Dev Agent Record (C-31-5-6). Manuel : `user-manual.tex:848` (« Kesh […] indique lequel rouvrir
   d'abord ») devient vrai — **aucune modification** ; le contrôler au PDF aplati.

### C — #504 : la communication de la QR-facture suit la langue du document

8. **AC8 — une clé, quatre locales.** Clé `invoice-qr-message-prefix` sans variable (`Facture` / `Rechnung` / `Fattura`
   / `Invoice`, en `fr-CH`, `de-CH`, `it-CH`, `en-CH`) — sans placeable pour qu'aucune marque d'isolation Fluent
   (U+2068/U+2069) n'entre dans la QR (C-31-5-8). `render_document` la résout dans **sa** `locale` (celle que reçoit
   déjà `build_i18n`) et la passe à `build_qrbill_inputs`, qui écrit `format!("{prefix} {n}")`, tronqué à 140 caractères
   comme aujourd'hui. La facture et un rappel rendus dans la même langue portent donc le même message (le test
   `invoice_pdf_service.rs:897-900` reste vert).
9. **AC9 — rouge d'abord.** (a) Test unitaire de `invoice_pdf_service` : `build_qrbill_inputs` avec le préfixe allemand
   → `unstructured_message == Some("Rechnung F-…")` ; (b) `crates/kesh-api/tests/invoice_frozen_pdf_e2e.rs`,
   `le_pdf_est_dans_la_langue_du_client` (l. 339) : ajouter `assert!(!pdf_contains(&pdf, "Facture"))` et retirer le
   commentaire qui contourne le défaut (l. 357-358). Les deux constatés rouges avant le correctif. *Si* `pdf_contains`
   ne voit pas le texte du récépissé/section de paiement (la QR elle-même est une image), le dev le constate et le
   remplace par une assertion sur le texte « Informations supplémentaires » ou sur `QrBillData` — il ne laisse pas une
   assertion qui passerait à vide.
10. **AC10 — les PDF déjà figés ne changent pas.** Un PDF émis (figé, Story 25-6-b) garde sa communication française
    jusqu'à un refigeage ; aucune migration, aucun refigeage d'office. Manuel `user-manual.tex:3258` : ajouter que la
    communication du QR-code suit aussi la langue de la facture, et qu'une facture émise avant la 0.15.0 garde la
    sienne. PDF régénéré.

### D — #469 : les refus du journal d'audit sont traduits

11. **AC11 — les six sites, inventaire clos.** `grep -n "AppError::Validation" crates/kesh-api/src/routes/audit_log.rs`
    rend aujourd'hui six lignes (179, 194, 201, 222, 224, 239). Chacune passe par `crate::errors::t` (sans variable) ou
    `t_args` (avec variable ; rendue `pub(crate)` dans `errors.rs:52`, qui retire déjà les marques d'isolation). Six clés
    `audit-log-filter-error-*` dans les **quatre** locales : plage inversée ; date au mauvais format (`{ $field }` — le
    détail `chrono` `({e})`, anglais, n'est plus montré) ; date hors de `[1000-01-01, 9999-12-31]` (`{ $field }`, les deux
    bornes écrites en clair) ; `entityId` non positif ; `entityId` sans `entityType` ; code trop long (`{ $field }`,
    `{ $max }`). Les noms de paramètre (`dateFrom`, `entityType`…) restent tels quels : ce sont des noms d'API. Après
    correctif, la même commande ne rend que des lignes dont l'argument est `t(` ou `t_args(`. Le code reste
    `VALIDATION_ERROR` (aucun changement d'API).
12. **AC12 — rouge d'abord.** `crates/kesh-api/tests/audit_log_e2e.rs` (application montée en `de-CH`, société en
    français, `verrou_i18n()` pris) : pour **chacun** des dix cas de `les_parametres_invalides_sont_refuses_en_400`
    (l. 346), le message est **égal** au rendu `de-CH` de la clé attendue (calculé depuis le catalogue chargé, marques
    d'isolation retirées — pas un texte allemand codé en dur) **et différent** du rendu `fr-CH` de la même clé (attrape
    une valeur allemande restée une copie du français). Constaté rouge avant le correctif. Le test existant
    `une_date_hors_plage_…` (l. 381, exige `1000-01-01` et `9999-12-31` dans le message) reste vert.

### E — transverses

13. **AC13 — i18n.** `crates/kesh-i18n` : la garde de parité (`loader.rs:771`, `parity_between_locales`) verte ; les
    sept clés nouvelles (six de l'AC11, une de l'AC8) présentes dans les quatre catalogues ; `npm run
    lint-i18n-ownership` vert (aucune clé frontend ne change).
14. **AC14 — gates au dernier commit de code.** `kesh-db` est touché (une fonction de dépôt) : **gate complet même en
    boucle de revue** (`CLAUDE.md` § *Exception kesh-db*) — `scripts/test-fast.sh` sous verrou, base remise à zéro
    d'abord ; les quatre gates frontend ; **E2E complet** (frontend et backend changent), jugé fichier par fichier
    contre `docs/testing.md` § *Les échecs attendus*. Ne déclarer que ce qui a tourné.
15. **AC15 — mutations** (sous verrou, restauration `git checkout -- f && touch f`) : chacune doit faire rougir au moins
    un test nommé.
    - M1 (#589) : `>` → `<` dans la sélection de `latestLaterClosed` → AC5 rouge.
    - M2 (#707) : retirer l'appel de `designate_vat_accounts_by_role_in_tx` dans `final_body` → AC2 rouge.
    - M3 (#707) : `VatPayable` → `VatSettlement` dans la fonction (deux passifs, le type ne le refuse pas) → l'assertion
      « crédite le compte de rôle `VatPayable` » rouge.
    - M4 (#707) : retirer la condition `invoice_settings_created` → aucun test ne rougit ? Si oui, la mutation est
      **équivalente** dans la démonstration (la ligne est toujours neuve à l'étape 2) : la déclarer telle, avec la raison.
    - M5 (#504) : rétablir `"Facture"` littéral → AC9 (a) et (b) rouges.
    - M6 (#469) : laisser **un** des six sites en français (le dernier, `parse_code`) → AC12 rouge sur le cas `action`.
16. **AC16 — symptôme grepé, valeur comprise.** Après chaque patch : `grep -rn 'format!("Facture' crates` → zéro hors
    tests de fixtures fournisseur (`supplier_invoices.rs:816`, `qr_decode.rs:225`, scans de factures **reçues**, sans
    rapport) ; `grep -rn "nearestLaterClosed" frontend/src` → zéro ; `grep -n "AppError::Validation(" audit_log.rs |
    grep -v "t(\|t_args("` → zéro.

## Tâches

- [ ] **T0** — Brancher n'est pas nécessaire (worktree `kesh-31-5`, branche `story/31-5`) ; base `kesh_315`,
  `kesh_e2e_315`, port attribué par l'orchestrateur, `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-31-5/target`
  (`docs/agents-orchestration.md` § 4).
- [ ] **T1 (#589, AC4-AC7)** — test rouge (AC5) ; `latestLaterClosed` ; commentaire du helper ; Vitest ciblé.
- [ ] **T2 (#707, AC1-AC2)** — tests rouges (AC2) ; `designate_vat_accounts_by_role_in_tx` (doc-comment : pourquoi la
  démonstration seulement, C-31-5-3) ; appel dans `final_body` ; gate **complet** (kesh-db).
- [ ] **T3 (#504, AC8-AC10)** — tests rouges (AC9) ; clé `invoice-qr-message-prefix` × 4 ; résolution dans
  `render_document`, paramètre de `build_qrbill_inputs` (ses sept appels de test, l. 705-943, reçoivent le préfixe
  `"Facture"`) ; commentaire de l. 344-347 (« ne dépendent PAS du document ») complété : *ils dépendent de la langue*.
- [ ] **T4 (#469, AC11-AC12)** — test rouge (AC12) ; `t_args` en `pub(crate)` ; six clés × 4 ; six sites.
- [ ] **T5 (AC3, AC7, AC10)** — manuel FR (trois retouches), `make fr` sous verrou, contrôle aplati :
  `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oF "dont les comptes de TVA"` (et
  les deux autres phrases) ; captures (AC7).
- [ ] **T6 (AC14-AC16)** — mutations M1-M6, gate complet, frontend, E2E complet au dernier commit de code ; Dev Agent
  Record : chiffres réellement exécutés.

## Dev Notes

### #707 — pourquoi la démonstration seulement, et comment

- La ligne `company_invoice_settings` de la démonstration est créée par `insert_with_defaults_in_tx`
  (`company_invoice_settings.rs:1069-1192`) : rôles `Receivable`, `DefaultRevenue`, `Payable` lus `FOR UPDATE`, puis
  `chart_designated_accounts` (arrondi, escompte, frais, perte). **Aucun** des trois comptes de TVA. Les colonnes
  viennent de `20260614000001_vat_accounts_config.sql`, nullables, et la Story 18-1a a laissé leur désignation à
  l'administrateur (`18-1a-comptes-tva.md:109-110`).
- Les rôles existent dans les trois plans livrés (`crates/kesh-core/assets/charts/{pme,independant,association}.json` :
  1171 `VatRecoverable` Asset, 2200 `VatPayable` Liability, 2206 `VatSettlement` Liability) ; la démonstration charge
  le plan PME (`user-manual.tex:182`).
- Exigences à l'usage : TVA due — `vat_payable_account_for_write` (actif, imputable, de la société) ; impôt préalable —
  `supplier_invoices.rs:149-152` (`ConfigurationRequired` si absent). Exigences à la désignation par la route :
  `routes/company_invoice_settings.rs:337-364` (types). La nouvelle fonction applique **les deux** (actif, imputable,
  type) : elle ne pose rien que la route aurait refusé.
- **Ordre des verrous** : `final_body` tient déjà l'état d'onboarding (`lock_state_in_tx`), puis `insert_with_defaults_in_tx`
  verrouille des comptes par rôle ; la nouvelle fonction verrouille trois comptes de plus, dans l'ordre fixe
  `VatPayable`, `VatRecoverable`, `VatSettlement`, puis met à jour la ligne de réglages créée par la même transaction.
  Aucune autre transaction ne peut tenir cette ligne (elle n'existait pas avant le `INSERT`). Le rejeu de
  `final_attempt` sur interblocage (`is_seed_retryable`) rejoue toute la transaction : la fonction est idempotente
  (`COALESCE`).
- Le journal d'audit : l'entrée de synthèse `installation.demo_seeded` n'est **pas** enrichie (périmètre minimal,
  C-31-5-5).
- `test_fixtures.rs:151-170` désigne déjà les comptes de TVA pour les tests : rien à y changer.

### #589 — la garde LIFO

- Serveur : `reopen` (`fiscal_years.rs:1258`) refuse si `find_later_closed_in_tx` trouve **un** clos postérieur ; la
  requête (`FIND_LATER_CLOSED_SQL`, l. 890-893, `ORDER BY start_date ASC LIMIT 1`) ne sert qu'à l'**existence**. Parmi
  les clos postérieurs à `fy`, le seul rouvrable est celui qui n'en a aucun après lui : le **max**.
- Comparaison de dates par chaîne `YYYY-MM-DD` (ordre lexicographique = chronologique), comme `outOfOrderState`
  (`fiscal-years.helpers.ts:86-110`, qui calcule déjà `latest` de la même façon : le dev peut réutiliser la boucle,
  DRY, sans fusionner les deux fonctions dont les entrées diffèrent).
- La sœur `earliestEarlierOpen` (clôture, l. 144-154) prend à juste titre le **min** : ne pas la toucher.

### #504 — la langue de la communication

- `render_document` (`invoice_pdf_service.rs:188-262`) reçoit `locale` : pour une facture, `client_language` de la
  facture (`issued_invoice_pdf.rs:150`, `:255`, qui tient compte de `pdf_language`) ; pour un rappel envoyé,
  `resolve_language(contact, company)` (`invoice_email.rs:546`, `:1253`). Une facture à langue **forcée** et son rappel
  peuvent donc porter deux préfixes différents : sans effet sur le rapprochement (contenance du numéro), et conforme à la
  langue de chaque document.
- Jeu de caractères SIX : les quatre préfixes sont ASCII.
- ⚠️ Fluent isole les placeables par défaut (aucun `set_use_isolating(false)` dans `crates/kesh-i18n/src`) : d'où une
  clé **sans** variable et un `format!` côté Rust (C-31-5-8).

### #469 — les refus

- `t` (`errors.rs:36`) et `t_args` (`errors.rs:52`, privée aujourd'hui) lisent le bundle global posé par
  `init_error_i18n` : la langue est celle de l'**installation**, ce que vérifie déjà le montage `de-CH` / société FR
  d'`audit_log_e2e.rs` (en-tête, l. 1-10).
- Précédent du même fichier : `RESULT_TOO_LARGE` → `audit-log-export-error-too-large` (traduit).
- Placement des clés : à côté de `audit-log-export-error-too-large` dans chaque catalogue (`fr-CH` l. 2713, `de-CH`
  l. 2651, `it-CH` l. 2652, `en-CH` l. 2657).

### Hors périmètre, et ce qui reste non résolu (inventaire)

- **Chemin production, comptes de TVA `NULL`** : voulu (C-31-5-3), documenté par l'AC3.
- **Autres `AppError::Validation` en français hors du journal d'audit** (`routes/projects.rs:107`, `:115`,
  `payment_batches.rs:176`, `contacts.rs:453`, `address_input.rs:62`, et d'autres — un `grep` grossier en compte
  289 lignes `AppError::Validation(` hors `t(`, relevé du 2026-10-24, qui surcompte les messages déjà traduits sur plusieurs lignes) : même classe de défaut, **non** couverte par #469 qui est bornée au journal d'audit,
  et non couverte par une issue ouverte à la date de la fiche. Demandée à l'orchestrateur (« on trace toujours »).
- **PDF déjà figés** en français : conservés (AC10).
- #687 et #645 : écartées (§ *Sélection*).

### Fichiers touchés (prévus)

`crates/kesh-db/src/repositories/company_invoice_settings.rs` ; `crates/kesh-seed/src/lib.rs` ;
`crates/kesh-api/src/routes/invoice_pdf_service.rs` ; `crates/kesh-api/src/routes/audit_log.rs` ;
`crates/kesh-api/src/errors.rs` (visibilité de `t_args`) ; `crates/kesh-api/tests/{onboarding_e2e,
invoice_frozen_pdf_e2e,audit_log_e2e}.rs` ; `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` ;
`frontend/src/routes/(app)/settings/fiscal-years/{+page.svelte,fiscal-years-page.test.ts}` ;
`frontend/src/lib/features/fiscal-years/fiscal-years.helpers.ts` (commentaire) ; `docs/manual/fr/user-manual.{tex,pdf}`.
CHANGELOG (`[0.15.0] — Non publié`, section *Fixed*, quatre entrées, `refs` des issues).

### Risques

- **#707 touche `kesh-db` et la transaction de la démonstration** : gate complet obligatoire ; la relecture porte sur
  les verrous et la naissance de la ligne (leçon des Epics 29-30).
- **#504 change le contenu d'un PDF émis désormais** (pas des PDF déjà figés) : la QR reste valide (même longueur
  bornée, ASCII).
- **#469** : un message attendu par un client de l'API change de texte, pas de code — aucun test frontend ne lit ces
  messages (`grep` des textes français : seuls `audit_log.rs` les porte).

### Références

`CLAUDE.md` § *Priorités des défauts*, § *Règle de splitting préventif*, § *Exception kesh-db* ;
`docs/agents-orchestration.md` § 3-7 ; `_bmad-output/planning-artifacts/epic-31-tenir-les-livres.md` ;
`18-1a-comptes-tva.md` ; issues #707, #589, #504, #469, #687, #645.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-10-24 — Fiche créée (Opus 5.5, autonomie) : quatre défauts retenus (#707, #589, #504, #469), deux écartés
  (#687, #645) ; prémisses de #707 et #589 corrigées ; choix C-31-5-0 à 12 ; validation P1 à lancer
  (`315-validate-prompt-p1.md`, Sonnet, lentilles R et F).
