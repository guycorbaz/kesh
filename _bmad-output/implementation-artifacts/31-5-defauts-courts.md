# Story 31.5 : quatre défauts courts avant la release (#707, #589, #504, #469)

Status: ready-for-dev

*Fiche rédigée le 2026-10-24, en autonomie (Opus 5.5), worktree `kesh-31-5`, branche `story/31-5` sur `origin/main`
`e71cd351`. Registre : `epic-31-choix-autonomes-315.md` (C-31-5-0 à C-31-5-17 ; 13 à 17 à la validation P1).
Clé courte : `315`.*

## Story

En tant que Guy, qui tient désormais sa comptabilité réelle dans Kesh, je veux que quatre défauts courts, à cause
établie, soient **réellement corrigés** avant le tag v0.15.0, afin que la démonstration valide une facture avec TVA,
que l'infobulle « Réouvrir » désigne l'exercice qu'il faut effectivement rouvrir, que la communication de la
QR-facture suive la langue du document, et que les refus du journal d'audit parlent la langue de l'installation.

## Origine

La règle de release de Guy (`CLAUDE.md` § *Priorités des défauts*) exige **moins de 136 issues ouvertes au tag
v0.15.0** (plafond écrit au fichier d'epic, `epic-31-tenir-les-livres.md` : 136 au tag v0.14.0).

**Recompte vivant** (validation P1, 2026-10-24, après l'ouverture de #708, #709 et #710 le même jour) :
`gh issue list --state open --limit 500 --json number --jq length` rend **141** (la fiche disait 140 à sa rédaction,
avant #710). Périmètre : toutes les issues ouvertes, à la date de la mesure, non au tag.

| Étape | Compte |
|---|---|
| ouvertes au 2026-10-24 | 141 |
| − #503 (fermée par la 31-1c) | 140 |
| − les quatre défauts de cette story (#707, #589, #504, #469) | **136** |
| + l'issue des LOW résiduels de la 31-1c, annoncée par l'orchestrateur et non encore ouverte | **137** |

**136 n'est pas sous le plafond (« moins de 136 ») ; 137 le dépasse de deux.** Cette story, à elle seule, ne tient
donc pas la règle de release : il manque **deux** retraits. L'arbitrage — corriger deux défauts de plus (dans cette
story ou une autre de l'epic), ou publier au-dessus du plafond — **revient au Project Lead** et lui est soumis par
l'orchestrateur (C-31-5-16). ⛔ **Jamais fermer un défaut réel pour tenir le chiffre**, et tout point non traité
s'ouvre en issue même s'il fait remonter le compte. Six candidates ont été proposées par l'orchestrateur ; chacune a été
établie au code (§ *Sélection*), quatre sont retenues.

## Sélection — établie au code le 2026-10-24

| Issue | Prio | Cause au code | Verdict |
|---|---|---|---|
| **#707** démo : facture avec TVA refusée | P3 | `insert_with_defaults_in_tx` (`crates/kesh-db/src/repositories/company_invoice_settings.rs:1069`) n'écrit **aucun** des trois comptes de TVA ; `seed_demo` (`crates/kesh-seed/src/lib.rs:391-392`) n'appelle que lui ; la validation d'une facture exige `default_vat_payable_account_id` dès que la TVA > 0 : `validate_invoice` le passe (`crates/kesh-db/src/repositories/invoices.rs:2364`) à `generate_invoice_journal_lines_rounded`, qui échoue en `ConfigurationRequired("default_vat_payable_account_id")` (`invoices.rs:1877-1881`), puis `designated.check_written` (`invoices.rs:2372`) exige le compte actif et imputable | **Retenue** |
| **#589** infobulle « Réouvrir » | P3 | `nearestLaterClosed` (`frontend/src/routes/(app)/settings/fiscal-years/+page.svelte:123-133`) prend le **min** `startDate` des postérieurs clos, alors que la garde LIFO du serveur (`crates/kesh-db/src/repositories/fiscal_years.rs:1294`, dans `reopen`) refuse tout exercice suivi d'un clos : seul le **plus récent** clos est rouvrable | **Retenue** |
| **#504** communication QR en français | P4 | `format!("Facture {n}")` en dur (`crates/kesh-api/src/routes/invoice_pdf_service.rs:372-376`), alors que le reste du PDF est résolu dans `locale` (`build_i18n`, l. 449) | **Retenue** |
| **#469** refus du journal d'audit non traduits | P4 | six `AppError::Validation("…français…")` dans `crates/kesh-api/src/routes/audit_log.rs` (l. 179, 194, 201, 222, 224, 239), rendus tels quels par le bras `VALIDATION_ERROR` (`crates/kesh-api/src/errors.rs:1530-1532`) | **Retenue** |
| #687 TTC fournisseur à plus de deux décimales | P3 | réelle (`supplier_invoices.rs:389`, `:415`, aucun arrondi ni borne d'échelle) | **Écartée** : l'issue elle-même n'offre « aucune [piste] mécanique » — arrondir à la création change HT et TVA saisis et doit couvrir la complétion d'une pièce importée, ou bien refondre `pay_in_tx` avec une ligne d'arrondi comme côté client (#476). Ni courte ni sans risque comptable, à trois semaines d'une comptabilité réelle (C-31-5-2) |
| #645 jetons de couleur non définis | P3 | réelle (`frontend/src/app.css`, bloc `@theme`) | **Écartée** : l'issue établit qu'elle « change l'apparence de tout l'écran » (bordures `border-input`, anneaux, infobulles ; « 39 fichiers » et « 19 règles `dark:` » sont les comptes **de l'issue** : recomptés le 2026-10-24, `grep -rl "border-input" frontend/src | wc -l` rend 10 et `grep -rno "dark:" frontend/src | wc -l` rend 37 occurrences dans 9 fichiers — le motif d'écart ne dépend pas du nombre), réactive des règles `dark:` et régresse l'accessibilité avec des valeurs par défaut ; elle demande une story propre avec mesures de contraste et captures avant/après (C-31-5-2) |

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

**Découpage préventif** : cinq modules de premier niveau — `kesh-db` (deux fonctions, C-31-5-13), `kesh-seed`, `kesh-api` (deux
fichiers de route et `errors.rs`), `kesh-i18n` (catalogues), `frontend` (une page) — plus le manuel utilisateur FR.
Le critère « plus de cinq modules » n'est pas franchi ; les quatre correctifs sont **indépendants** (aucun fichier de
code commun), si bien qu'une passe de revue peut les lire un par un (C-31-5-1). Aucune migration.

## Critères d'acceptation

### A — #707 : la démonstration valide une facture avec TVA

1. **AC1 — `seed_demo` désigne les comptes de TVA du plan, comptes verrouillés AVANT les réglages.** Deux fonctions
   de `kesh-db` (`crates/kesh-db/src/repositories/company_invoice_settings.rs`), pour tenir l'ordre documenté
   `accounts → company_invoice_settings` (`docs/MULTI-TENANT-SCOPING-PATTERNS.md:326`, ligne `seed_demo` du Pattern 5 ;
   doc de `final_body`, `crates/kesh-seed/src/lib.rs:362-367`) — C-31-5-13 :
   - `lock_vat_role_accounts_in_tx(tx, company_id) -> VatRoleAccounts` : **une** lecture verrouillante
     (`SELECT id, singleton_role, account_type, active, postable FROM accounts WHERE company_id = ? AND singleton_role IN
     (?, ?, ?) ORDER BY id FOR UPDATE` — ordre des verrous par clé primaire), qui retient pour chacun des rôles
     `VatPayable`, `VatRecoverable` et `VatSettlement` le compte **à la condition** qu'il soit actif, imputable et du
     type que la route exige au même réglage (`crates/kesh-api/src/routes/company_invoice_settings.rs:337-364` :
     `Liability`, `Asset`, `Liability`) ; un rôle sans compte utilisable donne `None`, sans erreur (la démonstration se
     charge quand même). Elle **ne lit ni n'écrit** `company_invoice_settings`.
   - `designate_vat_accounts_in_tx(tx, company_id, &VatRoleAccounts)` : un seul `UPDATE company_invoice_settings SET
     default_vat_payable_account_id = COALESCE(default_vat_payable_account_id, ?), … WHERE company_id = ?` ; elle **ne
     lit aucun compte** (aucun verrou de compte pris après la ligne de réglages) et n'écrase **jamais** une valeur posée.
   - `final_body` appelle la première **juste avant** `insert_with_defaults_in_tx` (après `clear_stub_in_tx`), la seconde
     juste après, **seulement si** `invoice_settings_created` est vrai (la ligne vient d'être créée par la même
     transaction, personne ne l'a lue : aucun `version` à bumper, C-31-5-4). La séquence devient `onboarding_state →
     companies → accounts (TVA, puis rôles de insert_with_defaults_in_tx) → company_invoice_settings → vat_rates →
     audit_log` : la ligne `seed_demo` du Pattern 5 et le doc-comment de `final_body` le disent (même PR). Le chemin
     production (`crates/kesh-api/src/routes/onboarding.rs`) **n'appelle ni l'une ni l'autre**.
2. **AC2 — rouge d'abord, test HTTP.** Dans `crates/kesh-api/tests/onboarding_e2e.rs`, un test (fixtures synthétiques)
   suit le chemin démonstration jusqu'à `seed-demo`, crée un contact et une facture d'une ligne à 8,1 %, la **valide** :
   `200`, et l'écriture produite crédite le compte de rôle `VatPayable` (2200 du plan PME) du montant de TVA. Le test est
   écrit et **constaté rouge** (`400 CONFIGURATION_REQUIRED`) avant le correctif. Un second test (ou une assertion du
   même) lit `GET` des réglages de facturation après `seed-demo` et vérifie les **trois** comptes (2200, 1171, 2206 —
   rôles relus en base, pas numéros codés en dur). Un test du chemin **production**, dans
   `crates/kesh-api/tests/onboarding_path_b_e2e.rs` (qui monte déjà la finalisation : `full_path_b_flow`, l. 216 —
   `onboarding_e2e.rs` ne porte que le chemin démonstration), vérifie que les trois réglages restent `NULL` après la
   finalisation (garde de C-31-5-3 ; vert avant comme après).
3. **AC3 — manuel utilisateur FR.** `docs/manual/fr/user-manual.tex` l. 185 : « Les quatre taux de TVA suisses et les
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
   clos », l. 113) est renommé.
6. **AC6 — inventaire clos des textes qui prescrivent de rouvrir un exercice** (établi le 2026-10-24, validation P1).
   **Critère de l'ensemble** : les lignes du catalogue `fr-CH` que rend
   `grep -nE "rouvr|Rouvr|réouv|Réouv" crates/kesh-i18n/locales/fr-CH/messages.ftl` — **30** lignes, dont deux
   commentaires (l. 969, 1463) —, plus le code client qui calcule un exercice à nommer. Tri, sans reste :
   - (a) **nomme l'exercice à rouvrir d'abord** : `fiscal-year-reopen-blocked-later-closed` (l. 976 ; la valeur
     `{ $name }` vient de la page) — **corrigée** par l'AC4 ; `fiscal-year-out-of-order-warning` (l. 952) nomme
     `{ $latest }`, que `outOfOrderState` calcule déjà comme le **max** des clos (`fiscal-years.helpers.ts:88-91`) :
     juste.
   - (b) **ne nomme aucun exercice à rouvrir et prescrit « en commençant par le plus récent »** (ils peuvent nommer
     l'exercice qui **bloque** ou le premier exercice, jamais celui à rouvrir) : l. 423, 424, 862, 863, 871, 872, 1035
     (`error-fiscal-year-reopen-blocked`, servie par `crates/kesh-api/src/routes/fiscal_years.rs:190-193`), 1040, 1069,
     1077, 2213, 2214 — justes.
   - (c) **réouverture sans ordre à prescrire** (bouton, dialogue, motif) : l. 964, 970-974, 1032, 1033 — sans objet.
   - (d) **hors famille** (compte bancaire, aperçu de rappel) : l. 1314, 1433, 1465, 1470, 1474, 2063.
   - Hors critère, relevées par la validation P1 : `reconciliation-failed-later-fiscal-year-closed{,-generic}`
     (l. 1321-1322) nomment l'exercice qui bloque et ne prescrivent aucune réouverture : sans objet.
   - Code client : `grep -rniE "laterClosed|later_closed|laterFiscal|blockingFiscal" frontend/src` ne rend que
     `+page.svelte` et `fiscal-years.helpers.ts` — aucun autre calcul du « clos à nommer ».
   Le renommage touche **quatre** sites : la définition (`+page.svelte:123`), l'appel (`+page.svelte:515`), le
   doc-comment de `earliestEarlierOpen` (`+page.svelte:137`, « Sœur de `nearestLaterClosed` ») et le commentaire
   `fiscal-years.helpers.ts:84`. Commande de contrôle : `grep -rn "nearestLaterClosed" frontend/src` → **zéro** ligne.
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
    `{ $max }`). Les noms de paramètre (`dateFrom`, `entityType`…) restent tels quels : ce sont des noms d'API. Le code
    reste `VALIDATION_ERROR` (aucun changement d'API). Chaque repli Rust (`default` de `t`/`t_args`) égale la valeur
    `fr-CH` de sa clé, variables réécrites (`{ $field }` → `{nom}`, `{ $max }` → `{maximum}`, les noms des paramètres
    actuels de `parse_date`/`parse_code`) : les six clés entrent à la garde **G9**
    (`crates/kesh-api/tests/textes_coherents.rs`, `les_replis_rust_suivent_le_catalogue`, l. 680 — une quatrième
    entrée `crates/kesh-api/src/routes/audit_log.rs` dans `TABLE`, dont le type passe de `[…; 3]` à `[…; 4]`), C-31-5-15.
    Le **contrôle de fin** est celui de l'AC16 (multi-ligne) ; le **garde** est le test de l'AC12, seul à prouver
    que le message rendu est traduit.
12. **AC12 — rouge d'abord.** `crates/kesh-api/tests/audit_log_e2e.rs` (application montée en `de-CH`, société en
    français, `verrou_i18n()` pris) : pour **chacun** des dix cas de `les_parametres_invalides_sont_refuses_en_400`
    (l. 346), le message est **égal** au rendu `de-CH` de la clé attendue (calculé depuis le catalogue chargé, marques
    d'isolation retirées — pas un texte allemand codé en dur) **et différent** du rendu `fr-CH` de la même clé (attrape
    une valeur allemande restée une copie du français). Constaté rouge avant le correctif. Le test existant
    `une_date_hors_plage_…` (l. 381, exige `1000-01-01` et `9999-12-31` dans le message) reste vert.

### E — transverses

13. **AC13 — i18n.** `crates/kesh-i18n` : la garde de parité (`loader.rs:771`, `parity_between_locales`) verte ; les
    sept clés nouvelles (six de l'AC11, une de l'AC8) présentes dans les quatre catalogues ; G9 vert avec les six clés
    du journal d'audit (AC11) — la clé de l'AC8 n'y entre pas : elle est résolue par `bundle.format` dans `render_document`,
    sans repli Rust ; `npm run lint-i18n-ownership` vert (aucune clé frontend ne change).
14. **AC14 — gates au dernier commit de code.** `kesh-db` est touché (deux fonctions de dépôt) : **gate complet même en
    boucle de revue** (`CLAUDE.md` § *Exception kesh-db*) — `scripts/test-fast.sh` sous verrou, base remise à zéro
    d'abord ; les quatre gates frontend ; **E2E complet** (frontend et backend changent), jugé fichier par fichier
    contre `docs/testing.md` § *Les échecs attendus*. Ne déclarer que ce qui a tourné.
15. **AC15 — mutations** (sous verrou, restauration `git checkout -- f && touch f`) : chacune doit faire rougir au moins
    un test nommé.
    - M1 (#589) : dans `latestLaterClosed`, la comparaison **de mise à jour du meilleur**
      (`other.startDate > best.startDate`) → `<` (c'est rétablir le défaut) → AC5 rouge (l'infobulle de 2024 nomme
      2025). La comparaison de filtre (`other.startDate > fy.startDate`) n'est pas la mutation visée.
    - M2 (#707) : retirer l'appel de `designate_vat_accounts_in_tx` dans `final_body` → AC2 rouge.
    - M3 (#707) : dans `lock_vat_role_accounts_in_tx`, attribuer au réglage de TVA due le compte du rôle
      `VatSettlement` au lieu de `VatPayable` (deux passifs, le type ne le refuse pas) → l'assertion
      « crédite le compte de rôle `VatPayable` » rouge.
    - M4 (#707) : retirer la condition `invoice_settings_created` → aucun test ne rougit ? Si oui, la mutation est
      **équivalente** dans la démonstration (la ligne est toujours neuve à l'étape 2) : la déclarer telle, avec la raison.
    - M5 (#504) : rétablir `"Facture"` littéral → AC9 (a) et (b) rouges.
    - M6 (#469) : laisser **un** des six sites en français (le dernier, `parse_code`) → AC12 rouge sur le cas `action`.
16. **AC16 — symptôme grepé, valeur comprise.** Commandes lancées depuis la racine du worktree (chemins relatifs à
    elle), après chaque patch — C-31-5-14 :
    - **#504.** `grep -rn 'format!("Facture' crates` rend aujourd'hui **deux** lignes (inventaire du 2026-10-24) :
      `crates/kesh-api/src/routes/invoice_pdf_service.rs:373` (le site corrigé) et
      `crates/kesh-db/src/repositories/supplier_invoices.rs:512` (`format!("Facture fournisseur {number_label} -
      {contact_name}")`, **libellé d'écriture comptable** d'une facture fournisseur : code de production, ni QR ni
      document adressé au débiteur — hors classe de #504, non résolu assumé, cf. *Hors périmètre*). Contrôle
      décidable : `grep -rn 'format!("Facture' crates | grep -v '^crates/kesh-db/src/repositories/supplier_invoices.rs:'`
      → **zéro** ligne, et `grep -c 'format!("Facture fournisseur' crates/kesh-db/src/repositories/supplier_invoices.rs`
      → **1** (l'exception reste seule et nommée ; un second site dans ce fichier rougirait ce compte). Les littéraux
      sans `format!` (`Some("Facture 42")` en test de `crates/kesh-api/src/routes/supplier_invoices.rs:816`,
      `Some("Facture test")` en test de `crates/kesh-api/src/qr_decode.rs:225`, et les fixtures de `kesh-qrbill`) ne
      sont pas attrapés par le motif et ne sont pas des exceptions : ce sont des QR **reçues** de test.
    - **#589.** `grep -rn "nearestLaterClosed" frontend/src` → zéro.
    - **#469.** Contrôle **multi-ligne**, insensible à la mise en forme de rustfmt (l'argument de
      `AppError::Validation(` est aujourd'hui sur la ligne suivante aux l. 179, 194, 201) :
      `tr '\n' ' ' < crates/kesh-api/src/routes/audit_log.rs | grep -oE 'AppError::Validation\( *[A-Za-z_:"]+'` rend
      **six** jetons — aujourd'hui trois `"…` et trois `format` — et, après correctif,
      `… | grep -vE '\( *(crate::errors::)?t(_args)?$'` → **zéro** (chaque argument commence par `t(` ou `t_args(`,
      qualifiés ou non). Ce contrôle dit la **forme** ; le **garde** reste le test de l'AC12, qui dit que le message
      rendu est traduit.

## Tâches

- [ ] **T0** — Brancher n'est pas nécessaire (worktree `kesh-31-5`, branche `story/31-5`) ; base `kesh_315`,
  `kesh_e2e_315`, port attribué par l'orchestrateur, `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-31-5/target`
  (`docs/agents-orchestration.md` § 4).
- [ ] **T1 (#589, AC4-AC7)** — test rouge (AC5) ; `latestLaterClosed` ; commentaire du helper ; Vitest ciblé.
- [ ] **T2 (#707, AC1-AC2)** — tests rouges (AC2, dont le test production dans `onboarding_path_b_e2e.rs`) ;
  `lock_vat_role_accounts_in_tx` et `designate_vat_accounts_in_tx` (doc-comment : pourquoi la démonstration seulement,
  C-31-5-3, et l'ordre des verrous, C-31-5-13) ; appels dans `final_body` (la lecture avant
  `insert_with_defaults_in_tx`, l'application après) ; doc-comment de `final_body` et ligne `seed_demo` de
  `docs/MULTI-TENANT-SCOPING-PATTERNS.md` (l. 326) mis à jour ; gate **complet** (kesh-db).
- [ ] **T3 (#504, AC8-AC10)** — tests rouges (AC9) ; clé `invoice-qr-message-prefix` × 4 ; résolution dans
  `render_document`, paramètre de `build_qrbill_inputs` (ses sept appels de test, l. 705-943, reçoivent le préfixe
  `"Facture"`) ; commentaire de l. 344-347 (« ne dépendent PAS du document ») complété : *ils dépendent de la langue*.
- [ ] **T4 (#469, AC11-AC12)** — test rouge (AC12) ; `t_args` en `pub(crate)` ; six clés × 4 ; six sites ; les six
  clés à la garde G9 (`textes_coherents.rs`, C-31-5-15).
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
  le plan PME (`user-manual.tex:183`).
- Exigences à l'usage, **à la validation d'une facture** (le chemin du refus de #707) : TVA due — absente →
  `ConfigurationRequired("default_vat_payable_account_id")` dans `generate_invoice_journal_lines_rounded`
  (`crates/kesh-db/src/repositories/invoices.rs:1877-1881`, appelée l. 2364), puis `designated.check_written`
  (`invoices.rs:2372` → `company_invoice_settings.rs:954-974` : actif et imputable, sinon `InactiveOrInvalidAccounts` ou
  `DesignatedAccountsNotPostable`) ; impôt préalable, à la validation d'une facture fournisseur —
  `crates/kesh-db/src/repositories/supplier_invoices.rs:149-152` (`ConfigurationRequired` si absent). *Même exigence,
  autre usage* : `vat_payable_account_for_write` (`company_invoice_settings.rs:612`) ne sert qu'à l'écart d'escompte ou
  de perte d'un règlement (`invoice_settlements_write.rs:601`), pas à la validation. Exigences à la désignation par la
  route : `crates/kesh-api/src/routes/company_invoice_settings.rs:337-364` (types). La nouvelle lecture applique **les
  deux** (actif, imputable, type) : elle ne pose rien que la route aurait refusé ni que `check_written` refuserait.
- **Ordre des verrous** (C-31-5-13) : `final_body` tient l'état d'onboarding (`lock_state_in_tx`), puis la société
  (`clear_stub_in_tx`) ; `lock_vat_role_accounts_in_tx` verrouille les trois comptes de TVA (une requête, `ORDER BY
  id`) ; `insert_with_defaults_in_tx` verrouille ses comptes de rôle (`company_invoice_settings.rs:1075-1105`) puis
  insère la ligne de réglages (`INSERT IGNORE`, l. 1126) ; `designate_vat_accounts_in_tx` ne fait qu'un `UPDATE` de
  cette ligne, **sans** lire de compte. Aucun verrou de compte n'est donc pris après la ligne de réglages : l'ordre
  documenté `accounts → company_invoice_settings` est tenu, sans exception à justifier. La forme écartée (verrouiller
  les comptes de TVA après l'`INSERT`) donnait `accounts → settings → accounts`, contraire au Pattern 5, et ne se
  défendait que par l'absence de concurrent sur une ligne neuve — pas sur les comptes. Les trois comptes sont verrouillés
  même quand la ligne existait déjà (`invoice_settings_created` faux, cas que l'étape 2 ne produit pas) : verrou inutile
  mais inoffensif, relâché au commit. Le rejeu de `final_attempt` sur interblocage (`is_seed_retryable`) rejoue toute
  la transaction : les deux fonctions sont idempotentes (`COALESCE`).
- Le journal d'audit : l'entrée de synthèse `installation.demo_seeded` n'est **pas** enrichie (périmètre minimal,
  C-31-5-5).
- `test_fixtures.rs:151-170` désigne déjà les comptes de TVA pour les tests : rien à y changer.

### #589 — la garde LIFO

- Serveur : `reopen` (`crates/kesh-db/src/repositories/fiscal_years.rs:1258`) refuse si `find_later_closed_in_tx`
  (même fichier, l. 1294) trouve **un** clos postérieur ; la requête (`FIND_LATER_CLOSED_SQL`, même fichier, l. 890-893,
  `ORDER BY start_date ASC LIMIT 1`) ne sert qu'à l'**existence**. Le refus est rendu par la route
  `crates/kesh-api/src/routes/fiscal_years.rs:190-193` (homonyme, autre crate). Parmi
  les clos postérieurs à `fy`, le seul rouvrable est celui qui n'en a aucun après lui : le **max**.
- Comparaison de dates par chaîne `YYYY-MM-DD` (ordre lexicographique = chronologique), comme `outOfOrderState`
  (`fiscal-years.helpers.ts:86-110`, qui calcule déjà `latest` de la même façon : le dev peut réutiliser la boucle,
  DRY, sans fusionner les deux fonctions dont les entrées diffèrent).
- La sœur `earliestEarlierOpen` (clôture, l. 144-154) prend à juste titre le **min** : ne pas la toucher.

### #504 — la langue de la communication

- `render_document` (`invoice_pdf_service.rs:188-262`) reçoit `locale` : pour une facture, `client_language` de la
  facture (appels de `render_document` à `issued_invoice_pdf.rs:151` et `:256`, la langue lue juste avant, qui tient
  compte de `pdf_language`) ; pour un rappel envoyé, `resolve_language(contact, company)` (langue à
  `invoice_email.rs:546` et `:1253`, appels à `:560` et `:1260`). Une facture à langue **forcée** et son rappel
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
  289 lignes `AppError::Validation(` hors `t(`, relevé du 2026-10-24, qui surcompte les messages déjà traduits sur plusieurs lignes) : même classe de défaut, **non** couverte par #469 qui est bornée au journal d'audit :
  tracée par **#710**, ouverte le 2026-10-24 (comprise dans le recompte de 141 de l'*Origine*), C-31-5-12.
- **Libellés d'écriture comptable en français en dur** — `crates/kesh-db/src/repositories/supplier_invoices.rs:512`
  (« Facture fournisseur … », seule exception du contrôle #504 de l'AC16), `:941` (« Règlement fournisseur … »),
  `invoice_settlements_write.rs:277` (« Règlement facture … »), `credit_notes.rs:590` (« Avoir … (annule …) ») : ce
  sont des descriptions d'écritures du journal, non des textes adressés au débiteur ; hors classe de #504 et hors
  périmètre. Aucune issue ouverte ne les couvre (`gh issue list` du 2026-10-24) ; la question de savoir s'ils doivent
  suivre la langue comptable de la société est **soumise à l'orchestrateur**, qui ouvre l'issue si oui (C-31-5-17).
- **PDF déjà figés** en français : conservés (AC10).
- #687 et #645 : écartées (§ *Sélection*).

### Fichiers touchés (prévus)

`crates/kesh-db/src/repositories/company_invoice_settings.rs` ; `crates/kesh-seed/src/lib.rs` ;
`crates/kesh-api/src/routes/invoice_pdf_service.rs` ; `crates/kesh-api/src/routes/audit_log.rs` ;
`crates/kesh-api/src/errors.rs` (visibilité de `t_args`) ; `crates/kesh-api/tests/{onboarding_e2e,
onboarding_path_b_e2e,invoice_frozen_pdf_e2e,audit_log_e2e,textes_coherents}.rs` ;
`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` ;
`frontend/src/routes/(app)/settings/fiscal-years/{+page.svelte,fiscal-years-page.test.ts}` ;
`frontend/src/lib/features/fiscal-years/fiscal-years.helpers.ts` (commentaire) ; `docs/manual/fr/user-manual.{tex,pdf}` ;
`docs/MULTI-TENANT-SCOPING-PATTERNS.md` (l. 326, ligne `seed_demo`). `docs/migrations-idempotence-audit.md` n'est pas
touché (aucune migration).
CHANGELOG : l'entrée `[0.15.0] — Non publié` n'a aujourd'hui que `### Ajouté`, `### Modifié` et `### Interne, invisible
de l'utilisateur` (l. 13, 40, 60) ; une section **`### Corrigé`** (intitulé des versions antérieures, ex. l. 102) y est
**à créer**, avant `### Interne…`, avec quatre entrées et les `refs` des issues.

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
- 2026-10-24 — **Validation P1** (Sonnet, deux lentilles en contexte frais ; rapports hors dépôt
  `kesh-gate-logs/315-validate-p1-{R,F}.md`). Lentille R : 0 CRITICAL / 0 HIGH / 4 MEDIUM / 5 LOW (R1-R4 MEDIUM,
  R5-R9 LOW ; son en-tête annonce « MEDIUM 3 · LOW 6 », son corps classe R4 en MEDIUM — retenu tel que dans le corps).
  Lentille F : 0 / 0 / 3 MEDIUM (F1-F3) / 7 LOW (F4-F10). Recoupements : R1 = F2/F3, R2 = F5, R3 = F1, R4 = F4 (R4
  MEDIUM, F4 LOW : traité au plus haut). Sort, chaque affirmation revérifiée au code avant écriture :
  - **R1/F2/F3** (contrôles de l'AC11 et de l'AC16 non décidables) — corrigés : inventaire des deux `format!("Facture`
    (`invoice_pdf_service.rs:373` corrigé, `supplier_invoices.rs:512` exception nommée et comptée), renvois faux
    `:816`/`qr_decode.rs:225` remplacés par ce qu'ils sont ; contrôle d'`audit_log.rs` multi-ligne par aplatissement,
    essayé contre le code (6 jetons aujourd'hui, 0 sur un exemple corrigé) ; l'AC11 renvoie à l'AC16, garde = AC12
    (C-31-5-14).
  - **R2/F5** (site de validation de #707) — corrigé : `invoices.rs:1877-1881`, `:2364`, `check_written` `:2372` ;
    `vat_payable_account_for_write` ramené à « même exigence, autre usage » (`invoice_settlements_write.rs:601`).
  - **R3/F1/R9** (budget d'issues) — corrigé : recompte 141, 136 après la story, 137 avec l'issue annoncée de la 31-1c ;
    déficit de deux déclaré, arbitrage au Project Lead (C-31-5-16) ; #710 citée comme ouverte (C-31-5-12 annoté).
  - **R4/F4** (ordre des verrous) — corrigé par la forme la plus sûre : comptes de TVA verrouillés avant la ligne de
    réglages, application sans lecture de compte ; Pattern 5 l. 326 et doc de `final_body` à préciser (C-31-5-13).
  - **LOW** — R5 (l. 185, l. 113, appels `render_document` l. 151/256 et 560/1260, chemins complets des deux
    `fiscal_years.rs`), R6 (AC6 clos sur le critère `rouvr|réouv` du catalogue fr-CH, 30 lignes triées), R7 (M1 nomme la
    comparaison de mise à jour du meilleur), R8 (comptes de #645 attribués à l'issue, recomptés 10 et 37), F6 (`### Corrigé`
    à créer), F7 (`+page.svelte:137` et `:515` dans l'inventaire du renommage), F8 (`onboarding_path_b_e2e.rs`), F9 (G9
    étendue aux six clés, C-31-5-15), F10 (= R5, chemins) : tous corrigés. Ajout hors findings : libellés d'écriture en
    français inventoriés au hors-périmètre, issue soumise à l'orchestrateur (C-31-5-17).
  - Propagation : symptômes grepés dans la fiche et le registre (`designate_vat_accounts_by_role`, `140`/`139`/`135`,
    `demandée`, `vat_payable_account_for_write`, numéros de ligne corrigés, `Fixed`) ; le prompt P1, relevé daté, n'est
    pas réécrit. Une passe P2 **complète** suit (Opus, lentilles R et F, `315-validate-prompt-p2.md`) : la boucle ne se
    clôt que sur une passe sans finding au-dessus de LOW.
