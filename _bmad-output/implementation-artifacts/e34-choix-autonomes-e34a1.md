# Epic 34 — choix autonomes de la story 34-11a1 (clé e34a1, compte sans IBAN, #972)

*Registre du développement (Opus 5.5, 2026-11-10). Les choix de la fiche sont au registre du groupe,
`e34-choix-autonomes-e341.md` (C-e341-13, 14, 17, 18, 19, 21, 29, 39) ; ceux-ci les complètent sans les contredire.*

**C-e34a1-1 — l'IBAN absent entre dans l'empreinte d'aperçu en `null` JSON.** Contexte : AC-7 exige un encodage distinct de toute
chaîne et stable, et l'empreinte d'un compte bancaire inchangée. Retenu : `bank_account.iban.as_deref().map(normalize_iban)` dans
le tableau sérialisé, si bien qu'un compte avec IBAN produit exactement la même chaîne JSON qu'en 0.17.5 (valeur figée par le test
U5) et qu'un compte sans IBAN y porte `null`, qu'aucune chaîne — vide, `"null"` — ne peut imiter (test U6). Autres : une chaîne
sentinelle (`"<none>"`), imitable par un IBAN forgé ; changer `DOMAIN_TAG`, qui périmerait les aperçus ouverts pendant la mise à
jour. Réversible : oui (changer la balise de domaine).

**C-e34a1-2 — forme de la migration.** Retenu : une seule instruction `ALTER TABLE` (`MODIFY COLUMN iban … NULL`, deux
`ADD COLUMN` en fin de table, quatre `ADD CONSTRAINT`), puis le bump. `account_kind VARCHAR(20) NOT NULL DEFAULT 'bank'`,
`card_last4 CHAR(4) NULL`. `chk_bank_accounts_iban_nonempty` **gardée telle quelle** : un `CHECK` qui vaut `NULL` est satisfait, elle
refuse toujours un IBAN vide et laisse passer `NULL` (éprouvé à la main sur la base `kesh_e34a1` et par le test
`sans_iban_les_invariants_du_schema`). Autres : la retirer et la reposer en `iban IS NULL OR …` (deux instructions de plus, même
effet) ; colonnes placées après `bank_name` (déplace l'ordre de `SELECT *` et du squash sans gain). Réversible : par migration.

**C-e34a1-3 — erreur de dépôt dédiée au lot, `DbError::PaymentBatchAccountHasNoIban`.** Contexte : le refus doit valoir à la
création du lot (dans la transaction de `create_batch`) et à la génération du `pain.001` (lot forgé). Retenu : une variante de
`DbError` rendue par les deux sites, mappée une fois en `409 PAYMENT_BATCH_ACCOUNT_HAS_NO_IBAN` (`details.bankAccountId`) dans
`AppError::Database`. Le contrôle de `create_batch` se place après « archivé » et « clos », avant « compte du grand livre non lié »,
sur la lecture non verrouillante existante (l'IBAN d'un compte ne devient jamais nul : pas de revérification sous verrou). Autres :
un pré-vol dans la route (la génération resterait à couvrir à part) ; `DbError::IllegalStateTransition` (code générique). Réversible : oui.

**C-e34a1-4 — refus CAMT par une fonction `camt_target_iban`.** Retenu : `fn camt_target_iban(&BankAccount) -> Result<&str, AppError>`,
appelée dans l'aperçu juste après l'aiguillage CSV (sa valeur sert à la sélection et à la relecture héritée), dans `create` juste après
l'aiguillage CSV (avant l'empreinte et le verrou) et relue dans `create_camt_under_lock` (refus déjà rendu par `create`, relu sans
`unwrap`). `details.bankAccountId`. Autre : un `expect` dans `create_camt_under_lock` (interdit par l'AC-3 pour l'esprit). Réversible : oui.

**C-e34a1-5 — l'instantané d'audit du `PUT` extrait en fonction, `full_update_audit_state`.** Contexte : AC-8 et M6 ; aucune route ne
peut encore créer ni modifier un compte sans IBAN, si bien qu'un `iban_present: true` rétabli à un site de route ne rougirait aucun
test d'intégration. Retenu : la paire `before`/`after` du `PUT` passe par une fonction (DRY au passage), testée à l'unité sur une carte
forgée en mémoire. Les deux sites de l'assistant (`onboarding.rs`) et celui de la création (`create_once`) lisent `iban.is_some()`
inline : vrais par construction (l'assistant et le `POST` exigent l'IBAN) ; M6 y est **équivalente** dans la 34-11a1, et redevient
tuable par la création d'une carte (34-11a2). Autre : changer la forme des détails de création pour la partager (contrat de la trace
modifié). Réversible : oui.

**C-e34a1-6 — affichage d'un compte sans IBAN : une forme, trois fonctions de `format.ts`.** Contexte : C-e341-21 veut « une fonction »
pour « •••• 1234 ». Retenu : `cardLast4Label` est cette fonction, **seule** à écrire la forme ; `bankAccountShortId` (listes à IBAN
court : accueil) et `bankAccountFullId` (colonne IBAN du tableau, *Paramètres*, sélecteurs via `bankAccountOptionLabel`) la composent
avec l'IBAN ; `shortIban` est gardée (`null` → `''`). Sans IBAN ni quatre chiffres, le nom seul. Réversible : oui.

**C-e34a1-7 — types de l'écran.** `BankAccountSummary` : `iban: string | null`, `accountKind` et `cardLast4` **obligatoires** (le
serveur les rend toujours ; `parseBankAccount` met `'bank'` et `null` pour une réponse d'un serveur antérieur). `BankAccountJson`
(*Paramètres*) : `iban: string | null`, `accountKind` et `cardLast4` **facultatifs** (type d'une réponse brute, sans parseur).
Payloads de création et de `PUT` : `iban: string` inchangé (C-e341-39). Réversible : oui.

**C-e34a1-8 — l'écran évite les deux refus.** Lots de paiement : le sélecteur du compte source filtre `iban !== null` en plus du compte
lié. Import : pour un compte sans IBAN choisi, le champ fichier n'accepte que le CSV (`accept`) et la zone de dépôt le dit (clé
`bank-import-labels-drop-zone-csv-only`, quatre langues). Un fichier `.xml` glissé quand même reçoit le refus du serveur. Autre :
masquer le compte dans le sélecteur de l'import (le CSV y est légitime). Réversible : oui.

**C-e34a1-9 — P6 : la frontière de `upgrade_path_preserves_data` reste à 34.** `total` 90 → 91 et le soustracteur 56 → 57 du même pas,
selon la règle écrite dans le test ; la fenêtre s'élargit à la migration neuve. Les autres sites positionnels (`P6`) sont tous résolus
par version, avec assertion de montage : inchangés. Réversible : oui.

**C-e34a1-10 — tri des assertions `"0.17.0"` (AC-2).** Gardées, parce qu'elles décrivent l'état d'une base ou d'une sauvegarde 0.17.0 :
`admin_full_import_e2e.rs:4845`, `:4848` (logique de `check_import_version_compat` à versions explicites) et `:5106` (forme d'une
sauvegarde de la rc1). Portées à `"0.17.6"`, parce qu'elles lisent le minimum **courant** d'une base migrée à neuf ou exportée :
`admin_full_import_e2e.rs:4649` (doc), `:4669`, `:4816` ; `migrations_fresh_install.rs:248` ; `migrations_upgrade_path.rs:549`
(binaire aligné, le test passe à `"0.17.6"`), `:565`, `:581`, `:600`, `:683`, `:1122`, `:1622`. Ajouté : `downgrade_protection_refuses_0_17_5`.
Réversible : oui.
