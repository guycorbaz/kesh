# Story 33-4 : les bornes d'années des dates reçues du client, et de l'expiration des clés d'API (#606, #742, #755)

Status: ready-for-dev

*Fiche rédigée le 2026-11-06 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`, branche `story/33-0-kickoff`,
numéros de ligne de `4ef19901`. Release **0.17.5**, Epic 33. **Aucune migration.** Registre :
`e33-choix-autonomes-e334.md`, choix C-e334-1 à 4 et C-e334-26. Issues en `refs` dans les commits ; à la PR : `closes #606`, `closes #755`.
⚠️ #742 est déjà **fermée**, reportée dans #606 à l'audit du 2026-11-06 : aucun mot-clé à poser sur elle.*

## Story

En tant qu'utilisateur ou client de l'API, je veux qu'une date dont l'année sort de la plage que la base sait tenir (1000 à
9999) soit refusée comme une date mal formée, et jamais rendue en 500 ni acceptée en silence ; en tant qu'Administrateur, je
veux qu'une clé d'API que j'ai voulue temporaire ne devienne jamais permanente.

## Ce que le code dit (prémisses établies)

- ⚠️ **Prémisse corrigée — `parse_as_of` n'est plus dans `letterings.rs`** : il vit à
  `crates/kesh-api/src/services/postes_ouverts.rs:61-77` depuis la couche de services de l'Epic 32. C'est lui qu'on réemploie.
- ⚠️ **Prémisse corrigée (validation P1, M-R4-1) — six champs sont déjà bornés, et non deux sites, et leur contrat est épinglé** :
  `asOf` (`routes/letterings.rs:312` → `parse_as_of`, testé `tests/open_items_e2e.rs:308`) ; l'argument `asOf` de l'outil MCP, passé en chaîne au
  même service (`mcp/outils/postes_ouverts.rs:32,240`) ; `dateFrom` et `dateTo` du journal d'audit (`routes/audit_log.rs:220-246`, bornes `:58-68`,
  testés `tests/audit_log_e2e.rs:395-397`) ; `closedOn` (`routes/bank_accounts.rs:1107-1110`, 400 JSON `VALIDATION_ERROR`, garanti par
  `docs/api-external.md:241`, testé `tests/bank_accounts_closed_e2e.rs:714-722`) ; l'expiration des clés MCP (`routes/mcp_keys.rs:292-299`). **Ces six-là
  gardent leur contrat, tests et documentation inchangés, et sortent du périmètre à poser.** Quatre copies de la même règle
  seraient trop : seul le prédicat se met en commun (`annee_stockable`).
- ⚠️ **Un septième contrat porte sur une date encore ouverte** : `POST /invoices` avec `+262142-10-01` rend **400** JSON, par le dépassement de
  `checked_add_signed` (`routes/invoices.rs:683`, `tests/contact_payment_terms_e2e.rs:473-498`). Un `deserialize_with` sur `invoices.rs:91` le ferait passer à 422
  en texte. Et le même calcul a une face non couverte : `9999-12-31` plus le délai du contact donne une échéance en l'an 10000, que `checked_add_signed`
  laisse passer vers la base. D'où l'AC-13.
- ⚠️ **Le commentaire `audit_log.rs:55-57` dit qu'une année négative ou au-delà de 65535 fait « paniquer » la liaison.** Le code
  lu de `sqlx-mysql-0.8.6/src/types/chrono.rs:263-264` rend une **erreur** (`u16::try_from(...).map_err(...)`, propagée par
  `encode_by_ref`, `:171-177`) : donc vraisemblablement un 500, non une panique. **À établir par T1**, et le commentaire se
  corrige sur le résultat.
- **Entre 10000 et 65535**, l'année s'encode (`u16`) et part à MariaDB ; ce que la base en fait n'est pas lu au code : T1 le dit.
- **Entre 0 et 999**, MariaDB accepte en général, et le filtre rend un résultat au lieu d'un refus (`audit_log.rs:62-64`) : pas un
  500, mais un écart aux deux bornes, à fermer aussi.
- ⚠️ **Prémisse corrigée — « attendu : 400 » n'est pas ce que rend un corps JSON.** Les corps passent par `axum::Json` : une
  donnée invalide y rend **422** en texte (`JsonDataError`), ce que le dépôt sait déjà (`routes/invoices.rs:1174-1175`) ; les
  `Query<…>` rendent **400** en texte ; les dates lues en chaîne, **400** JSON `VALIDATION_ERROR`. Le critère décidable est donc, sur un site
  qui ne refuse encore rien de ces valeurs en JSON : **une date hors plage reçoit le même refus qu'une date mal formée sur le même site**. Sur un
  site qui en refuse déjà certaines en JSON, le contrat existant l'emporte (classes A et B, C-e334-1 révisé).
- ⚠️ **#742 (commentaire) parle des filtres de factures « clientes et fournisseur »** : la liste fournisseur **n'a aucun filtre de
  date** (`routes/supplier_invoices.rs:640-646`). Seules les listes clientes sont concernées.
- **#755** : `routes/api_keys.rs:140-146` ne contrôle que `exp <= Utc::now()` ; la jumelle MCP porte la garde
  (`routes/mcp_keys.rs:173-176`, `:292-299`, i18n `admin-mcp-keys-error-expiry-out-of-range`), et y a constaté à l'exécution un
  201 avec `expiresAt: null` sans elle (#755). Non exécuté sur `api_keys` : c'est T2.

- ⚠️ **Prémisse corrigée (validation P2, M-R4-1) — `sent_at` est seul de son type, et de classe B.** `routes/dunning_reminders.rs:121` est un `NaiveDateTime` : une date seule y rend 422 pour sa forme, jamais pour son année. Une année future y a déjà un refus JSON de domaine, épinglé (`:261-263`, `errors.rs:1904-1906`, **422** `REMINDER_DATE_IN_FUTURE`, `tests/dunning_reminders_e2e.rs:407-414`), et `0999-…` ou `-0001-…` passent le contrôle : classe B, et aucun corps ni aucune requête ne garde de `NaiveDateTime` pour la classe C.

## Inventaire clos des sites qui reçoivent une date du client

Méthode : tout champ `NaiveDate`/`NaiveDateTime`/`DateTime<…>` d'un item `Deserialize` de `crates/kesh-api/src/` (structs et
variantes d'enum), toute chaîne de date analysée à la main, tout analyseur de fichier reçu. **68 champs de requête, 3 analyseurs** ;
**6 de ces champs sont déjà bornés** et restent intacts, **62 restent à classer** par T1 — décompte à refaire par le développeur avec la garde de T5, écart à
déclarer au Change Log.

| Canal | Fichier (ligne des champs) | Forme |
|---|---|---|
| **Déjà borné, intact (6)** | `routes/letterings.rs:312` → `services/postes_ouverts.rs:61` ; `routes/audit_log.rs:82,84` ; `routes/bank_accounts.rs:138` ; `routes/mcp_keys.rs:102` ; `mcp/outils/postes_ouverts.rs:32` → même service | chaîne, corps, expiration, MCP |
| Requête, chaîne | `routes/journal_entries.rs:317,319` → `services/ecritures.rs:83-92` (`analyser_date`) | chaîne |
| Requête, serde (17) | `routes/invoices.rs:131,133,945,947,949` ; `routes/reports.rs:65,66,87,88,106,107,147,148,175,176,545,546` | `Query` |
| Corps (29) | `routes/companies.rs:374,383` ; `credit_notes.rs:142` ; `dunning_reminders.rs:121` (`NaiveDateTime`, B d'office) ; `fiscal_years.rs:81,82` ; `invoices.rs:91,93,108,110,1060,1181` ; `journal_entries.rs:84,95` ; `payment_batches.rs:132,139` ; `projects.rs:74,75,85,86` ; `reconciliation.rs:838` (variante `Split`),`4718,5142` ; `supplier_invoices.rs:346,347,611` (`UpdateSupplierInvoiceRequest` par `flatten`, `:652-654`) ; `vat.rs:216,218,228` | `Json` |
| Expiration (1) | `routes/api_keys.rs:67` (#755) | `DateTime<Utc>` |
| MCP, par le service (2) | `mcp/outils/ecritures.rs:105,106` : passés en chaîne (`:241-242`) à `analyser_date`, donc bornés par la ligne *Requête, chaîne* | refus de validation du service |
| MCP, direct (11) | `factures.rs:153,154,167,168,169` ; `grand_livre.rs:27,28` ; `rapports.rs:36,37,44,45` | `from_value` → `ArgumentsInvalides` (`mcp/outil.rs:181`) |
| Fichiers reçus | `kesh-core/src/bank_imports/manual.rs:729` (saisie manuelle : forme à quatre chiffres vérifiée, `0999` passe) ; `kesh-import/src/camt053/mod.rs:863-873` ; `kesh-import/src/csv/parser.rs:603` | analyseurs |

**Faux positifs écartés, à déclarer dans la garde** : `BankProfileResponse`, `ReconciliationRuleResponse` (réponses dérivant aussi
`Deserialize`). `routes/test_endpoints.rs` ne porte aucun champ daté `Deserialize` (relevé en validation P1) : il n'y a pas à l'exempter. **Hors périmètre, écrit** : les dates de `.keshbackup`, restaurées telles quelles.

## Conception (C-e334-1, 2, 3, 26)

- **`kesh_core::dates`** (neuf) : `ANNEE_MIN = 1000`, `ANNEE_MAX = 9999`, `annee_stockable(NaiveDate) -> bool`, `expiration_stockable(DateTime<Utc>) -> bool`. Pure, sans I/O.
- **Règle de pose, par classe, établie par T1 sur le code inchangé** (C-e334-1, révisé). Chaque champ à classer reçoit une classe au tableau de T1 ; une classe
  se déduit du relevé, non d'une lecture :
  - **A, déjà refusé** : les valeurs de l'AC-2 sont toutes refusées en 4xx, jamais 500 ni 2xx. **Rien ne se pose** : statut et corps restent ; le champ entre à la liste
    d'exceptions de la garde, motif « classe A » et nom du test T1. Les six champs ci-dessus sont A d'office.
  - **B, refus JSON partiel** : au moins une valeur hors plage reçoit déjà un refus JSON (400 de validation, ou 4xx de domaine), ou un test existant ou `docs/api-external.md` épingle un tel
    refus, mais une autre valeur passe (500 ou 2xx). **Contrôle dans le handler**, avant tout usage de la date : `annee_stockable`, `AppError::Validation`, 400 JSON, message portant
    `1000 à 9999`. Jamais de `deserialize_with`, qui rendrait 422 en texte là où ce site répond en JSON. Si un test épingle un code de domaine, le contrôle se place après ce
    refus. **`routes/invoices.rs:91,93,108,110` sont B d'office** : la création par le test cité, la modification pour que les deux gestes aient le même refus. **`routes/dunning_reminders.rs:121` aussi** : `annee_stockable(sent_at.date())`, après le refus du futur, qui garde son code.
  - **C, ouvert** : aucune de ces valeurs n'a de refus JSON. **`deserialize_with`**, et le refus est celui d'une date mal formée sur le site (400 en `Query`, 422 en corps).
  - Un même corps ne mêle pas B et C : B l'emporte, et le Change Log le dit.
- **`crates/kesh-api/src/dates_recues.rs`** (neuf, classe C) : `date`, `date_opt` pour `#[serde(deserialize_with = …)]`, qui
  délèguent à la désérialisation de chrono puis refusent par `serde::de::Error::custom("année hors de 1000 à 9999")`.
  ⚠️ **Un `Option` en `deserialize_with` perd le « champ absent = `None` »** : chaque champ optionnel porte `default` en plus. Un
  oubli fait de l'optionnel un champ requis, et un 422 sur des appels qui marchaient : l'AC-6 et la garde T5 l'attrapent.
- `parse_as_of`, `audit_log::parse_date` et le contrôle de `closedOn` emploient `annee_stockable` ; `borne_basse`/`borne_haute` disparaissent ; **messages, statuts et clés i18n existants inchangés**.
  `analyser_date` prend le refus de plage (`{nom} invalide (année hors de 1000 à 9999)`) : c'est sa seule nouveauté.
- **MCP** : les onze arguments directs portent `dates_recues`, donc `ArgumentsInvalides`, dont le message est fixe (`mcp/outil.rs:181`) ; les trois arguments qui passent par un service borné (`postes_ouverts`, `ecritures`) n'en portent pas : leur refus reste celui du service, traduit par `depuis_app_error`, avec son message.
- **`kesh-import`** n'a aucune dépendance interne, et la CI le vérifie (`ci.yml:140`) : il porte sa propre constante, avec un test croisé côté `kesh-api` qui égale les deux bornes. `manual.rs` emploie `kesh_core::dates`.
- **La frontière avec la lecture héritée (C-e334-26, M-R4-2), au code lu** : `parse_date` (`camt053/mod.rs:863`), `handle_text`, `flush_pending` et `parse_with_version` (`:180`) sont communs aux deux lectures et **ne changent pas d'une ligne**. La borne se pose dans une fonction neuve, `parse_corrige(reader, version)` = `parse_with_version(reader, version, TextMode::Joined)` puis `refuser_annees_hors_plage(&stmts)` (période, date de comptabilisation, date de valeur du relevé rendu, `CamtError::InvalidDate`). `v04::parse` et `v08::parse`, seuls appelants du mode `Joined`, passent par elle ; `parse_legacy_0173` (`:89`) continue d'appeler `parse_with_version` directement, et **ne voit donc pas la borne**. Une date hors plage est refusée par la lecture corrigée, qui passe avant (`routes/bank_imports.rs:1050,1288`) : la relecture ne la rencontre jamais dans un import, et la clé de doublon qu'elle recalcule reste celle de la 0.17.3.
- **#755** : `routes/api_keys.rs` et `routes/mcp_keys.rs` emploient `expiration_stockable` ; `EXPIRY_MAX_YEAR` (`mcp_keys.rs:176`, un seul usage, `:293`) disparaît. `mcp_keys.rs` garde sa clé i18n. `api_keys.rs` écrit son refus en français dans le code, comme tous ses voisins (`:140-147`) : **ni clé neuve, ni emprunt de l'espace `admin-mcp-keys-*`**.

## Acceptance Criteria

1. **AC-1 (premier geste, avant tout correctif)** : T1 et T2 tournent sur le code inchangé ; le statut relevé par site est écrit au
   Change Log. Si aucun site ne rend de 500 et que #755 ne se reproduit pas, la priorité se reclasse (P3) : l'orchestrateur en est
   averti au rapport, la borne reste à poser.
2. **AC-2** : chaque champ à classer du tableau s'éprouve **avec des valeurs de son type réel** : `NaiveDate`, `%2B10000-01-01`, `%2B70000-01-01`, `-0001-01-01`,
   `0999-12-31` (en requête ; `+10000-01-01`… en JSON) ; `NaiveDateTime` (`sent_at`), `+10000-01-01T00:00:00`, `+70000-01-01T00:00:00`, `-0001-01-01T00:00:00`, `0999-12-31T00:00:00` ; `DateTime<Utc>` : l'AC-8. Chacune rend le refus de sa classe : A, l'existant, relevé par T1 ; B, 400 JSON `VALIDATION_ERROR` ; C, 400 en `Query`, 422 en corps. Jamais 500 ni 2xx.
   **Refus de domaine conservés** : une année future sur `sent_at` garde `REMINDER_DATE_IN_FUTURE`. **Lot `POST …/accept`, `Split.value_date`, classe C** : une date hors plage refuse tout le lot en 400, comme une date mal formée au même extracteur ; l'exception « corps invalide » de *Pattern batch* (`CLAUDE.md`), déclarée ici. Si T1 relève un `200` : `failed[]` portant cette proposition vaut refus, `accepted[]` vaut échec.
   ⚠️ **Dans une chaîne de requête, `+` se lit comme un espace** et rendrait un 400 qui ne prouve rien : encoder `%2B`
   (`tests/open_items_e2e.rs:309`).
3. **AC-3** : le refus nomme la plage : le corps de la réponse contient `1000 à 9999`, sur les classes B et C, sauf le lot `POST …/accept` (extracteur à message fixe, `routes/reconciliation.rs:926` : 400 seul, sur une proposition
   par ailleurs valide), les onze arguments MCP directs (message fixe, AC-5), et les refus de domaine conservés de l'AC-2. La classe A garde son message tel quel. ⚠️ **Sans ce marqueur, le test d'un corps partiel de classe C passerait à vide** : un champ requis manquant rend aussi 422.
4. **AC-4** : témoins acceptés, aux deux bornes, sur un site de chaque canal : `1000-01-01` et `9999-12-31` ne sont pas refusés pour
   leur plage.
5. **AC-5** : outils MCP : même plage et aucune donnée lue. Onze arguments directs : `ArgumentsInvalides`, sur un jeu d'arguments complet et valide hors la seule date fautive, avec le témoin de forme : ce refus, à message fixe (`mcp/outil.rs:178-184`), est aussi celui d'un champ manquant. `kesh_postes_ouverts` et la liste des écritures (trois arguments) : refus de validation du service, message conservé, et les tests existants de `mcp_outils_e2e` passent sans modification.
6. **AC-6 — non-régression du champ absent** : chaque `Option` touché reste facultatif : un test par route lit sans ce champ et rend
   ce qu'il rendait.
7. **AC-7 — fichiers** : un relevé CAMT, un CSV et une saisie manuelle portant une année hors plage sont refusés avec l'erreur de date
   invalide de leur format (`CamtError::InvalidDate`, `CsvLineErrorCode::InvalidDate`, `ManualStatementReason::InvalidDate`), rien n'est écrit. Le CAMT s'éprouve par `v04::parse` **et** `v08::parse`.
8. **AC-8 — #755** : `POST /api/v1/settings/api-keys` (`lib.rs:964`) avec `9999-12-31T23:59:59-05:00` → 400 ; rien n'est
   créé (compte de lignes de `api_keys` inchangé). Témoin `9999-12-31T23:59:59Z` → 201 et `expiresAt` non nul. La garde jumelle de `mcp_keys.rs` rend toujours sa clé i18n.
9. **AC-9 — garde** (T5) : la liste des champs date reçus **non bornés** est vide, et la garde rougit sur une sonde.
10. **AC-10** : `audit_log.rs:55-57` dit ce que T1 a établi.
11. **AC-11 — la lecture héritée reste celle de la 0.17.3 (frontière de C-e334-26)**, test `kesh-import` : (a) un relevé à `<Dt>0999-12-31</Dt>` : `parse` → `InvalidDate`, `parse_legacy_0173` → `Ok`, date `0999-12-31` ; (b) `<Dt>&#x32;026-10-10</Dt>` : `parse` → `Ok`, 2026-10-10 ; `parse_legacy_0173` → `Ok`, année 26, comme la 0.17.3 (chrono lit `%Y` sur 1 à 4 chiffres, `format/parse.rs:416`, à confirmer par ce test) ; (c) `parse` et `parse_legacy_0173` restent égales à leurs références `camt053_golden`, sans régénération. `git diff` de la story ne touche ni `parse_date`, ni `parse_with_version`, ni `parse_legacy_0173`, ni `handle_text`, ni `flush_pending` (à citer au Change Log).
12. **AC-12 — contrats existants** : sans modification de leurs corps, `bank_accounts_closed_e2e`, `audit_log_e2e`, `open_items_e2e`, `contact_payment_terms_e2e` et `mcp_outils_e2e` passent, et `docs/api-external.md:241` ne change pas. T1 relève d'abord tout test qui épingle une date hors plage : `git grep -nE '\+10000|\+262142|\+70000|0999-|-0001-' -- crates/kesh-api/tests`, liste au Change Log (quatre fichiers à ce jour, relevés le 2026-11-06 : les quatre premiers cités ; `mcp_outils_e2e` n'épingle aucune date hors plage, il doit seulement rester vert). `dunning_reminders_e2e` passe aussi sans modification : son futur est calculé, hors de ce grep.
13. **AC-13 — échéance calculée** : `POST /invoices` à `date = 9999-12-31`, sans `dueDate`, contact à délai de 30 jours → 400 JSON nommant la plage, jamais 500, rien d'écrit. Sans délai au contact, la même date n'est pas refusée pour sa plage.

## Tâches

- [ ] **T1 — la mesure d'abord** : `crates/kesh-api/tests/dates_bornes_e2e.rs`, table `(canal, méthode, chemin, champ, classe, statut_attendu)`,
  un test par champ. Lancée sur le code inchangé, sortie gardée dans `kesh-gate-logs/e334/`, relevé par valeur et classe déduite au Change Log. ⚠️ **Chaque table porte un témoin de forme** : une valeur valide du même type (`2026-01-01`, `2026-01-01T00:00:00`) est acceptée sur ce champ, sans quoi le relevé ne mesure rien. ⚠️ **Le classement exige un corps complet et valide** : un corps partiel rend 422 sur le champ manquant et masque le statut réel (C-e334-4, révisé). Un corps trop coûteux à monter se classe par lecture du handler, ligne citée, et se déclare. Ensuite seulement, la preuve post-correctif d'un site C peut se contenter du corps partiel à marqueur. **Date calculée, relevée en validation P3** : `kesh-db/src/repositories/reconciliation.rs:527-528` lie `d + tolerance_days` en SQL (`:660`, `:707`) ; une transaction au `9999-12-31`, que cette story admet, y donne l'an 10000. T1 relève le statut : si 500, saturation à `9999-12-31` (limite courte, mais `kesh-db` : gate complet) ; sinon, le dire au Change Log.
- [ ] **T2** : même fichier, `POST` de clé d'API, cas de #755 et témoin (AC-8). Rouge attendu si l'analogie tient.
- [ ] **T3** : `kesh_core::dates`, `dates_recues.rs`, pose par classe (outil Edit, site par site), `analyser_date`,
  `parse_as_of`, `audit_log::parse_date`, `closedOn`, l'échéance calculée (AC-13), tests unitaires des bornes (999/1000, 9999/10000, négative).
- [ ] **T4** : analyseurs de fichiers et leurs tests, dont `parse_corrige` et l'AC-11 ; `api_keys.rs`, `mcp_keys.rs` ; correction du commentaire.
- [ ] **T5** : `crates/kesh-api/tests/dates_recues_garde.rs`, garde lexicale sur le modèle de `tests/mcp_gardes_lexicales.rs`, sur
  `src/routes`, `src/services`, `src/mcp` : tout champ daté d'un item `Deserialize` porte `dates_recues::` ou figure à la liste des
  exceptions, écrite avec son motif (`BankProfileResponse`, `ReconciliationRuleResponse`, classes A et B, MCP par le service, `api_keys.rs:67`, borné dans le handler par `expiration_stockable`). Trois sondes dans le test, sur des extraits inline : un champ sans borne est relevé ; un `date_opt` **sans `default`** est relevé ; une exception **sans cible** dans le code fait rougir, sans quoi la liste se périmerait en silence. Une exception de classe B exige un appel à `annee_stockable` dans le même fichier (*un contrôle vide se
  prouve par une sonde*).
- [ ] **T6** : `docs/api-external.md` : une phrase générale, plage des dates et statut par forme ; `:241` reste tel quel. CHANGELOG `## [0.17.5] — Non publié`, *Corrigé* : `refs #606`, `refs #755`.

### Tests qui rougissent avant correctif

T1 (les sites de classes B et C), T2, AC-13. La classe A reste verte avant et après, et c'est sa preuve. **Le statut relevé, non la prédiction, fait foi.**

### Mutations nommées (P2)

- **M1** : `annee_stockable` borne haute à `10000` → rouge `dates_bornes_e2e`, unitaires.
- **M2** : borne basse à `0` → rouge sur `0999-12-31`.
- **M3** : retirer `deserialize_with` d'un `Option` de classe C (`routes/invoices.rs:133`, ou le site que T1 classe ainsi) → rouge T1 et garde T5.
- **M4** : retirer `default` d'un `Option` de classe C (`projects.rs:74`, même réserve) → rouge AC-6 et garde T5.
- **M5** : retirer la garde de `api_keys.rs` → rouge AC-8.
- **M6** : `v04::parse`, puis `v08::parse`, sans `refuser_annees_hors_plage` → rouge AC-7, chacun. **M7** : la borne déplacée dans `parse_date` → rouge AC-11 (a).
- **M8** : `deserialize_with` posé sur `closedOn` → rouge `bank_accounts_closed_e2e`. **M9** : `deserialize_with` sur `invoices.rs:91` → rouge `contact_payment_terms_e2e`. **M10** : échéance calculée non contrôlée → rouge AC-13. **M11** : contrôle de `sent_at` retiré → rouge AC-2 sur `0999-12-31T00:00:00`.

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

`routes/invoices.rs` (33-6, autres lignes) ; ⚠️ `kesh-import/src/camt053/mod.rs` (**33-2**) : la frontière de l'AC-11 fait qu'aucune cause d'échec neuve n'entre dans son inventaire E1 à E4 ; une date en référence relue d'une autre année, sans erreur, relève de son contrôle de désalignement (E4), non d'ici. La 33-2 écrit la réciproque (son AC-S5, C-e332-22 : la relecture héritée sans borne d'années). Ordre de fusion indifférent ; à la seconde, rejouer `camt053_golden`, `bank_imports_e2e` et les tests de ces deux AC.
`routes/supplier_invoices.rs` (33-1a/b). `routes/reports.rs` : **33-3a** y ajoute `get_homepage_summary`, sans date reçue du client ; `services/rapports.rs` (33-3a, 33-7) n'est pas touché ici ; `mcp/outils/rapports.rs` (33-7). Lignes distinctes partout ; union à l'intégration.

## Hors périmètre

Le texte plat des rejets `Query` et `Json` (non JSON, non traduit) : comportement de toutes les erreurs de forme, inchangé ici.

## Change Log

- 2026-11-06 : fiche créée (Opus 5.5).
- 2026-11-06 : validation P1 (Sonnet 5.5, lentilles R et F), remédiation (Opus 5.5). Deux MEDIUM, quatre LOW, prémisses lues au code, aucun réfuté. **M-R4-1** : six champs déjà bornés sortis du périmètre, trois classes A, B, C au lieu d'une pose unique (arbitrage de l'orchestrateur ; lus : `bank_accounts.rs:1107`, `api-external.md:241`, `bank_accounts_closed_e2e.rs:714`, `contact_payment_terms_e2e.rs:490`, `invoices.rs:683`). **M-R4-2** : borne hors du chemin commun, AC-11 (arbitrage ; lus : `camt053/mod.rs:71-96,180,472,530,863`, `v04.rs:22-26` ; chrono `format/parse.rs:416`, lu et non exécuté). **L-R4-3** : garde T5 élargie à `default` et aux exceptions sans cible. **L-R4-4** : AC-5 et conception distinguent le refus par le service (`mcp/outils/ecritures.rs:241`, `postes_ouverts.rs:240`) de `ArgumentsInvalides`. **L-F4-1** : recoupements 33-3a. **L-F4-2** : `EXPIRY_MAX_YEAR` retirée, aucune clé empruntée. **Trouvé en remédiant** : l'échéance calculée (AC-13) ; et le relevé sur corps partiel qui masquait le statut réel (T1). Décomptes recomptés au tableau : Query 17, corps 29, expiration 1, MCP 2 + 11, chaînes 2, déjà bornés 6 : 68. Aucune compilation ni exécution : passe 2 requise.
- 2026-11-06 : validation P2 (Opus 5.5, lentilles R et F), remédiation (Opus 5.5). Un MEDIUM, quatre LOW, un réfuté. **M-R4-1** : AC-2 par type, témoin de forme à T1, `sent_at` classé B (lus : `dunning_reminders.rs:121,261-263`, `errors.rs:1904-1906`, `dunning_reminders_e2e.rs:407-414`). ⚠️ Le refus existant est un **422** JSON, non un 400, statut que le rapport n'avait pas lu : la définition de la classe B s'élargit au refus de domaine. **Retiré** : `date_heure`, sans appelant. **L-R4-2** : `api_keys.rs:67` aux exceptions (lu : `:67`). **L-R4-3** : lot `accept` déclaré (lus : `reconciliation.rs:838,926`). **L-R4-4** : v08, AC-7 et M6. **L-R4-5 réfuté** : `journal_entries.rs:3991,3999,4189,4197,4322,4330` sont dans `mod tests` (ouvert à `:3149`) ; le grep de `Duration::days`, `checked_add_*` en production (`kesh-api`, `kesh-core`, `kesh-db`, `kesh-report`) ne trouve aucune autre date calculée vers la base que `invoices.rs:683`, l'AC-13 ; ce grep ne couvre pas les formes `+ Months` ni les calculs SQL, angle mort déclaré. Mutations : M11 ajoutée, soit 11. Aucune compilation ni exécution : le statut réel de `sent_at` reste à relever par T1.
- 2026-11-06 : validation P3 (Sonnet 5.5, lentilles R et F), remédiation (Opus 5.5). Aucun finding au-dessus de LOW : deux LOW, aucun réfuté. **L-R4-1** : AC-3 et AC-5 (lu : `mcp/outil.rs:178-184`). **L-R4-2** : renvoyé à T1, le statut réel n'étant pas établi (lus : `reconciliation.rs:525-531,660`) ; il complète le grep de l'entrée P2, qui ne couvrait pas `Duration::days` hors de `checked_add_*` et concluait trop vite à un site unique. Mutations : 11, inchangées.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
