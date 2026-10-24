# Story 31.5 : quatre défauts courts avant la release (#707, #589, #504, #469)

Status: review

*Fiche rédigée le 2026-10-24, en autonomie (Opus 5.5), worktree `kesh-31-5`, branche `story/31-5` sur `origin/main`
`e71cd351`. Registre : `epic-31-choix-autonomes-315.md` (C-31-5-0 à C-31-5-22 ; 13 à 17 à la validation P1, 18 à 20 à la validation P2, 21 et 22 à la validation P3).
Clé courte : `315`.*

## Story

En tant que Guy, qui tient désormais sa comptabilité réelle dans Kesh, je veux que quatre défauts courts, à cause
établie, soient **réellement corrigés** avant le tag v0.15.0, afin que la démonstration valide une facture avec TVA,
que l'infobulle « Réouvrir » désigne l'exercice qu'il faut effectivement rouvrir, que la communication de la
QR-facture suive la langue du document, et que les refus du journal d'audit parlent la langue de l'installation.

## Origine

La règle de release de Guy (`CLAUDE.md` § *Priorités des défauts*) exige **moins de 136 issues ouvertes au tag
v0.15.0** (plafond écrit au fichier d'epic, `epic-31-tenir-les-livres.md` : 136 au tag v0.14.0).

**Recompte vivant** (validation P3, 2026-10-24, après l'ouverture de #711 — LOW résiduels de la 31-1c — et de #712 —
libellés des écritures générées écrits en dur en français, C-31-5-17 — le même jour) :
`gh issue list --state open --limit 500 --json number --jq length` rend **143**. Relevés datés antérieurs, laissés
tels : 140 à la rédaction, 141 à la validation P1 (après #710), 142 à la validation P2 (après #711). Périmètre : toutes
les issues ouvertes, à la date de la mesure, non au tag.

| Étape | Compte |
|---|---|
| ouvertes au 2026-10-24 (validation P3), #711 et #712 comprises | 143 |
| − #503 (ouverte à la mesure ; attendue fermée par la PR de la 31-1c, avant le tag) | 142 |
| − les quatre défauts de cette story (#707, #589, #504, #469) | **138** — attendu au tag si rien d'autre ne s'ouvre ni ne se ferme |

**Plafond : moins de 136.** Au tag, le compte attendu est **138** : il manque **trois** retraits. Cette story, à elle
seule, ne tient pas la règle de release. L'arbitrage — corriger d'autres défauts (dans cette story ou une autre de
l'epic), ou publier au-dessus du plafond — **revient au Project Lead** et lui est soumis par l'orchestrateur
(C-31-5-16). La règle de type des comptes de TVA n'ajoute pas d'issue : elle est tenue en une seule copie par cette
story (AC1, C-31-5-19). ⛔ **Jamais fermer un défaut réel pour tenir le chiffre**, et tout point non traité s'ouvre en issue même
s'il fait remonter le compte. Six candidates ont été proposées par l'orchestrateur ; chacune a été établie au code
(§ *Sélection*), quatre sont retenues.

## Sélection — établie au code le 2026-10-24

| Issue | Prio | Cause au code | Verdict |
|---|---|---|---|
| **#707** démo : facture avec TVA refusée | P3 | `insert_with_defaults_in_tx` (`crates/kesh-db/src/repositories/company_invoice_settings.rs:1069`) n'écrit **aucun** des trois comptes de TVA ; `seed_demo` (`crates/kesh-seed/src/lib.rs:391-392`) n'appelle que lui ; la validation d'une facture exige `default_vat_payable_account_id` dès que la TVA > 0 : `validate_invoice` le passe (`crates/kesh-db/src/repositories/invoices.rs:2364`) à `generate_invoice_journal_lines_rounded`, qui échoue en `ConfigurationRequired("default_vat_payable_account_id")` (`invoices.rs:1877-1881`), puis `designated.check_written` (`invoices.rs:2372`) exige le compte actif et imputable | **Retenue** |
| **#589** infobulle « Réouvrir » | P3 | `nearestLaterClosed` (`frontend/src/routes/(app)/settings/fiscal-years/+page.svelte:123-133`) prend le **min** `startDate` des postérieurs clos, alors que la garde LIFO du serveur (`crates/kesh-db/src/repositories/fiscal_years.rs:1294`, dans `reopen`) refuse tout exercice suivi d'un clos : seul le **plus récent** clos est rouvrable | **Retenue** |
| **#504** communication QR en français | P4 | `format!("Facture {n}")` en dur (`crates/kesh-api/src/routes/invoice_pdf_service.rs:372-376`), alors que le reste du PDF est résolu dans `locale` (`build_i18n`, l. 449) | **Retenue** |
| **#469** refus du journal d'audit non traduits | P4 | six `AppError::Validation("…français…")` dans `crates/kesh-api/src/routes/audit_log.rs` (l. 179, 194, 201, 222, 224, 239), rendus tels quels par le bras `VALIDATION_ERROR` (`crates/kesh-api/src/errors.rs:1530-1532`) | **Retenue** |
| #687 TTC fournisseur à plus de deux décimales | P3 | réelle (`crates/kesh-db/src/repositories/supplier_invoices.rs:400` et `:426`, `line_total = line.quantity * line.unit_price`, aucun arrondi ni borne d'échelle — l'issue citait `:389`/`:415`, périmés) | **Écartée** : l'issue elle-même n'offre « aucune [piste] mécanique » — arrondir à la création change HT et TVA saisis et doit couvrir la complétion d'une pièce importée, ou bien refondre `pay_in_tx` avec une ligne d'arrondi comme côté client (#476). Ni courte ni sans risque comptable, à trois semaines d'une comptabilité réelle (C-31-5-2) |
| #645 jetons de couleur non définis | P3 | réelle (`frontend/src/app.css`, bloc `@theme`) | **Écartée** : l'issue établit qu'elle « change l'apparence de tout l'écran » (bordures `border-input`, anneaux, infobulles ; « 39 fichiers » et « 19 règles `dark:` » sont les comptes **de l'issue** : recomptés le 2026-10-24, `grep -rl "border-input" frontend/src | wc -l` rend 10 et `grep -rno "dark:" frontend/src | wc -l` rend 37 occurrences dans 9 fichiers — le motif d'écart ne dépend pas du nombre), réactive des règles `dark:` et régresse l'accessibilité avec des valeurs par défaut ; elle demande une story propre avec mesures de contraste et captures avant/après (C-31-5-2) |

**Prémisses corrigées** (à commenter sur les issues par l'orchestrateur) :

- **#707** — « démonstration seulement » est vrai du **symptôme**, pas de la cause : le chemin **production**
  (`routes/onboarding.rs:830`, finalisation) appelle la même `insert_with_defaults_in_tx` et laisse lui aussi les
  trois comptes de TVA à `NULL`. Les colonnes sont nullables depuis la Story 18-1a (`18-1a-comptes-tva.md:109-110`,
  qui renvoie à la 18-1b la question du blocage), et la 18-1b a décidé que la validation d'une facture avec TVA est
  refusée en `CONFIGURATION_REQUIRED` tant que la TVA due n'est pas désignée (`18-1b-comptabilisation-ventes.md:122-125`,
  AC5 ; la décision aux l. 123-124). La désignation par l'administrateur dans *Administration → Facturation*, et le
  garde-fou qui en résulte pour un non-assujetti — Guy l'est : une facture saisie par erreur avec un taux est refusée
  au lieu d'écrire de la TVA due —, en sont la **conséquence**, lecture de cette fiche, non le texte des deux fiches. Seule la démonstration, dont
  l'issue attend un essai « sans réglage préalable » (formule de l'issue, que le produit ne porte nulle part), est
  corrigée (C-31-5-3) ; le chemin production reçoit une phrase au
  manuel (AC3).
- **#589** — la reproduction de l'issue (« A ouvert, B clos, C clos : l'infobulle de A dit B ») est **inexacte** : le
  bouton « Réouvrir » n'existe que pour un exercice **clos** (`+page.svelte:514`, `fy.status === 'Closed'`), et avec
  A ouvert l'infobulle de B nomme C, ce qui est juste. Le défaut exige **trois exercices clos** A < B < C :
  l'infobulle de A nomme B, que le serveur refusera à son tour.
- **#504** — l'avertissement de l'issue (« vérifier qu'aucun rapprochement ne s'appuie sur le texte « Facture » ») est
  levé : le score de référence (`crates/kesh-reconciliation/src/matching.rs:231-256`) teste la **contenance** du numéro
  de facture normalisé dans la référence de la transaction : la contenance du numéro décide (score 1.0), et le
  préfixe n'y entre pas **tant que le numéro est contenu** — le cas de toute communication `<préfixe> <numéro>`. La
  branche suivante (préfixe commun d'au moins quatre caractères, score 0.5, `matching.rs:247-254`, après la contenance de la l. 242) n'est évaluée que
  si la contenance échoue ; un format de numéro personnalisé commençant par `Facture`/`Rechnung` pourrait y entrer,
  sans changer la conclusion pratique : aucun test de rapprochement à ajouter (C-31-5-7).

**Découpage préventif** : cinq modules de premier niveau — `kesh-db` (deux fonctions, deux types et une constante,
C-31-5-13, C-31-5-22), `kesh-seed`, `kesh-api` (trois fichiers de route — dont `company_invoice_settings.rs`, trois
arguments, validation P3 — et `errors.rs`), `kesh-i18n` (catalogues), `frontend` (une page) — plus le manuel utilisateur FR.
Le critère « plus de cinq modules » n'est pas franchi ; les quatre correctifs sont **indépendants** (aucun fichier de
code commun), si bien qu'une passe de revue peut les lire un par un (C-31-5-1). Aucune migration.

## Critères d'acceptation

### A — #707 : la démonstration valide une facture avec TVA

1. **AC1 — `seed_demo` désigne les comptes de TVA du plan, comptes verrouillés AVANT les réglages.** Deux fonctions
   de `kesh-db` (`crates/kesh-db/src/repositories/company_invoice_settings.rs`), pour tenir l'ordre documenté
   `accounts → company_invoice_settings` (`docs/MULTI-TENANT-SCOPING-PATTERNS.md:326`, ligne `seed_demo` du Pattern 5 ;
   doc de `final_body`, `crates/kesh-seed/src/lib.rs:364-369`) — C-31-5-13, C-31-5-18 :
   - `lock_vat_role_accounts_in_tx(tx, company_id) -> Result<VatRoleAccounts, DbError>` : **trois lectures
     ponctuelles** verrouillantes, sur le patron des trois lectures d'`insert_with_defaults_in_tx`
     (`company_invoice_settings.rs:1076-1105`) — `SELECT id, account_type, postable FROM accounts WHERE company_id = ?
     AND singleton_role = ? FOR UPDATE`, une par rôle, **dans l'ordre écrit dans le code** : `VatRecoverable`, puis
     `VatPayable`, puis `VatSettlement` (ordre permuté à la validation P3, C-31-5-21 : il suit, au plan standard,
     l'ordre croissant des identifiants, celui du verrou partagé de `lock_designated_accounts_in_tx`). Chaque lecture est
     un accès `const` à l'index unique
     `uq_accounts_company_singleton_role (company_id, singleton_role)` ; elle verrouille l'entrée de cet index puis
     l'enregistrement du compte (clé primaire), et, pour un rôle absent, l'intervalle de l'index où il serait — le même
     verrou que les lectures existantes. L'**ordre réel des verrous** est donc celui du code (1171, 2200, 2206 dans le
     plan PME) ; il **coïncide** avec l'ordre des identifiants au plan standard sans le **garantir** (un rôle réattribué
     à un autre compte peut le défaire) : c'est ce que dit le doc-comment, sans rien promettre de plus. Lecture typée
     `sqlx::query_as::<_, (i64, AccountType, bool)>` — `AccountType` implémente `Type`/`Decode` pour MySQL
     (`crates/kesh-db/src/entities/account.rs:43-66`), `postable` (`tinyint(1)`) se décode en `bool`. La colonne
     générée `singleton_role` est déjà `active`-aware (`company_invoice_settings.rs:1019-1027`) : un compte archivé n'est
     pas lu. La fonction retient le compte de chaque rôle **à la condition** qu'il soit imputable (`postable`) et du type
     que `VAT_SETTING_ACCOUNT_TYPES` fixe **pour ce réglage** — `Liability`, `Asset`, `Liability`, les types qu'exige
     aujourd'hui la route (`crates/kesh-api/src/routes/company_invoice_settings.rs:337-364`) ; un rôle sans compte utilisable donne `None`, sans erreur (la démonstration se
     charge quand même). Ce type n'est **pas** `accepts_account_type` (`crates/kesh-core/src/chart_of_accounts/mod.rs:161`,
     appliquée à l'écriture du compte par `crates/kesh-db/src/repositories/accounts.rs:89`) : cette règle est celle du
     **rôle**, qui admet `Asset` **ou** `Liability` pour les trois rôles de TVA et que tout compte portant un rôle
     satisfait déjà — l'y réappliquer serait une tautologie et laisserait passer un `VatPayable` porté par un actif. La
     règle requise est celle du **réglage**, aujourd'hui écrite à la seule route. Elle s'écrit **une fois**, en `kesh-db`,
     et **la route l'emploie** (C-31-5-19) : `kesh-api` dépend de `kesh-db` et la route importe déjà
     `kesh_db::entities::account::AccountType` (`crates/kesh-api/src/routes/company_invoice_settings.rs:25-26`) et le
     module `kesh_db::repositories::company_invoice_settings` (l. 28) ; ses trois appels de `validate_account` (l. 337-364)
     passent `expected: AccountType` par valeur (signature l. 152-158 ; `AccountType` est `Copy`), si bien que les trois
     arguments `AccountType::Liability`, `AccountType::Asset`, `AccountType::Liability` deviennent
     `company_invoice_settings::VAT_SETTING_ACCOUNT_TYPES.payable`, `.recoverable`, `.settlement` — aucune autre ligne de
     la route ne change, son comportement non plus. Il n'y a donc **pas de jumelle** à tenir d'accord. Précédent du même
     fichier, qui garde, lui, deux copies : `designated_account_id` (`company_invoice_settings.rs:495-520`, « Mêmes
     exigences que sa validation (`validate_account_of`, `kesh-api`) », `account_type IN (?, ?)` lié à `Expense`,
     `Revenue`), sans garde qui les tienne d'accord — la forme à ne pas reproduire ici (C-31-5-19).
     **Définitions** (C-31-5-22), les deux types et la constante `pub` dans `company_invoice_settings.rs`, atteignables par
     `kesh_db::repositories::company_invoice_settings::…` (`kesh-seed` et la route en sont les consommateurs) :
     `#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)] pub struct VatRoleAccounts { pub payable: Option<i64>, pub
     recoverable: Option<i64>, pub settlement: Option<i64> }` (le compte retenu pour chacun des trois réglages, dans
     l'ordre des colonnes `default_vat_payable_account_id`, `default_vat_recoverable_account_id`,
     `default_vat_decompte_account_id`) ; `#[derive(Debug, Clone, Copy)] pub struct VatSettingAccountTypes { pub payable:
     AccountType, pub recoverable: AccountType, pub settlement: AccountType }` et `pub const VAT_SETTING_ACCOUNT_TYPES:
     VatSettingAccountTypes = VatSettingAccountTypes { payable: AccountType::Liability, recoverable: AccountType::Asset,
     settlement: AccountType::Liability }`, doc-comment : « règle du **réglage** (non du rôle, cf.
     `accepts_account_type`), lue par `lock_vat_role_accounts_in_tx` et par la route `PUT /api/v1/company/invoice-settings` (`update_invoice_settings`) ».
     `lock_vat_role_accounts_in_tx` **ne lit ni n'écrit** `company_invoice_settings`.
   - `designate_vat_accounts_in_tx(tx, company_id, &VatRoleAccounts) -> Result<(), DbError>` : un seul `UPDATE company_invoice_settings SET
     default_vat_payable_account_id = COALESCE(default_vat_payable_account_id, ?), … WHERE company_id = ?` ; elle **ne
     lit aucun compte** (aucun verrou de compte pris après la ligne de réglages) et n'écrase **jamais** une valeur posée.
   - `final_body` appelle la première **juste avant** `insert_with_defaults_in_tx` (après `clear_stub_in_tx`), la seconde
     juste après, **seulement si** `invoice_settings_created` est vrai (la ligne vient d'être créée par la même
     transaction, personne ne l'a lue : aucun `version` à bumper, C-31-5-4). La séquence devient `onboarding_state →
     companies → accounts (TVA, puis rôles de insert_with_defaults_in_tx) → company_invoice_settings → vat_rates →
     audit_log` : la ligne `seed_demo` du Pattern 5 et le doc-comment de `final_body` le disent (même PR). Le chemin
     production (`crates/kesh-api/src/routes/onboarding.rs`) **n'appelle ni l'une ni l'autre**.
   - **Test `kesh-db` de la lecture** (C-31-5-19), `#[sqlx::test(migrations = "./test-schema")]` sur le squash, dans
     `crates/kesh-db/tests/company_invoice_settings_repository.rs` (le fichier de tests de ce dépôt ; il monte déjà un
     plan PME par `kesh_core::chart_of_accounts::load_chart("Pme")` et `accounts::bulk_create_from_chart`, l. 34-35 —
     `company_invoice_settings.rs` n'a pas de `mod tests`) : une société au plan PME chargé ; (i) chemin heureux — les trois rôles retenus, chacun
     égal au compte qui porte le rôle (relu en base) ; (ii) le compte `VatPayable` retypé en `Asset` (permis par la règle
     du rôle) → `payable == None`, `recoverable` et `settlement` retenus ; (iii) le compte `VatRecoverable` rendu non
     imputable (`postable = FALSE`) → `recoverable == None`, les deux autres retenus ; (iv) le compte `VatSettlement`
     archivé (sa colonne `singleton_role` devient `NULL`) → `settlement == None`, sans erreur. Les états (ii) à (iv) se
     posent par `UPDATE` direct en montage (le test éprouve la lecture, non les routes qui y mènent). Chaque cas est un test
     distinct ou une assertion nommée, de sorte que la mutation qui le vise (M7a, M7b, AC15) rougisse un cas identifiable.
2. **AC2 — rouge d'abord, test HTTP.** Dans `crates/kesh-api/tests/onboarding_e2e.rs`, un test (fixtures synthétiques)
   suit le chemin démonstration jusqu'à `seed-demo`, crée un contact et une facture d'une ligne à 8,1 %, la **valide** :
   `200`, et l'écriture produite crédite le compte de rôle `VatPayable` (2200 du plan PME) du montant de TVA. Le test est
   écrit et **constaté rouge** (`400 CONFIGURATION_REQUIRED`) avant le correctif. Un second test (ou une assertion du
   même) lit `GET` des réglages de facturation après `seed-demo` et vérifie les **trois** comptes (2200, 1171, 2206 —
   rôles relus en base, pas numéros codés en dur). Un test du chemin **production**, dans
   `crates/kesh-api/tests/onboarding_path_b_e2e.rs` (qui monte déjà la finalisation : `full_path_b_flow`, l. 216 —
   `onboarding_e2e.rs` ne porte que le chemin démonstration), vérifie que les trois réglages restent `NULL` après la
   finalisation (garde de C-31-5-3 ; vert avant comme après). C'est une **assertion à écrire** : `full_path_b_flow`
   n'interroge aujourd'hui que le **nombre** de lignes de réglages (`settings_rows == 1`, l. 306-312) ; le test lit les
   trois colonnes `default_vat_payable_account_id`, `default_vat_recoverable_account_id` et
   `default_vat_decompte_account_id` de la ligne de la société et exige `None` pour **chacune** (une assertion nommée par
   colonne). Vert dès son écriture, il ne prouve qu'il sait rougir que par la mutation M8 (AC15).
3. **AC3 — manuel utilisateur FR.** `docs/manual/fr/user-manual.tex` l. 185 : « Les quatre taux de TVA suisses et les
   réglages de facturation par défaut, **dont les comptes de TVA** (TVA due, impôt préalable, décompte) ». Au chemin B
   (production, après le paragraphe de la l. 205, « Dès que vous avez choisi la langue comptable… ») : une phrase — *les comptes de TVA ne sont pas désignés d'office ; un assujetti les désigne
   dans Administration → Facturation avant sa première facture avec TVA, sans quoi la validation est refusée (« Configuration
   incomplète : configurez les paramètres de facturation avant de valider. »)* — texte actuel de
   `error-configuration-required`, qui ne nomme ni le compte ni le menu. Un **administrateur** voit déjà le menu :
   l'écran de la facture y ajoute « — Configurez les comptes par défaut dans Administration → Facturation. »
   (`frontend/src/routes/(app)/invoices/[id]/+page.svelte:409-411`) ; un autre rôle est renvoyé à son administrateur, et
   un client de l'API ne reçoit que le message. Le manuel dit donc le compte et le menu pour tous, et le message ne change
   pas (périmètre minimal). PDF régénéré (`make fr`, sous verrou) et contrôlé aplati.
   **Capture** (recette visuelle, l'état d'un écran change) : thème clair, base de démonstration synthétique après
   `seed-demo`, l'écran *Administration → Facturation* montrant les trois sélecteurs de comptes de TVA remplis (2200,
   1171, 2206), jointe au Dev Agent Record.

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
6. **AC6 — inventaire clos des textes qui prescrivent de rouvrir un exercice** (établi le 2026-10-24, validation P1 ; critère élargi à `rouvert` à la validation P2).
   **Critère de l'ensemble** : les lignes du catalogue `fr-CH` que rend
   `grep -nE "rouvr|Rouvr|réouv|Réouv|rouvert|Rouvert" crates/kesh-i18n/locales/fr-CH/messages.ftl` — **34** lignes
   (30 sans `rouvert`, qui ne contient pas `rouvr`), dont deux
   commentaires (l. 969, 1463) —, plus le code client qui calcule un exercice à nommer. Tri, sans reste :
   - (a) **nomme l'exercice à rouvrir d'abord** : `fiscal-year-reopen-blocked-later-closed` (l. 976 ; la valeur
     `{ $name }` vient de la page) — **corrigée** par l'AC4 ; `fiscal-year-out-of-order-warning` (l. 952) nomme
     `{ $latest }`, que `outOfOrderState` calcule déjà comme le **max** des clos (`fiscal-years.helpers.ts:88-91`) :
     juste.
   - (b) **ne nomme aucun exercice à rouvrir et prescrit « en commençant par le plus récent »** (ils peuvent nommer
     l'exercice qui **bloque** ou le premier exercice, jamais celui à rouvrir) : l. 423, 424, 862, 863, 871, 872, 1035
     (`error-fiscal-year-reopen-blocked`, servie par `crates/kesh-api/src/routes/fiscal_years.rs:190-193`), 1040, 1069,
     1077, 2213, 2214 — justes.
   - (c) **réouverture sans ordre à prescrire** (bouton, dialogue, motif, confirmation, libellé d'audit) : l. 964,
     970-974, 975 (`fiscal-year-reopened`), 1032, 1033, 2597 (`audit-log-action-fiscal-year-reopened`) — sans objet.
   - (d) **hors famille** (compte bancaire, aperçu de rappel) : l. 1314, 1433, 1465, 1470, 1474, 1478, 2063, 2645.
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
   commentaire qui contourne le défaut (l. 357-358). **Seul (b) établit le défaut** sur le code actuel : il compile tel
   quel et se constate rouge avant tout changement. (a), qui passe un préfixe à une fonction qui n'en prend pas encore
   (`invoice_pdf_service.rs:264-273`), ne compile pas avant le correctif — un échec de compilation n'est pas un rouge.
   Il se constate rouge **en deux temps** : ajouter d'abord le paramètre, ignoré, le `"Facture"` littéral conservé
   (les sept appels de test mis à jour) ; (a) rougit alors sur la valeur ; puis le correctif (le paramètre employé)
   le fait passer. *Si* `pdf_contains`
   ne voit pas le texte du récépissé/section de paiement (la QR elle-même est une image), le dev le constate et le
   remplace par une assertion sur le texte « Informations supplémentaires » ou sur `QrBillData` — il ne laisse pas une
   assertion qui passerait à vide.
10. **AC10 — les PDF déjà figés ne changent pas.** Un PDF émis (figé, Story 25-6-b) garde sa communication française
    jusqu'à un refigeage ; aucune migration, aucun refigeage d'office. Manuel `user-manual.tex:3258` : ajouter que la
    communication du QR-code suit aussi la langue de la facture, et qu'une facture émise avant la 0.15.0 garde la
    sienne. PDF régénéré.
    *Annotation du 2026-10-24, revue de code P1 (A2, B1, E4) : « émise avant la 0.15.0 » est trop large — seule une
    facture dont le PDF est **déjà figé** garde « Facture » ; une facture validée mais jamais téléchargée ni envoyée
    prend le préfixe de sa langue à son premier PDF, et ses rappels, régénérés à chaque envoi, portent toujours le
    préfixe de la langue du client. Manuel (l. 3262) et CHANGELOG reformulés en conséquence ; la première phrase de
    l'AC10 (« un PDF émis (figé) ») était juste.*

### D — #469 : les refus du journal d'audit sont traduits

11. **AC11 — les six sites, inventaire clos.** `grep -n "AppError::Validation" crates/kesh-api/src/routes/audit_log.rs`
    rend aujourd'hui six lignes (179, 194, 201, 222, 224, 239). Chacune passe par `crate::errors::t` (sans variable) ou
    `t_args` (avec variable ; rendue `pub(crate)` dans `errors.rs:52`, qui retire déjà les marques d'isolation). Six clés
    `audit-log-query-error-*` dans les **quatre** locales (préfixe choisi sans collision, C-31-5-20 :
    `audit-log-filter-*` désigne les libellés du formulaire de l'écran, `fr-CH` l. 2719-2726, et
    `audit-log-error-date-range`, l. 2739, le contrôle de plage fait par l'écran ; `grep -rn "audit-log-query"
    crates frontend/src` ne rend rien au 2026-10-24) : plage inversée ; date au mauvais format (`{ $field }` — le
    détail `chrono` `({e})`, anglais, n'est plus montré) ; date hors de `[1000-01-01, 9999-12-31]` (`{ $field }`, les deux
    bornes écrites en clair) ; `entityId` non positif ; `entityId` sans `entityType` ; code trop long (`{ $field }`,
    `{ $max }`). Les noms de paramètre (`dateFrom`, `entityType`…) restent tels quels : ce sont des noms d'API. Le code
    reste `VALIDATION_ERROR` (aucun changement d'API). Chaque repli Rust (`default` de `t`/`t_args`) égale la valeur
    `fr-CH` de sa clé, variables réécrites (`{ $field }` → `{nom}`, `{ $max }` → `{maximum}`, les noms des paramètres
    actuels de `parse_date`/`parse_code`). Pour une clé **à variable**, le repli s'écrit `&format!("…{nom}…")` — un
    littéral égal au catalogue réécrit, passé à `format!` avec les variables `nom`/`maximum` en portée —, **jamais** le
    littéral nu `"…{nom}…"` : celui-ci passerait G9 (qui ne compare que le littéral, `textes_coherents.rs:774-805`) mais
    afficherait des accolades brutes sans bundle (`t_args` rend alors `default.to_string()`, `errors.rs:52-59`) ;
    précédent : le repli `format!` de `error-later-fiscal-year-closed` (`errors.rs:3565-3567`). Les six clés entrent à la garde **G9**
    (`crates/kesh-api/tests/textes_coherents.rs`, `les_replis_rust_suivent_le_catalogue`, l. 680 — une quatrième
    entrée `crates/kesh-api/src/routes/audit_log.rs` dans `TABLE`, dont le type passe de `[…; 3]` à `[…; 4]`), C-31-5-15.
    Le **contrôle de fin** est celui de l'AC16 (multi-ligne) ; le **garde** est le test de l'AC12, seul à prouver
    que le message rendu est traduit.
12. **AC12 — rouge d'abord.** `crates/kesh-api/tests/audit_log_e2e.rs` (application montée en `de-CH`, société en
    français, `verrou_i18n()` pris) : pour **chacun** des dix cas de `les_parametres_invalides_sont_refuses_en_400`
    (l. 346), le message est **égal** au rendu `de-CH` de la clé attendue (calculé depuis le catalogue chargé, marques
    d'isolation retirées — pas un texte allemand codé en dur) **et différent** du rendu `fr-CH` de la même clé (attrape
    une valeur allemande restée une copie du français). Constaté rouge avant le correctif — **après** l'ajout des six
    clés aux quatre catalogues (ordre de T4) : sans elles, le calcul de la valeur attendue échouerait sur une clé absente
    et le rouge ne démontrerait pas que le message rendu est français. Le test existant
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
    - M7a (#707) : dans `lock_vat_role_accounts_in_tx`, retirer le filtre de type → le cas (ii) du test `kesh-db` de
      l'AC1 rouge (`payable` retenu alors qu'il porte un actif).
    - M7b (#707) : retirer le filtre `postable` → le cas (iii) rouge.
    - M8 (#707, garde du chemin production) : faire appeler `designate_vat_accounts_in_tx` (précédée de
      `lock_vat_role_accounts_in_tx`) par la finalisation production (`crates/kesh-api/src/routes/onboarding.rs`, après
      `insert_with_defaults_in_tx`, l. 830) → le test de `onboarding_path_b_e2e.rs` de l'AC2 rouge (les trois réglages ne
      sont plus `NULL`). Sans cette mutation, rien ne montre que ce test « vert avant comme après » sait rougir.
16. **AC16 — symptôme grepé, valeur comprise.** Commandes lancées depuis la racine du worktree (chemins relatifs à
    elle), après chaque patch — C-31-5-14 :
    - **#504.** `grep -rn 'format!("Facture' crates` rend aujourd'hui **deux** lignes (inventaire du 2026-10-24) :
      `crates/kesh-api/src/routes/invoice_pdf_service.rs:373` (le site corrigé) et
      `crates/kesh-db/src/repositories/supplier_invoices.rs:512` (`format!("Facture fournisseur {number_label} -
      {contact_name}")`, **libellé d'écriture comptable** d'une facture fournisseur : code de production, ni QR ni
      document adressé au débiteur — hors classe de #504, non résolu assumé, cf. *Hors périmètre*). Contrôle
      décidable : `grep -rn 'format!("Facture' crates | grep -v '^crates/kesh-db/src/repositories/supplier_invoices.rs:'`
      → **zéro** ligne, et `grep -c 'format!("Facture' crates/kesh-db/src/repositories/supplier_invoices.rs`
      → **1** (l'exception reste seule et nommée : un second `format!("Facture…` dans ce fichier, quel qu'il soit,
      porterait ce compte à 2). Les littéraux
      sans `format!` (`Some("Facture 42")` en test de `crates/kesh-api/src/routes/supplier_invoices.rs:816`,
      `Some("Facture test")` en test de `crates/kesh-api/src/qr_decode.rs:225`, et les fixtures de `kesh-qrbill`) ne
      sont pas attrapés par le motif et ne sont pas des exceptions : ce sont des QR **reçues** de test.
    - **#589.** `grep -rn "nearestLaterClosed" frontend/src` → zéro.
    - **#469.** Contrôle **multi-ligne**, insensible à la mise en forme de rustfmt (l'argument de
      `AppError::Validation(` est aujourd'hui sur la ligne suivante aux l. 179, 194, 201) :
      `tr '\n' ' ' < crates/kesh-api/src/routes/audit_log.rs | grep -oE 'AppError::Validation\( *[A-Za-z_:"]+'` rend
      **six** jetons — aujourd'hui trois `"…` et trois `format` — et, après correctif, **toujours six** (un site
      supprimé ou passé à une autre variante d'`AppError` sortirait du compte sans rougir le filtre : le compte de six
      est donc exigé **avec** le filtre), et `… | grep -vE '\( *(crate::errors::)?t(_args)?$'` → **zéro** (chaque
      argument commence par `t(` ou `t_args(`, qualifiés ou non). Ce contrôle dit la **forme** ; le **garde** reste le test de l'AC12, qui dit que le message
      rendu est traduit.

## Tâches

- [ ] **T0** — Brancher n'est pas nécessaire (worktree `kesh-31-5`, branche `story/31-5`) ; base `kesh_315`,
  `kesh_e2e_315`, port attribué par l'orchestrateur, `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-31-5/target`
  (`docs/agents-orchestration.md` § 4).
- [ ] **T1 (#589, AC4-AC7)** — test rouge (AC5) ; `latestLaterClosed` ; commentaire du helper ; Vitest ciblé.
- [ ] **T2 (#707, AC1-AC2)** — tests rouges (AC2, dont le test production dans `onboarding_path_b_e2e.rs`) ;
  `lock_vat_role_accounts_in_tx` et `designate_vat_accounts_in_tx` (doc-comment : pourquoi la démonstration seulement,
  C-31-5-3 ; l'ordre **réel** des verrous — trois lectures ponctuelles dans l'ordre du code, `VatRecoverable`,
  `VatPayable`, `VatSettlement`, sur l'index `uq_accounts_company_singleton_role`, qui coïncide au plan standard avec
  l'ordre des identifiants sans le garantir —, C-31-5-13, C-31-5-18 et C-31-5-21) ; `VatRoleAccounts`,
  `VatSettingAccountTypes` et `VAT_SETTING_ACCOUNT_TYPES` (C-31-5-22) ; la route
  `crates/kesh-api/src/routes/company_invoice_settings.rs` emploie la constante (trois arguments, l. 337-364,
  C-31-5-19) ; test `kesh-db` de la lecture dans `crates/kesh-db/tests/company_invoice_settings_repository.rs` (AC1,
  cas i-iv) ; appels dans `final_body` (la lecture avant
  `insert_with_defaults_in_tx`, l'application après) ; doc-comment de `final_body` et ligne `seed_demo` de
  `docs/MULTI-TENANT-SCOPING-PATTERNS.md` (l. 326) mis à jour ; gate **complet** (kesh-db).
- [ ] **T3 (#504, AC8-AC10)** — tests rouges (AC9) ; clé `invoice-qr-message-prefix` × 4 ; résolution dans
  `render_document`, paramètre de `build_qrbill_inputs` — ajouté d'abord, ignoré, pour constater (a) rouge (AC9) — (ses sept appels de test, l. 705-943, reçoivent le préfixe
  `"Facture"`) ; commentaire de l. 345-348 (« ne dépendent PAS du document ») complété : *ils dépendent de la langue*.
- [ ] **T4 (#469, AC11-AC12)** — dans cet ordre : six clés `audit-log-query-error-*` × 4 d'abord ; puis le test de
  l'AC12, écrit et **constaté rouge** sur le seul message français rendu ; puis `t_args` en `pub(crate)` et les six
  sites (replis `&format!(…)` pour les clés à variable, AC11) ; enfin les six clés à la garde G9 (`textes_coherents.rs`,
  C-31-5-15).
- [ ] **T5 (AC3, AC7, AC10)** — manuel FR (trois retouches), `make fr` sous verrou, contrôle aplati :
  `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oF "dont les comptes de TVA"` (et
  les deux autres phrases) ; captures (AC3 : *Administration → Facturation* de la démonstration ; AC7).
- [ ] **T6 (AC14-AC16)** — mutations M1-M8 (M7a, M7b), gate complet, frontend, E2E complet au dernier commit de code ; Dev Agent
  Record : chiffres réellement exécutés.

## Dev Notes

### #707 — pourquoi la démonstration seulement, et comment

- La ligne `company_invoice_settings` de la démonstration est créée par `insert_with_defaults_in_tx`
  (`company_invoice_settings.rs:1069-1192`) : rôles `Receivable`, `DefaultRevenue`, `Payable` lus `FOR UPDATE`, puis
  `chart_designated_accounts` (arrondi, escompte, frais, perte). **Aucun** des trois comptes de TVA. Les colonnes
  viennent de `20260614000001_vat_accounts_config.sql`, nullables (`18-1a-comptes-tva.md:109-110`, qui renvoie la
  question du blocage à la 18-1b) ; la 18-1b a décidé le refus `CONFIGURATION_REQUIRED` d'une facture avec TVA sans
  compte de TVA due (`18-1b-comptabilisation-ventes.md:122-125`, AC5). La désignation par l'administrateur en est la
  conséquence.
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
  deux** (actif, imputable, type **du réglage** et non du rôle, C-31-5-19) : elle ne pose rien que la route aurait refusé
  ni que `check_written` refuserait.
- **Ordre des verrous** (C-31-5-13) : `final_body` tient l'état d'onboarding (`lock_state_in_tx`), puis la société
  (`clear_stub_in_tx`) ; `lock_vat_role_accounts_in_tx` verrouille les trois comptes de TVA par **trois lectures ponctuelles**
  `singleton_role = ? FOR UPDATE`, dans l'ordre du code — `VatRecoverable`, `VatPayable`, `VatSettlement` (1171, 2200,
  2206 au plan PME ; ordre permuté à la validation P3, C-31-5-21) —, chacune sur l'index unique `uq_accounts_company_singleton_role` puis sur l'enregistrement du
  compte, un rôle absent ne laissant qu'un verrou d'intervalle sur cet index (C-31-5-18). ⚠️ Un `IN (…) ORDER BY id FOR
  UPDATE`, prescrit à la validation P1, **ne donnait pas** l'ordre des identifiants : InnoDB verrouille au fil de l'index
  parcouru, le tri vient après (le dépôt connaît ce piège : `FORCE INDEX (PRIMARY)` de `company_invoice_settings.rs:897-905`,
  inapplicable ici faute de liste d'`id`). Le seul autre site qui verrouille ces comptes dans un ordre fixe est le
  verrou partagé par identifiant croissant de `lock_designated_accounts_in_tx` (`company_invoice_settings.rs:865-916`) ;
  mais ses appelants ne verrouillent jamais 1171 et 2200 **ensemble** — la validation d'une facture prend `SALE`
  (créance, `VatPayable`), la saisie fournisseur `PURCHASE` (dette, `VatRecoverable`), `company_invoice_settings.rs:696-700`
  —, si bien qu'aucun ordre opposé n'existe entre deux comptes de TVA. La permutation de la validation P3 (C-31-5-21)
  est donc **sans effet sur les interblocages** : elle aligne seulement l'ordre de lecture sur celui des identifiants au
  plan standard, par lisibilité ; une opposition imprévue serait de toute façon rejouée par `is_seed_retryable`
  *(rectifié à la validation P4, P4-1 : la P3 invoquait à tort un « unique ordre opposé » 1171 → 2200)*. L'ordre **entre tables** est ce que tient le Pattern 5, l'ordre **entre comptes** n'est que celui du code,
  écrit tel. `insert_with_defaults_in_tx` verrouille ensuite ses comptes de rôle de la même
  façon (`company_invoice_settings.rs:1076-1105`) puis
  insère la ligne de réglages (`INSERT IGNORE`, l. 1123) ; `designate_vat_accounts_in_tx` ne fait qu'un `UPDATE` de
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

- **Chemin production, comptes de TVA `NULL`** : conséquence des Stories 18-1a et 18-1b, gardée (C-31-5-3), documentée
  par l'AC3.
- **Autres `AppError::Validation` en français hors du journal d'audit** (`routes/projects.rs:107`, `:115`,
  `payment_batches.rs:176`, `contacts.rs:453`, `address_input.rs:62`, et d'autres — un `grep` grossier,
  `grep -rn "AppError::Validation(" crates | grep -v "t(" | grep -v audit_log.rs | wc -l`, en compte **283** hors du
  journal d'audit — 289 sans le dernier filtre, dont les six d'`audit_log.rs` —, relevé du 2026-10-24, qui surcompte les
  messages déjà traduits sur plusieurs lignes) : même classe de défaut, **non** couverte par #469 qui est bornée au journal d'audit :
  tracée par **#710**, ouverte le 2026-10-24 (comprise dans les recomptes de l'*Origine* depuis la validation P1 — 141, puis 143 à la validation P3), C-31-5-12.
- **Libellés d'écriture comptable en français en dur** — `crates/kesh-db/src/repositories/supplier_invoices.rs:512`
  (« Facture fournisseur … », seule exception du contrôle #504 de l'AC16), `:941` (« Règlement fournisseur … »),
  `invoice_settlements_write.rs:277` (« Règlement facture … »), `credit_notes.rs:590` (« Avoir … (annule …) ») : ce
  sont des descriptions d'écritures du journal, non des textes adressés au débiteur ; hors classe de #504 et hors
  périmètre. Tracés par **#712** (« Les libellés des écritures générées (facture fournisseur, règlement, avoir) sont
  écrits en dur en français »), ouverte le 2026-10-24 par l'orchestrateur et comprise dans le recompte de 143 de
  l'*Origine* — C-31-5-17.
- **PDF déjà figés** en français : conservés (AC10).
- #687 et #645 : écartées (§ *Sélection*).

### Fichiers touchés (prévus)

`crates/kesh-db/src/repositories/company_invoice_settings.rs` ; `crates/kesh-db/tests/company_invoice_settings_repository.rs`
(test de la lecture, AC1) ; `crates/kesh-api/src/routes/company_invoice_settings.rs` (les trois arguments de type
remplacés par la constante, AC1, C-31-5-19) ; `crates/kesh-seed/src/lib.rs` ;
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
- **La route `PUT /api/v1/company/invoice-settings` lit ses trois types de TVA dans la constante de `kesh-db`** : mêmes
  valeurs, comportement inchangé ; `crates/kesh-api/tests/company_invoice_settings_postable_e2e.rs` (table des champs,
  « Compte TVA due » `Liability`, « Compte TVA récupérable » `Asset`, « Compte décompte TVA » `Liability`, l. 150-170)
  crée des comptes du type exigé et reste vert. Une mutation de la constante change désormais la route **et** la
  lecture ensemble : le cas (ii) du test `kesh-db` (AC1) la voit.
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

Opus 5.5 (Claude Code), en autonomie, 2026-10-24 — worktree `kesh-31-5`, bases `kesh_315` / `kesh_e2e_315`, port 3315,
`CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-31-5/target`. Journaux sous `/home/gcorbaz/devel/kesh-gate-logs/315-*`.

### Completion Notes List

**Commits de code** (dans l'ordre) : `05f8a944` (#589), `89f35d52` (#707), `46504f33` (#504, et les six clés de #469
posées d'abord), `4a35ec1c` (#469, garde G9 et C-31-5-23) — **dernier commit de code** ; puis `d8bdffd5` (manuel FR, PDF,
CHANGELOG) et le commit de ce compte rendu, documentaires.

**Rouges constatés avant chaque correctif** (sorties copiées des journaux) :

- **#589 (AC5)** — `315-rouge-589.log` : `× trois clos : l'infobulle nomme le plus récent clos, pas le plus proche` —
  `Expected: "Exercice 2026"` / `Received: "Rouvrez d'abord l'exercice « Exercice 2025 », plus récent et encore
  clôturé."` (1 failed | 1 passed | 27 skipped).
- **#707 (AC2)** — `315-rouge-707.log` : `demo_validates_an_invoice_with_vat` → `left: 400 / right: 200`, corps
  `{"error":{"code":"CONFIGURATION_REQUIRED",…}}` ; `demo_designates_the_three_vat_accounts` → `TVA due — left: None /
  right: Some(42)` ; `full_path_b_flow` vert (garde du chemin production, vert avant comme après ; éprouvée par M8).
- **#504 (AC9 b)** — `315-rouge-504-b.log` : `le_pdf_est_dans_la_langue_du_client` rouge à la nouvelle assertion
  (`invoice_frozen_pdf_e2e.rs:359`, `!pdf_contains(&pdf, "Facture")`) : `pdf_contains` **voit** donc la communication
  imprimée — l'assertion ne passe pas à vide. **(AC9 a)** — `315-rouge-504-a.log`, en deux temps (paramètre ajouté,
  ignoré, sept appels de test à `"Facture"`) : `left: Some("Facture F-2026-0001") / right: Some("Rechnung F-2026-0001")`.
- **#469 (AC12)** — `315-rouge-469.log`, après l'ajout des six clés aux quatre catalogues : `message de
  ?dateFrom=pas-une-date … left: "dateFrom invalide (input contains invalid characters)" / right: "dateFrom ist
  ungültig: Datum im Format JJJJ-MM-TT erwartet"` ; `une_date_hors_plage_…` vert.

**Correctifs** :

- **#707** — `lock_vat_role_accounts_in_tx` (trois lectures ponctuelles `singleton_role = ? FOR UPDATE`, ordre
  `VatRecoverable`, `VatPayable`, `VatSettlement` ; filtre imputable **et** type du réglage), `designate_vat_accounts_in_tx`
  (un `UPDATE … COALESCE`, aucune lecture de compte), `VatRoleAccounts`, `VatSettingAccountTypes`,
  `VAT_SETTING_ACCOUNT_TYPES` ; `final_body` : lecture avant `insert_with_defaults_in_tx`, désignation après, sur ligne
  neuve seulement ; la route `PUT /company/invoice-settings` lit la constante (C-31-5-25) ; Pattern 5 et doc-comment de
  `final_body` mis à jour. Le chemin production n'appelle ni l'une ni l'autre.
- **#589** — `latestLaterClosed` (max `startDate`), doc-comment réécrit, quatre sites renommés.
- **#504** — clé `invoice-qr-message-prefix` × 4, résolue dans la `locale` de `render_document`, passée à
  `build_qrbill_inputs` (`format!("{qr_message_prefix} {n}")`, troncature 140 conservée) ; commentaire du rapprochement
  complété.
- **#469** — six sites en `t` / `t_args` (`pub(crate)`), replis `&format!(…)` égaux au fr-CH, six clés × 4, entrée
  `audit_log.rs` à G9 (`TABLE` `[…; 4]`) ; formulation du refus de format : C-31-5-24.

**Écarts à la fiche, et incidents** :

- **Garde des comptes cités (C-31-5-23)** : le gate complet d'après T2 (`315-gate-t2.log`, 3914 exécutés, 3913 passés,
  1 échoué) a rougi `textes_coherents::les_comptes_cites_en_exemple_existent_dans_les_plans_livres` sur la clé neuve
  `audit-log-query-error-date-out-of-range` (« 9999 » lu comme un compte). Correctif mécanique de la garde (une date ISO
  `NNNN-MM-JJ` n'est pas un compte), commité avec T4. Défaut non prévu par la fiche.
- **Route (C-31-5-25)** : une variable locale `vat_types` et deux lignes de commentaire, au lieu de trois chemins
  complets — écart à la lettre « aucune autre ligne ne change », comportement identique.
- **Commit `46504f33`** : les catalogues portent ensemble la clé de #504 et les six clés de #469 (un même fichier par
  locale ; aucun ajout partiel par commit sans `git add -p`, indisponible ici).
- **Outil d'édition** : les deux tests HTTP de #707 (fin de `onboarding_e2e.rs`) ont été ajoutés par un `cat >>`, contre
  la consigne « Edit uniquement » ; contenu vérifié par le gate, aucun autre fichier du dépôt n'a été écrit hors Edit
  (hors `cargo fmt` et `make fr`).
- **Premier run E2E à montage incomplet** : le runner n'avait pas `KESH_INBOX_DIR` / `KESH_DOCUMENTS_DIR` — 298 passés /
  11 échoués / 17 ignorés / 10 non exécutés, dont quatre échecs `KESH_INBOX_DIR = /tmp/kesh-e2e/inbox : répertoire
  absent…` (`backup-restore-new-install.spec.ts:140`, `:262`, `inbox-import.spec.ts:59`, `supplier-documents.spec.ts:78`).
  Journal gardé : `315-e2e-run1-montage-incomplet.log`. Relancé, base remise à zéro, montage corrigé : voir ci-dessous.
  `docs/testing.md` (recette du terminal 2) ne nomme pas ces deux variables côté runner — la recette du `CLAUDE.md` non
  plus : **issue à ouvrir** (documentation de la recette E2E).
  *⚠️ Annotation du 2026-10-24, revue de code P1 (finding A1, MEDIUM) : la phrase précédente est fausse.
  `docs/testing.md` l. 244-250 (depuis la Story 30-2a) dit que le runner lit `KESH_INBOX_DIR` et `KESH_DOCUMENTS_DIR`
  avec pour défauts ceux de la recette, et qu'un backend lancé avec d'autres dossiers exige de passer les mêmes
  valeurs au runner. Le run 1 venait d'un backend lancé sur des dossiers propres au worktree, non passés au runner :
  la recette était juste, le défaut était dans `docs/agents-orchestration.md` § 4, point 2 (montage E2E), qui ne
  disait la chose qu'à moitié — précisé par l'orchestrateur au commit `adff757c`, défaut voisin intégré (C-31-5-26).
  **Aucune issue n'est à ouvrir.***

**Mutations** (`315-mutations.log`, un journal `315-mut-<id>.log` chacune ; appliquées par Edit sur motif unique,
rejouées sous verrou, restaurées par `git checkout -- f && touch f`, diff résiduel 0) :

| Id | Mutation | Résultat |
|---|---|---|
| M1 | `other.startDate > best.startDate` → `<` | **tuée** : `trois clos : l'infobulle nomme le plus récent clos…` |
| M2 | appel de `designate_vat_accounts_in_tx` retiré | **tuée** : `demo_validates_an_invoice_with_vat`, `demo_designates_the_three_vat_accounts` |
| M3 | TVA due lue sur `VatSettlement` | **tuée** : `demo_validates_…` (« l'écriture crédite le compte du rôle VatPayable de 8.10 » — `left: None`), `demo_designates_…`, et quatre tests `vat_role_accounts_*` |
| M4 | `if invoice_settings_created` → `if true` | **équivalente** (71 tests verts : `onboarding_e2e`, `onboarding_audit_e2e`, `onboarding_path_b_e2e`, `kesh-seed`) : à l'étape 2, la ligne de réglages est toujours neuve, et `COALESCE` n'écraserait de toute façon aucune valeur posée — la seule différence observable serait une désignation sans bump de `version` sur une ligne préexistante, que la démonstration ne produit pas |
| M5 | préfixe `"Facture"` rétabli en dur | **tuée** : `qr_message_uses_the_document_language_prefix`, `le_pdf_est_dans_la_langue_du_client` |
| M6 | `parse_code` laissé en français | **tuée** : `les_parametres_invalides_sont_refuses_en_400`, cas `?action=…` |
| M7a | filtre de type neutralisé | **tuée** : `vat_role_accounts_reject_payable_of_wrong_type` |
| M7b | filtre `postable` neutralisé | **tuée** : `vat_role_accounts_reject_non_postable_recoverable` |
| M8 | la finalisation production désigne la TVA | **tuée** : `full_path_b_flow` (« TVA due non désignée en production », `left: Some(40)`) |

*Annotation du 2026-10-24, revue de code P1 (finding A4, LOW) : M1 figure deux fois dans `315-mutations.log` pour
**une seule exécution**. M1 est une mutation Vitest, jouée à la main avant l'écriture du script `315-mut-run.sh`
(07:11:25), qui ne sait lancer que `cargo nextest` ; `315-mut-M1.log` ne porte qu'un run (`Start at 07:11:00`, un
seul résumé `1 failed | 28 passed (29)`). La première ligne est l'extrait brut du journal (signe `×`, apostrophe
typographique, durée `20ms`), la seconde sa recopie normalisée, ajoutée par erreur à côté au lieu de la remplacer.
Les deux décrivent le même rouge ; le journal, relevé daté, n'est pas réécrit.*

**Tests ajoutés** — recomptés aux deux bornes (`git show origin/main:<f> | grep -cE '#\[(sqlx::)?test'` contre le
fichier à `HEAD`, `it(` pour Vitest), périmètre `origin/main` (`e71cd351`) → `HEAD` : `onboarding_e2e.rs` 14 → 16 (+2),
`company_invoice_settings_repository.rs` 25 → 29 (+4), `invoice_pdf_service.rs` 9 → 10 (+1), `fiscal-years-page.test.ts`
26 → 27 (+1) ; soit **7 tests Rust et 1 test Vitest neufs**. Tests modifiés sans changement de nombre :
`onboarding_path_b_e2e.rs` (`full_path_b_flow`, trois assertions), `invoice_frozen_pdf_e2e.rs`
(`le_pdf_est_dans_la_langue_du_client`), `audit_log_e2e.rs` (`les_parametres_invalides_sont_refuses_en_400`),
`textes_coherents.rs` (G9, `nombres_isoles`).

**Gates au dernier commit de code** (`4a35ec1c`, arbre identique à `d8bdffd5` pour le code), base remise à zéro avant :

- **Backend** — `scripts/test-fast.sh --ci` sous verrou (`315-gate-final-backend.log`) : garde des droits, `fmt`,
  `clippy -D warnings`, nextest **3915 exécutés, 3915 passés, 7 ignorés**.
- **Frontend** (`315-gate-final-*.log`) : `npm run check` 0 erreur (29 avertissements, préexistants, aucun dans les
  fichiers touchés) ; `lint-i18n-ownership` vert ; `test:unit` **159 fichiers, 1867 tests passés** ; `build` vert.
- **E2E complet** (`315-e2e.log`, montage de `docs/testing.md`, port 3315, `kesh_e2e_315` remise à zéro) : **312 passés /
  7 échoués / 17 ignorés** (12,7 min). Les sept, jugés fichier par fichier : `mode-expert.spec.ts:26`, `:41`,
  `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150` — **les sept KF-029 (#97)** de la liste
  des échecs attendus ; ni KF-052 ni pollution. `git log origin/main..HEAD` sur ces trois fichiers : zéro commit.
- **Gates ciblés** entre les étapes (non déclarés comme complets) : `315-vert-707-cible.log` (64/64),
  `315-vert-504.log` (38/38), `315-vert-469.log` (570/570, dont `textes_coherents` et la parité i18n).
- **Gate complet intermédiaire** (exception `kesh-db`, après T2) : `315-gate-t2.log`, 3913/3914 — l'échec est celui de
  C-31-5-23, corrigé ensuite.

**Contrôles de l'AC16** (après correctif) : `grep -rn 'format!("Facture' crates | grep -v '^crates/kesh-db/src/repositories/supplier_invoices.rs:'`
→ zéro ligne ; `grep -c 'format!("Facture' crates/kesh-db/src/repositories/supplier_invoices.rs` → 1 ;
`grep -rn "nearestLaterClosed" frontend/src` → zéro ; contrôle multi-ligne d'`audit_log.rs` → six jetons (`t` × 3,
`t_args` × 3), zéro hors `t`/`t_args`.

**Manuel** : `user-manual.tex` (l. 185, paragraphe après la l. 205, FAQ de la langue d'une facture), `make fr` sous
verrou, `user-manual.pdf` seul gardé (`admin-manual.pdf` et `marketing-brochure.pdf` régénérés sans changement de
source, restaurés). PDF aplati : « dont les comptes de TVA » 1, « suit elle aussi la langue de la facture » 1,
« Configuration incomplète : configurez les paramètres de facturation avant de valider » 1 ; « Les comptes de TVA ne
sont pas désignés d’office » et « indique lequel rouvrir d’abord » présents à l'apostrophe typographique et à la
ligature `ﬀ` près.

**Captures** (thème clair, base de démonstration synthétique — `_test/seed` `fresh`, puis `seed-demo` par l'API),
`/home/gcorbaz/devel/kesh-gate-logs/315-captures/` : `315-ac3-facturation-demo.png` (*Administration → Facturation* :
TVA due 2200, impôt préalable 1171, décompte 2206) ; `315-ac7-exercices-trois-clos.png` (2024, 2025, 2026 clos :
deux boutons *Réouvrir* désactivés, celui de 2026 actif) ; `315-ac7-survol-plus-ancien.png`. Infobulles relevées au DOM
(`315-captures-notes.txt`) : 2024 et 2025 — `title` et enveloppe « Rouvrez d’abord l’exercice « Exercice 2026 », plus
récent et encore clôturé. » ; 2026 — actif, sans `title`.

**Issues à ouvrir** (demandées à l'orchestrateur) : recette E2E — `KESH_INBOX_DIR` et `KESH_DOCUMENTS_DIR` à passer
aussi au runner (`docs/testing.md`, `CLAUDE.md`).
*Annotation du 2026-10-24, revue de code P1 (A1) : demande retirée — `docs/testing.md` le disait déjà ; la précision
manquante était dans `docs/agents-orchestration.md` § 4, portée par `adff757c` (C-31-5-26). Aucune issue à ouvrir.*

### File List

- `crates/kesh-db/src/repositories/company_invoice_settings.rs`
- `crates/kesh-db/tests/company_invoice_settings_repository.rs`
- `crates/kesh-seed/src/lib.rs`
- `crates/kesh-api/src/routes/company_invoice_settings.rs`
- `crates/kesh-api/src/routes/invoice_pdf_service.rs`
- `crates/kesh-api/src/routes/audit_log.rs`
- `crates/kesh-api/src/errors.rs`
- `crates/kesh-api/tests/onboarding_e2e.rs`
- `crates/kesh-api/tests/onboarding_path_b_e2e.rs`
- `crates/kesh-api/tests/invoice_frozen_pdf_e2e.rs`
- `crates/kesh-api/tests/audit_log_e2e.rs`
- `crates/kesh-api/tests/textes_coherents.rs`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`
- `frontend/src/routes/(app)/settings/fiscal-years/+page.svelte`
- `frontend/src/routes/(app)/settings/fiscal-years/fiscal-years-page.test.ts`
- `frontend/src/lib/features/fiscal-years/fiscal-years.helpers.ts`
- `docs/MULTI-TENANT-SCOPING-PATTERNS.md`
- `docs/agents-orchestration.md` (§ 4, point 2 — commit `adff757c` de l'orchestrateur, défaut voisin intégré, C-31-5-26)
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/user-manual.pdf`
- `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/31-5-defauts-courts.md`, `epic-31-choix-autonomes-315.md`,
  `315-review-prompt-p1.md`, `315-review-prompt-p2.md`, `sprint-status.yaml`

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
- 2026-10-24 — **Validation P2** (Opus, passe complète, deux lentilles en contexte frais ; rapports hors dépôt
  `kesh-gate-logs/315-validate-p2-{R,F}.md`). Lentille R : 0 CRITICAL / 0 HIGH / 2 MEDIUM (R1, R2) / 7 LOW (R3-R9).
  Lentille F : 0 / 0 / 2 MEDIUM (F1, F2) / 8 LOW (F3-F10). Recoupements : R1 = F1, R7 = F8, R4 ⊃ F5 (position l. 205).
  Trois MEDIUM distincts (R1 = F1, R2, F2), contre sept en P1 : défauts distincts, dont deux nés de la remédiation P1
  (R1 = F1, F2) — aucun recyclage. Sort, chaque affirmation revérifiée au code avant écriture :
  - **R1 = F1** (MEDIUM, « `ORDER BY id` — ordre des verrous par clé primaire », faux : InnoDB verrouille au fil de
    l'index parcouru, `uq_accounts_company_singleton_role`) — corrigé : trois lectures ponctuelles
    `singleton_role = ? FOR UPDATE` dans l'ordre du code (`VatPayable`, `VatRecoverable`, `VatSettlement`), patron
    `company_invoice_settings.rs:1076-1105` ; l'ordre réel des verrous écrit à l'AC1, aux Dev Notes, à T2 (consigne du
    doc-comment) et au registre (C-31-5-13 annoté, C-31-5-18).
  - **R2** (MEDIUM, branche `None` sans test ni mutation) — corrigé : test `kesh-db` de la lecture à quatre cas (AC1),
    mutations M7a/M7b (AC15) ; **F10** traité avec lui : le filtre de type est celui du **réglage**, non
    `accepts_account_type`, qui est la règle du rôle et ne refuse pas un `VatPayable` porté par un actif — écrit une
    fois en `kesh-db`, jumelle de la route nommée (C-31-5-19). ⚠️ Écart assumé à la consigne de l'orchestrateur (« réutiliser
    `accepts_account_type` ») : la réutilisation rendrait le filtre inopérant et le cas (ii) de R2 intestable.
  - **F2** (MEDIUM, budget conditionnel sans le dire) — corrigé : *Origine* inconditionnelle, 137 attendues au tag si
    rien d'autre ne s'ouvre, 138 si l'issue des libellés (C-31-5-17, non ouverte) s'ouvre, plafond < 136, arbitrage au
    Project Lead (C-31-5-16 annoté, C-31-5-17 annoté).
  - **LOW** — R3 (M8, garde du chemin production) ; R4 (doc de `final_body` l. 364-369 ; #687 `:400`/`:426` ; AC3 après
    la l. 205) ; R5 (critère de l'AC6 élargi à `rouvert` : 34 lignes, 975 et 2597 en (c), 1478 et 2645 en (d)) ; R6
    (« toujours six » jetons exigés avec le filtre) ; R7 = F8 (AC9 : seul (b) établit le défaut, (a) en deux temps,
    paramètre ignoré d'abord) ; R8 (« promesse » de la démonstration attribuée à l'issue, fiche et C-31-5-3) ; R9
    (préfixe `audit-log-query-error-*`, sans collision, C-31-5-20) ; F3 (`grep -c 'format!("Facture'`) ; F4 (`Result`
    aux deux signatures) ; F5 (l'écran de la facture nomme déjà le menu à un administrateur, justification nuancée) ; F6
    (283 hors journal d'audit, commande écrite, C-31-5-12 annoté) ; F7 (l. 345-348) ; F9 (capture d'*Administration →
    Facturation* de la démonstration, AC3, T5) : tous corrigés.
  - Propagation : symptômes grepés dans la fiche et le registre (`ORDER BY id`, `clé primaire`, `ordre des verrous`,
    `289`, `344-347`, `362-367`, `l. 204`, `:389`, `-> VatRoleAccounts`, `Facture fournisseur'`, `audit-log-filter-error`,
    `promet`, `M1-M6`, « qui ouvre l'issue ») ; occurrences restantes légitimes (entrées datées P1, mentions de la forme
    écartée). Tendance : P1 7 MEDIUM distincts → P2 3. Une passe P3 **complète** suit (Sonnet, lentilles R et F,
    `315-validate-prompt-p3.md`) : la boucle ne se clôt que sur une passe sans finding au-dessus de LOW.
- 2026-10-24 — **Validation P3** (Sonnet, passe complète, deux lentilles en contexte frais ; rapports hors dépôt
  `kesh-gate-logs/315-validate-p3-{R,F}.md`). Lentille R : 0 CRITICAL / 0 HIGH / 0 MEDIUM / 8 LOW (R1-R8). Lentille F :
  0 / 0 / 2 MEDIUM (F1, F2) / 4 LOW (F3-F6). Recoupements : F1 = R4 (traité au plus haut, MEDIUM), F3 = R1, F4 = R8.
  Deux MEDIUM distincts, contre trois en P2 : F1 né de la remédiation P2 (C-31-5-19), F2 hérité de la P1 — aucun
  recyclage. Sort, chaque affirmation revérifiée au code avant écriture :
  - **F1 = R4** (MEDIUM, règle de type du réglage en deux copies sans garde) — corrigé : la constante
    `VAT_SETTING_ACCOUNT_TYPES` de `kesh-db` est **employée par la route** (`routes/company_invoice_settings.rs`, qui
    importe déjà `AccountType` l. 25-26 et le module l. 28 ; trois arguments de `validate_account`, l. 337-364, passés
    par valeur, `AccountType` `Copy`) : une seule copie, plus de jumelle ; précédent `designated_account_id`
    (`company_invoice_settings.rs:495-520`) cité comme la forme à ne pas reproduire ; fichier ajouté aux *Fichiers
    touchés* et aux *Risques* (C-31-5-19 annoté).
  - **F2** (MEDIUM, issue des libellés non ouverte) — corrigé : **#712** ouverte par l'orchestrateur, citée au
    *Hors périmètre*, à l'*Origine* et à C-31-5-17.
  - **F4 = R8** (LOW, recompte périmé) — corrigé : **143** ouvertes (#711 et #712 comprises), 143 − #503 − 4 = **138**
    attendues au tag, trois retraits manquants sous le plafond < 136, arbitrage au Project Lead, écrit sans conditionnel
    (C-31-5-16 annoté) ; les relevés antérieurs (140, 141, 142) restent datés.
  - **LOW** — F3 = R1 (test `kesh-db` situé dans `crates/kesh-db/tests/company_invoice_settings_repository.rs`, ajouté
    aux fichiers touchés) ; F5 (prémisse « voulu depuis la 18-1a » : colonnes nullables à la 18-1a,
    `18-1a-comptes-tva.md:109-110`, refus décidé par la 18-1b, `18-1b-comptabilisation-ventes.md:122-125`, AC5 — la
    décision aux l. 123-124 ; garde-fou du non-assujetti donné comme conséquence ; fiche, Dev Notes, *Hors périmètre*,
    C-31-5-3) ; F6 (`VatRoleAccounts`, `VatSettingAccountTypes`, `VAT_SETTING_ACCOUNT_TYPES` définis : noms, champs,
    `pub`, chemin ; décodage `(i64, AccountType, bool)`, C-31-5-22) ; R2 (`INSERT IGNORE` l. 1123) ; R3 (ordre
    **permuté** : `VatRecoverable`, `VatPayable`, `VatSettlement`, qui suit au plan standard l'ordre croissant des
    identifiants et celui de `lock_designated_accounts_in_tx`, sans le garantir hors plan standard, C-31-5-21) ; R5
    (contenance du numéro, branche du préfixe commun `matching.rs:247-254` nommée) ; R6 (replis `&format!(…)` des clés
    à variable, précédent `errors.rs:3565-3567`) ; R7 (T4 ordonnée : clés, test constaté rouge, sites, G9 ; AC12) ;
    assertion « trois réglages `NULL` » de l'AC2 écrite comme **nouvelle** (`full_path_b_flow` ne compte aujourd'hui
    que les lignes, l. 306-312), prouvée par M8 : tous corrigés.
  - Propagation : symptômes grepés dans la fiche et le registre (`137`, `141`, `142`, `non ouverte`, `soumise au`,
    `jumelle`, `mod tests`, `VatPayable`, puis, `2200, 1171`, `1126`, `voulu`, `deux fichiers de route`) ; occurrences
    restantes légitimes (entrées datées P1/P2 du Change Log et du registre, mentions de la forme écartée). Tendance :
    P1 7 MEDIUM distincts → P2 3 → P3 2. Une passe P4 **ciblée** suit (Haiku, une lentille *regression hunter* sur le
    seul diff de cette remédiation, `315-validate-prompt-p4.md`) : la boucle ne se clôt que sur une passe sans finding
    au-dessus de LOW.
- 2026-10-24 — **Validation P4** (Haiku, passe ciblée, une lentille *regression hunter* sur `ee140ef5..eec02e1f` ;
  rapport hors dépôt `315-validate-p4.md`) : 0/0/2/1. Remédiation par l'orchestrateur, fiche et registre seulement :
  - P4-1 (MEDIUM), **vérifié au code** par l'orchestrateur (`company_invoice_settings.rs:696-700` : `SALE` =
    créance + `VatPayable`, `PURCHASE` = dette + `VatRecoverable`) : aucun appelant de `lock_designated_accounts_in_tx`
    ne verrouille 1171 et 2200 ensemble ; la justification « unique ordre opposé » de C-31-5-21 et des Dev Notes était
    fausse. Réécrite : la permutation est gardée pour la lisibilité, sans effet sur les interblocages (C-31-5-21
    annoté, Dev Notes, l'« Écartée » de C-31-5-21).
  - P4-2 (MEDIUM) : l'*Origine* disait #503 « fermée par la 31-1c » ; elle est ouverte à la mesure, attendue fermée par
    la PR de la 31-1c avant le tag. Libellé corrigé.
  - P4-3 (LOW) : le *Contexte* de C-31-5-16 daté (relevé de la P1) et renvoyé au recompte vivant.
  - Propagation : `oppos` grepé dans la fiche et le registre ; restent l'entrée datée du *Contexte* de C-31-5-21 (suivie
    de sa rectification) et les entrées datées du Change Log. Tendance : P1 7 → P2 3 → P3 2 → P4 2 MEDIUM, ces deux
    derniers nés de la remédiation précédente. Une passe P5 **ciblée** (Sonnet) suit, sur le seul diff de cette
    remédiation.
- 2026-10-24 — **Développement** (Opus 5.5, autonomie) : T1 à T6 ; quatre rouges constatés avant correctif ; mutations
  M1-M8 (M4 équivalente, les huit autres tuées) ; gate complet backend 3915/3915, frontend 1867/1867, E2E 312 / 7 KF-029
  / 17 ignorés au dernier commit de code ; choix C-31-5-23 à 25 ; statut `review`. Revue de code P1 à lancer
  (`315-review-prompt-p1.md`, Sonnet, lentilles B, E, A).
- 2026-10-24 — **Revue de code P1** (Sonnet, trois lentilles en contexte frais, lecture seule ; rapports hors dépôt
  `kesh-gate-logs/315-review-p1-{B,E,A}.md`). Lentille B : 0 CRITICAL / 0 HIGH / 0 MEDIUM / 3 LOW (B1-B3). Lentille E :
  0 / 0 / 0 / 7 LOW (E1-E7). Lentille A : 0 / 0 / 1 MEDIUM (A1) / 3 LOW (A2-A4). Recoupements : B1 = E4 (même constat,
  facture figée et rappel) et rejoint A2 ; B3 = E3. Aucun défaut de code de production. Remédiation **documentaire
  seulement** (aucune ligne de `crates/`, `frontend/src/` ni des tests), chaque affirmation revérifiée avant écriture :
  - **A1** (MEDIUM, Record faux sur `docs/testing.md`) — corrigé par annotation datée des deux paragraphes du Record
    (premier run E2E, *Issues à ouvrir*) : `docs/testing.md` l. 244-250 disait déjà que le runner reçoit les mêmes
    dossiers que le backend ; le défaut était dans `docs/agents-orchestration.md` § 4, point 2, précisé par
    l'orchestrateur (`adff757c`) ; fichier ajouté à la File List ; demande d'issue retirée (C-31-5-26).
  - **A2** (LOW), avec la partie documentaire de **B1/E4** — vérifié au code (`issued_invoice_pdf.rs` : un PDF figé est
    servi tel quel, `get_or_freeze` l. 95-105 ; une facture non figée est rendue au premier PDF par le code courant ;
    les rappels, `PdfDocument::Reminder`, sont rendus à chaque envoi, `invoice_email.rs:566`, `:1266`) : manuel
    (`user-manual.tex` l. 3262) et CHANGELOG (l. 64) visent désormais la facture **dont le PDF est déjà figé**, disent
    qu'une facture validée mais jamais téléchargée ni envoyée prend le préfixe de sa langue, et que ses rappels portent
    celui de la langue du client ; AC10 annotée. `make fr` sous verrou (`315-make-fr-p1.log`) : seul `user-manual.pdf`
    reconstruit (2 passes, stable), `admin-manual.pdf` et `marketing-brochure.pdf` inchangés. PDF aplati : 0 `??` ;
    « figé avant la version 0.15.0 » 1, « portent toujours celle de la langue du client » 1, « peut donc porter
    « Rechnung F-… » » 1, « non le mot qui le précède » 1, « suit elle aussi la langue de la facture » 1, « émise
    avant la version 0.15.0 » 0 ; la phrase « …jamais téléchargée ni envoyée prend la communication de sa / langue à
    son premier PDF » est coupée par un saut de page (pied de page entre les deux), chacune de ses deux moitiés présente.
  - **A4** (LOW, M1 écrit deux fois) — expliqué au Record (annotation sous le tableau des mutations) : une seule
    exécution, deux lignes (extrait brut et recopie normalisée).
  - **A3** (LOW, trois commits sans `refs`) — sans effet au squash ; à rappeler au titre de la PR. Non traité ici
    (réécrire des commits poussés ou non n'est pas de la remédiation documentaire).
  - **Non traités, tracés** : B1-commentaire (commentaire de `invoice_pdf_service.rs`, du code), B2, B3 = E3, E1, E2,
    E5, E6, E7 — liste exacte, avec fichier et ligne, dans `kesh-gate-logs/315-low-residuels.md`, pour une issue que
    l'orchestrateur ouvrira (règle « on trace toujours »).
  - Propagation : `émise avant`, `garde sa communication`, `garde la sienne`, `issue à ouvrir`, `Issues à ouvrir`,
    `recette E2E` grepés dans le dépôt (code, fiche, registre, manuel, CHANGELOG, `website/`) ; restent l'AC10
    (annotée) et les relevés datés du Record (annotés, non réécrits).
  - **Gate ciblé** (sous verrou, `kesh_315`, cible du worktree, `scripts/mem-guard.sh`, `315-review-p1-gate.log`) :
    `cargo fmt --all -- --check` vert, `cargo clippy --workspace --all-targets -- -D warnings` vert,
    `cargo nextest run -E 'binary(textes_coherents) | binary(configuration_transmise)'` **47 exécutés, 47 passés**
    (17 + 30). Ni gate complet ni E2E rejoués : aucun code n'a changé depuis le dernier commit de code (`4a35ec1c`).
  - La passe a rendu un MEDIUM : une passe P2 suit — **ciblée** (Haiku, une lentille *regression hunter*, sur le seul
    diff aplati `adff757c..HEAD`, manuel et PDF nommés, `315-review-prompt-p2.md`) ; la boucle ne se clôt que sur une
    passe sans finding au-dessus de LOW.
