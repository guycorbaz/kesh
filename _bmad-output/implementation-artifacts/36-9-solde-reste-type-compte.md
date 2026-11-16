# Story 36-9 : le solde du reste d'un règlement refuse un compte de TVA due retypé, comme la validation d'une facture (#1044)

Status: review

*Fiche rédigée le 2026-11-15 par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-9`, branche
`story/36-9-solde-reste-type-compte`, par le workflow `bmad-create-story`. Release **0.18.0**, phase 2, rang 9 (fichier
d'epic, § *Phase 2 — après la 0.17.8*) : « même patron que la 35-3 (Q6) ; #525 reste dans le report de la TVA ».
**Aucune migration.** Code lu à la tête du worktree, **`0bf3c3b4b`** (`origin/epic/36-0.18.0`). Le registre du kickoff
(`e36-choix-autonomes-e360.md`) n'a pas d'étape 0 pour la phase 2 ; seul C-e360-12 nomme la story. Registre de la story :
`e36-choix-autonomes-36-9.md` (C-36-9-1 à C-36-9-4). Voie de base : **voie 1** ; bases `kesh_e369`, `kesh_e2e_e369`,
`kesh_ci_e369`. Validation non lancée.* *(Annoté le 2026-11-15, revue P1, A3 : le registre porte C-36-9-1 à -6 ; la
validation est terminée (P1 Sonnet, 0/0/0/5), le développement aussi ; revue de code P1 remédiée, voir le Change Log.)*

## Story

En tant que titulaire qui solde le reste d'une facture en escompte ou en perte sur débiteur, je veux que Kesh refuse de
corriger la TVA sur le compte de TVA due des réglages si ce compte n'est plus un passif, et me dise lequel et où le
corriger, comme il le fait depuis la 0.17.7 à la validation d'une facture, afin qu'un compte de TVA due retypé en actif
ou en charge ne reçoive pas, en silence, la correction de TVA de mes soldes.

Sans effet sur les livres de Guy tant qu'aucune de ses factures ne porte de TVA (il n'est pas assujetti) : la correction
de TVA d'un solde n'existe que pour une facture à TVA (`write_off_vat_shares`, `crates/kesh-core/src/accounting/vat.rs:184`).

## Ce qui se lit au code (`0bf3c3b4b`)

**Le solde du reste relit la TVA due sans son type.** `write_off_invoice`
(`crates/kesh-db/src/repositories/invoice_settlements_write.rs:521`), pour une nature qui corrige la TVA et une facture
qui en porte, lit le compte :

```
678	    let vat_account_id = if shares.is_empty() {
681	        let vat =
682	            company_invoice_settings::vat_payable_account_for_write(&mut tx, company_id).await?;
683	        // Story 15-6b — la TVA due n'est pas la créance. Aucun contrôle de type ne
684	        // garde ce compte au moment d'écrire : cette comparaison est sa seule
685	        // garde contre `D 1100 / C 1100`.
```

`vat_payable_account_for_write` (`crates/kesh-db/src/repositories/company_invoice_settings.rs:669-692`) exige un compte
« actif, imputable, de la société, `FOR UPDATE` — comme le compte de la nature, sans contrainte de type » (`:665-666`) :

```
682	    sqlx::query_scalar::<_, i64>(
683	        "SELECT id FROM accounts WHERE id = ? AND company_id = ? AND active = TRUE \
684	         AND postable = TRUE FOR UPDATE",
```

Absent, archivé ou non imputable → `ConfigurationRequired("default_vat_payable_account_id")` (`:673`, `:691`) : la
« divergence assumée (C28) » que nomme le doc-comment de la garde à l'usage (`:952-954`). Seule autre garde : la
comparaison à la créance de la vente (`refuse_if_claim_account`, `invoice_settlements_write.rs:686-693`).

**Le retypage passe** (constaté à la 35-3, inchangé) : un rôle de bilan admet `Asset` ou `Liability`
(`crates/kesh-core/src/chart_of_accounts/mod.rs:161-166`), si bien qu'un compte de rôle *TVA due* se retype en actif en
gardant son rôle ; un compte sans rôle, ou dont on retire le rôle dans la même requête, se retype en charge.

**Ce que la 35-3 a posé, et que cette story réemploie** (aucune règle neuve) :
- le type attendu : `DesignatedRole::VatPayable.expected_type()` (`company_invoice_settings.rs:777-784`) lit
  `VAT_SETTING_ACCOUNT_TYPES.payable` (`:1352-1356`, `payable: AccountType::Liability`), la constante que lit aussi la
  route de désignation (`crates/kesh-api/src/routes/company_invoice_settings.rs:399`) ; `DesignatedRole::setting()`
  (`:788-795`) rend `DesignatedSetting::VatPayable` ; les trois sont `pub(in crate::repositories)` ou publics et
  visibles depuis `company_invoice_settings.rs`, où vit l'accesseur ;
- le refus : `DbError::DesignatedAccountsWrongType(WrongTypeDesignatedAccounts)` (`crates/kesh-db/src/errors.rs:1050`),
  liste construite par `WrongTypeDesignatedAccounts::new` (`errors.rs:732`), code `DESIGNATED_ACCOUNT_WRONG_TYPE`
  (`:1922`) ;
- la réponse HTTP : `designated_account_wrong_type_response` (`crates/kesh-api/src/errors.rs:1633-1663`), appelée par le
  bras générique `DbError::DesignatedAccountsWrongType(accounts) =>` (`:4763`) de la conversion de `AppError::Database`,
  **quelle que soit la route** ; la route du solde (`write_off_invoice_handler`, `crates/kesh-api/src/routes/invoices.rs:1247-1285`)
  rend toute `DbError` par `?`, sans cas particulier ;
- le message : une phrase par compte (`error-designated-account-wrong-type`, `crates/kesh-i18n/locales/fr-CH/messages.ftl:343`)
  et une phrase d'action (`:344`), qui finit par « puis la validation pourra être relancée telle quelle » — **fausse pour
  un solde** (D4).

**L'écran affiche déjà le message.** `handleWriteOffConfirm` (`frontend/src/routes/(app)/invoices/[id]/+page.svelte:554-598`)
met `err.message` dans le dialogue pour tout code autre que `OPTIMISTIC_LOCK_CONFLICT` (`:574-581`), relit la facture,
et ferme le dialogue seulement si elle n'est plus soldable (`:584-595`) : une facture refusée pour le type reste
validée et soldable, le dialogue reste ouvert avec le message. **Aucun changement d'écran.**

### Inventaire par le symptôme — toute écriture sur le compte de TVA due des réglages

Symptôme : une écriture passe sur le compte **désigné** comme TVA due (`default_vat_payable_account_id`) sans que son
type soit comparé à celui du réglage. Détecteur par lecture, parti du symptôme : toute lecture du réglage dans le code de
production, puis l'appelant de chaque accesseur.

```
$ git grep -n 'default_vat_payable_account_id' -- 'crates/*/src/*' 'crates/*/src/**'
```

Sortie (`0bf3c3b4b`, 40 lignes, recomptées par `| wc -l` ; commentaires et entités omis ci-dessous, chaque site lu) :

| Site | Ce qu'il fait | Écrit une écriture ? | Type contrôlé | Verdict |
|---|---|---|---|---|
| `invoices.rs:2420` (validation d'une facture) | rôle `VatPayable` de `check_written` | oui | **oui** (35-3) | **sonde positive** : le détecteur trouve le site connu |
| `company_invoice_settings.rs:675` (`vat_payable_account_for_write`) ← `invoice_settlements_write.rs:682` (seul appelant : `git grep -n vat_payable_account_for_write -- 'crates/*/src/**'`) | solde du reste, escompte et perte | oui | **non** | **dans le périmètre** |
| `credit_notes.rs:502`, `:672` (avoir) | verrou puis lignes de contre-passation de la TVA | oui | non | **hors périmètre** : #525, P1 du jalon de la TVA en report assumé (`CLAUDE.md`, § *Priorités des défauts*) ; l'avoir « ne contrôle que l'état actif » par choix (C35, `company_invoice_settings.rs:946-951`) |
| `company_invoice_settings.rs:1487` (`COALESCE`, démonstration) | désignation | non | oui (`lock_vat_role_accounts_in_tx`, `:1425-1446`, par `VAT_SETTING_ACCOUNT_TYPES`) | sans objet |
| `routes/company_invoice_settings.rs:403-404`, `:504` | désignation | non | oui (`:399`) | sans objet |
| `kesh-report/src/vat_report.rs:174` | lecture du solde du compte (décompte) | non | — | sans objet |
| `exports/csv_tables.rs:965`, `:990` | export | non | — | sans objet |
| `invoices.rs:1919`, `credit_notes.rs:276` | refus `ConfigurationRequired` | non | — | sans objet |
| `test_fixtures.rs:151-159`, `:317`, `:776` | montages de test | — | — | sans objet |

Écritures qui touchent ce compte **sans lire le réglage**, écartées du symptôme : l'**annulation d'un solde** passe une
contre-passation de l'écriture d'origine (`reverse_owned_in_tx`, `invoice_settlements_write.rs:1031`), sur les comptes
qu'elle portait — elle défait, elle n'impute pas ; un **règlement par compte interne** et le **rapprochement** écrivent
sur un compte choisi dans la requête, non désigné. Côté achat, la TVA récupérable n'est écrite que par la validation
fournisseur, gardée par la 35-3 (`git grep -n -E 'default_(vat_recoverable|receivable|payable)_account_id' --
'crates/kesh-db/src/repositories/*'` : `invoices.rs:2158`, `supplier_invoices.rs:163`, `:1725`, `:1732`, tous gardés).

**Angles morts du détecteur** : un accès au réglage par un autre nom de colonne (aucun : le réglage n'a qu'une colonne,
`company_invoice_settings.rs:35`) ; un compte de TVA due trouvé par **rôle** et non par réglage — `git grep -n
'AccountRole::VatPayable' -- 'crates/*/src/**'` ne rend que la démonstration (`company_invoice_settings.rs:1438`, typée),
le bilan (`kesh-report/src/balance_sheet.rs:419`, lecture) et des tests.

## Décisions

- **D1 — Le type se contrôle dans l'accesseur, comme l'arrondi et les natures** (C-36-9-1). `vat_payable_account_for_write`
  lit `id, number, postable, account_type` dans **sa** requête `FOR UPDATE` (le type lu sous le verrou, D5 de la 35-3), et
  rend `DbError::DesignatedAccountsWrongType(WrongTypeDesignatedAccounts::new([…]))` avec
  `setting = DesignatedRole::VatPayable.setting()`, `expected_type = DesignatedRole::VatPayable.expected_type()`. Aucune
  constante, aucune variante, aucun code, aucune clé de phrase par compte neufs : ceux de la 35-3. Signature inchangée
  (`Result<i64, DbError>`).
  *Écartée* : passer le solde par `lock_designated_accounts_in_tx` et `check_written` — le verrou deviendrait partagé
  (l'ordre des verrous de ce flux, documenté `invoice_settlements_write.rs:641-654`, serait à refaire), et un compte archivé
  rendrait `INACTIVE_OR_INVALID_ACCOUNTS` au lieu de `CONFIGURATION_REQUIRED` : un second changement de contrat, hors de
  l'objet de #1044 (C28).
- **D2 — Ordre des refus du compte de TVA due** (C-36-9-3, aligné sur D3 de la 35-3) : réglage vide, compte absent,
  étranger ou **archivé** → `ConfigurationRequired` (inchangé) ; sinon **type** faux →
  `DesignatedAccountsWrongType` ; sinon **non imputable** → `ConfigurationRequired` (inchangé). Un compte à la fois
  retypé et non imputable n'est nommé que par le refus de type. Le filtre `postable = TRUE` sort donc du `WHERE` et se
  juge en Rust ; `active = TRUE` y reste.
  *Écartée* : garder `postable` dans le `WHERE` (retypé et non imputable → `CONFIGURATION_REQUIRED`) : plus court d'une
  ligne, mais l'ordre divergerait de celui de la validation pour le même compte.
- **D3 — Le type avant la comparaison à la créance** (C-36-9-2) : le contrôle vit dans l'accesseur, donc il précède
  `refuse_if_claim_account` (`:686`), comme pour la nature et l'arrondi, dont l'accesseur juge le type avant leur
  comparaison (`usable_designated_account`, `company_invoice_settings.rs:699-723`). Une TVA due désignée sur la créance
  **non retypée** (un actif) rend le refus de type ; une TVA due désignée sur une créance **retypée en passif** (le cas
  de `a_vat_payable_account_that_is_the_receivable_is_refused`, `crates/kesh-db/tests/invoice_write_off.rs:1439-1468`)
  passe le type et rend toujours `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT` : la comparaison reste nécessaire.
  *Écartée* : la comparaison d'abord — il faudrait sortir le type de l'accesseur et l'écrire au site d'appel, une seconde
  forme du même contrôle.
- **D4 — La phrase d'action devient neutre** (C-36-9-4) : « … puis la validation pourra être relancée telle quelle »
  (`messages.ftl:344` ×4, repli `crates/kesh-api/src/errors.rs:1654-1657`) devient « … puis l’opération pourra être
  relancée telle quelle. » (de-CH : « danach kann der Vorgang unverändert erneut ausgeführt werden » ; it-CH : « poi
  l’operazione potrà essere rilanciata così com’è » ; en-CH : « and the operation can then be run again as it is »). Le
  bras HTTP est générique et ne connaît pas la route : une phrase vraie pour les trois routes est la seule qui tienne sans
  porter de contexte dans l'erreur. « Telle quelle » est vrai pour le solde : un refus n'incrémente pas la `version`
  (AC7).
  *Écartées* : une variante `DesignatedAccountsWrongType { context, … }` et une seconde clé d'action — un champ neuf dans
  `kesh-db`, tous les constructeurs de `check_written` touchés, pour un mot ; garder la phrase telle quelle — elle dit
  « la validation » à qui vient de cliquer *Solder le reste*.
- **D5 — Un E2E Playwright joue le refus dans le navigateur** : le défaut se voit à la frontière HTTP et dans le dialogue.
  Le compte de TVA due est retypé **par l'API** (`PUT /api/v1/accounts/{id}`, chemin réel, confirmation de retypage s'il
  porte des écritures), et rendu à son type dans un `finally` (la base est partagée entre specs ; `workers: 1`,
  `frontend/playwright.config.ts:53`). *Écartée* : s'en tenir au test HTTP de `kesh-api` — il ne voit pas que le message
  arrive dans le dialogue.

## Critères d'acceptation

1. **AC1 — TVA due retypée en actif, rôle gardé.** Montage `company` de `invoice_write_off.rs` (TVA due `2000`, passif,
   `test_fixtures.rs:131`, `:166`), facture validée à 8.1 %. Le test pose d'abord le rôle `VatPayable` sur `2000` par
   `accounts::update` et vérifie qu'il est posé ; puis `accounts::update` retype `2000` en `Asset` **en gardant son rôle**
   (`confirm_retype = true` : il porte la TVA de la vente). `write_off_invoice` en escompte rend
   `DesignatedAccountsWrongType` à une entrée `(id de 2000, "2000", VatPayable, Liability, Asset)` ; **rien n'est écrit**
   (`assert_nothing_written`, `invoice_write_off.rs:1255-1289` : facture non payée, version inchangée, aucune ligne de
   règlement, aucune écriture, aucun audit). Même refus en perte sur débiteur.
2. **AC2 — Compte sans rôle retypé seul en charge** (`accounts::update`, rôle `None`, type `Expense`) : même refus,
   `actualType = Expense`.
3. **AC3 — Ce qui ne corrige pas la TVA passe.** TVA due retypée en actif : un solde en **frais bancaires**, un solde en
   **reste d'arrondi**, et un solde en escompte d'une facture **sans TVA** réussissent, et l'écriture n'a aucune ligne sur
   `2000` (le compte n'est pas lu, `shares.is_empty()`, `:678`).
4. **AC4 — Ordre** (D2, D3) : TVA due retypée **et archivée** → `ConfigurationRequired` ; retypée **et non imputable** →
   `DesignatedAccountsWrongType` ; non imputable au bon type → `ConfigurationRequired`
   (`an_archived_vat_payable_account_is_refused`, `invoice_write_off.rs:1040-1063`, vert sans retouche) ; TVA due désignée
   sur la créance `1100` **non retypée** → `DesignatedAccountsWrongType` (`expectedType = Liability`,
   `actualType = Asset`) ; désignée sur la créance retypée en passif → `SettlementCounterpartyIsClaimAccount`, rôle
   `VatPayable` (test existant `:1439-1468`, vert sans retouche) ; une nature égale à la créance précède le refus de type
   (`a_nature_on_the_receivable_is_refused_before_a_missing_vat_payable`, `:1406-1437`, et un cas neuf avec la TVA due
   retypée).
5. **AC5 — Le type se lit frais, sous le verrou** (patron `the_type_is_read_fresh_under_the_lock`,
   `crates/kesh-db/tests/invoices_validate_vat.rs:2209-2255`) : une transaction tient `2000` `FOR UPDATE` et le retype en
   `Asset` sans conclure ; le solde, lancé en tâche, est vu en attente sur l'accesseur
   (`attendre_une_requete_en_cours`, `crates/kesh-db/src/test_fixtures.rs:566`, motifs à choisir dans la requête neuve de
   D1 et à vérifier uniques — ni l'accesseur de la validation, ni `usable_designated_account`) ; la transaction valide ;
   le solde rend le refus de type. Exception écrite à « retypage par `accounts::update` » : celui-ci commite lui-même.
6. **AC6 — Le chemin nominal ne bouge pas** : tous les tests existants de `invoice_write_off.rs`,
   `crates/kesh-report/tests/vat_report_write_off.rs`, `crates/kesh-api/tests/invoice_settlement_response_e2e.rs` et des
   autres fichiers qui soldent (`git grep -l -E 'write_off_invoice|/write-off' -- 'crates/*/tests/*'`, douze fichiers à
   `0bf3c3b4b`) passent **sans retouche de leurs attendus**. Étape 0 : pour chaque montage de ces fichiers qui désigne
   une TVA due ou retype un compte (`git grep -n -E 'default_vat_payable_account_id|SET account_type' --` ces fichiers :
   `accounts_designated_archive_e2e.rs:429`, `:1098`, `:1181`, `invoice_echeancier_e2e.rs:1418`, `:1931`,
   `invoice_write_off.rs:575`, `:1228`, `:1418`, `:1447`), écrire au Change Log s'il solde en escompte ou en perte après
   le retypage, et le verdict.
7. **AC7 — HTTP** (`crates/kesh-api/tests/designated_account_type_e2e.rs`, qui monte l'application et le catalogue) :
   `POST /api/v1/invoices/{id}/write-off` (`nature = discount`, la `version` lue) sur une facture à TVA dont la TVA due est
   retypée en actif rend **400 `DESIGNATED_ACCOUNT_WRONG_TYPE`**, `details.rejected = [{accountId, accountNumber: "2000",
   setting: "vatPayable", expectedType: "Liability", actualType: "Asset"}]`, et un message fr-CH égal à « Le compte 2000,
   désigné comme compte TVA due dans Administration → Facturation, est un compte d’actif ; ce réglage demande un compte de
   passif. » suivi, après une espace, de la phrase d'action de D4. Puis le compte est rendu à son type et **la même
   requête, même `version`**, réussit (200) : « telle quelle » est vrai.
8. **AC8 — La phrase d'action, quatre langues et G9** (D4) : la clé `error-designated-account-wrong-type-action` change
   dans `fr-CH`, `de-CH`, `it-CH`, `en-CH` (`parity_between_locales` vert) ; le repli Rust (`errors.rs:1656`) égale la
   valeur fr-CH (G9, `crates/kesh-api/tests/textes_coherents.rs:857`, `les_replis_rust_suivent_le_catalogue`) ; la
   constante `ACTION` de `designated_account_type_e2e.rs:29` suit, et les trois tests de validation de ce fichier
   restent verts. Greper l'ancienne fin de phrase (« la validation pourra être relancée », « die Validierung unverändert »,
   « la convalida potrà », « the validation can then ») sur `crates`, `frontend`, `docs` et écrire le verdict de chaque
   occurrence. *(Étape 0, V1-2)* Le Vitest `frontend/src/routes/(app)/invoices/[id]/invoice-validate-page.test.ts:188`
   simule le message du serveur avec l'ancienne phrase : il est **aligné** sur la phrase neutre (C-36-9-6), si bien que
   `npm run test:unit` entre dans les gates du frontend (complet au passage en `review`).
9. **AC9 — Le navigateur** (D5) : un scénario de `frontend/tests/e2e/invoice-write-off.spec.ts` retype la TVA due par
   l'API, ouvre *Solder le reste*, choisit *escompte accordé*, confirme : le dialogue **reste ouvert** et montre le message
   du serveur (le numéro du compte et « Administration → Facturation » au moins) ; la facture reste « validée », reste dû
   inchangé ; le compte est rendu à son type dans un `finally`. Capture de cet état (thème clair, base jetable) dans
   `/home/gcorbaz/devel/kesh-gate-logs/e36-9/captures/`.
10. **AC10 — Doc-comments et commentaires** reformulés, non effacés : `company_invoice_settings.rs:664-668` (« sans
    contrainte de type ») ; `:952-954` (la divergence C28 reste pour l'absent, l'archivé et le non imputable, le type est
    désormais celui de la validation) ; `invoice_settlements_write.rs:508-519` (« la TVA due n'en a aucun au moment
    d'écrire » ; l'ordre des refus gagne le type) et `:683-685` (« Aucun contrôle de type ne garde ce compte … sa seule
    garde ») ; `crates/kesh-db/src/errors.rs:1030-1035` (le refus vient aussi du solde du reste) ;
    `crates/kesh-api/src/errors.rs:4756-4762` (« la validation veut y écrire ») ; le doc du test
    `invoice_write_off.rs:1437-1438`. Greper `sans contrainte de type`, `n'en a aucun`, `seule garde`, `la validation veut
    y écrire` sur `crates/` et écrire le verdict de chaque occurrence au Change Log.
11. **AC11 — Documentation et manuels** (`.tex` dans la story ; PDF régénérés à l'intégration, contrôlés aplatis) :
    - `docs/api-external.md:596` *(étape 0, V1-1 : `:595` est la ligne de `WRITE_OFF_ACCOUNT_NOT_CONFIGURED`)* : une ligne
      neuve **après** celle de `CONFIGURATION_REQUIRED` du tableau des refus du solde
      — « Compte de TVA due retypé depuis sa désignation — n'est plus un passif (escompte, perte) *(depuis la v0.18.0)* |
      `DESIGNATED_ACCOUNT_WRONG_TYPE` | `400`, `details.rejected` » ; la ligne `CONFIGURATION_REQUIRED` dit que le refus de
      type passe avant le non imputable ; la ligne `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT` (`:597`) dit qu'une TVA due
      égale à la créance **et** qui n'est pas un passif rend désormais le refus de type (*étape 0, V1-3* : changement de
      contrat, D3) ; `:1054` (ligne de `DESIGNATED_ACCOUNT_WRONG_TYPE` du tableau des erreurs) nomme
      la route `POST /invoices/{id}/write-off` (TVA due, escompte et perte), son ordre propre (absent ou archivé →
      `CONFIGURATION_REQUIRED`, type, non imputable → `CONFIGURATION_REQUIRED`), le changement de contrat *(depuis la
      v0.18.0)*, et que « le `message` … puis dit qu'un administrateur doit corriger le réglage » reste vrai ;
    - `docs/manual/fr/user-manual.tex:494` : la phrase « Le \textbf{solde du reste} d'un règlement relit lui aussi la TVA
      due dans les réglages sans en contrôler le type (limite connue, \#1044). » devient l'énoncé du contrôle (*depuis la
      0.18.0*) ; `:1473-1492` (*Solder le reste*) : le refus d'un compte de TVA due changé de type, pour l'escompte et la
      perte, et les deux voies pour corriger ;
    - `docs/manual/fr/admin-manual.tex:3846` (« fait refuser ces validations ») et `:2556` (*Après la mise à jour : les
      comptes désignés*) : le solde du reste en escompte ou en perte est refusé de même, *depuis la 0.18.0* ;
    - *(étape 0, V1-5)* `admin-manual.tex`, section *Passer à la 0.18.0* : elle n'existe pas à `0bf3c3b4b` (`:2538` est
      *Passer à la 0.17.8*) ; **la story l'ouvre** (`\subsubsection{Passer à la 0.18.0}\label{sec:maj-0-18-0}`, avant
      `:2538`) avec le seul paragraphe **« Pour les intégrations »** de ce changement de contrat — route, code, ordre,
      phrase d'action ; l'introduction de la section (sauvegarde, retour) revient à l'intégration de la release
      (C-36-9-5) ;
    - le PDF se contrôle par la seule forme permise (E6) :
      `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'limite connue, #1044'`
      rend `0` après la régénération.
12. **AC12 — Note de version** : la section ci-dessous ; **aucune** écriture de `CHANGELOG.md` dans la branche (C-e360-11).

## Tests à écrire, et la preuve du rouge

La variante, le code, la constante et la réponse HTTP existent déjà (35-3) : les tests neufs **compilent sur le code de
départ** et y rougissent par leur attendu. Écrire d'abord, lancer sur `0bf3c3b4b`, garder le journal
`/home/gcorbaz/devel/kesh-gate-logs/e36-9/rouges.log`.

- `crates/kesh-db/tests/invoice_write_off.rs`, un module à côté de `a_vat_payable_account_that_is_the_receivable_is_refused`
  (`:1439`) : AC1, AC2, AC3, AC4, AC5. **Rouge attendu** : AC1, AC2, AC4 (retypé et non imputable, rouge par
  `ConfigurationRequired` ; créance non retypée, rouge par la comparaison) et AC5 — le solde **réussit** et débite `2000`
  retypé ; le test l'écrit en commentaire, avec l'identifiant de l'écriture produite au journal du rouge. AC3 est vert au
  départ et le reste (garde de non-régression, déclarée telle).
- `crates/kesh-api/tests/designated_account_type_e2e.rs` : AC7 (rouge : 200) ; AC8 (rouge par `ACTION` changée avant la
  clé et le repli).
- `frontend/tests/e2e/invoice-write-off.spec.ts` : AC9 (rouge : le solde réussit, le dialogue se ferme sur « Reste soldé »).
- Retypage par `accounts::update` (le chemin réel), sauf AC5 (exception écrite).

**Mutations** (code commité d'abord ; scripts dans `/home/gcorbaz/devel/kesh-gate-logs/e36-9/mut/` ; restauration
`git restore --worktree -- <fichier>` puis `touch`) : **M1** le contrôle de type retiré de l'accesseur (AC1, AC2, AC7,
AC9) ; **M2** `postable = TRUE` remis dans le `WHERE` (AC4, retypé et non imputable) ; **M3** le type lu par une requête
séparée non verrouillante après la lecture verrouillante (AC5) ; **M4** *(reformulée à l'étape 0, V1-4 : la
comparaison prend l'identifiant que rend l'accesseur et ne peut pas le précéder)* ~~le contrôle de type sorti de
l'accesseur et jugé **après** la comparaison à la créance, au site d'appel~~ — *telle que jouée (revue P1, E4)* : une
comparaison à la créance **ajoutée avant** l'appel de l'accesseur, sur l'identifiant du réglage relu sans verrou ;
l'accesseur garde son contrôle de type, si bien que la comparaison précède le type (`mut/muter.py`, clé `M4`) — même
effet observable sur l'ordre (AC4, créance non retypée : rouge par `SettlementCounterpartyIsClaimAccount`) ; **M5** le type attendu écrit en dur `AccountType::Asset` dans
l'accesseur au lieu de `DesignatedRole::VatPayable.expected_type()` (AC1 et AC3 nominal rougissent ; rend visible qu'aucune
seconde source n'est écrite) ; **M6** le repli Rust de la phrase d'action laissé à l'ancienne valeur (AC8, G9).

## Tâches

- [x] Étape 0 : relire à la tête du worktree les lignes citées ; recompter l'inventaire (la commande et sa sortie au
      Change Log) ; verdicts d'AC6 ; motifs d'AC5.
- [x] Tests ci-dessus, rouges consignés.
- [x] `kesh-db` : `vat_payable_account_for_write` lit le type et l'imputabilité sous son verrou (D1, D2) ; doc-comments
      (AC10).
- [x] `kesh-api` et `kesh-i18n` : phrase d'action neutre ×4 et son repli (D4, AC8) ; commentaire du bras (AC10).
- [x] E2E et capture (AC9) ; `docs/api-external.md`, deux manuels (AC11, `.tex` seulement : PDF à l'intégration).
- [x] Gates (ci-dessous) ; mutations consignées ; Dev Agent Record recompté ; prompt de la revue P1.

**Gates** (voie 1, `flock /tmp/station-heavy.lock flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock
scripts/mem-guard.sh …`, une commande par prise, `free -g` avant) : un **repository de `kesh-db`** est touché, donc
`scripts/test-fast.sh --ci` **complet** à chaque commit de code, même en cours de boucle (`CLAUDE.md`, § *Exception
`kesh-db`*), bases `kesh_e369` remises à neuf ; au **passage en `review`** : `test-fast --ci` et
`scripts/gate-ci.sh kesh_ci_e369`, et `npm run check` et `npm run lint-i18n-ownership` (seule une spec E2E est touchée
au frontend ; aucun `i18nMsg(` neuf) ; aux **remédiations** : `test-fast --ci` ; à la **clôture** : `gate-ci` **et** E2E
complet sur `kesh_e2e_e369` recréée, au dernier commit de code, échecs nommés un par un contre `docs/testing.md`
§ *Les échecs attendus*. Chaque chiffre cite son journal (`/home/gcorbaz/devel/kesh-gate-logs/e36-9/…`) et son commit.

## Note de version

Rubrique **Corrigé** (section `## [0.18.0] — Non publié`, assemblée par l'orchestrateur) :

> - **Le solde du reste d'une facture refuse un compte de TVA due changé de type.** Solder le reste en escompte ou en perte
>   sur débiteur corrige la TVA sur le compte de TVA due d'*Administration → Facturation* ; si ce compte a été changé de
>   type depuis sa désignation (il n'est plus un passif), le solde est maintenant refusé, sans rien écrire, avec le
>   message de la validation d'une facture, qui nomme le compte et le réglage. Désignez un compte de passif, ou rendez au
>   compte son type, puis soldez à nouveau. ⚠️ **Changement de contrat de l'API** : `POST /api/v1/invoices/{id}/write-off`
>   rend `400 DESIGNATED_ACCOUNT_WRONG_TYPE` là où il réussissait (et là où il rendait `400 CONFIGURATION_REQUIRED` pour
>   un compte à la fois retypé et non imputable, ou `400 SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT` pour un compte de TVA
>   due désigné sur la créance qui n'est pas un passif : le type est jugé avant) ; la dernière phrase du message de ce code dit désormais « l’opération »
>   au lieu de « la validation », sur les trois routes qui le rendent. Le solde en frais bancaires ou en reste d'arrondi
>   n'est pas concerné, ni un reste de quelques centimes dont la part de TVA, au centime, est nulle : seul un solde qui
>   corrige réellement de la TVA lit ce compte. L'avoir ne contrôle toujours pas ce compte ([#525](https://github.com/guycorbaz/kesh/issues/525)).
>   ([#1044](https://github.com/guycorbaz/kesh/issues/1044))

`refs #1044` ; la PR de release porte `closes #1044`.

## Dev Notes

### Hors périmètre

- **L'avoir** (#525, P1 du jalon « Vague 2 », report assumé) : il relit la TVA due dans les réglages du moment sans en
  contrôler le type ni l'imputabilité (C35, `company_invoice_settings.rs:946-951`) — voulu pour qu'une facture reste
  annulable, et le compte qu'il devrait viser (celui de la vente) est l'objet même de #525.
- **Le compte que vise la correction de TVA du solde** : elle est « débitée sur le compte de TVA due **courant** (comme
  l'avoir) » (`invoice_settlements_write.rs:503-504`), non sur celui de la vente. Même classe que #525, mais #525 ne nomme
  que l'avoir : **issue demandée à l'orchestrateur** (rapport de la fiche), à trier dans le jalon de la TVA —
  ouverte : **#1198** (revue P1, A6).
- **Un avertissement au retypage d'un compte désigné** : #1047.
- **Le frontend** : aucun changement (le dialogue affiche déjà `err.message`), donc aucun Vitest neuf.

### Modules touchés

`kesh-db` (un repository, doc de `errors.rs`, un fichier de tests), `kesh-api` (repli de la phrase, un commentaire, un
fichier de tests), `kesh-i18n` (une clé ×4), `frontend` (une spec E2E seule), `docs` (`api-external.md`, deux manuels) :
**cinq**, sous le seuil de découpage.

### Fichiers communs avec les autres stories de la 0.18.0

Le tableau du fichier d'epic ne couvre que la phase 1 ; recensés ici, à confirmer par l'orchestrateur :

| Fichier | Ici | Voisins probables | Règle |
|---|---|---|---|
| `crates/kesh-i18n/locales/*/messages.ftl` ×4 | la ligne 344 seule, sur place | 36-15 (vocabulaire « compte analytique »), 36-18 | aucune ligne ajoutée en fin de fichier ; une ligne modifiée en place |
| `docs/manual/fr/user-manual.tex` | `:494`, `:1473-1492` | 36-15, 36-16, 36-18 | paragraphes distincts ; PDF après l'union |
| `docs/manual/fr/admin-manual.tex` | `:2556`, `:3846` | 36-15 | idem |
| `docs/api-external.md` | `:595`, `:1054` | 36-10 (audit), 36-18 | lignes distinctes |
| `crates/kesh-api/src/errors.rs` | `:1654-1657`, `:4756-4762` | 36-18 (refus de montant ?) | sites distincts |

`textes_coherents.rs` n'est pas modifié (la clé de G9 ne change pas, seule sa valeur) ; `CHANGELOG.md` n'est pas touché.

### Ce qui doit rester vrai

- Le refus de type ne s'atteint qu'en escompte ou en perte, quand le solde corrige de la TVA (une part non nulle au
  centime, `write_off_vat_shares` ; revue P1, E1) ; rien n'est écrit, la
  `version` ne bouge pas, le rejeu sur interblocage (`retry_on_deadlock`, `routes/invoices.rs:1262`) est inchangé.
- L'ordre des verrous du flux (nature, arrondi, TVA due, exercice, `invoice_settlements_write.rs:641-654`) est inchangé :
  la requête neuve verrouille la même ligne par clé primaire, au même moment.
- Les trois tests de validation de la 35-3 (`ac7_*`, `designated_account_type_e2e.rs:249`, `:281`, `:317`) ne changent que par `ACTION`.
- Aucune donnée réelle : fixtures synthétiques (`seed_accounting_company`, seed E2E).

### Références

- Fiche 35-3 : `_bmad-output/implementation-artifacts/35-3-comptes-designes-type.md` (D2 à D5, inventaire, Hors
  périmètre : « Le solde du reste … #1044, qui reprendra la constante et le code de cette story »).
- Issue #1044 (bug, P3) ; #537 (35-3) ; #525 ; #1047.
- `CLAUDE.md`, § *Priorités des défauts*, § *Migration breaking policy* (aucune migration ici).

## Dev Agent Record

### Agent Model Used

Opus 5.5 (agent de développement de l'Epic 36), le 2026-11-15, worktree `/home/gcorbaz/devel/kesh-36-9`, voie 1.

### Debug Log References

Journaux sous `/home/gcorbaz/devel/kesh-gate-logs/e36-9/` (hors dépôt) :

- **Rouges** (code de départ, tests neufs écrits, commit `ce95b8b5a` + tests non commités) : `rouges.log` — 35 tests, 27
  passés, **8 échecs** : AC1 et AC2 (le solde **réussit** : `WriteOffOutcome { journal_entry_id: 3, amount: 108.1000, … }`
  et `journal_entry_id: 2` — la correction de TVA débitée sur `2000` retypé), AC4 (`attendu DesignatedAccountsWrongType,
  obtenu ConfigurationRequired("default_vat_payable_account_id")`, le cas retypé et non imputable), AC5 (le solde n'est
  jamais vu en attente sur la requête neuve : `moins de 1 requête(s) contenant ["SELECT id, number, postable,
  account_type FROM accounts WHERE id", "FOR UPDATE"]`), et les trois `ac7_*` de validation de la 35-3 (AC8 : `ACTION`
  changée avant la clé). AC7 (`ac7_write_off_names_the_retyped_vat_payable`) rougissait d'abord sur son **montage** (409
  `ACCOUNT_HAS_ENTRIES` du retypage, voir *Écarts*) ; rejoué après la correction du helper : `rouges-ac7-bis.log`, rouge
  pour la bonne raison (`left: 200, right: 400`, le solde passe). AC3 et AC4 « créance non retypée » : verts au départ pour
  le premier (garde de non-régression), le second n'a pas été atteint (le test s'arrête au cas non imputable).
  *(Annoté à la revue P1, A2 : le rouge d'AC5 au code de départ l'est **par le motif**, la requête neuve n'existant pas
  encore, non par le défaut ; la preuve que le test voit un type lu hors verrou est la mise à mort par **M1** et
  **M3**, `mut/M1.log`, `mut/M3.log`. A1 : le rouge de l'AC9 (E2E) prescrit par la fiche n'avait **pas** été joué à
  cette date, malgré la case cochée — joué à la remédiation, ci-dessous.)*
- **Vert ciblé** : `cible-vert.log` (commit de travail, avant `dec591150`, contenu identique) — `invoice_write_off`,
  `designated_account_type_e2e`, `textes_coherents`, `kesh-i18n` : 102 tests, 102 passés.
- **Clippy** : `clippy-dev.log`, workspace `--all-targets -D warnings`, vert.
- **Mutations** (code commité `dec591150` d'abord ; scripts `mut/muter.py`, `mut/jouer.sh` ; restauration
  `git restore --worktree` puis `touch`, chaque journal finit par « restauré : 0 fichier(s) modifié(s) restant(s) ») :
  | Mutation | Journal | Tuée par |
  |---|---|---|
  | M1 contrôle de type retiré | `mut/M1.log` | AC1, AC2, AC4, AC5, AC7 (5 échecs sur 35) |
  | M2 `postable = TRUE` remis au `WHERE` | `mut/M2.log` | AC4 (`the_type_comes_after_archived_and_before_non_postable_and_claim`) |
  | M3 type relu par une requête séparée non verrouillante | `mut/M3.log` | AC5 (`the_vat_payable_type_is_read_fresh_under_the_lock`) |
  | M4 comparaison à la créance avant le type (V1-4) | `mut/M4.log` | AC4 (même test, cas « créance non retypée » : seule partie que M4 change) |
  | M5 type attendu écrit en dur `Asset` | `mut/M5.log` | 12 échecs : les soldes nominaux à TVA, AC1, AC2, AC4, AC5, AC7 |
  | M6 repli Rust à l'ancienne phrase | `mut/M6.log` | G9 `les_replis_rust_suivent_le_catalogue` |
  *(Annoté à la revue P1 : A1 — la fiche nommait aussi l'AC9 (E2E) parmi les tueurs de M1, retiré ici sans le dire ;
  E4 — M4 jouée est une comparaison à la créance **ajoutée avant** l'accesseur, non le type sorti de l'accesseur ;
  A4 — ces journaux ne gardaient pas le message du rouge. Les journaux `mut/M*.log` ont été **réécrits** par le rejeu
  de la remédiation, ci-dessous, qui remplace ce relevé.)*
- **Gates du passage en `review`**, au dernier commit de code **backend** `dec591150` (le commit suivant, `84246f671`, ne
  touche qu'une spec E2E) :
  - bases `kesh_e369` remises à neuf (DROP/CREATE, migrations, seed) ; `scripts/test-fast.sh --ci` :
    `test-fast-ci-dec591150.log` — **5459 exécutés, 5459 passés, 13 ignorés** ;
  - `scripts/gate-ci.sh kesh_ci_e369` : premier run `gate-ci-dec591150.log` — **rouge** sur
    `mcp_transport_e2e::abandon_au_delai_le_permis_reste_pris` (« témoin : sonde servie », KF-061, **#1126**, minutage ;
    une suite de la voie 2 tournait en même temps) : 2122 passés, 1 échec, `cargo test` arrêté à ce binaire ; second
    run `gate-ci-dec591150-r2.log` — **5466 passés, 0 échec, 17 ignorés** ;
  - frontend complet (le Vitest aligné, C-36-9-6) : `front-check-dec591150.log` (0 erreur, 29 avertissements, tous
    hors des fichiers touchés), `front-lint-i18n-ownership-dec591150.log` (vert), `front-test-unit-dec591150.log`
    (**238 fichiers, 3709 tests passés**), `front-build-dec591150.log` (vert) ; après `84246f671` :
    `front-check-r2.log` (0 erreur).
  - **E2E** : la suite complète **n'a pas tourné** (à la clôture). La seule spec `invoice-write-off.spec.ts` a été jouée
    pour la capture, sur `kesh_e2e_e369` recréée, backend port 3369 : `e2e-spec-write-off.log` (1 échec, le scénario neuf
    — voir *Écarts*), `e2e-spec-write-off-r3.log` au fichier de `84246f671` : **2 passés**.
  *(Annoté à la revue P1, A4 : `gate-ci-dec591150*.log` ne portent pas de commit en tête ; leur rattachement à
  `dec591150` tient aux horodatages (18:13 et 19:41, entre `dec591150` 16:10 et `84246f671` 20:09). `front-check-r2.log`
  (20:09:23) est « au fichier de `84246f671` », non « après ». `e2e-spec-write-off-r2.log` (2 passés, avant la
  restauration de Prettier) n'était pas cité.)*

#### Remédiation de la revue P1 (2026-11-16)

Commits `dd9bccfdf` (tests, spec, textes) et `34a2dbee1` (la capture attend la fin du fondu du bouton) ; tous les
journaux ci-dessous portent en tête `# commit …`.

- **Ciblé** : `rem1-cible.log`, `invoice_write_off` au fichier de `dd9bccfdf` : 31 passés.
- **E2E, preuves de l'AC9** (A1), script versionné `mut/e2e-preuve.sh` (montage de `docs/testing.md`, `kesh_e2e_e369`
  recréée à chaque run, port 3369, sous verrou ; restauration `git restore --worktree` puis `touch`) :
  - `e2e-preuve-depart.log` (commit `dd9bccfdf`, production rétablie à `4c5019c5d`, spec de HEAD) : **rouge** —
    `Expected: 400 / Received: 200`, `invoice-write-off.spec.ts:153` (le solde réussit) ; l'autre scénario passe ;
  - `e2e-preuve-M1.log` (`dd9bccfdf`, M1) : **M1 tuée par la spec**, même message ;
  - `e2e-preuve-vert.log` (commit `34a2dbee1`) : **2 passés** ; capture reprise :
    `captures/write-off-tva-due-retypee.png` — bouton *Solder le reste* actif et opaque (E3 ; le premier rejeu à
    `dd9bccfdf` l'avait encore pris pâle, `disabled:opacity-50` sous `transition-all`, d'où `34a2dbee1`).
- **Mutations rejouées** à `34a2dbee1` (A4 : message du rouge gardé, commit en tête), `mut/M1.log` à `mut/M7.log` :
  | Mutation | Tuée par (message au journal) |
  |---|---|
  | M1 contrôle de type retiré | 6 échecs sur 36 : AC1, AC2, AC2 produit, AC4, AC5, AC7 ; et l'E2E (`e2e-preuve-M1.log`) |
  | M2 `postable = TRUE` au `WHERE` | AC4 — `obtenu ConfigurationRequired("default_vat_payable_account_id")` |
  | M3 type relu hors verrou | AC5 |
  | M4 comparaison à la créance ajoutée avant l'accesseur | AC4 — `obtenu SettlementCounterpartyIsClaimAccount { … role: VatPayable … }` |
  | M5 type attendu `Asset` en dur | 13 échecs sur 36 |
  | M6 repli Rust à l'ancienne phrase | G9 `les_replis_rust_suivent_le_catalogue` (31 tests) |
  | M7 *(neuve, proposée par A et E)* refus du non imputable retiré | `an_archived_vat_payable_account_is_refused` — le solde réussit |
- **Gate de remédiation** : `test-fast-ci-34a2dbee1.log` (voir le Change Log pour le chiffre). Vitest non rejoué : aucun
  fichier de `frontend/src` ne change (seule la spec E2E). `gate-ci` et E2E complet : à la clôture.

### Completion Notes List

- **D1 à D5 appliquées sans écart.** `vat_payable_account_for_write` lit `id, number, postable, account_type` sous
  `FOR UPDATE`, `active = TRUE` gardé au `WHERE` ; ordre absent/archivé → type → non imputable ; le refus
  `DesignatedAccountsWrongType` réemploie `DesignatedRole::VatPayable.{expected_type, setting}` et
  `WrongTypeDesignatedAccounts::new` ; signature inchangée. Aucune migration.
- **Changement de contrat** écrit dans `docs/api-external.md` (tableau du solde, deux lignes modifiées et une neuve ;
  § 10), au manuel d'administration (section neuve *Passer à la 0.18.0*, « Pour les intégrations », C-36-9-5) et dans la
  note de version — non au `CHANGELOG.md`.
- **Manuels** : `.tex` seulement ; les PDF se régénèrent à l'intégration (`CLAUDE.md`, § *Travailler à plusieurs
  développeurs*). Le PDF publié porte encore « limite connue, #1044 » (`pdftotext docs/manual/fr/user-manual.pdf - | tr
  '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'limite connue, #1044'` → `1`) : le contrôle de l'AC11 (`0`) se fait à
  l'intégration.
- **Capture** (AC9, thème clair, base jetable `kesh_e2e_e369`) :
  `/home/gcorbaz/devel/kesh-gate-logs/e36-9/captures/write-off-tva-due-retypee.png` — le dialogue *Solder le reste*
  ouvert, *Escompte accordé* coché, le message du serveur en entier, phrase d'action neutre comprise.
- **AC8, grep de l'ancienne phrase** (`git grep -n -e 'la validation pourra être relancée' -e 'die Validierung
  unverändert' -e 'la convalida potrà' -e 'the validation can then' -- crates frontend docs`) : **vide** après
  `dec591150`. **AC10** (`git grep -n -e 'sans contrainte de type' -e "n'en a aucun" -e 'seule garde' -e 'la validation
  veut y écrire' -- crates/`) : douze lignes, aucune sur ce sujet hormis `invoice_write_off.rs:1439` (doc reformulée du
  test 15-6b, juste : la créance retypée en passif passe le type, la comparaison est alors la seule garde) ; les onze
  autres portent sur d'autres gardes (lettrage, réconciliation, MCP…).
- **Écarts** :
  1. *Helper `retype` de la 35-3* (`designated_account_type_e2e.rs`) : il envoyait `"confirmRetype": true`, champ que la
     route ignore (`routes/accounts.rs:109` : `confirm_account_retype`) — sans effet tant que les comptes retypés ne
     portaient aucune écriture. Corrigé en `confirmAccountRetype` dans le fichier que la story touche ; même champ dans
     la spec E2E. Défaut de test de la 35-3, corrigé sur place, sans issue.
  2. *Scénario E2E* : il lisait d'abord `invoice-amount-due`, ligne que la fiche n'affiche qu'après un premier règlement
     (`+page.svelte`, `amountSettled > 0`) ; corrigé (`84246f671`) : le reste se lit dans le dialogue
     (`write-off-amount`) et par l'API, avec l'absence de tout règlement.
  3. *Prettier* : lancé une fois en `--write` sur les deux fichiers du frontend touchés, il a réécrit 427 lignes du
     dépôt (le dépôt n'a pas de configuration Prettier et ces fichiers ne la suivent pas) ; rétabli par `git restore
     --worktree` et les deux modifications de la spec rejouées par l'outil Edit. Aucune trace dans les commits.
  4. Consigne Edit : code, tests, fiche par l'outil Edit ; `cargo fmt --all` lancé (permis) ; scripts Python seulement
     pour les mutations.
- **Branche** : en retard de trois commits documentaires sur `origin/epic/36-0.18.0` (`CLAUDE.md`, `ci.yml`, fichier
  d'epic) ; non rebasée (interdit), sans recouvrement avec ce diff.

### File List

- `crates/kesh-db/src/repositories/company_invoice_settings.rs` — l'accesseur ; doc de la garde à l'usage.
- `crates/kesh-db/src/repositories/invoice_settlements_write.rs` — doc-comment de `write_off_invoice`, commentaire de
  l'appel.
- `crates/kesh-db/src/errors.rs` — doc de `WrongTypeDesignatedAccount` et de `DesignatedAccountsWrongType`.
- `crates/kesh-db/tests/invoice_write_off.rs` — module `tva_due_type` (six tests, 21 → 27 `sqlx::test`), doc d'un test.
- `crates/kesh-api/src/errors.rs` — repli de la phrase d'action, commentaire du bras.
- `crates/kesh-api/tests/designated_account_type_e2e.rs` — `ACTION`, helper `retype`, un test (4 → 5).
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` — ligne 344.
- `frontend/src/routes/(app)/invoices/[id]/invoice-validate-page.test.ts` — simulacre aligné.
- `frontend/tests/e2e/invoice-write-off.spec.ts` — un scénario (1 → 2).
- `docs/api-external.md`, `docs/manual/fr/user-manual.tex`, `docs/manual/fr/admin-manual.tex`.
- `_bmad-output/implementation-artifacts/36-9-solde-reste-type-compte.md`, `e36-choix-autonomes-36-9.md` (C-36-9-5,
  -6), `sprint-status.yaml`, `36-9-solde-reste-type-compte-review-prompt-p1.md`.

## Change Log

- **2026-11-15 — Création** (agent de rédaction des fiches de l'Epic 36, Opus 5.5) : fiche écrite sur `0bf3c3b4b` ;
  inventaire par le symptôme (40 lignes de `git grep`, un site dans le périmètre, l'avoir hors périmètre par #525) ;
  décisions D1 à D5, registre `e36-choix-autonomes-36-9.md` (C-36-9-1 à C-36-9-4) ; aucune migration ; cinq modules.
  Validation non lancée.
- **2026-11-15 — Validation terminée** : **une passe**, P1 Sonnet (complète), **0 CRITICAL / 0 HIGH / 0 MEDIUM / 5 LOW**
  (V1-1 à V1-5), rapport `/home/gcorbaz/devel/kesh-gate-logs/e36-9/validation-p1.md`. Trend : P1 0 > LOW — boucle
  close sur une passe sans finding au-dessus de LOW (`CLAUDE.md`, § *Review Iteration Rule*). Les cinq LOW traités à
  l'étape 0 ci-dessous, aucun reclassement.
- **2026-11-15 — Développement, étape 0** (agent de développement, Opus 5.5, worktree `kesh-36-9`, tête `4c5019c5d`).
  Les cinq LOW de la validation, chacun avec la ligne lue :
  - **V1-1** — `/usr/bin/grep -n 'Compte de TVA due absent ou inutilisable' docs/api-external.md` →
    `596:| Compte de TVA due absent ou inutilisable — archivé, non imputable (escompte, perte) | CONFIGURATION_REQUIRED | 400 |`
    (`:595` : « Aucun compte utilisable désigné pour la nature »). AC11 cite désormais `:596` et `:597`.
  - **V1-2** — `frontend/src/routes/(app)/invoices/[id]/invoice-validate-page.test.ts:188` : `const TYPE = "Le compte
    1100, … puis la validation pourra être relancée telle quelle.";` (simulacre de `validateInvoiceMock`). Aligné sur
    la phrase neutre (C-36-9-6) ; AC8 le nomme ; `npm run test:unit` entre dans les gates.
  - **V1-3** — `docs/api-external.md:597` : « … `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT` | `400`, `details.role`
    (`write_off_nature`, `rounding`, `vat_payable`) ; refusés dans cet ordre, le premier nommé » ; et
    `invoice_settlements_write.rs:682-693` (`vat_payable_account_for_write` puis `refuse_if_claim_account(.., vat, ..)`) :
    avec le type dans l'accesseur, une TVA due égale à la créance et restée un actif rend le refus de type. Écrit à AC11
    (ligne `:597`) et à la note de version.
  - **V1-4** — `invoice_settlements_write.rs:681-693` : `let vat = …vat_payable_account_for_write(…).await?;` puis
    `refuse_if_claim_account(&mut tx, company_id, vat, …)` : la comparaison consomme l'identifiant rendu par
    l'accesseur, M4 telle qu'écrite était irréalisable. M4 reformulée : le type jugé après la comparaison, au site
    d'appel (rouge attendu : le cas « créance non retypée » d'AC4).
  - **V1-5** — `admin-manual.tex:2538` : `\subsubsection{Passer à la 0.17.8}\label{sec:maj-0-17-8}`, première section de
    mise à jour ; aucune section *Passer à la 0.18.0* (`/usr/bin/grep -n 'Passer à la 0.18' docs/manual/fr/admin-manual.tex`
    vide). La story l'ouvre avec le seul paragraphe « Pour les intégrations » (C-36-9-5, AC11).
  Recomptes : `git grep -n 'default_vat_payable_account_id' -- 'crates/*/src/*' 'crates/*/src/**' | wc -l` → **40**
  (comme la fiche) ; `git grep -n vat_payable_account_for_write -- 'crates/*/src/**'` → définition
  `company_invoice_settings.rs:669`, doc `:953`, **un** appelant `invoice_settlements_write.rs:682`. AC6 :
  `git grep -l -E 'write_off_invoice|/write-off' -- 'crates/*/tests/*'` → **douze** fichiers (onze de tests et
  `crates/kesh-db/tests/support/lettering_documents.rs` ; le « 11 » de la validation exclut ce dernier). Verdicts des
  montages qui désignent une TVA due ou retypent un compte : `accounts_designated_archive_e2e.rs:429` (archive la TVA
  due, puis **valide** ; ne solde pas), `:1098` et `:1181` (réglages seulement, aucun solde) ;
  `invoice_echeancier_e2e.rs:1418` (compte d'arrondi retypé en produit, puis règlement ; ne solde pas),
  `:1931` (créance retypée en charge, désignée escompte, facture à 8.10 % — `create_validated_invoice`, `vat_rate:
  dec!(8.10)` — : refus par la nature, comparée avant la lecture de la TVA due, `2000` n'étant pas retypé) ; `invoice_write_off.rs:575` (TVA due vidée → `ConfigurationRequired`, inchangé), `:1228`
  (`retype_receivable`, la créance), `:1418` (TVA due vidée, nature égale à la créance : la nature précède), `:1447`
  (TVA due désignée sur la créance **retypée en passif** : le type passe, la comparaison refuse) — **aucun ne voit son
  attendu changer**. Motifs d'AC5 : `"SELECT id, number, postable, account_type FROM accounts WHERE id"` et
  `"FOR UPDATE"`, uniques — le verrou de la validation lit `SELECT id, number, active, postable, account_type FROM
  accounts FORCE INDEX` (`company_invoice_settings.rs:999`), celui des natures `SELECT id FROM accounts WHERE id = ?`
  (`:713`).
- **2026-11-15 — Développement terminé, passage en `review`** (Opus 5.5). Commits `dec591150` (code, tests, documents)
  et `84246f671` (scénario E2E corrigé). Rouges, mutations M1 à M6 (toutes tuées), gates et écarts : *Dev Agent Record*.
  Gates au dernier commit de code backend `dec591150` : `test-fast --ci` 5459/5459 (13 ignorés) ; `gate-ci` 5466 passés,
  17 ignorés, au second run (le premier rouge sur KF-061, #1126) ; frontend complet vert (Vitest 3709). **Non lancés** :
  l'E2E complet et le `gate-ci` de clôture (à la clôture) ; la seule spec `invoice-write-off.spec.ts` a tourné (2 passés).
  Prompt de la revue P1 : `36-9-solde-reste-type-compte-review-prompt-p1.md`.
