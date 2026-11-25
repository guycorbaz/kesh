# Story 53-4 : l'assistant ne réécrit pas le compte bancaire (#1004)

Status: drafted

*Fiche rédigée le 2026-11-25 (Sonnet 5.5), remédiée le même jour après la validation P1 (Opus), worktree
`/home/gcorbaz/devel/kesh-53-4`, branche `story/53-4-assistant-compte-bancaire`, numéros de ligne de `4d540a6ea` (base
`origin/epic/53-ecritures-justes`). Epic 53 « Écritures justes », release visée **v0.21.0** (vendredi 2026-12-04). **Aucune migration,
aucun changement de contrat de l'API, aucune ligne de serveur.** Commits en `refs #1004` ; à la PR, le mot-clé de fermeture n'apparaît que
dans le titre ou le corps de la PR (CLAUDE.md § Issue Tracking Rule).*

## Story

En tant qu'Administrateur qui revient à l'étape 6 de l'assistant alors qu'un compte bancaire principal existe déjà, je veux que l'étape
reparte de ce compte (banque, IBAN, QR-IBAN, description, compte lié), pour que la renvoyer ne vide pas sa description, n'efface pas son
QR-IBAN et ne remplace pas son compte du plan par une présélection, sans que je l'aie demandé.

## Décisions du Project Lead (2026-11-25)

- **Q1 — P1 maintenu.** Le chemin réel est ouvert à tout Administrateur (Étape 0, point 4) ; l'effet est un compte du plan changé sans
  avertissement.
- **Q2 — non : le serveur ne change pas**, pas de changement de contrat. Le QR-IBAN n'est pas une double option (absent et `null` valent
  « effacé ») : **l'écran le renvoie toujours, prérempli**.
- **Q3 — oui : l'étape est bloquée, avec « Réessayer », si la lecture du compte existant échoue**, et le message est clair : il dit ce qui a
  échoué et pourquoi l'étape attend, sans jargon (AC-5, vérifié sur capture). *Configurer plus tard* reste actif pendant le blocage.
- **Couplage Q2/Q3 (résolu).** Sans lecture, l'écran ne peut pas envoyer le QR-IBAN sans l'écraser ; Q2 interdit de changer le serveur ;
  Q3 en tire la conséquence : pas de lecture, pas d'envoi. Les trois décisions tiennent ensemble et ne se rouvrent pas séparément.

## Étape 0 — prémisses relues au code (à rejouer au développement)

Le constat de #1004 est **confirmé pour l'écran** et **démenti pour le serveur**, qui fait déjà ce qu'on lui demande. Un troisième champ
s'ajoute à ceux que l'issue nomme.

1. **Le serveur ne réécrit que ce qu'on lui envoie.** `set_bank_account` (`crates/kesh-api/src/routes/onboarding.rs:635-746`) passe
   à `upsert_primary_in_tx` (`crates/kesh-db/src/repositories/bank_accounts.rs:389-470`) un `PrimaryDetails` à **doubles options**
   (`:317-322`) : `description` et `journalAccountId` **absents = inchangés** (l'`UPDATE` ne nomme pas la colonne, `:422-430`),
   **`null` = effacés**, valeur = posée. Le court-circuit no-op (`is_no_op_change`, `:285-298`) ne compare que les champs présents.
   C'est le contrat de la Story 34-1 (C-e341-34), éprouvé par `bank_step_replays` et `bank_step_update_without_new_fields_keeps_them`
   (`crates/kesh-api/tests/onboarding_bank_account_e2e.rs:321`, `:492`). **Le serveur n'a aucun défaut ici.**
2. **L'écran envoie toujours des valeurs, jamais « absent ».** `submitBankAccount`
   (`frontend/src/routes/onboarding/+page.svelte:474-505`) envoie `description: bankDescription.trim() || null` et
   `journalAccountId: bankJournalAccountId` ; `setBankAccount` (`onboarding.api.ts:80-87`) recopie les cinq champs. Les états
   `bankName`, `bankIban`, `bankQrIban`, `bankDescription` partent de `''` (`:230-248`) et `bankJournalAccountId` de `null` ; la
   **seule** lecture du serveur à l'étape 6 est la liste du plan (`loadBankLedgerOptions`, `:270-297`), qui présélectionne le `1020`
   (`preselectedBankLedgerId`) sans jamais lire un compte bancaire. Sur un compte principal existant, l'envoi écrit donc : description
   `null`, lien = présélection, QR-IBAN `null` si le champ est resté vide.
3. **Inventaire des colonnes de `bank_accounts` écrites par cet `UPDATE`** (`:422-430`, colonnes de `columns!()`, `:52-56`) :

   | Colonne | Écrasée par l'envoi de l'écran ? | Source de l'écrasement |
   |---|---|---|
   | `description` | **oui**, vidée (`null`) | l'écran n'a pas de valeur de départ |
   | `journal_account_id` | **oui**, remplacée par la présélection (1020 s'il existe) ou `null` | idem |
   | `qr_iban` | **oui**, effacée si le champ est resté vide — **non nommé par #1004** | pas une double option : absent et `null` valent « effacé » (`routes/onboarding.rs:600-604`, `:672-679`) |
   | `bank_name`, `iban` | réécrites par la valeur saisie (obligatoires à l'écran, donc retapées) ; sans effet si identiques | le formulaire démarre vide |
   | `version` | +1 si un champ diffère (verrou optimiste) | — |
   | `updated_at` | rafraîchie par tout `UPDATE` (`ON UPDATE current_timestamp(3)`, schéma `:214`) | effet de bord, sans enjeu |
   | `account_kind`, `card_last4`, `is_primary`, `archived`, `closed_on`, `provider`, `provider_account`, `fee_account_id`, `company_id` | non : l'`UPDATE` ne les nomme pas | — |

   Aucun trigger (schéma de test). Les écritures comptables ne sont pas touchées (l'issue le dit ; `upsert_primary_in_tx` n'écrit que
   `bank_accounts`, plus l'audit `bank_account.updated` et `installation.step_completed`). Ce que l'écrasement change dans les livres, c'est
   **le compte du plan par lequel passeront les écritures de réconciliation et d'import**.
4. **Quand le cas se produit — chemin réel, par l'interface seule.** `lock_state_at_step(&mut tx, 6, true)` ne laisse passer l'étape qu'à
   `step_completed = 6`, et l'écran n'a aucun bouton de retour. Mais **à l'étape 6 l'application entière est ouverte** :
   `frontend/src/routes/(app)/+layout.ts:42` ne redirige vers l'assistant que pour `!isDemo && step < 6`. Un Administrateur peut donc
   ouvrir *Comptes bancaires*, y **créer un compte principal** (route Comptable+, `lib.rs:1090-1093`), puis revenir à `/onboarding` (bouton
   Précédent du navigateur, URL) : le formulaire de l'étape 6 est vide, et le renvoyer écrase le compte qu'il vient de créer. La connexion
   mène à `/` (`login/+page.svelte:74`), non à l'assistant, ce qui rend ce va-et-vient naturel. Une clé d'API ou un second onglet y mènent
   aussi. **Chemins écartés** : la restauration ou l'import d'installation ne tombe pas à l'étape 6 (`backup.rs:1058` : l'état passe à 8) ;
   le rechargement après une réponse perdue ne rejoue pas l'étape (upsert et `complete_step` sont dans la même transaction : état 7 après
   commit, rien d'écrit sinon). **À sonder au développement (T0)**, par exécution : cette fiche n'exécute ni serveur ni `curl` (§ 2.1 de
   `docs/agents-orchestration.md`).
5. **La lecture du compte existant est possible à l'étape 6** : `GET /api/v1/companies/current` (`lib.rs:1322`, `routes/companies.rs:650-665`)
   rend `bankAccounts` avec `bankName`, `description`, `iban` (entier), `qrIban`, `accountKind`, `provider`, `isPrimary`,
   `journalAccountId` (`BankAccountJson`, `:607-629`) ; `fetchCompanyCurrent` (`settings.api.ts:26`) l'appelle déjà au récapitulatif. **Aucune
   divergence entre cette lecture et la ligne que l'upsert réécrit** : `list_by_company` (défaut) et le `SELECT … FOR UPDATE` excluent tous
   deux les archivés, et un principal ne peut être ni clos (`chk_bank_accounts_closed_not_primary`, schéma `:231` ; `close_in_tx` refuse
   un principal, `ClosureRefusal::Primary`), ni une carte (`chk_bank_accounts_card_shape`, `:233`), ni un compte de prestataire
   (`chk_bank_accounts_provider`, `:235`). Un compte principal est donc toujours un compte bancaire ordinaire ; **le tri à l'écran se
   réduit à `isPrimary`**.
6. **Le lien existant peut viser un futur compte de créance.** À l'étape 6 les réglages de facturation n'existent pas ; `POST /bank-accounts`
   laisse donc passer un lien vers le compte de rôle `Receivable` ou `Payable` (`routes/onboarding.rs:745`, « vers le futur compte débiteurs
   passerait »). **La finalisation, elle, refuse** un compte de rôle lié à un compte bancaire
   (`DefaultInvoiceAccountCause::RoleAccountLinkedToBankAccount`, `company_invoice_settings.rs:1584` ; « Obligatoire → refus », `:1682`).
   Un tel lien ne peut pas être « conservé » : l'écran le dit et le délie (D3-1).

## Décisions

- **D1 — le correctif est à l'écran, le serveur ne change pas** (décision Q2). Le contrat « absent = inchangé, `null` = effacé » est publié
  (manuel, note d'API, CHANGELOG de la 0.17.6) et testé ; le rendre « `null` = inchangé » casserait le client qui veut effacer.
- **D2 — l'écran repart du compte existant.** À l'étape 6, il lit `fetchCompanyCurrent()` une fois, retient le compte principal
  (`isPrimary`) et en préremplit les cinq champs. Un avis (`data-testid="onboarding-bank-existing-notice"`) dit « Un compte bancaire
  principal existe déjà : le formulaire en reprend les valeurs. » La présélection du `1020` ne vaut que **sans** compte principal.
  **Le QR-IBAN est toujours renvoyé, prérempli** (Q2). *Écartée* : omettre `description` et `journalAccountId` tant qu'on n'y touche pas
  (laisse l'écran vide et le QR-IBAN effacé).
- **D3 — ce que le serveur refuserait, ou ne pourrait pas relire, se traite à l'écran.** Le choix dépend de **deux lectures** (compte existant ;
  liste du plan, rôles compris), terminées dans n'importe quel ordre : l'écran n'en décide qu'à la fin des deux.
  1. *Lien existant de rôle `Receivable` / `Payable`* (Étape 0, point 6) : il ne peut pas être conservé. Le sélecteur affiche « Non configuré »,
     **`null` est envoyé**, et un avis (`data-testid="onboarding-bank-link-claim-notice"`) le dit avant l'envoi : « Le compte lié actuel est
     un compte de créance, qui ne peut pas être lié à un compte bancaire : il sera délié. Choisissez un autre compte si vous le souhaitez. »
     *Écartée* : conserver et dire « conservé » (la finalisation refuserait).
  2. *Lien existant archivé ou devenu non imputable* (absent de la liste liable, rôle ordinaire) *ou liste du plan en échec* : `journalAccountId`
     est **omis** (absent = inchangé), le sélecteur est vide d'affichage, et une phrase sous le champ (`data-testid="onboarding-bank-link-kept"`)
     dit : « Le compte lié actuel n'a pas pu être proposé ici : il est laissé tel quel. » Angle mort assumé : si la liste est en échec, le rôle
     n'est pas connu, et la finalisation (qui affiche son propre remède) peut encore refuser un lien de créance.
  3. *Lecture du compte existant en échec* (décision Q3) : l'étape **ne s'envoie pas** ; l'écran affiche le message de AC-5 et *Réessayer*
     (clé `onboarding-retry` existante). **Configurer plus tard reste actif** : il n'écrit aucun compte (`skip_bank` ne touche que l'étape).
- **D4 — le serveur gagne des tests, pas une ligne** : le corps exact que l'écran enverra après D2, et le contrat du QR-IBAN (T1).
- **D5 — une seule route de lecture** : `fetchCompanyCurrent` est réutilisé (DRY).
- **Angles morts assumés (écrits).** (a) *Lecture périmée* : entre la lecture et l'envoi, une modification faite ailleurs est réécrite par les
  valeurs lues, sans contrôle de version (le corps de l'étape n'en porte pas) ; pas pire qu'aujourd'hui. (b) *Plusieurs principaux non archivés* :
  aucune contrainte ne l'empêche (seul le basculement applicatif l'évite) ; l'upsert (`LIMIT 1 FOR UPDATE`, sans `ORDER BY`) et la liste
  de l'écran (`is_primary DESC, id`) pourraient désigner deux comptes ; anomalie improbable, hors story (serveur inchangé).

## Acceptance Criteria

1. **AC-1 — l'écran repart du compte existant.** À l'étape 6 avec un compte principal existant, `#bank-name`, `#bank-iban`, `#bank-qr-iban`,
   `#bank-description` et `#bank-journal-account` portent ses valeurs ; l'avis `onboarding-bank-existing-notice` s'affiche. Sans compte
   principal (y compris avec d'autres comptes non principaux dans la liste) : écran vide, présélection du `1020` s'il existe, pas d'avis.
2. **AC-2 — rien n'est écrasé sans le dire.** Renvoyer l'étape **sans rien changer** sur un compte existant (description, lien et QR-IBAN
   posés) rend 200, `step_completed = 7`, et le compte est identique : mêmes valeurs, **même `version`**, aucune entrée
   `bank_account.updated` (seulement `installation.step_completed`). Modifier **un** champ ne change que celui-là.
3. **AC-3 — la valeur chargée prime, aussi dans le temps.** Un compte existant lié à un compte autre que le `1020` reste lié à celui-ci à
   l'écran et à l'envoi ; un choix fait dans le sélecteur avant la fin du chargement ne se perd pas ; **le préremplissage tardif ne
   remplace pas** un champ texte que l'utilisateur a déjà modifié (ou les champs sont désactivés pendant la lecture : choix du développeur,
   écrit au Dev Agent Record).
4. **AC-4 — le lien non proposable.** (a) Lien de rôle `Receivable` / `Payable` : l'avis `…link-claim-notice` s'affiche, « Non configuré » est
   sélectionné, le corps envoyé porte `journalAccountId: null`. (b) Lien archivé / non imputable ou liste du plan en échec : le corps ne porte
   **pas** la clé `journalAccountId`, le lien en base est inchangé, la phrase `…link-kept` s'affiche, et le sélecteur n'a pas de valeur périmée
   liée (Svelte 5 : `<select bind:value>` sur une valeur sans option fait `selectedIndex = -1` et garde l'id périmé ; l'id ne part jamais).
5. **AC-5 — lecture du compte en échec : un message clair.** `fetchCompanyCurrent` rejette : l'étape ne s'envoie pas (bouton *Enregistrer*
   désactivé), sans toast (garde `onboarding-catch-guard.test.ts`) ; sous le formulaire, un message en langage simple, sans jargon technique :
   **« La lecture du compte bancaire déjà enregistré a échoué. L'étape attend qu'elle réussisse, pour ne pas écraser ce compte. Réessayez,
   ou choisissez « Configurer plus tard ». »**, suivi du motif du client s'il en dit davantage ; *Réessayer* relance la lecture et, si elle
   réussit, l'étape redevient utilisable ; *Configurer plus tard* reste actif et fonctionne. **Vérifié sur capture** (recette visuelle, 04).
6. **AC-6 — le compte neuf ne change pas.** Sans compte principal, le corps envoyé est celui de la 34-1 ; les AC de la 34-1 (liste filtrée,
   présélection, « modifiable plus tard ») restent vrais.
7. **AC-7 — un compte principal est toujours un compte bancaire ordinaire**, par le schéma : `chk_bank_accounts_closed_not_primary`,
   `chk_bank_accounts_card_shape` et `chk_bank_accounts_provider`. Le tri se réduit à `isPrimary` ; un test Vitest « comptes présents, aucun
   principal » en fixe la frontière.
8. **AC-8 — quatre locales, manuel.** Les clés neuves existent en FR, DE, IT, EN (§ Sites) ; le manuel d'utilisation FR dit le comportement
   (§ Manuel) ; les PDF se régénèrent à la release, non dans la story.
9. **AC-9 — recette visuelle** de chaque état touché, thème clair, base de démonstration : captures jointes au Dev Agent Record et au prompt de
   revue (§ Recette visuelle).
10. **AC-10 — le serveur est inchangé.** `git diff origin/epic/53-ecritures-justes -- 'crates/*/src/*'` (**pathspec entre apostrophes**, pour git, non
    pour le shell) est vide ; les tests Rust neufs passent sans l'avoir touché.

## Tâches

- [ ] **T0 — sonder le chemin réel (Étape 0, point 4)** : sur une installation jetable (base `kesh_534`, jamais celle du NAS), amener l'étape 6 **par
  l'API**, ouvrir *Comptes bancaires*, **créer un compte principal par l'écran**, revenir à `/onboarding`, constater les effacements (description,
  lien, QR-IBAN) ; rouge de référence au journal `/home/gcorbaz/devel/kesh-gate-logs/e534/sonde-t0.txt`. Variante API : `POST /api/v1/bank-accounts`
  avec **`isPrimary: true`** (facultatif, faux par défaut : sans lui le compte n'est pas principal et le test passe à vide).
- [ ] **T1 — Rust, tests seuls** (`crates/kesh-api/tests/onboarding_bank_account_e2e.rs`) : (a) `bank_step_replay_with_full_screen_body_is_unchanged` :
  créer un compte par l'étape (description « Personnel », lien 1020, **QR-IBAN valide**), `set_step(6)`, renvoyer les cinq champs identiques, le
  QR-IBAN **espacé** comme l'afficherait un formulaire : `Unchanged`, `version` 1, actions `["installation.step_completed"]` ; (b)
  `bank_step_absent_or_null_qr_iban_erases_it` : même point de départ ; envoi sans `qrIban` → QR-IBAN `NULL`, `Updated` ; **reposer le QR-IBAN**
  (renvoi avec valeur, `Updated`) ; envoi avec `qrIban: null` → `NULL`, `Updated`. Sans cette repose, le second envoi serait `Unchanged`.
  Aucun rouge à produire (serveur juste) : leur valeur est de fixer le contrat que l'écran respecte ; la preuve de leur tranchant est M7 et M8.
- [ ] **T2 — l'écran** (`+page.svelte`, éventuellement `onboarding.svelte.ts`) : état `existingPrimary` (compte `isPrimary` de la liste), lecture une
  fois à `stepCompleted === 6` (drapeau non réactif, patron `bankLedgerRequested`), **les deux lectures** (compte, liste du plan) attendues
  avant toute décision de D3-1/D3-2, garde « pas d'envoi avant la fin des lectures », préremplissage qui ne touche pas un champ déjà modifié
  (AC-3), D3-1 à D3-3. `BankAccountStepPayload.journalAccountId` devient `number | null | undefined` (absent = inchangé) ; `setBankAccount`
  n'ajoute la clé que si elle est définie. Le tri `isPrimary` vit dans une fonction pure de `lib/features/bank-accounts/format.ts` (testée). **Commentaires
  à réécrire, devenus faux** : `onboarding.api.ts:66-68` (« toujours envoyés ») et `+page.svelte:489-490` (« description et compte lié toujours
  envoyés »).
- [ ] **T3 — Vitest** (`onboarding-bank-step.test.ts`, `bank-accounts/format.test.ts`) : AC-1 (dont « comptes présents, aucun principal »), AC-3
  (champ texte modifié avant la fin de la lecture ; lectures dans les deux ordres), AC-4 (a) et (b) — lien archivé **et** liste en échec —, AC-5
  (texte exact du message, bouton désactivé, *Réessayer*, *Configurer plus tard* actif), AC-6 ; le test de non-écrasement vérifie le **corps
  envoyé** à `api.setBankAccount`.
- [ ] **T4 — Playwright** (`onboarding-bank-account.spec.ts`, bloc neuf, patron de `beforeEach`) : étape 6 par l'API, principal créé par
  `POST /api/v1/bank-accounts` (**`isPrimary: true`**, description, lien non 1020, QR-IBAN), recharger `/onboarding`, constater le
  préremplissage, soumettre sans rien changer, relire par `GET /api/v1/bank-accounts` : description, lien, QR-IBAN et `version` identiques.
  Un second test joue le chemin réel de T0 par l'écran *Comptes bancaires* si le montage le permet. Sélecteurs d'`id`. Spec jouée **seule**
  d'abord, puis suite complète au dernier commit de code.
- [ ] **T5 — i18n et manuel** : clés × 4 locales ; `user-manual.tex` (§ Manuel) ; Notes de version.
- [ ] **T6 — mutations (M1 à M10)** et grep du symptôme, un journal par mutation.
- [ ] **T7 — recette visuelle** et Dev Agent Record.

## Dev Notes

- **Sites.** Lus, non modifiés : `crates/kesh-api/src/routes/onboarding.rs:600-746`, `crates/kesh-db/src/repositories/bank_accounts.rs:285-298`,
  `:317-322`, `:389-470`. Modifiés : `frontend/src/routes/onboarding/+page.svelte:229-303`, `:474-505`, `:781-875` ;
  `frontend/src/lib/features/onboarding/onboarding.api.ts:63-87` ; `onboarding.svelte.ts:134-142` ;
  `frontend/src/lib/features/bank-accounts/format.ts` ; catalogues `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` près de
  `onboarding-bank-editable-later` (`:3540` en FR) ; `docs/manual/fr/user-manual.tex`. Lu : `settings.api.ts:26`.
- **L'epic ne déclare pas tout** : `epic-53-ecritures-justes.md:32` nomme `kesh-api/src/routes/onboarding.rs` (non touché) et omet
  `features/onboarding/`, `features/bank-accounts/`, les catalogues i18n et le manuel ; l'orchestrateur aligne le fichier d'epic (§ *Fusions et
  fichiers communs*).
- **Clés i18n neuves** : `onboarding-bank-existing-notice`, `onboarding-bank-existing-load-error` (le message d'AC-5), `onboarding-bank-link-claim-notice`,
  `onboarding-bank-link-kept` ; *Réessayer* réutilise `onboarding-retry`. Un repli par clé (gardes `i18n-un-repli-par-cle.test.ts`,
  `i18n-repli-divergent-actif.test.ts`) ; recompter `sitesTotal` de `i18n-keys.test.ts` et le compteur de `i18n-libelle-en-dur.test.ts`
  depuis la source ; `npm run lint-i18n-ownership`.
- **Le sélecteur.** Svelte 5.55.1 (`bindings/select.js:45`) : une valeur liée sans option correspondante met `selectedIndex = -1` (rien
  d'affiché) et **garde l'id périmé** dans la variable ; elle ne la convertit pas en `null`. Les cas D3-1 et D3-2 ne lient donc jamais un
  id absent des options : états séparés, `bankJournalAccountId` à `null`.
- **IBAN à l'écran** : le compte existant le rend en entier ; le préremplir dans un champ de saisie est voulu. Aucun IBAN réel dans une capture
  ou une issue : IBAN de démonstration de la suite, captures hors dépôt.
- **Aucun mot-clé de fermeture** dans les commits ; `refs #1004`.
- **Agents** : le développeur ne touche ni la base `kesh_dev` ni le conteneur ; bases `kesh_534` et `kesh_e2e_534`, port à demander à
  l'orchestrateur ; journaux sous `/home/gcorbaz/devel/kesh-gate-logs/e534/`.

## Tests et mutations

Chaque mutation : appliquer, lancer le seul test nommé, constater le rouge, `git checkout --` **puis `touch`** (cargo garde sinon le binaire
muté), journal `kesh-gate-logs/e534/mut/Mn.log`.

| # | Mutation | Test qui doit rougir |
|---|---|---|
| M1 | `is_no_op_change` ignore la description (`is_none_or` → `true`) | `bank_step_replays` (d) : déjà tuée en 34-1 (M8). Ne dit rien de T1 (a) (un court-circuit plus permissif laisse `Unchanged` vert) : jouée pour mémoire, non pour T1 |
| M2 | l'écran ne lit pas le compte existant | Vitest AC-1 ; Playwright T4 |
| M3 | la présélection du `1020` écrase la valeur chargée | Vitest AC-3 |
| M4 | lien archivé / non imputable → envoyé à `null` au lieu d'être omis | Vitest AC-4 (b) |
| M5 | l'écran n'attend pas la lecture avant d'envoyer | Vitest AC-5 (bouton désactivé) |
| M6 | `bankQrIban` non prérempli | Vitest AC-1 ; Playwright T4 (QR-IBAN relu inchangé) |
| M7 | `is_no_op_change` : `existing.qr_iban == new.qr_iban` forcé à `false`, **ou** normalisation du QR-IBAN retirée (`qr.trim().to_string()` au lieu de `as_iban().as_str()`, avec un QR-IBAN espacé) | T1 (a) : `Unchanged` devient `Updated` |
| M8 | `qr_iban` retiré de l'`UPDATE` (`:422`) | T1 (b) : le QR-IBAN n'est plus effacé |
| M9 | liste du plan en échec → `journalAccountId: null` envoyé au lieu d'être omis | Vitest AC-4 (b), cas « liste en échec » |
| M10 | tri « compte principal » remplacé par « premier compte de la liste » | Vitest AC-1, cas « comptes présents, aucun principal » |

Une mutation qui survit est écrite comme telle, avec sa raison (une équivalente se démontre, non se suppose). Une mutation sur un filtre
redondant avec le schéma (clos, carte, prestataire) serait équivalente : non jouée.

## Manuel

`docs/manual/fr/user-manual.tex:225` (étape *Compte bancaire* de l'assistant) : ajouter, après « Cette étape peut aussi être passée… », « Si un
compte bancaire principal existe déjà quand vous arrivez à cette étape, le formulaire en reprend les valeurs ; le renvoyer tel quel ne change rien. »
(exact : no-op, `version` et `updated_at` inchangés). `:2948` ne reçoit **pas** de phrase (la description s'y règle « à l'assistant ou ensuite » :
vrai). **keshnote `:2952`** : elle dit que `POST /onboarding/bank-account` laisse inchangés description et lien omis ; **ajouter** que le QR-IBAN
omis ou `null` est **effacé** (le contrat que T1 (b) épingle). `:225` (« présélectionné s'il existe — une simple proposition ») reste vrai
pour un compte neuf. **DE/EN/IT** : `docs/manual/{de,en,it}/` ne contiennent qu'un `README.md`, sans manuel : rien à y mettre. **PDF** : relus
aplatis (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) pour `présélectionn`, `se modifient ensuite`, `simple proposition`, `laisse inchangés` ; **non
commités** (CLAUDE.md § Fusions et fichiers communs).

## Recette visuelle (CLAUDE.md § *Recette visuelle*)

Base de démonstration jetable (`kesh_e2e_534`), thème clair, captures sous `/home/gcorbaz/devel/kesh-gate-logs/e534/captures/` :

1. `01-compte-neuf.png` — pas de principal : formulaire vide, 1020 présélectionné, « modifiable plus tard », **pas d'avis**.
2. `02-compte-existant.png` — principal avec description, lien ≠ 1020, QR-IBAN : cinq champs préremplis, avis visible.
3. `03a-lien-de-creance.png` — lien de rôle `Receivable`/`Payable` : avis « sera délié », « Non configuré » sélectionné.
3. `03b-lien-laisse-tel-quel.png` — lien archivé : phrase « laissé tel quel », sélecteur sans valeur périmée.
3. `03c-compte-existant-liste-en-echec.png` — compte existant **et** liste du plan en échec : avis, phrase « laissé tel quel », motif de la liste.
4. `04-lecture-en-echec.png` — **le message d'AC-5 en entier, lisible**, *Réessayer*, *Enregistrer* désactivé, *Configurer plus tard* actif ;
   la capture sert à juger que le message est clair, sans jargon.
5. `05-chargement.png` — pendant les lectures : bouton désactivé.
6. `06-apres-envoi.png` — « Configuration terminée », puis *Comptes bancaires* montrant description, lien et QR-IBAN conservés.

## Notes de version

*(Pour l'assemblage du CHANGELOG ; sans numéro d'epic.)* **Corrigé** — L'assistant de démarrage ne réécrit plus un compte bancaire principal qui
existe déjà. Si l'on revient à l'étape du compte bancaire après avoir créé un compte principal ailleurs (par exemple dans *Comptes bancaires*, ou
par l'API), le formulaire en reprend la banque, l'IBAN, le QR-IBAN, la description et le compte comptable lié, au lieu de les remplacer par un
formulaire vide et par la présélection du compte 1020. Si la lecture du compte existant échoue, l'étape attend plutôt que de l'écraser. Un lien vers un
compte de créance, que la finalisation aurait refusé, est délié et signalé. Sans compte existant, rien ne change. Aucune migration ; l'API
`POST /api/v1/onboarding/bank-account` est inchangée.

## Dev Agent Record

*(à remplir au développement)*

## Change Log

- 2026-11-25 : fiche rédigée (Sonnet 5.5), `drafted`. Validation P1 : prompt `53-4-assistant-compte-bancaire-validation-prompt-p1.md`, passe
  complète, **Opus**, contexte frais.
- 2026-11-25 : **validation P1 remédiée** (rapport `kesh-gate-logs/e534/validation-p1.md` : 0 CRITICAL, 0 HIGH, **5 MEDIUM**, 12 LOW). Décisions du Project
  Lead du même jour écrites comme tranchées (Q1 P1 maintenu, Q2 serveur inchangé, Q3 blocage avec message clair). Prémisses relues au code avant
  remédiation : `backup.rs:1058` (import → état 8) ; schéma `:231`, `:233`, `:235` (trois CHECK) ; `company_invoice_settings.rs:1584` ;
  `(app)/+layout.ts:42` ; `user-manual.tex:2952`. **M-1** : chemin réel par l'interface (Étape 0 point 4, Q1, Note de version, T0/T4), cas
  « restauration/import » retiré. **M-2** : D3-3 et AC-7 d'origine retirés, divergence démentie par les trois CHECK, aucune issue à ouvrir. **M-3** :
  D3-1 séparé (lien de créance délié et signalé / archivé ou liste en échec omis), AC-4 (a)/(b), M9. **M-4** : M7 (T1 a), M8 (T1 b), M1 ramenée à ce
  qu'elle prouve, T1 (b) repose le QR-IBAN. **M-5** : couplage Q2/Q3 écrit comme résolu ; *Configurer plus tard* reste actif ; AC-5 dédié (message,
  capture). **LOW** : L-1 `updated_at` au tableau ; L-2 phrase Svelte corrigée (Dev Notes) ; L-3 pathspec entre apostrophes ; L-4 et L-12 angles
  morts écrits ; L-5 AC-3 et T2 (lectures dans les deux ordres, champs déjà modifiés) ; L-6 M9, M10, cas Vitest « aucun principal » ; L-7 les deux commentaires
  nommés en T2 ; L-8 keshnote `:2952`, `:2948` sans phrase, DE/EN/IT sans manuel ; L-9 `isPrimary: true` ; L-10 captures 03a-c ; L-11 note au fichier
  d'epic, à aligner par l'orchestrateur. Prochaine passe : validation P2, Sonnet, complète.
