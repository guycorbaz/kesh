# Story v017-4a : factures fournisseur — brouillon, validée, payée — le backend

Status: review

*Sous-fiche issue du découpage de la Story v017-4 à sa rédaction (2026-10-31, C-v017-4-1). Worktree
`/home/gcorbaz/devel/kesh-0175`, branche `story/v017-4-facture-fournisseur-brouillon` (partie de `8e126cfd`) ; la 4a
d'abord, puis la 4b. Registre **commun** : `v017-4-choix-autonomes.md` (C-v017-4-1 à 42 ; C-v017-4-43 et suivants au
développement). Issues : **#849**, #853 — en `refs` seulement. **Le texte des AC, le contexte, les décisions, le
contrat de l'API, les codes d'erreur, la précédence, les verrous, l'inventaire clos, les limites et les questions au
Project Lead sont dans la fiche parente `v017-4-facture-fournisseur-brouillon.md`** : ils valent ici sans être recopiés.
Les numéros de ligne cités sont ceux de `8e126cfd`.*

## Story

En tant que Guy, je veux que l'API de Kesh tienne le cycle brouillon → validée → payée d'une facture fournisseur — et
que la 0.17.0 monte sans perte une base 0.16.x ou rc1 peuplée —, afin que l'écran de la `v017-4b` n'ait plus qu'à le
présenter.

## Périmètre

`kesh-db` (migration `20261031000001_supplier_invoices_draft`, dépôt des factures fournisseur, K8, `delete_in_tx` —
doc-comment —, lettrage et lots — types), `kesh-api` (routes `PUT`, `DELETE`, `validate`, `unvalidate`, filtre de la
liste, création et complétion en brouillon, règles « document » extraites, erreurs, audit, MCP, export CSV, registres),
`kesh-i18n` (messages d'erreur, libellés d'audit, quatre catalogues), `kesh-seed`, `docs/api-external.md`,
`docs/migrations-idempotence-audit.md`, les tests Rust, et les textes liés à la migration : CHANGELOG (en-tête et
*Modifié*), manuel d'administration et son PDF. *(Corrigé à la validation P2, F10, R13 : la première rédaction disait
« livrable seul ».)* **Testable seule en HTTP, publiée avec la 4b** (dérogation C-v017-4-31) ; elle écrit aussi le
helper E2E `createSupplierInvoiceViaApi` qui valide, le helper du relevé saisi par l'API et les étapes par l'API de la
spec `supplier-invoice-draft-cycle.spec.ts`, qu'elle joue en E2E ciblés.

## Critères d'acceptation

**AC-1 à AC-20** de la fiche parente, sous leur numéro d'origine. Mutations **M1 à M20**, dont M12a et M12c (M12b retirée à la validation P2,
mutant équivalent) et M20-bis à M20-quater ; et le **début de l'AC-30** (helpers E2E, étapes API de la spec neuve,
validation P3). **Dérogation écrite** (fiche parente, § *Découpage*, C-v017-4-31) : la 4a ne se publie
qu'avec la 4b, sur la même branche.

## Tâches

- [x] Étape 0 — relire au code les prémisses citées par la fiche parente (§ *Ce que le code dit*, § *Inventaire
      clos*) ; tout écart au Change Log, ligne lue citée.
- [x] Tests rouges d'abord (commit séparé, sortie rouge au Dev Agent Record) : AC-1 (contraintes), AC-2 (deux montées),
      AC-3, AC-5 à AC-16.
- [x] Migration (dont la colonne `link_origin`, et le repli du `MODIFY COLUMN` joué deux fois), P5 à P8, squash,
      `migrations.sha384` (AC-1).
- [x] Entité et lecteurs de l'écriture d'achat (AC-4) ; inventaire clos tenu au Dev Agent Record.
- [x] Dépôt : `create_draft_in_tx`, `update_draft`, `delete_draft`, `validate_in_tx`, `unvalidate`, `create_validated*`
      composé (AC-6 à AC-12) ; tête `SupplierInvoiceDraft` (AC-13).
- [x] Routes, règles « document » extraites, #853 serveur, filtre de liste (AC-6 à AC-11, AC-14, AC-15).
- [x] K8 (AC-16) ; MCP (AC-17) ; audit, registres, messages (AC-18).
- [x] Appelants des tests et du seed (AC-12).
- [x] `api-external.md` (AC-19) ; CHANGELOG, manuel d'administration et PDF (AC-20).
- [x] *(Validation P3, R8 = F7.)* Début de l'AC-30 : helper `createSupplierInvoiceViaApi` qui valide après la création
      (`frontend/tests/e2e/helpers/api-fixtures.ts:249`), helper du relevé saisi par l'API (équilibré au centime), spec
      `supplier-invoice-draft-cycle.spec.ts` réduite à ses étapes par l'API — joués en E2E ciblés.
- [x] Mutations M1 à M20 (M12a, M12c ; M20-bis à M20-quater, tests pilotés).
- [x] Gate complet backend (base reconstruite), Vitest et **E2E ciblés** de ses routes au dernier commit de code de la
      4a ; l'**E2E complet** se joue au dernier commit de code de la **4b** (dérogation, C-v017-4-31) — le Dev Agent Record
      le dit ainsi ; décomptes recomptés.

## Dev Agent Record

*Développement du 2026-10-31, Opus 5.5, en autonomie, worktree `kesh-0175`, sur mandat
`kesh-gate-logs/prompts/v0174a-dev.md`. Journaux sous `/home/gcorbaz/devel/kesh-gate-logs/v0174a/`.*

### Commits (de `8e126cfd` exclu à `HEAD`, partie 4a)

`6e9bb99c` étape 0 (six LOW de la validation P5 appliqués à la fiche parente, statut `in-progress`) ; `c805d527` tests
rouges ; `acd07ecc` implémentation ; `d82a5975` PDF du manuel d'administration ; `bf66dba6` correction de la spec E2E neuve ; le commit de ce compte rendu. Tous en
`refs #849`, aucun mot-clé de fermeture.

### Étape 0 — prémisses et tests rouges

- Prémisses de la fiche relues au code : tenues, sauf les écarts listés plus bas (§ *Écarts à la fiche*).
- **Rouges** (`rouges-1.log`, `rouges-compil.log` ; tests ajoutés seuls, implémentation remisée par `git stash -u`) :
  sur les binaires qui compilaient, **24 tests lancés : 23 rouges, 1 vert** — le vert est
  `export_and_supplier_candidates_with_a_draft`, non-régression voulue. `supplier_invoices_draft` (**53 erreurs** de
  compilation : `create_draft`, `update_draft`, `delete_draft`, `validate`, `validate_in_tx`, `unvalidate`,
  `create_validated` absents) et `migrations_upgrade_path` (**5 erreurs**) ne compilaient pas — rouges par absence.

### Migration `20261031000001_supplier_invoices_draft` (C-v017-4-43)

- Forme (1) `MODIFY COLUMN purchase_journal_entry_id BIGINT NULL` seule : **acceptée** par MariaDB 10.11, la clé
  étrangère gardée sous son nom. Forme (2), repli au même nom : **refusée** (`ERROR 1005 … errno 121`), table intacte.
  Forme (3), repli au nom neuf : acceptée, mais l'index **prend** le nom `_v2` — la fiche disait le contraire.
  **Retenue : (1).** Ordre des `CHECK` (le neuf avant le retrait de l'ancien) : accepté.
- Breaking (P1, `MODIFY COLUMN`) : dernière instruction `UPDATE _kesh_version SET kesh_version_min_required =
  '0.17.0'` ; crates restées à 0.17.0 (P2-bis tenu). P5 : ligne d'audit (`tracked-by-sqlx`). P6 : sites positionnels
  de `migrations_upgrade_path.rs` repris (`total` 87 → 88, `total - 53` → `total - 54`). P7 : `EXEMPT_MIGRATIONS`
  (motif « Durable » : le seul `UPDATE` vise `_kesh_version`). P8 : aucune migration existante touchée. Squash
  `test-schema/` régénéré (88 migrations, 46 tables) ; `migrations.sha384` complété.
- **Aucune écriture comptable réécrite** : la migration ne touche ni `journal_entries` ni `journal_entry_lines`.

### Démarrage réel sur une base rc1 peuplée (`kesh_rc1_v0174a`)

- Le binaire **rc1** (construit depuis `8e126cfd`, `kesh-gate-logs/v0174a/kesh-api-rc1`) a peuplé une base neuve de
  données synthétiques (`peupler-rc1.sh`) : huit factures fournisseur (ouverte, ouverte avec TVA, payée, annulée,
  payée-annulée, née d'une complétion, document rattaché, lot confirmé). État figé en `rc1-etat87.sql` (87 migrations).
- Le binaire **neuf** a démarré dessus, appliqué la 88e, et l'instantané des 46 tables (`instantane.sh`) *(« 45 » jusqu'à la revue P1, A-5 ; recompté : `grep -c '^==' instantane-avant.txt` = 46)* est
  **identique** avant et après, à une colonne près : `_kesh_version.updated_at` (horodatage du démarrage).
  `link_origin` est `NULL` sur les deux documents.
- **Le cas de Guy**, joué par l'API (`cas-de-guy.sh`, `.log`) : factures 1 et 7 dévalidées (brouillon, écriture
  supprimée), `PUT` du total corrigé (99.95 et 249.95 ; IBAN changé pour la 7), revalidées (`open`, écritures 14 et 15).
  Backend arrêté par son PID.
- **Seconde application** : base rechargée depuis `rc1-etat87.sql`, `sqlx migrate run` → instantané identique au
  premier (`instantane-seconde.txt`).
- Sauvegardes : une `.keshbackup` à la forme 0.16 et une à la forme rc1, **sans** `link_origin`, s'importent
  (`admin_full_import_e2e`, tests `v0174a`), et un aller-retour avec brouillon passe.

### Gates au dernier commit de code (`acd07ecc`, PDF `d82a5975`, spec E2E `bf66dba6`)

| Gate | Résultat |
|---|---|
| Base `kesh_v0174a` remise à zéro, puis `scripts/test-fast.sh --ci` (fmt + clippy `-D warnings` + nextest) | vert — **4636 lancés, 4636 réussis, 8 ignorés** (`nextest-final.log`) |
| `cargo build --workspace --all-targets` | vert (`build-final.log`) |
| `npm run check` | 0 erreur, 29 avertissements dans 5 fichiers — aucun touché par la 4a (elle ne modifie aucun `.svelte`) |
| `npm run lint-i18n-ownership` | vert |
| `npm run test:unit` | vert — **190 fichiers, 2291 tests** |
| `npm run build` | vert |
| E2E complet (port 3178, base `kesh_e2e_v0174a` neuve, répertoires `/tmp/kesh-e2e-v0174a-1/`) | voir § suivant |

Avant ce gate final, le run `nextest-4.log` (4636 lancés, 2 rouges : `bank_account_closed_inventory`,
`lettering_documents_backfill`) a conduit à C-v017-4-49 et au recompte de l'inventaire (34 → 38).
Décompte des tests (périmètre `8e126cfd..HEAD`, `git diff | grep '^+#[…test…]'`) : **+53 attributs de test Rust, 0
retiré** ; **+2 tests Playwright** (spec `supplier-invoice-draft-cycle.spec.ts`).

### E2E complet

Joué au dernier commit de code (`bf66dba6`), base `kesh_e2e_v0174a` neuve, répertoires `/tmp/kesh-e2e-v0174a-2/`
passés **au backend et au runner** (`e2e-full-2.log`) : **322 réussis, 17 échecs, 18 ignorés** (15,0 min).

- **7 attendus, KF-029 (#97)**, la liste de `docs/testing.md` § *Les échecs attendus* : `mode-expert.spec.ts:26`,
  `:41` ; `onboarding-path-b.spec.ts:65`, `:92` ; `onboarding.spec.ts:57`, `:77`, `:150`. Ni KF-052 ni pollution.
- **10 dus au changement de contrat, à la charge de la 4b (AC-30)** :
  - K8, huitième ligne du panneau de clôture (`expected 7, received 8`) : `fiscal-year-closing-journey.spec.ts:166`
    (assertion `:162`), `fiscal-years.spec.ts:474` (clair et sombre, assertion `:498`), `fiscal-years.spec.ts:519`
    (assertion `:543`). ⚠️ **Ces deux specs manquent à l'inventaire de l'AC-30** de la fiche parente, qui ne nomme
    que le test Vitest `fiscal-years-page.test.ts:460`.
  - Création par `POST` sans validation (la facture naît brouillon) : `payment-batches.spec.ts:122` (lot vide,
    `body.batch` nul), `supplier-invoices.spec.ts:135`, `:168` (pas de paiement offert sur un brouillon), `:197`
    (statut `draft` affiché au lieu de « Ouverte »).
  - Complétion d'un document en brouillon, sans écriture d'achat : `supplier-documents.spec.ts:401` (E7), `:442` (E6).
- Un **premier run** (`e2e-full-1.log`, avant la correction de la spec) comptait 21 échecs : quatre venaient de mon
  montage — `KESH_INBOX_DIR` et les deux autres répertoires n'étaient pas passés au runner
  (`backup-restore-new-install.spec.ts:140`, `:262`, `inbox-import.spec.ts:59`, `supplier-documents.spec.ts:78`, tous
  verts au second run) — et deux de la spec neuve de la 4a (`supplier-invoice-draft-cycle.spec.ts` : un seul compte de
  charge au seed), corrigée par `bf66dba6`, verte en ciblé (`e2e-cible-cycle.log`, 2/2) puis au second run.

### Mutations (24, toutes tuées — `mutations.log`)

| Mutation | Tuée par |
|---|---|
| M1, M20-bis | `supplier_invoices_draft::a_draft_writes_no_entry_and_carries_the_ttc` |
| M2 | `supplier_invoices_draft::validation_passes_the_rc1_purchase_entry` |
| M3, M4, M7 | `supplier_invoices_draft::unvalidation_precedence_version_batch_match_fiscal_year` |
| M5, M19 | `supplier_invoices_draft::unvalidation_deletes_the_purchase_entry` |
| M6 | `supplier_invoices_draft::unvalidation_status_refusals` |
| M8 | `supplier_invoices_draft::update_draft_refusals` |
| M9 | `supplier_invoices_draft::deleting_a_draft_releases_its_document` |
| M10 | `supplier_invoices_draft::deleting_refusals` |
| M18 | `supplier_invoices_draft::a_draft_cannot_be_cancelled` (par l'erreur `Invariant`) |
| M20 | `supplier_invoices_draft::concurrent_validations_the_loser_reads_validated_under_the_lock` — **par le délai de la sonde** (B attendait sur l'`INSERT IGNORE` des réglages, aucun `FOR UPDATE` vu), non par l'assertion de code |
| M20-quater | `supplier_invoices_draft::deleting_a_draft_during_an_attachment_does_not_deadlock` |
| M15 | `supplier_invoices_draft_schema::a_validated_invoice_without_a_purchase_entry_is_refused` (via le squash) |
| M16 | `closing_checks::t_k8_brouillons_fournisseur_dans_l_exercice` |
| M11, M13 | `supplier_invoice_draft_e2e::put_of_a_draft_born_from_a_qr_document_keeps_the_qr_rules` |
| M12a, M12c | `supplier_invoice_draft_e2e::reconciliation_ignores_drafts` |
| M20-ter | `supplier_invoice_draft_e2e::put_of_a_draft_with_an_attached_document_imposes_nothing` |
| M14 | `supplier_invoice_draft_e2e::due_date_before_invoice_date_is_refused` |
| M17 | `supplier_invoice_draft_e2e::list_filters_by_status` (mutation au service, `&[]`) |

Chaque mutation restaurée par `git checkout -- <fichier> && touch <fichier>`.

### Compteurs (ancien → nouveau, recomptés à la source)

Migrations 87 → 88 ; audit d'idempotence : en-tête et `Total` 87 → 88, `tracked-by-sqlx` 76 → 77, `yes` 11 inchangé ;
`EXEMPT_MIGRATIONS` 22 → 23 ; `migrations_upgrade_path` `total - 53` → `total - 54` ; registre des routes
auditées 138 → 142 / 131 → 135 / 141 → 145 / 113 → 117 (rejouées 25) ; point (vi) 18 → 22 ; couples refusés aux clés
49 → 50, `(14, 35)` → `(14, 36)` ; sites d'écriture du compte bancaire clos 34 → 38 ; `services_motif` 7/8 → 11/13 ;
contrôles de clôture 7 → 8 (K8 en dernier).

### Choix ajoutés au registre

C-v017-4-43 à C-v017-4-51 (`v017-4-choix-autonomes.md`).

### Écarts à la fiche

1. **Repli (3) de la migration** : l'index prend le nom `_v2` (la fiche disait qu'il gardait le sien) — sans effet,
   la forme (1) étant retenue.
2. **`migrations_upgrade_path.rs:44/:46/:63-64/:289`** : la fiche les donnait pour historiques ; ils décrivent la
   fenêtre courante et ont suivi le compteur.
3. **AC-16** : les cas sémantiques de K8 sont testés au dépôt (`closing_checks`) ; la route ne l'est que par
   `closing_checks_e2e` (huit codes).
4. **M20** tuée par le délai de la sonde, non par l'assertion : sous mutation, B bloque plus tôt (sur l'`INSERT
   IGNORE` des réglages) et la sonde ne voit pas le `FOR UPDATE` attendu ; le test rougit, pour une raison voisine.
   *(Levé à la revue P1, E-3 : le brouillon du test porte un projet ; M20 est désormais tuée par l'assertion — § *Revue
   P1*.)*
5. **Audits du `DELETE`** écrits au dépôt (C-v017-4-47), non à la route.
6. **`SUPPLIER_INVOICE_IN_PAYMENT_BATCH`** porte désormais `details` aussi à l'annulation (ajout sans retrait,
   C-v017-4-45).
7. `inbox_import_e2e::complete_closed_fiscal_year_keeps_to_complete` change de sens (son nom date d'avant la 0.17.0) :
   la complétion crée un brouillon, le refus d'exercice clos se lit à la **validation**.
8. **E2E** : le mandat exige l'E2E complet au dernier commit de code de la 4a ; la sous-fiche le renvoyait à la 4b
   (dérogation C-v017-4-31). Joué ici ; il reste à rejouer au dernier commit de code de la 4b.
9. *(Revue P1, A-8.)* **AC-3 (a)** demandait d'**étendre** le test v016-2a (`admin_full_import_e2e.rs`) : deux tests
   neufs l'ont été à la place (`full_import_accepts_a_0_16_backup_with_supplier_invoices`,
   `full_import_accepts_an_rc1_backup_with_supplier_invoices`), l'ancien ne gagnant qu'un doc-comment corrigé ; le fond
   (colonne retirée du manifeste et des lignes, `link_origin` nul après import, dévalidation puis revalidation) est tenu.
10. *(Revue P1, A-8.)* **AC-2 (5)** (« se modifie par `PUT` … aucune règle de la complétion ne s'impose ») est joué au
    **dépôt** (`migrations_upgrade_path.rs`, `update_draft`), où les règles « document » de la route ne s'appliquent
    jamais ; la preuve par la route vient de `put_of_a_draft_with_a_pre_017_link_imposes_nothing`
    (`supplier_invoice_draft_e2e.rs`, lien nul avec QR, par `PUT`) et du démarrage réel (`cas-de-guy.log`, IBAN
    changé pour la facture 7).
11. *(Revue P1, A-3.)* **M-A3a, mutation équivalente** : retirer le refus du compte de charge inactif de
    `validate_in_tx` ne change rien d'observable — `journal_entries::create_in_tx` refuse le compte archivé avec le même
    code, `InactiveOrInvalidAccounts` (`journal_entries.rs:165-166`), et la transaction est annulée. Le re-contrôle
    `active` de la validation est une défense en profondeur ; le refus de la non-imputabilité (`enforce_postable =
    false` côté journal) et celui du projet (`new.project_id` non revalidé par le journal, `journal_entries.rs:300-305`)
    ne le sont pas, et leurs mutations sont tuées.

Aucun incident mémoire (pas de 137) pendant le développement.

### Revue P1 — remédiation (2026-10-31)

#### AC-4 — inventaire clos des lecteurs, recompté à `e7400d76` (revue P1, A-4)

Commande 1 de la fiche : `grep -rn 'purchase_journal_entry_id' crates --include=*.rs | grep -v '/tests/'` → **29**
lignes (27 à `8e126cfd`), dans 12 fichiers (`kesh-gate-logs/v0174a/inventaire-ac4-ecriture.txt`). Chaque site :

| Site (à `e7400d76`) | Devant un brouillon | Verdict |
|---|---|---|
| `entities/supplier_invoice.rs:45` | `Option<i64>` | adapté |
| `routes/supplier_invoices.rs:87`, `:234` | `purchaseJournalEntryId: null` (AC-4, `get()` asserté) | adapté |
| `repositories/supplier_invoices.rs:38`, `:49`, `:2589` (`FIND_SCOPED_SQL`, colonnes de liste, `list_all_by_company`) | colonne lue en `Option` | adapté (type) |
| `repositories/supplier_invoices.rs:85` (`snapshot_json`) | `null` à l'audit | adapté (type) |
| `repositories/supplier_invoices.rs:520`, `:547` (`create_draft_in_tx`) | insère `NULL` | neuf |
| `repositories/supplier_invoices.rs:947`, `:984` (`unvalidate_in_tx`) | `Invariant` si `open` sans écriture (inatteignable, CHECK) ; remet `NULL` | neuf |
| `repositories/supplier_invoices.rs:1313` (`validate_in_tx`) | pose l'écriture | neuf |
| `repositories/supplier_invoices.rs:1528` (`pay_in_tx_as`) | statut refusé avant ; `Invariant` sinon | adapté |
| `repositories/supplier_invoices.rs:1975` (`supplier_invoice_cancel_blocker`) | rang 1 `SUPPLIER_INVOICE_DRAFT` avant toute lecture de l'écriture | adapté |
| `repositories/supplier_invoices.rs:2069` (`cancel_in_tx`) | refus du brouillon (`:2064`) avant ; `Invariant` sinon | adapté |
| `repositories/letterings.rs:1847` (`SYNC_SUPPLIER_INVOICE_SQL`) | jamais atteint (statut refusé par les trois appelants) ; `Invariant` explicite | adapté |
| `repositories/payment_batches.rs:377`, `:394` | `Option<i64>`, extraite après `status != "open"` (`:406`) → `failed[]` | adapté (C-v017-4-38) |
| `repositories/journal_entries.rs:2082` (doc), `:2262-2263` (`document_owners`) | `NULL IN (…)` ne correspond à rien | sans effet |
| `repositories/journal_entries.rs:2776` (`reverse_in_tx_inner`) | `= ?` sur une écriture existante : un brouillon n'en a pas | sans effet |
| `repositories/imported_supplier_invoices.rs:406` (`find_source_document_of_entry`) | `= ?` sur une écriture : un brouillon n'en a pas | sans effet |
| `routes/reconciliation.rs:3410` (`accept_one_supplier_invoice`, audit) | refus `NotOpen` avant | sans effet |
| `exports/csv_tables.rs:1216`, `:1243` | cellule vide (`fmt_opt_i64`) | adapté |
| `mcp/outils/factures_fournisseur.rs:235` | `ecritureAchatId: null` (testé, A-2) | adapté |
| `kesh-reconciliation/src/matching.rs:799` | champ de test, `Some(999)` | adapté (type) |
| `kesh-seed/src/pieges.rs:690` | `create_validated`, `ok_or` `Invariant` | adapté |

Commande 2 de la fiche (`status\s*(=|<>|!=|IN)|\.status\s*(==|!=)|status == "`) sur les fichiers de l'inventaire :
**123** lignes dans 14 fichiers (dont 0 dans `routes/supplier_invoices.rs`, `csv_tables.rs` et les deux fichiers MCP).
Celles qui jugent une **facture fournisseur** :

| Site | Lit | Verdict |
|---|---|---|
| `repositories/reconciliation.rs:825`, `:885`, `:935` | `si.status = 'open'` | brouillon exclu des propositions |
| `repositories/contacts.rs:401` | `si.status <> 'cancelled'` | brouillon **compté** (C-v017-4-11) |
| `repositories/journal_entries.rs:1090` | `si.status = 'cancelled'` | sans effet |
| `repositories/payment_batches.rs:406`, `:420` | `status != "open"`, lot `generated` | brouillon → `failed[]` `SUPPLIER_INVOICE_NOT_OPEN` |
| `repositories/reconciliation_cancel.rs:117` | `status = 'paid'` | sans effet (un brouillon n'a pas de règlement) |
| `repositories/closing_checks.rs:504` | `si.status = 'draft'` | K8, neuf |
| `routes/reconciliation.rs:2383` | `invoice.status != "validated"` (facture **client**) | hors sujet |
| `repositories/supplier_invoices.rs` — `:229` (filtre de liste), `:605`, `:666`, `:826` (`PUT`, `DELETE`), `:953`, `:984-986` (dévalidation), `:1118`, `:1313-1315` (validation), `:1393`, `:1522`, `:1700-1703` (paiement), `:1885`, `:1914` (lot), `:1986`, `:1993` (motif d'annulation), `:2064`, `:2149-2152` (annulation), `:2283`, `:2345`, `:2425-2428` (annulation du règlement) | statut | neufs (gestes du brouillon) ou refus du brouillon avant tout effet |

Les autres lignes de la commande 2 jugent d'autres tables (factures client, transactions, documents, exercices, lots,
avoirs) : sans objet.

#### AC-12 — fichiers de test d'API, triés (revue P1, A-4)

- Création par HTTP (`grep -rln '"/api/v1/supplier-invoices"' crates/kesh-api/tests`, à `e7400d76`, 6 fichiers) :
  `admin_full_import_e2e.rs` (tests `v0174a` neufs : créent, valident) ; `inbox_import_e2e.rs` (valide) ;
  `rejeu_interblocage_e2e.rs` (valide ; rejeu de la validation) ; `services_rollout.rs` (lecture de liste seulement :
  sans objet) ; `supplier_invoice_draft_e2e.rs` (neuf) ; `supplier_settlement_cancel_e2e.rs` (valide avant de payer).
- Complétion (`grep -rln '/complete' crates/kesh-api/tests`, 5 fichiers) : `inbox_import_e2e.rs` (valide) ;
  `mcp_enveloppe_e2e.rs` (complète puis lit les coordonnées par l'outil, sans payer : sans objet) ;
  `mcp_transport_e2e.rs` et `opening_balances_e2e.rs` (autres routes `…/complete` : sans objet) ;
  `supplier_invoice_draft_e2e.rs` (neuf).
- `grep -rn "create_validated" crates/kesh-api/src` → 0 : aucune route ne l'appelle.

#### P6 — sites positionnels (revue P1, A-4)

`grep -rn "migrations.len()\|apply_migrations_up_to" crates/` : un seul site **positionnel**,
`migrations_upgrade_path.rs:124` (`total`) et `:217` (`total - 54`), au couplage assumé et gardé par `assert_eq!(total,
88)` (`:126`) ; tous les autres appels (`migrations_upgrade_path.rs:614`, `:693`, `:900`, `:1163`, `:1327`, `:1669` ;
`accounts_role_backfill.rs`, `audit_log_company_id_backfill.rs`, `invoice_lines_revenue_account_backfill.rs`,
`closing_accounts_backfill.rs`, `lettering_documents_backfill.rs`, `journal_entry_number_sequences_bootstrap.rs`)
résolvent **par version** (`migrations_before`). Copies de `migrations_upgrade_path.rs` par `grep -nE
'\b(53|54|87|88)\b|total - 5[34]'` : `:44`, `:46`, `:63-64` (fenêtre courante, mises à jour — écart 2), `:88`, `:89`
(historiques, laissées), `:122-123`, `:209-210` (lignes d'historique, `:123` et `:210` ajoutées), `:126-127`, `:130`,
`:138`, `:145`, `:147-148`, `:217`, `:220`, `:289` (valeurs courantes, mises à jour).

#### Mutations de la revue P1 (`mutations-p1.log`, diff et lignes de l'échec journalisés — A-6)

| Mutation | Diff | Verdict |
|---|---|---|
| M-A3a | `validate_in_tx` : `_ => return Err(InactiveOrInvalidAccounts)` → `_ => {}` | **survit — équivalente** (écart 11) |
| M-A3b | `validate_in_tx` : `if !non_postable.is_empty()` → `if false && …` | tuée — `validation_rechecks_accounts_and_project_changed_since_the_draft` (`:567`, la validation passe) |
| M-A3c | `validate_in_tx` : `validate_taggable_in_tx(…)` retiré | tuée — même test (`:567`) |
| M20 (rejouée) | `validate_in_tx` : `{FIND_SCOPED_SQL} FOR UPDATE` → `{FIND_SCOPED_SQL}` | tuée **par l'assertion** — `concurrent_validations_the_loser_reads_validated_under_the_lock` (`:689`, « le perdant doit relire `open` sous le verrou : OptimisticLockConflict »), en 0,18 s |

Le journal `mutations.log` de la première campagne (24 mutations) ne garde ni diff ni message ; il reste tel quel.

#### Gates de la remédiation

- Gate ciblé après le code (`p1-cible-1.log`) : `cargo fmt`, `cargo clippy --workspace --all-targets -D warnings`, puis
  `binary(supplier_invoices_draft) | binary(mcp_outils_e2e) | binary(supplier_invoice_draft_e2e)` — 58 lancés, 58 réussis.
- **Gate complet** au dernier commit de code (`e7400d76`), base `kesh_v0174a` remise à zéro, `scripts/test-fast.sh
  --ci` (`nextest-p1.log`) : **4638 lancés, 4638 réussis, 8 ignorés** — 4636 + les deux tests neufs
  (`validation_rechecks_accounts_and_project_changed_since_the_draft`, `un_brouillon_lu_par_les_outils`) ; `cargo build
  --workspace --all-targets` vert (`build-p1.log`).
- Frontend et helpers E2E non touchés : ni Vitest ni E2E rejoués (consigne de l'orchestrateur).
- PDF du manuel d'administration régénéré (`make-admin-p1.log`) ; contrôle aplati : « verrou de période, ligne lettrée
  — 409 ENTRY_LETTERED, à délettrer d'abord » et « période, et une ligne lettrée ».

## Change Log

- 2026-10-31 — Sous-fiche créée au découpage de la v017-4 (C-v017-4-1). Prochaine étape : validation P1 de la fiche
  parente et des deux sous-fiches (`v017-4-validate-prompt-p1.md`).
- 2026-10-31 — Validation P1 remédiée dans la fiche parente (Change Log de la parente ; C-v017-4-26 à 33) : K8 en dernier,
  total du brouillon en `kesh-core`, régime de verrous et de rejeu, règles de la complétion au `PUT` selon `link_origin`,
  `sourceDocument` enrichi, dérogation E2E, règle « Validée » aux textes de la 4a. Prochaine : validation P2.
- 2026-10-31 — Validation P2 remédiée dans la fiche parente (C-v017-4-34 à 36) : `DELETE` dans l'ordre facture puis
  document, contrainte de statut sous un nom neuf, sauvegarde 0.16.x forgée sans `link_origin`, codes réels de la
  réconciliation, M12b retirée ; la 4a écrit les helpers E2E et les étapes API de la spec neuve. Prochaine : validation P3.
- 2026-10-31 — Validation P3 remédiée dans la fiche parente (C-v017-4-37 à 40) : rejeu gardé sur la création, le `PUT` et
  le `DELETE` ; décodage tolérant de `payment_batches.rs:378-385` ; tests pilotés de M20 et M20-quater ; repli du
  `MODIFY COLUMN` au nom neuf ; tâche des artefacts E2E. Prochaine : validation P4.
- 2026-10-31 — Validation P4 remédiée dans la fiche parente (C-v017-4-41, 42) : clé et texte du motif brouillon posés à
  l'AC-13 ; test piloté du `DELETE` au niveau du dépôt, plus un test de route ; `retry_app_on_deadlock` pour le `PUT` ;
  point (vi) à vingt-deux. Prochaine : validation P5 (ciblée).
- 2026-10-31 — Développement (Opus 5.5, autonomie) : migration `20261031000001` (forme `MODIFY COLUMN` seule), dépôt,
  routes `PUT`/`DELETE`/`validate`/`unvalidate`, filtre de liste, K8, MCP, export, audit, API externe, CHANGELOG,
  manuel d'administration et PDF ; 24 mutations tuées ; démarrage réel sur base rc1 peuplée, relecture identique ;
  gate complet vert (4636/4636), Vitest 2291, E2E complet 322 / 17 (7 KF-029, 10 à la charge de la 4b). Choix
  C-v017-4-43 à 51. Statut `review`. Prochaine : revue de code P1 (`v017-4a-review-prompt-p1.md`, Opus, lentilles
  B, E, A).
- 2026-10-31 — Revue de code P1 (Opus, lentilles B, E, A : 0 CRITICAL, 0 HIGH, 6 MEDIUM, 12 LOW) remédiée : A-3 (test
  des refus de la validation après changement d'état ; M-A3b et M-A3c tuées, M-A3a équivalente), A-2 (brouillon lu par
  l'outil MCP), E-1, E-2, E-5, A-1 (`api-external.md` : codes réels de la validation, rangées `ACCOUNT_NOT_POSTABLE` et
  `INVALID_PAYMENT_DETAILS`, `PUT`, règle Q4 ; fiche parente annotée `:169`), A-4 (inventaire de l'AC-4, liste de
  l'AC-12, tri P6 au Dev Agent Record), E-3 (M20 tuée par l'assertion), B1 (ligne d'audit et Dev Notes), B2 (manuel et
  PDF), B3, B4 = E-4 (doc-comments), E-6 (numéro blanc, une ligne de production, testée), A-5, A-6, A-7 (sous-fiche
  4b), A-8. Gate complet 4638/4638. Prochaine : revue P2 ciblée (`v017-4a-review-prompt-p2.md`, Sonnet, une lentille).
