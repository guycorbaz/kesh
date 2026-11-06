# Story 33-4 : les bornes d'années des dates reçues du client, et de l'expiration des clés d'API (#606, #742, #755)

Status: ready-for-dev

*Fiche rédigée le 2026-11-06 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`, branche `story/33-0-kickoff`,
numéros de ligne de `4ef19901`. Release **0.17.5**, Epic 33. **Aucune migration.** Registre :
`e33-choix-autonomes-e334.md`, choix C-e334-1 à 4. Issues en `refs` dans les commits ; à la PR : `closes #606`, `closes #755`.
⚠️ #742 est déjà **fermée**, reportée dans #606 à l'audit du 2026-11-06 : aucun mot-clé à poser sur elle.*

## Story

En tant qu'utilisateur ou client de l'API, je veux qu'une date dont l'année sort de la plage que la base sait tenir (1000 à
9999) soit refusée comme une date mal formée, et jamais rendue en 500 ni acceptée en silence ; en tant qu'Administrateur, je
veux qu'une clé d'API que j'ai voulue temporaire ne devienne jamais permanente.

## Ce que le code dit (prémisses établies)

- ⚠️ **Prémisse corrigée — `parse_as_of` n'est plus dans `letterings.rs`** : il vit à
  `crates/kesh-api/src/services/postes_ouverts.rs:61-77` depuis la couche de services de l'Epic 32. C'est lui qu'on réemploie.
- **Deux sites sont déjà bornés** : `parse_as_of` (ci-dessus) et `routes/audit_log.rs:220-246` (`parse_date`, bornes
  `:58-68`). Trois copies de la même règle seraient une de trop : la règle se met en commun.
- ⚠️ **Le commentaire `audit_log.rs:55-57` dit qu'une année négative ou au-delà de 65535 fait « paniquer » la liaison.** Le code
  lu de `sqlx-mysql-0.8.6/src/types/chrono.rs:263-264` rend une **erreur** (`u16::try_from(...).map_err(...)`, propagée par
  `encode_by_ref`, `:171-177`) : donc vraisemblablement un 500, non une panique. **À établir par T1**, et le commentaire se
  corrige sur le résultat.
- **Entre 10000 et 65535**, l'année s'encode (`u16`) et part à MariaDB ; ce que la base en fait n'est pas lu au code : T1 le dit.
- **Entre 0 et 999**, MariaDB accepte en général, et le filtre rend un résultat au lieu d'un refus (`audit_log.rs:62-64`) : pas un
  500, mais un écart aux deux bornes, à fermer aussi.
- ⚠️ **Prémisse corrigée — « attendu : 400 » n'est pas ce que rend un corps JSON.** Les corps passent par `axum::Json` : une
  donnée invalide y rend **422** en texte (`JsonDataError`), ce que le dépôt sait déjà (`routes/invoices.rs:1174-1175`) ; les
  `Query<…>` rendent **400** en texte ; les dates lues en chaîne, **400** JSON `VALIDATION_ERROR`. Le critère décidable est donc :
  **une date hors plage reçoit le même refus qu'une date mal formée sur le même site** (C-e334-1).
- ⚠️ **#742 (commentaire) parle des filtres de factures « clientes et fournisseur »** : la liste fournisseur **n'a aucun filtre de
  date** (`routes/supplier_invoices.rs:640-646`). Seules les listes clientes sont concernées.
- **#755** : `routes/api_keys.rs:140-146` ne contrôle que `exp <= Utc::now()` ; la jumelle MCP porte la garde
  (`routes/mcp_keys.rs:173-176`, `:292-299`, i18n `admin-mcp-keys-error-expiry-out-of-range`), et y a constaté à l'exécution un
  201 avec `expiresAt: null` sans elle (#755). Non exécuté sur `api_keys` : c'est T2.

## Inventaire clos des sites qui reçoivent une date du client

Méthode : tout champ `NaiveDate`/`NaiveDateTime`/`DateTime<…>` d'un item `Deserialize` de `crates/kesh-api/src/` (structs et
variantes d'enum), toute chaîne de date analysée à la main, tout analyseur de fichier reçu. **68 champs de requête, 3 analyseurs**
— décompte à refaire par le développeur avec la garde de T5, écart à déclarer au Change Log.

| Canal | Fichier (ligne des champs) | Forme |
|---|---|---|
| Requête, **déjà borné** | `routes/letterings.rs:312` → `services/postes_ouverts.rs:61` ; `routes/audit_log.rs:82,84` | chaîne |
| Requête, chaîne | `routes/journal_entries.rs:317,319` → `services/ecritures.rs:83-92` (`analyser_date`) | chaîne |
| Requête, serde (17) | `routes/invoices.rs:131,133,945,947,949` ; `routes/reports.rs:65,66,87,88,106,107,147,148,175,176,545,546` | `Query` |
| Corps (30) | `routes/bank_accounts.rs:138` ; `companies.rs:374,383` ; `credit_notes.rs:142` ; `dunning_reminders.rs:121` (`NaiveDateTime`) ; `fiscal_years.rs:81,82` ; `invoices.rs:91,93,108,110,1060,1181` ; `journal_entries.rs:84,95` ; `payment_batches.rs:132,139` ; `projects.rs:74,75,85,86` ; `reconciliation.rs:838` (variante `Split`),`4718,5142` ; `supplier_invoices.rs:346,347,611` (`UpdateSupplierInvoiceRequest` par `flatten`, `:652-654`) ; `vat.rs:216,218,228` | `Json` |
| Expirations | `routes/api_keys.rs:67` (#755) ; `routes/mcp_keys.rs:102` (déjà borné) | `DateTime<Utc>` |
| MCP (14) | `mcp/outils/ecritures.rs:105,106` ; `factures.rs:153,154,167,168,169` ; `grand_livre.rs:27,28` ; `postes_ouverts.rs:32` ; `rapports.rs:36,37,44,45` | `from_value` → `ArgumentsInvalides` (`mcp/outil.rs:181`) |
| Fichiers reçus | `kesh-core/src/bank_imports/manual.rs:729` (saisie manuelle : forme à quatre chiffres vérifiée, `0999` passe) ; `kesh-import/src/camt053/mod.rs:863-873` ; `kesh-import/src/csv/parser.rs:603` | analyseurs |

**Faux positifs écartés, à déclarer dans la garde** : `BankProfileResponse`, `ReconciliationRuleResponse` (réponses dérivant aussi
`Deserialize`), `test_endpoints.rs` (mode test). **Hors périmètre, écrit** : les dates de `.keshbackup`, restaurées telles quelles.

## Conception (C-e334-1, 2, 3)

- **`kesh_core::dates`** (neuf) : `ANNEE_MIN = 1000`, `ANNEE_MAX = 9999`, `annee_stockable(NaiveDate) -> bool`. Pure, sans I/O.
- **`crates/kesh-api/src/dates_recues.rs`** (neuf) : `date`, `date_opt`, `date_heure` pour `#[serde(deserialize_with = …)]`, qui
  délèguent à la désérialisation de chrono puis refusent par `serde::de::Error::custom("année hors de 1000 à 9999")`.
  ⚠️ **Un `Option` en `deserialize_with` perd le « champ absent = `None` »** : chaque champ optionnel porte `default` en plus. Un
  oubli fait de l'optionnel un champ requis, et un 422 sur des appels qui marchaient : l'AC-6 l'attrape.
- `parse_as_of`, `analyser_date` et `audit_log::parse_date` emploient `annee_stockable` ; `borne_basse`/`borne_haute` disparaissent ;
  **messages et clés i18n existants inchangés**, sauf `analyser_date`, qui prend le refus de plage (`{nom} invalide (année hors de 1000 à 9999)`).
- `kesh-import` n'a aucune dépendance interne, et la CI le vérifie (`ci.yml:140`) : il porte sa propre constante, avec un test
  croisé côté `kesh-api` qui égale les deux bornes. `manual.rs` emploie `kesh_core::dates`.
- **#755** : `routes/api_keys.rs` reprend la garde de `mcp_keys.rs` (`exp.year() > EXPIRY_MAX_YEAR`) et sa clé i18n, sans nouvelle
  clé. La constante reste à sa place ; une seule fonction `expiration_stockable` sert les deux routes.

## Acceptance Criteria

1. **AC-1 (premier geste, avant tout correctif)** : T1 et T2 tournent sur le code inchangé ; le statut relevé par site est écrit au
   Change Log. Si aucun site ne rend de 500 et que #755 ne se reproduit pas, la priorité se reclasse (P3) : l'orchestrateur en est
   averti au rapport, la borne reste à poser.
2. **AC-2** : pour chaque site de requête ou de corps du tableau, les valeurs `%2B10000-01-01`, `%2B70000-01-01`, `-0001-01-01`,
   `0999-12-31` (en requête ; `+10000-01-01`… en JSON) rendent le refus du site : 400 en `Query` et en chaîne, 422 en corps, jamais
   500 ni 2xx. ⚠️ **Dans une chaîne de requête, `+` se lit comme un espace** et rendrait un 400 qui ne prouve rien : encoder `%2B`
   (`tests/open_items_e2e.rs:309`).
3. **AC-3** : le refus nomme la plage : le corps de la réponse contient `1000 à 9999`, sauf `audit_log` (message existant, traduit,
   testé tel quel) et le lot `POST …/accept` (extracteur à message fixe, `routes/reconciliation.rs:926` : 400 seul, sur une proposition
   par ailleurs valide). ⚠️ **Sans ce marqueur, le test d'un corps partiel passerait à vide** : un champ requis manquant rend aussi 422.
4. **AC-4** : témoins acceptés, aux deux bornes, sur un site de chaque canal : `1000-01-01` et `9999-12-31` ne sont pas refusés pour
   leur plage.
5. **AC-5** : outils MCP (14 arguments) : même plage, refus `ArgumentsInvalides`, sans donnée lue.
6. **AC-6 — non-régression du champ absent** : chaque `Option` touché reste facultatif : un test par route lit sans ce champ et rend
   ce qu'il rendait.
7. **AC-7 — fichiers** : un relevé CAMT, un CSV et une saisie manuelle portant une année hors plage sont refusés avec l'erreur de date
   invalide de leur format (`CamtError::InvalidDate`, `CsvLineErrorCode::InvalidDate`, `ManualStatementReason::InvalidDate`), rien n'est écrit.
8. **AC-8 — #755** : `POST /api/v1/settings/api-keys` (`lib.rs:964`) avec `9999-12-31T23:59:59-05:00` → 400 ; rien n'est
   créé (compte de lignes de `api_keys` inchangé). Témoin `9999-12-31T23:59:59Z` → 201 et `expiresAt` non nul.
9. **AC-9 — garde** (T5) : la liste des champs date reçus **non bornés** est vide, et la garde rougit sur une sonde.
10. **AC-10** : `audit_log.rs:55-57` dit ce que T1 a établi.

## Tâches

- [ ] **T1 — la mesure d'abord** : `crates/kesh-api/tests/dates_bornes_e2e.rs`, table `(canal, méthode, chemin, champ, statut_attendu)`,
  un test par site. Lancée rouge, sur le code inchangé, sortie gardée dans `kesh-gate-logs/e334/`.
- [ ] **T2** : même fichier, `POST` de clé d'API, cas de #755 et témoin (AC-8). Rouge attendu si l'analogie tient.
- [ ] **T3** : `kesh_core::dates`, `dates_recues.rs`, pose sur les sites du tableau (outil Edit, site par site), `analyser_date`,
  `parse_as_of`, `parse_date`, tests unitaires des bornes (999/1000, 9999/10000, négative).
- [ ] **T4** : analyseurs de fichiers et leurs tests ; `api_keys.rs` ; correction du commentaire.
- [ ] **T5** : `crates/kesh-api/tests/dates_recues_garde.rs`, garde lexicale sur le modèle de `tests/mcp_gardes_lexicales.rs`, sur
  `src/routes`, `src/services`, `src/mcp` : tout champ daté d'un item `Deserialize` porte `dates_recues::` ou figure à la liste des
  exceptions, écrite avec son motif. Sonde positive dans le test : un extrait inline sans borne doit être relevé (*un contrôle vide se
  prouve par une sonde*).
- [ ] **T6** : `docs/api-external.md` : une phrase, plage des dates et statut. CHANGELOG `## [0.17.5] — Non publié`, *Corrigé* : `refs #606`, `refs #755`.

### Tests qui rougissent avant correctif

T1 (les sites encore ouverts), T2. **Le statut relevé, non la prédiction, fait foi.**

### Mutations nommées (P2)

- **M1** : `annee_stockable` borne haute à `10000` → rouge `dates_bornes_e2e`, unitaires.
- **M2** : borne basse à `0` → rouge sur `0999-12-31`.
- **M3** : retirer `deserialize_with` de `routes/invoices.rs:93` (un `Option`) → rouge T1 et garde T5.
- **M4** : retirer `default` d'un `Option` (`projects.rs:74`) → rouge AC-6.
- **M5** : retirer la garde de `api_keys.rs` → rouge AC-8.
- **M6** : `camt053::parse_date` sans borne → rouge AC-7.

Après une mutation restaurée : `touch` du fichier (cargo garde sinon le binaire muté).

## Gates

Gate complet au dernier commit de code : `flock /tmp/station-heavy.lock flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock
scripts/mem-guard.sh scripts/test-fast.sh`, base remise à neuf. **E2E complet** au dernier commit de code (la règle ne l'exempte
pas, même sans écran touché) : rouge jugé contre `docs/testing.md`. `cargo publish --dry-run -p kesh-import` (CI). Aucune capture :
aucun contrôle visible.

## Manuels

Aucun comportement d'écran ne change : les champs de date du navigateur n'émettent que des années à quatre chiffres. **À contrôler,
et à déclarer** : `/usr/bin/grep -n '9999\|1000-01-01' docs/manual/fr/*.tex` ; un `.tex` touché entraîne son PDF. La phrase de
`docs/api-external.md` suffit pour l'API.

## Recoupements avec les autres fiches

`routes/invoices.rs` (33-6, autres lignes) ; ⚠️ `kesh-import/src/camt053/mod.rs` (**33-2** : sa relecture héritée `parse_legacy_0173`
passe aussi par `parse_date` : la borne y change ce que la relecture refuse, à vérifier sur la caractérisation
`tests/fixtures/camt053_golden/` et à l'intégration, sur une date à année hors plage) ;
`routes/supplier_invoices.rs` (33-1a/b) ; `routes/reports.rs` et `mcp/outils/rapports.rs` voisins de `services/rapports.rs` (33-7, autre
fichier). Lignes distinctes partout ; union à l'intégration.

## Hors périmètre

Le texte plat des rejets `Query` et `Json` (non JSON, non traduit) : comportement de toutes les erreurs de forme, inchangé ici.

## Change Log

- 2026-11-06 : fiche créée (Opus 5.5).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
