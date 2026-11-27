# Story 39-11b : réglages du compte Wise et import des relevés, ventilé par devise

Status: done

*Fiche rédigée le 2026-11-20, branche `epic/39-wise`, numéros de ligne relevés au code de `origin/main` (`fd04099f`),
revérifiés à la validation P1. Epic 39, **phase 1 KISS** (G2, G3, G4, G10, G11). **Dépend de** : 39-11a (lecteur),
**38-11b** (colonnes `provider`, `provider_account`, `fee_account_id`, contrainte qui admet déjà `'wise'`, réglages
prestataire d'un compte bancaire, cœur de l'import ventilé ; fiche validée P1 lue dans le worktree de l'Epic 38 le
2026-11-20). **Suivie par** : 39-11c (rapprochement), 39-11d (écrans). **Aucune migration** (décision de
l'orchestrateur du 2026-11-20, Q-39-10 tranchée).*

## Story

En tant que **Comptable** (les routes des comptes bancaires sont des routes Comptable et plus, `comptable_routes`,
`crates/kesh-api/src/lib.rs:579`, `:1053-1060`), je veux faire d'un compte bancaire Kesh le solde, dans une devise,
de mon compte Wise — sans IBAN obligatoire, lié à un compte du plan en devise, avec le compte de frais — puis importer
d'un geste les relevés CSV de ce compte Wise (une devise par fichier, plusieurs fichiers à la fois), sans doublon,
afin que chaque ligne attende son rapprochement (39-11c).

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-11b-1` | Un compte bancaire porte des réglages de prestataire `wise` : **libellé du compte Wise** et **compte de frais** — les colonnes de 38-11b, sans champ propre à Wise. | AC-1, AC-2 |
| `FR-39-11b-2` | Un compte Wise a au plus un compte bancaire par devise ; sans IBAN, lié à un compte du plan en devise (levée de C-37-9 de 38-11b étendue à `wise`) ; refusé aux imports bancaires génériques et aux règlements comme un compte PayPal. | AC-2 |
| `FR-39-11b-3` | Import à la demande (route et bouton) d'un ou plusieurs relevés d'un compte Wise : chaque fichier va au compte bancaire de sa devise ; une devise sans compte refuse tout l'import en la nommant. | AC-3, AC-4 |
| `FR-39-11b-4` | Une ligne = une transaction ; doublons (même identifiant Wise sur le même compte) ignorés et comptés ; soldes d'ouverture et de clôture posés ; un import qui ne prolonge pas le précédent est signalé. | AC-4, AC-5 |
| `NFR-39-11b-1` | Aucune migration : `chk_bank_accounts_provider` de 38-11b admet `'wise'`. | AC-1 |
| `NFR-39-11b-2` | Code commun avec 38-11b factorisé, non recopié ; PayPal inchangé (ses tests verts sans retouche de leurs attentes). | AC-2, AC-4, AC-6 |

## Critères d'acceptation

1. **AC-1 — `provider = 'wise'`, sans migration.** La migration de 38-11b (`20261120000001_paypal_import.sql`) pose
   `chk_bank_accounts_provider` avec `BINARY provider IN ('paypal', 'wise', 'stripe')` et un test qui admet `'wise'` au
   schéma : **aucune migration** ici, `kesh_version_min_required` et versions Cargo inchangés, aucune ligne à
   `docs/migrations-idempotence-audit.md`. Vérifié au T0 sur 38-11b telle que développée ; si elle n'admettait pas
   `'wise'`, c'est à l'Epic 38 de corriger sa migration **avant toute publication** (P8), non à cette story d'en ajouter
   une. Sauvegarde, réimport et export CSV : rien de neuf (colonnes de 38-11b, AC-7 de 38-11b).
2. **AC-2 — Réglages du compte Wise** (routes existantes `POST /api/v1/bank-accounts` et `PUT
   /api/v1/bank-accounts/{id}`, `lib.rs:1053-1060`, Comptable et plus). Le corps gagne un objet facultatif `wise:
   { account: string, feeAccountId: number | null }`, **de même forme** que `paypal` de 38-11b AC-2 (absent au `PUT` =
   inchangé, `null` = retiré, pose `provider = 'wise'`) ; `paypal` et `wise` ensemble → 400 `VALIDATION_ERROR`. La
   réponse gagne `wise: { account, feeAccountId } | null`. **Factorisation (G10)** : la fonction unique de gardes de
   38-11b AC-2 et AC-3 (état résultant ; libellé rogné 1 à 120 caractères et `refuse_card_number`,
   `crates/kesh-api/src/routes/bank_accounts.rs:387` ; compte lié obligatoire ; un compte bancaire par devise et par
   libellé **du même prestataire** ; compte de frais actif, imputable, de charge, CHF ou de la devise du compte lié ;
   réglages figés dès qu'une transaction existe ; IBAN facultatif mais contrôlé, QR-IBAN refusé ; jamais principal ;
   levée des deux refus `ACCOUNT_IN_FOREIGN_CURRENCY`, `locked_ledger_type` `:435-458` et le pré-vol `:680-691` ;
   `PATCH` refusé `BANK_ACCOUNT_FIELD_NOT_ALLOWED` ; refus `BANK_ACCOUNT_IS_PROVIDER` aux imports CAMT et CSV, au
   relevé saisi, au règlement d'une facture client et au paiement d'une facture fournisseur) est **paramétrée par le
   prestataire** ; chaque garde lit `provider IS NOT NULL` (ou le prestataire passé), jamais `= 'paypal'` en dur —
   chaque site de 38-11b qui le ferait est généralisé, relevé par `grep -n "paypal" crates/kesh-api/src/routes/` au
   Change Log. **Codes d'erreur** : ceux de 38-11b, **génériques** (`PROVIDER_ACCOUNT_NOT_FOUND`, `PROVIDER_ACCOUNT_IN_USE`, `PROVIDER_CURRENCY_DUPLICATE`, `PROVIDER_FEE_ACCOUNT_INVALID`, `PROVIDER_LEDGER_REQUIRED`, `PROVIDER_CURRENCY_ACCOUNT_MISSING`, servis par 38-11b/38-11c2 ; Q-39-10 satisfaite), réutilisés tels quels ; seuls les codes propres au format Wise sont `WISE_*`. Messages aux quatre locales (`crates/kesh-i18n/locales/*`), choix au Change Log.
3. **AC-3 — Route d'import** `POST /api/v1/wise-imports` (multipart : un ou plusieurs `file`, `account` = libellé),
   Comptable et plus, clés d'API `read-write` admises (comme `POST /api/v1/bank-imports`, `lib.rs:946`), limite du
   corps entier = `bank_import_max_mib` (même `DefaultBodyLimit` que `lib.rs:946-950`). Inscrite à `LIB_ROUTES`
   (`crates/kesh-api/tests/audit_route_registry.rs:440`) en `Traced, SansEcritureAuJournal` comme celle de 38-11b ; le
   compte de `the_registry_partition_is_what_the_story_declares` (157 au relevé de `origin/main`, plus les routes des
   Epics 38 et suivants à la reprise) relevé de un avec sa raison, et la phrase « N des M routes à verbe mutant » du
   manuel d'administration (`docs/manual/fr/admin-manual.tex`, lue par `le_manuel_dit_le_compte_du_registre`)
   **recomptée** — `.tex` seul, PDF à la release. Si 38-11b a été développée avec une route d'import prestataire
   générique, Wise s'y branche au lieu d'une route neuve (G10), choix au Change Log. Handlers `wise_imports::create` (clé du registre). Ordre des refus, rien d'écrit à
   aucun (codes `WISE_*` nommés ci-dessous, ou les `PROVIDER_*` génériques de 38-11b selon la règle de l'AC-2) : 400 `VALIDATION_ERROR` (champ manquant, aucun fichier) → 404 compte Wise introuvable (aucun compte bancaire
   non archivé `provider = 'wise'` de ce libellé ; code de l'AC-2) → 422 `WISE_CSV_INVALID` (erreur de 39-11a,
   `details` : nom du fichier et code fermé de 39-11a, puis colonnes manquantes ou erreurs de ligne, forme de
   `CsvLineErrorPayload`, `crates/kesh-api/src/errors.rs:1541`) → 422 `WISE_DUPLICATE_CURRENCY_FILE` (deux fichiers de
   même devise, `details: { currency, filenames }`) → 422 devise sans compte (`details: { currencies }`, jamais de
   solde créé d'office) → 409 doublon par devise (deux comptes bancaires non archivés de ce libellé pour une devise :
   la course de 38-11b AC-2) → 409 `BANK_ACCOUNT_CLOSED` → 422 `BANK_IMPORT_DUPLICATE_FILE` (empreinte de l'AC-5).
4. **AC-4 — Rangement** *(amendé le 2026-11-21, revue P1 M1 : le montant de l'autre devise est écrit au segment de conversion du libellé, voir le Change Log)*, par le **cœur commun de 38-11b** (AC-5 et AC-6 de 38-11b : `with_account_locks` neuf de
   `crates/kesh-reconciliation/src/mutex.rs`, à côté de `with_account_lock` `:75`, verrous pris par identifiant
   croissant ; une transaction de base, tout ou rien ; recherche de doublon par identifiant ; empreinte). Par fichier,
   **une** entrée `bank_imports` (`create_with_transactions`, `crates/kesh-db/src/repositories/bank_imports.rs:72`)
   au compte bancaire de sa devise : `source_format = 'CSV'` (valeur de 38-11b, aucune neuve), `filename`, période =
   `first_date`..`last_date`, `opening_balance` et `closing_balance` de 39-11a AC-5 (colonnes existantes,
   `migrations/20260504000001_bank_imports.sql:29-30`). Chaque ligne donne une `NewBankTransaction`
   (`crates/kesh-db/src/entities/bank_transaction.rs:113`, champs de 38-11b compris) : `amount` = `Amount` (signé),
   `currency`, `booking_date` = `Date`, `value_date = None`, `transaction_id` = identifiant Wise, `reference` =
   `Payment Reference` (sinon `None`), `end_to_end_id = None`, `counterparty_name` = `Merchant`, sinon `Payee Name`
   (débit) ou `Payer Name` (crédit), `counterparty_iban` = `Payee Account Number` sans espaces et en majuscules **s'il
   a la forme d'un IBAN** (deux lettres, deux chiffres, 11 à 30 caractères alphanumériques, ≤ 34, colonne
   `VARCHAR(34)`, `20260504000001_bank_imports.sql:68`) — sinon `None` et le numéro va aux détails (les relevés 2024
   portent aussi des numéros à huit chiffres et des numéros masqués d'étoiles) —, `counterparty_email = None`,
   `provider_type` = `id_prefix` (filtre de règle de 38-11b, G5), `details` = `Description`, `Transaction Details
   Type`, numéro de compte non IBAN, `Exchange From`→`Exchange To` au cours `Exchange Rate`, `Note`, non vides, joints
   par « · », tronqués comme les autres imports (à la frontière d'un caractère : U+202F tient sur trois octets).
   `Total fees` n'est **jamais** une transaction (les frais sont leurs propres lignes, 39-11a). Somme des transactions
   insérées d'un fichier sans doublon = clôture − ouverture (test). Trace d'audit : `bank_import.created` par fichier,
   comme l'import CSV, plus `wise_import.created` (`details` : libellé, fichiers, compte rendu), sous l'acteur
   (`NewAuditLogEntry::for_actor`, utilisateur ou clé d'API ; garde `audit_acteur_garde.rs`).
5. **AC-5 — Doublons, continuité, compte rendu.** Empreinte : la fonction de 38-11b AC-6 (SHA-256 de `fichier ‖ ":" ‖
   devise`), refus de l'AC-3 si `find_by_company_and_hash` (`bank_imports.rs:308`) la trouve. Doublon de transaction :
   même `transaction_id` déjà sur ce compte bancaire → ligne ignorée (la fenêtre date-montant-référence des imports
   bancaires, `find_in_dedup_window`, `crates/kesh-db/src/repositories/bank_transactions.rs:83`, ne s'applique pas,
   comme pour PayPal) ; un fichier tout en doublons ne crée pas d'entrée. **Continuité avec l'import précédent** du
   même compte, **une seule définition** (servie aussi par les écarts de 39-11c, qui ne peut pas la lire des doublons :
   le fichier n'est pas conservé, `transaction_count` ne compte que les lignes insérées), jugée sur le **premier jour** du
   fichier seul : un fichier **prolonge par chevauchement** un relevé du compte quand `overlapping_statements(company, compte,
   first_date, first_date)` (`bank_imports.rs:386`, `period_from <= first_date AND period_to >= first_date`, relevés saisis ou
   importés ; à la revue, l'import jugé exclu) en rend au moins un — pas d'avertissement. Jamais le chevauchement de toute la
   période (`first_date..last_date`) : un relevé **postérieur** qui chevauche la fin d'un import masquerait, à la revue,
   l'écart du début de cet import, et un import rattrapé après coup qui chevauche un relevé plus récent ne serait jamais
   jugé. Sinon, par la fonction
   existante `previous_statement_closing` (`bank_imports.rs:358` : le relevé à solde du compte dont `period_to` <
   `first_date`, le plus tardif) : si elle en rend un et que sa clôture diffère de `opening_balance` du fichier →
   avertissement `gap` au compte rendu (l'import passe) ; aucun relevé antérieur → rien. Le jugement est celui de l'instant de l'import : la revue de 39-11c (AC-6) relit la même règle sur l'état présent des relevés du compte, un relevé importé ou saisi depuis peut donc changer sa réponse, rien n'est stocké. La clôture lue est celle d'un
   relevé à solde : les imports Wise en ont toujours une, un relevé saisi sans solde est ignoré par la fonction. Réponse `201` : `{ account, files: [{ filename, currency, bankAccountId,
   importId | null, inserted, duplicates, openingBalance, closingBalance, warnings: [ { code: "gap", expected, found,
   previousImportId } ] }] }`.
6. **AC-6 — Non-régression et factorisation.** Tests de 38-11b (PayPal) et des imports CAMT, CSV, manuel inchangés
   en assertions ; le Change Log nomme chaque fonction de 38-11b réutilisée ou généralisée, et confirme qu'aucune
   n'est recopiée. Aucun outil MCP (comme 38-11b).

## Tâches

- [x] T0 — Reprendre `main` et `epic/38-paypal` ; lire 38-11b telle que développée (contrainte, codes, route) ;
  relever les sites `paypal` en dur.
- [x] T1 — Gardes prestataire paramétrées, objet `wise`, refus aux autres chemins pour `wise` (AC-2).
- [x] T2 — Route d'import, rangement par fichier, doublons, continuité, compte rendu ; registre des routes et phrase du
  manuel d'administration (AC-3 à AC-5).
- [x] T3 — i18n, non-régression (AC-6).

## Stratégie de tests

- **HTTP** (`crates/kesh-api/tests/wise_import_e2e.rs`, `#[sqlx::test]` sur le squash) avec les fixtures de 39-11a :
  société multi-devises, trois comptes bancaires du libellé fictif ; import des trois fichiers → trois entrées,
  nombres et soldes attendus ; réimport → 422 `BANK_IMPORT_DUPLICATE_FILE` ; fichier chevauchant → seules les lignes
  neuves insérées, aucun `gap` ; fichier suivant discontinu → `gap` ; import rattrapé après coup dont la fin chevauche un relevé plus récent et
  dont le début ne prolonge pas le précédent → `gap` ; devise sans compte → 422, rien d'écrit ; deux
  fichiers de même devise → 422 ; compte clos → 409 ; clé d'API `read` → 403 ; numéro de compte non IBAN → `None` et
  détails. Réglages : chaque refus de l'AC-2 pour `wise` ; `paypal` et `wise` ensemble → 400 ; import CAMT, relevé
  saisi, règlement de facture et paiement fournisseur sur un compte Wise → 409 `BANK_ACCOUNT_IS_PROVIDER` ; PayPal
  inchangé.
- Gate complet au push.

## Modules touchés (4)

`kesh-api` (routes `bank_accounts`, `wise_imports`, erreurs), `kesh-reconciliation` (appel de `with_account_locks`,
sans changement), `kesh-import` (lecture de 39-11a), `kesh-i18n`. Plus une phrase du manuel d'administration (`.tex`).
*(Au développement : `kesh-db` aussi — l'entité `BankAccountProvider` sert l'objet `wise`, et la règle de continuité
unique `bank_imports::continuity_gap` vit au dépôt, pour que 39-11c la relise ; cinq modules, sous le seuil de
découpage.)*

## Note de version

*Ajouté* — Un compte bancaire peut être le solde, dans une devise, d'un compte Wise : sans IBAN, lié à un compte du
plan en devise, avec son compte de frais. Les relevés CSV de Wise s'importent d'un geste, plusieurs devises à la
fois ; les doublons sont ignorés et un trou entre deux relevés est signalé. Un compte Wise ne reçoit ni relevé
bancaire ni règlement de facture.
*Migration* — aucune.

## Notes de développement

- Ancienne 39-1 (client Wise, connexion et coffre, `superseded`) : remplacée par les réglages de 38-11b.
- ⚠️ 38-11b est `drafted` (validée P1) au 2026-11-20 : ses noms (objet `paypal`) peuvent changer
  avant son développement ; relire au T0 (codes génériques `PROVIDER_*` acquis, phase 1 validée P3, `28623871`).

## Change Log

- 2026-11-20 — fiche rédigée (phase 1 KISS), non validée.
- 2026-11-20 — validation P1 (Claude Opus 5.5, contexte frais) : 0 CRITICAL, 2 HIGH, 4 MEDIUM, 3 LOW, remédiés dans la
  fiche (rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-11b-compte-wise-et-import-P1.md`). HIGH : continuité
  lue sur « le solde courant de la dernière transaction », qui n'est stocké nulle part → `previous_statement_closing`
  existant, sauté quand le fichier chevauche ; migration de repli retirée (Q-39-10 tranchée : 38-11b admet `'wise'`).
  MEDIUM : rôle Administrateur → Comptable et plus (38-11b validée) ; registre `LIB_ROUTES` et phrase du manuel
  d'administration absents ; refus `BANK_ACCOUNT_IS_PROVIDER` et `PATCH` non étendus à `wise` ; verrou
  `with_account_lock` au lieu de `with_account_locks` de 38-11b. LOW : limite du corps multi-fichiers, place du refus
  de devise en double (après la lecture), troncature UTF-8 des détails.
- 2026-11-20 — validation P2 (Claude Sonnet 5.5, contexte frais) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 1 LOW, remédiés (rapport
  `kesh-gate-logs/epic-39-40-kiss/validation/39-11b-compte-wise-et-import-P2.md`). Remédiation P1 relue au code d'`origin/main`
  et à 38-11b (validée P2) : rien à reprendre. MEDIUM : la règle de continuité « aucune ligne du fichier n'est un doublon »
  n'était pas relisible par les écarts de 39-11c (les doublons ne sont pas stockés) et, relue à la revue, signalait à tort
  l'import qui en chevauchait un autre (`previous_statement_closing` rend alors un relevé plus ancien) → chevauchement par
  `overlapping_statements`, une seule définition. LOW : codes nommés `WISE_*` à l'AC-3 contre la règle de l'AC-2, clé du
  registre des routes absente.
- 2026-11-20 — validation P3 ciblée (Claude Opus 5.5, contexte frais, commit `02ed58c6` seul) : 0 CRITICAL, 0 HIGH, 1 MEDIUM,
  0 LOW, remédié (rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-11-P3-ciblee.md`). MEDIUM : le chevauchement jugé
  sur toute la période (`first_date..last_date`) ne donnait pas la même réponse à l'import et à la revue — un relevé
  postérieur chevauchant la fin d'un import masquait à la revue l'écart du début, et un import rattrapé après coup n'était
  jamais jugé → chevauchement du **premier jour** seul (`overlapping_statements(…, first_date, first_date)`, l'import jugé
  exclu à la revue), complémentaire de `previous_statement_closing` (`period_to < first_date`) ; test ajouté.
- 2026-11-20 — Validation P5 ciblée (Claude Sonnet 5.5, passe ciblée, contexte frais, rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-11-P5-ciblee.md`) : 0 C/H/M, 2 LOW remédiés — G4, 39-11b AC-5 et 39-11c AC-6 disent la même chose (l'import juge à son instant, la revue relit la même règle sur l'état présent) ; 39-11c AC-3 : 40-7c n'écrit le texte qu'en dernier recours.
- 2026-11-20 — alignement sur la phase 1 de l'Epic 38 (Claude Sonnet 5.5, contexte frais ; fiches 38-11c1, 38-11c2, 38-11d et rapport de validation P3 de `28623871` lus) : codes d'erreur de 38-11b génériques `PROVIDER_*` (liste nommée), plus de branche `PAYPAL_*` ; note finale sur les noms de 38-11b mise à jour. Rapport `kesh-gate-logs/epic-39-40-kiss/validation/alignement-38-P3.md`.

- 2026-11-21 — développée (Claude Opus 5.5) : T0 à T3, AC-1 à AC-6 ; gates au Dev Agent Record ; passée en `review`, prompt de
  la revue P1 : `kesh-gate-logs/s39-11b/revue-p1-prompt.md`.

## Dev Agent Record

### Agent

Claude Opus 5.5 (`bmad-dev-story`), worktree `kesh-wt3911b`, branche `story/39-11b` partie d'`epic/39-wise` (`c0dd8731`).

### T0 — relevés au code

- AC-1 vérifiée : `20261120000001_paypal_import.sql` pose `BINARY provider IN ('paypal', 'wise', 'stripe')` ; aucune
  migration, `kesh_version_min_required` et versions Cargo inchangés, aucune ligne à l'audit d'idempotence.
- Sites `paypal` en dur de la 38-11b (`grep -n "paypal" crates/kesh-api/src/routes/`) : dans `bank_accounts.rs`,
  `validate_bank_input`, `normalize_paypal_settings`, `resulting_paypal`, `check_provider_guards_in_tx`
  (`PROVIDER_PAYPAL` à la garde d'unicité), `refuse_provider_change_in_use` (`.paypal()`), les deux `INSERT`/`UPDATE`
  (`PROVIDER_PAYPAL` lié) ; dans `paypal_imports.rs`, la relecture sous verrou et les traces. **Déjà génériques**, sans
  retouche : `refuse_provider_account` (imports CAMT et CSV, `provider_settings.is_provider()`), relevé saisi, règlement
  de facture (`invoice_settlements_write.rs`, `provider.is_some()`), paiement fournisseur (`supplier_invoices.rs`), lot
  pain.001 (`payment_batches.rs`), `PATCH` (`patch_bank_account_journal_link`, `is_provider()`), garde de devise du
  compte du plan (`accounts.rs`, `provider IS NOT NULL`), levée d'`ACCOUNT_IN_FOREIGN_CURRENCY`
  (`locked_ledger_type`, booléen `provider`). Après la story, `PROVIDER_PAYPAL` ne reste dans `bank_accounts.rs` que
  dans la correspondance corps → prestataire (`provider_of_create`, `resulting_provider`).
- Cœur d'import : celui de la 38-11b vivait dans le handler PayPal ; la 38-12a l'extrait (`provider_imports.rs`,
  `story/38-12a`, `d64688fc`, lecture seule), **non fusionnée** dans `epic/39-wise`. Choix : paramétrer le cœur de la
  38-11b **là où il est** (`paypal_imports.rs`), sans recopie, et écrire ce qu'il faudra raccorder (ci-dessous).

### Fonctions de la 38-11b réutilisées ou généralisées (AC-6 ; aucune recopiée)

- `bank_accounts.rs` : `normalize_paypal_settings` → `normalize_provider_settings` ; `resulting_paypal` →
  `resulting_provider` ; `check_provider_guards_in_tx` et `refuse_provider_change_in_use` paramétrées par
  `ProviderChoice` (prestataire + réglages ; changer de prestataire compte comme un changement de libellé) ;
  `validate_bank_input`, `recheck_ledger_type_in_tx`, `locked_ledger_type`, `validate_journal_account_id` inchangées
  hors le champ `provider` ; neuves : `ProviderChoice`, `provider_of_create`, `both_providers_refusal`.
- `kesh-db` : `PaypalSettings` → `ProviderSettings` ; `BankAccountProvider::settings_for`, `served`, `wise` (et
  `paypal` réécrite dessus) ; `PROVIDER_WISE` ; JSON `wise`.
- `paypal_imports.rs` : le cœur sorti du handler et paramétré — `resolve_currency_balances` (refus devise manquante,
  doublon de devise, compte clos), `file_currency_hash` (empreinte), `run_provider_import` / `import_under_locks` /
  `recheck_targets_locked` / `write_under_locks` (verrous, rejeu, relecture, empreinte, `COMMIT`),
  `insert_currency_import` (entrée `bank_imports` et `bank_import.created`), trait `ProviderWrite` (écriture propre au
  prestataire, future en boîte, patron de `mail::MailFuture`) ; `truncate_chars`, `truncate_bytes` et les bornes de
  colonnes passent `pub(crate)`. Le handler PayPal (`create`, `PaypalWriter`) les appelle : ordre des refus, réponse,
  traces et opération de rejeu inchangés.
- Réutilisées telles quelles : `list_provider_balances(_locked)`, `existing_transaction_ids`,
  `find_by_company_and_hash`, `create_with_transactions`, `insert_canonical_audit_log`, `with_account_locks`,
  `retry_app_on_deadlock`, `overlapping_statements`, `previous_statement_closing` (composées dans `continuity_gap`).

### Choix

- Codes : `PROVIDER_*` de la 38-11b tels quels ; neufs `WISE_CSV_INVALID` (422, `details.filename` + `reason` +
  détail du motif du lecteur) et `WISE_DUPLICATE_CURRENCY_FILE` (422) ; `paypal` et `wise` ensemble →
  `400 VALIDATION_ERROR` (`error-provider-settings-both`). Messages aux quatre locales, plus
  `audit-log-action-wise-import-created`.
- Route neuve `POST /api/v1/wise-imports` (la 38-11b n'a pas de route prestataire générique).
- `PUT` : `wise: null` sur un compte PayPal (et `paypal: null` sur un compte Wise) est sans effet ; poser l'objet de
  l'autre prestataire fait passer le compte, sous la garde 5.
- Continuité : `kesh_db::repositories::bank_imports::continuity_gap(conn, société, compte, first_date, ouverture,
  exclude_import_id)` — la règle unique de l'AC-5 ; 39-11c l'appelle avec l'identifiant de l'import jugé. Jugée sous
  les verrous, dans la transaction, avant l'insertion du fichier.
- Un fichier tout en doublons : pas d'entrée, mais il figure au compte rendu (soldes, avertissement éventuel) et à la
  trace `wise_import.created`.
- Règles R1 à R3 (`wise_duplicates.rs`, `rules: Option<&dyn DuplicateRules>`) : **hors de cette story** — elles
  n'ont d'objet qu'avec des lignes téléchargées (39-12a3 (fiche sortie de l'epic, branche `futur/wise-api`)) ; aucune ne se déclenche sur un import CSV seul. Rien n'est
  écrit qui les empêche (le cœur reçoit déjà l'écriture du prestataire).
- Manuel utilisateur : non touché (39-11d, écrans et manuel).

### À raccorder à la fusion avec `story/38-12a` (cœur extrait `provider_imports.rs`)

1. `routes/paypal_imports.rs` : prendre la version de la 38-12a (cœur extrait). Disparaissent alors de cette branche
   `resolve_currency_balances`, `file_currency_hash`, `LockedTarget`, `ProviderWrite`/`WriteFuture`,
   `run_provider_import`, `import_under_locks`, `recheck_targets_locked`, `write_under_locks`,
   `insert_currency_import`, `CurrencyImport`, `PaypalWriter` ; leurs équivalents sont `load_balances`,
   `import_lines`, `ImportRequest`, `ProviderLine`, `file_hash`, `dedupe_groups`.
2. Étendre `provider_imports.rs` pour Wise (que le cœur de la 38-12a ne sait pas encore faire) :
   a. **nom et empreinte par devise** (`ImportRequest` n'a qu'un `filename` et un `ImportSource::File(bytes)` pour
      toutes les devises ; Wise : un fichier, donc un nom et des octets, par devise — ce que 39-12a3 attend « depuis
      39-11b ») ;
   b. **solde d'ouverture, solde de clôture et période du fichier** par devise (`import_lines` pose
      `opening_balance: None`, la période des lignes gardées et la clôture de `balance_after`) ;
   c. **`ProviderLine.counterparty_iban`** (`into_new_bank_transaction` le pose à `None`) ;
   d. **avertissements de continuité** : appel de `bank_imports::continuity_gap` par devise, sous les verrous, avant
      l'insertion, rendu au compte rendu (`CurrencyReport.warnings`, omis vide pour garder la réponse PayPal octet
      pour octet) ;
   e. le point d'accroche `rules: Option<&dyn DuplicateRules>` de 39-12a3 (Wise CSV : `WiseDuplicateRules`).
3. `routes/wise_imports.rs` : `WiseWriter` remplacé par la construction des `ProviderLine` (un groupe = une ligne,
   `group_id = transaction_id`, à partir de `transaction_of_row`) et l'appel de `import_lines` ; le compte rendu
   `FileReport` construit depuis `CurrencyReport`.
4. `tests/audit_route_registry.rs` : point (vi), l'enveloppe de rejeu vit dans `provider_imports::import_lines` ;
   compte des routes, des tracées et phrase du manuel (`151 des 159`, `151 + 4 + 4 = 159`) **recomptés** après la
   fusion (la 38-12a ajoute les routes de téléchargement PayPal).
5. Catalogues i18n : les deux branches écrivent après `error-paypal-csv-invalid` — garder les deux blocs.
6. *(revue P1, M2)* **Registre fermé des actions de trace** : `provider_imports::AUDIT_ACTIONS` (la 38-12a refuse en
   `AppError::Internal` une action inconnue) doit recevoir `wise_import.created`, qui s'inscrit aussi à
   `audit_labels::ACTIONS`, aux quatre `.ftl` et à l'inventaire `SITES_INDIRECTS` d'`audit_label_registry.rs` ; sans cela
   tout import Wise rend 500 après le raccord.
7. **Forme de la trace** : le cœur de la 38-12a écrit lui-même `currencies` (ses `CurrencyReport`) aux `audit_details` ;
   la trace `wise_import.created` porte ici `files` (nom, soldes, avertissements). `CurrencyReport` n'a ni `filename`,
   ni `openingBalance`/`closingBalance`, ni `warnings` : décider ce que devient la clé `files` (le point 3 ne le dit pas).
8. **Entrée `bank_imports`** : la 38-12a prend la période des lignes **gardées** et `opening_balance: None` ; la règle de
   continuité de la 39-11c relit `period_from` et `opening_balance` **stockés**. Contrainte : période et ouverture du
   **fichier entier**, tenues par un test (fichier à doublons partiels).
9. **Conflits de texte attendus** (inventaire) : `lib.rs` (routes), `errors.rs` (variantes), `audit_labels.rs`,
   `audit_route_registry.rs`, et dans `kesh-import` : `csv/amount.rs`, `lib.rs`, `paypal.rs` (touchés par la 38-12a et
   la 39-11a).
10. **Épic 40** : `BankAccountProvider::served()` est la liste unique des prestataires servis (`PROVIDER_PAYPAL`,
    `PROVIDER_WISE`) ; un prestataire réservé (`stripe`) voit aujourd'hui un `PUT` effacer ses réglages (`provider =
    NULL`) — inatteignable par l'API aujourd'hui (revue P1, L1), à étendre avec la 40-x. Et la désérialisation
    `json.paypal.or(json.wise)` ne contrôle pas `provider` (L2) : prendre l'objet nommé par `provider`.

Le raccord est porté par la story **`39-14-raccord-epic-38`** (registre `sprint-status.yaml`, `backlog`), à la fusion
d'`epic/38` dans `epic/39`.

### Tests ajoutés (recomptés, `epic/39-wise` → dernier commit de code `368369e3`)

- `crates/kesh-api/tests/wise_import_e2e.rs` (neuf) : **7** tests `#[sqlx::test]` — import de trois relevés et
  doublon de fichier ; continuité (chevauchement, `gap`, relevé rattrapé après coup, relevé qui prolonge) et doublons
  de lignes, fichier tout en doublons ; refus dans l'ordre de l'AC-3, rôle Consultation, clés `read` et
  `read-write` ; rejeu sur interblocage ; création d'un solde Wise et chaque garde ; `PUT` (absent, `null`, passage
  à PayPal, figé après import, compte de frais) et `PATCH` ; refus aux autres chemins (CAMT, relevé saisi, règlement).
- `crates/kesh-api/src/routes/wise_imports.rs` : **8** tests unitaires (forme d'IBAN, contrepartie et frais,
  conversion, somme = clôture − ouverture, troncature, fichiers de même devise).
- Assertions changées, déclarées : `kesh-db` `les_reglages_de_prestataire_en_json` (le JSON gagne `wise` ; le cas
  « prestataire réservé » passe de Wise à Stripe), `bank_account_opening_balance_e2e::creation_sans_solde_comportement_d_avant`
  (dix-huit clés, `wise` en plus), `audit_route_registry` (159 routes de `lib.rs`, 151 tracées, 163 au registre,
  130 sans écriture au journal). Aucune assertion des tests PayPal (`paypal_import_e2e`) changée.

### Sondes négatives (`kesh-gate-logs/s39-11b/mutations.sh`, `mut-*.log`) — six mutations, six rouges

premier jour → chevauchement sur 60 jours (`import_continuite_et_doublons_de_lignes`) ; unicité par prestataire →
`PROVIDER_PAYPAL` en dur (`creer_un_solde_wise_et_ses_refus`) ; filtre des doublons retiré
(`import_continuite_et_doublons_de_lignes`) ; IBAN de 8 caractères admis (`la_forme_d_un_iban`) ; `paypal: null`
qui retire tout prestataire (`modifier_un_solde_wise`) ; changement de prestataire ignoré par la garde 5
(`modifier_un_solde_wise`).

### Gates (cible `<cibles>/kesh-wt3911b`, `CARGO_INCREMENTAL=0`, file `heavy.sh`)

Tous au **dernier commit de code `385adfaf`** (remédiation du LOW L1 de la revue P2), journaux `kesh-gate-logs/s39-11b/` ;
Claude Sonnet 5.5, 2026-11-21. Les gates joués à `368369e3` (avant les remédiations P1 et P2) sont remplacés.

- **Gate ciblé** (fmt, clippy workspace `-D warnings`, nextest `test(wise) | binary(wise_import_e2e)`) : vert,
  **19 passés** (`cible-p2.log`, `cible-p2.console`).
- **Gate complet** `scripts/test-fast.sh --ci`, base `kesh_s3911b` remise à zéro (`kesh-mariadb-dev5`) : fmt et clippy verts,
  nextest **6271 passés, 15 ignorés** (`gate-backend.log`).
- **`scripts/gate-ci.sh`** (copie `git archive` de `385adfaf`, enveloppes lisant `gate-ci-wrap/place.env`, écrit par le
  script, sans forçage de dev2) : **6290 passés, 0 échoué, 19 ignorés** (`gate-ci.log`, code 0). Le premier passage
  (`gate-ci-1-unknowndb.log`) a rougi sur 37 tests (`Unknown database 'kesh_ci_s3911b'`, base disparue du serveur en cours
  de gate : environnement, comme à `368369e3`) ; rejoué sans autre changement, vert. La cause de la disparition de la base
  n'est pas élucidée ici.
- **E2E complet** (`LANG=fr_CH.UTF-8`, base `kesh_e2e_s3911b`, backend port 3911) : **477 passés, 10 échecs, 18 ignorés** —
  les dix attendus et eux seuls : les sept de la KF-029 (#97 : `mode-expert.spec.ts:26`, `:41`,
  `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`), `backup-restore-new-install.spec.ts:262`
  et `:381` (KF-065, #1282), `opening-balances.spec.ts:156` (KF-067, #1284) (`e2e.log`).
- Frontend : non touché par la story (aucun fichier de `frontend/`), gates frontend non relancés ; le build du frontend a
  servi l'E2E (`e2e-build.log`).
- Les journaux `mut-*.log` et `mutations*.log` des six sondes négatives sont ceux du passage à `368369e3` ; la mutation du
  test neuf `une_longue_description_ne_rogne_pas_la_conversion` n'a pas été jouée (à faire à la passe ciblée P3).

### Revue de code P1 (Claude Opus 5.5, 2026-11-21) — remédiation

- **M1** : `Exchange To Amount` est écrit au segment de conversion du libellé, forme fixe `DE→VERS [MONTANT]
  [au cours COURS]` (le montant est celui reçu dans la devise VERS ; sans montant ni cours : `DE→VERS`). Aucun champ
  structuré de `bank_transactions`/`NewBankTransaction` ne convient (ni montant d'origine, ni contre-valeur) ; aucune
  migration. Test `le_segment_de_conversion_a_une_forme_fixe`, attente de `une_conversion_porte_son_montant_et_son_cours`
  mise à jour (« EUR→USD 109.00 au cours 1.09 »).
- **M2** : liste « À raccorder » complétée (points 6 à 10) ; story `39-14-raccord-epic-38` au registre.
- **L1, L2** : notés au point 10 pour l'Epic 40. **L3** : paiement fournisseur et lot pain.001 ajoutés à
  `les_autres_chemins_refusent_un_compte_wise`. **L4** : deux lignes d'`api-external.md` (« ou Wise »). **L6** :
  `is_paypal()` supprimé (aucun appelant). **L7** : `gate-ci.sh` à rejouer avant le push (voir plus haut).
- **L5 — textes du journal changés, déclarés** : le message de journal serveur « import PayPal : libération des verrous
  en échec » devient « import de prestataire : … » (avec champ `provider`), idem « verrous des comptes non acquis », et
  l'erreur interne « import PayPal : trop de transactions » devient « import {provider} : trop de transactions ». Ni
  réponse, ni trace d'audit, ni ordre des refus ne changent ; « PayPal inchangé » s'entend hors ces textes.
- Gate ciblé de la remédiation : fmt, clippy workspace, nextest de `wise_import_e2e`, `paypal_import_e2e`, `audit_route_registry`, des tests `wise_imports` et `bank_account` : 58 verts sur 59, le rouge (facture fournisseur sans compte fournisseur par défaut dans le test neuf, corrigé) puis rejoué vert (`cible-p1-rem.log`, `cible-p1-rem-2.log`). Gate complet au push.

### Revue de code P2 et LOW L1 (Claude Sonnet 5.5, 2026-11-21)

- P2 : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 5 LOW (`revue-p2-rapport.md`). **L1 corrigé** (`385adfaf`) : `transaction_of_row` rogne la
  tête du libellé (description, type de détail, numéro non IBAN) pour réserver la place du segment de conversion ; la note
  reste rognée en premier. Test `une_longue_description_ne_rogne_pas_la_conversion`. Le code de production étant touché, la
  boucle se rouvre sur une **passe ciblée P3** de ce seul commit (`revue-p3-prompt.md`) ; statut `review`.
- L2 (deux fichiers de la 38-12a non nommés à la liste « À raccorder »), L3 (sonde négative du paiement fournisseur non
  rejouée), L4 (fmt/clippy de la remédiation P1 sans journal : couverts par le gate ciblé et le gate complet ci-dessus) :
  non corrigés, sans effet sur le code. La relisibilité du segment par la 39-11c (L1, seconde partie) est à fixer dans sa
  fiche.

### Passe ciblée P3 et clôture (2026-11-21)

Passe ciblée P3 sur le seul commit `385adfaf` (correctif L1, une lentille « régression du correctif », Claude Haiku 5.5),
rapport `kesh-gate-logs/s39-11b/revue-p3-rapport.md` : **0 CRITICAL · 0 HIGH · 0 MEDIUM · 3 LOW**. Sonde par mutation
(rognage de la tête retiré, `wise_imports.rs:198`) : `une_longue_description_ne_rogne_pas_la_conversion` rougit
(`mut-p3.log`), tête non mutée 10/10 verte. Les trois LOW (séparateur coupé en bout de tête, segment de conversion
au-delà de 500 caractères, tête vide avec conversion non testée) ne décrivent aucun comportement faux observé : ils vont
en issue `technical-debt`, non corrigés ici, pour ne pas rouvrir la boucle sur la production.

**Tendance de la boucle de revue de code** : P1 (Opus, complète) → P2 (Sonnet, complète, 0 au-dessus de LOW, L1 corrigé
dans la production) → P3 (Haiku, ciblée sur `385adfaf`) : 0 au-dessus de LOW. Boucle close sur P3. Gates du dernier
commit de code `385adfaf` : backend complet 6271 passés / 15 ignorés ; `gate-ci.sh` 6290 / 0 / 19 (rejoué après un
« Unknown database » d'environnement) ; E2E 477 passés, les 10 échecs attendus, 18 ignorés.
