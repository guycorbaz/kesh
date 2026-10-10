# Epic 26 — registre des choix faits en autonomie

Registre des choix tranchés sans question au Project Lead pendant l'Epic 26 (recette de la
v0.13.0, cible v0.13.1), **pour sa revue en fin d'epic**. Même règle que le registre de
l'Epic 15 : chaque entrée est écrite **au moment où le choix est fait** et dit le contexte,
l'option retenue, les options écartées et ce que coûterait de revenir dessus.

Numérotation `C-<story>-k`. Plusieurs stories de l'epic avancent en parallèle sur des
branches distinctes : ce fichier **se fusionne par union**, jamais par écrasement ; un
numéro en double se renumérote, il ne s'écrase pas.

---

## Story 26-2 — description d'un compte bancaire (#622)

### C-26-2-1 — Découpage proposé, fiche unique

- **Contexte** : l'inventaire touche bien plus de cinq modules (kesh-db, `kesh-api/routes/bank_accounts`,
  `routes/companies`, `errors`, `exports`, kesh-i18n, et sept dossiers du frontend). La § *Règle de
  splitting préventif* du `CLAUDE.md` impose alors le découpage avant la spec.
- **Retenu** : **proposer** le découpage 26-2a (socle : colonne, dépôt, API, export, fonction de libellé,
  saisie, écran *Comptes bancaires*, i18n) / 26-2b (rollout mécanique sur les autres écrans, message serveur,
  garde d'inventaire, manuel, CHANGELOG — porte `closes #622`), et écrire **une seule fiche** dont les AC
  sont déjà partagées entre les deux parties. La clé `26-2-description-compte-bancaire` reste seule au
  registre de sprint, en `ready-for-dev`, jusqu'à la décision de l'orchestrateur (qui seul crée des
  clés et des branches).
- **Écarté** : écrire deux fiches d'office (l'orchestrateur avait demandé une proposition, et le
  découpage change les branches et les PR, qui lui reviennent) ; ne pas découper (la règle l'impose,
  et aucune dérogation ne se justifie : 26-2a se teste seule).
- **Réversibilité** : totale — la fiche se scinde en recopiant les AC de chaque partie.

### C-26-2-2 — Facultative, et absente de l'onboarding

- **Retenu** : la description est facultative (décision du Project Lead, 2026-10-10, commentaire de #622).
  Elle se saisit **seulement** dans *Administration → Comptes bancaires*, à la création et à la
  modification. L'étape « Compte bancaire » de l'onboarding n'en reçoit pas : `upsert_primary` et
  `POST /onboarding/bank-account` restent inchangés et **ne touchent pas** la colonne.
- **Écarté** : un champ à l'onboarding — il élargit la story à `routes/onboarding`, au dépôt d'upsert
  (court-circuit KF-004, audit 15-7a2) et au premier écran d'un nouvel utilisateur, pour un premier compte
  qui n'a encore rien à distinguer.
- **Réversibilité** : ajout ultérieur possible sans migration.

### C-26-2-3 — 60 caractères, espaces retirés, vide = absente, caractères de contrôle refusés

- **Retenu** : colonne `VARCHAR(60) NULL`. Le serveur retire les espaces de tête et de queue ; une valeur
  vide après ce retrait est enregistrée `NULL` ; plus de 60 caractères (comptés en caractères Unicode,
  `chars().count()`, comme `VARCHAR` en `utf8mb4`) ou un caractère de contrôle (`char::is_control`, dont
  le saut de ligne) → `400` de validation. Pas de contrainte `CHECK` en base.
- **Écarté** : 255 comme `bank_name` (la demande dit « courte » ; une description longue casse les
  `<select>`) ; 30 (trop court pour « Compte épargne logement ») ; un `CHECK` (la normalisation vit
  dans `validate_bank_input`, comme celle de `bank_name` ; un `CHECK` interdirait `ALGORITHM=INSTANT`).
- **Réversibilité** : élargir est une migration `MODIFY COLUMN` vers un type plus large. *(Rectifié en validation P1,
  R8 : `CLAUDE.md` P3 range tout `MODIFY COLUMN <type>` parmi les opérations breaking — un élargissement exige donc
  le bump `min_required` **ou** un `-- breaking-skip-bump` justifié et confirmé en revue (P4) ; « non breaking » était
  une affirmation, pas la règle.)*

### C-26-2-4 — Pas d'unicité

- **Retenu** : deux comptes peuvent porter la même description, et la même paire banque + description.
  Les sélecteurs gardent l'IBAN (C-26-2-5) : deux homonymes restent distinguables.
- **Écarté** : une unicité par société — elle refuserait un cas légitime (deux comptes « Épargne » dans
  deux banques) et ajouterait un index et un code d'erreur pour rien.

### C-26-2-5 — Forme du libellé, et l'unique changement visible sans description

- **Retenu** : deux fonctions, et elles seules :
  - **nom du compte** : `UBS — Personnel` avec description, `UBS` sans — pour les listes qui montrent
    l'IBAN à part (écran *Comptes bancaires*, *Paramètres*, tuile de l'accueil) et pour le message
    serveur du refus de désignation ;
  - **libellé d'option** : nom du compte, `" — "`, IBAN complet — `UBS — Personnel — CH93…` avec,
    `UBS — CH93…` sans — pour tous les `<select>` et l'en-tête du formulaire de lien au plan comptable.
- **Constat** : l'affichage d'aujourd'hui est **la banque et l'IBAN complet** dans les sélecteurs, la
  banque seule dans les listes (l'IBAN complet ou raccourci en dessous) — et non « la fin de l'IBAN »
  comme le dit le commentaire de #622. Sans description, ces affichages sont conservés au caractère près,
  **à une exception** : le sélecteur de l'import bancaire écrivait `UBS (CH93…)`, seul de son espèce ; il
  passe à `UBS — CH93…` pour que tous les sélecteurs sortent de la même fonction.
- **Écarté** : garder la parenthèse (une variante de la fonction pour un seul site, ce que la règle DRY
  du `CLAUDE.md` interdit) ; retirer l'IBAN des sélecteurs quand une description existe (deux homonymes
  redeviendraient indiscernables, C-26-2-4).
- **Réversibilité** : une ligne dans chaque fonction.

### C-26-2-6 — Place de la colonne *(complété par C-26-2-16)*

- **Retenu** : `AFTER bank_name` en base ; colonne `description` juste après `bank_name` dans
  `bank_accounts.csv` de l'export global.
- **Écarté** : en fin de table (l'export se lit par en-tête, et la lecture humaine du CSV gagne à voir la
  description à côté de la banque).

### C-26-2-7 — `NewBankAccount` inchangé

- **Retenu** : la description ne rejoint **pas** `NewBankAccount` ; elle est passée à part au `POST` et à
  `update_for_company`, comme `journal_account_id` l'est déjà. `upsert_primary_in_tx` ne la nomme ni à
  l'`INSERT` ni à l'`UPDATE`, et son court-circuit KF-004 (`is_no_op_change`) ne la compare pas.
- **Écarté** : un champ de `NewBankAccount` — une soixantaine de littéraux de struct dans 25 fichiers de
  tests à retoucher, et l'upsert de l'onboarding forcé de choisir entre écraser et relire.

### C-26-2-8 — `PUT` en remplacement complet

- **Retenu** : `PUT /bank-accounts/{id}` sans `description` l'efface, comme il efface `qrIban` : la route
  est une édition complète depuis la v014-1. `PATCH` (lien au plan comptable) et `DELETE` (archivage) la
  conservent. Le frontend envoie toujours le champ.
- **Écarté** : `Option<Option<String>>` pour distinguer « absent » de « null » — une sémantique de plus,
  pour un seul champ, sur une route qui n'en a jamais eu.

### C-26-2-9 — Le refus de désignation nomme le compte par son libellé

- **Retenu** : `CLAIM_ACCOUNT_LINKED_TO_BANK_ACCOUNT` (« Le compte 1020 est lié au compte bancaire … »)
  rend le **nom du compte** (C-26-2-5) dans son message et dans la variable `$bank` des quatre locales ;
  `first_active_bank_account_linked_to` lit `description` et rend ce nom ; le champ `bank_name` de
  `DbError::ClaimAccountLinkedToBankAccount` devient `bank_label`. La clé JSON `details.bankName` est
  **conservée** (contrat de l'API) et porte ce nom.
- **Écarté** : laisser la banque seule (deux comptes UBS rendraient un message ambigu — c'est le défaut
  même que #622 ferme) ; ajouter une clé `details.bankDescription` (aucun client ne la lit).

### C-26-2-10 — La description en clair dans le journal d'audit

- **Retenu** : `bank_account.created` porte `description` ; `bank_account.updated` (`trigger:
  full_update`) la porte dans `before` et `after`. Une modification de la seule description est donc
  tracée et lisible.
- **Écarté** : un booléen `description_present` à la manière de `iban_present` — l'IBAN est masqué parce
  qu'il est une donnée bancaire ; une description choisie par l'utilisateur ne l'est pas, et un booléen
  ne dirait pas ce qui a changé.

### C-26-2-11 — Messages de validation serveur en français, comme leurs voisins

- **Retenu** : les deux refus neufs (trop longue, caractère de contrôle) suivent le patron de
  `validate_bank_input` — `AppError::Validation` en dur, en français. Le champ du formulaire porte
  `maxlength="60"` : le refus de longueur n'est pas atteignable depuis l'interface.
- **Écarté** : les traduire seuls, quand les refus voisins (banque vide, IBAN invalide) ne le sont pas —
  la traduction de `validate_bank_input` est un sujet entier, hors de cette story.

### C-26-2-12 — Angles morts assumés : trois écrans n'affichent aucun compte

- **Retenu** : la liste des imports (`BankImportList.svelte`), le détail d'un import
  (`bank-import/[id]`) et le détail d'un lot de paiement (`payment-batches/[id]`) **n'affichent pas** le
  compte bancaire concerné. Ils ne sont pas des sites « qui affichent un compte sans sa description » :
  la story ne leur ajoute rien. Avec deux comptes UBS, savoir à quel compte appartient un import y reste
  impossible ; c'est un manque d'avant cette story, que l'orchestrateur peut ouvrir en issue (CR, P3).
- **Écarté** : y ajouter le compte — une colonne, une jointure côté serveur ou une résolution côté
  client par écran : trois petites fonctionnalités, hors de #622.

### C-26-2-13 — Le manuel : une section neuve avant « Import bancaire »

- **Retenu** : `\section{Comptes bancaires}\label{sec:comptes-bancaires}`, insérée juste avant
  `\section{Import bancaire}` du manuel utilisateur — le manuel n'a aujourd'hui **aucune** section sur
  *Administration → Comptes bancaires*. La 26-3 (#616) réécrit deux lignes de la section suivante
  (CAMT « fourni par toutes les banques », « marqué Compte de banque ») : hunks distincts ; celle des deux
  stories qui fusionne la seconde régénère le PDF et peut renvoyer à `\S\ref{sec:comptes-bancaires}`.
- **Écarté** : une sous-section de l'onboarding (la description ne s'y saisit pas, C-26-2-2).

### C-26-2-14 — Migration neuve, et la course avec la 26-1 *(remplacé par C-26-2-25)*

- **Retenu** : `crates/kesh-db/migrations/20261011000001_bank_accounts_description.sql`, version à
  confirmer au développement contre `origin/main` : la 26-1 (#620, #621) peut elle aussi ajouter une
  migration, régénérer le squash de test, et porter les compteurs de 78 à 79. **Celle des deux qui
  fusionne la seconde** rebase, choisit une version strictement postérieure si besoin (P8 n'interdit rien
  tant que rien n'est publié — reconstruire alors les bases de dev), régénère le squash, recompte la
  ligne d'audit P5 et les cinq compteurs, et reporte le `total`/`N` de `migrations_upgrade_path.rs`.
- **Écarté** : une version lointaine pour éviter la collision (elle ne dispense d'aucun recompte).

## C-26-2-15 — Découpage matérialisé (orchestrateur, 2026-10-10)

**Contexte** : la fiche dépasse cinq modules (§ Règle de splitting préventif) ; C-26-2-1 proposait le découpage sans le
matérialiser. **Option retenue** : deux clés au registre (`26-2a-description-socle`, `26-2b-description-ecrans`), une
fiche unique en deux parties, deux PR successives (`refs #622` puis `closes #622`). **Alternatives** : deux fiches
séparées (précédent 15-1c) — écartée, la fiche est déjà partagée et l'inventaire clos doit rester d'un seul tenant.
**Réversibilité** : totale (registre). Le changement de libellé du menu d'import sans description (C-26-2-5, « UBS (CH…) »
→ « UBS — CH… ») est accepté : il aligne ce menu sur les autres.

## Story 26-2 — validation P1 remédiée (2026-10-10)

### C-26-2-16 — La colonne en fin de table

- **Contexte** : F6 (P1) — `AFTER bank_name` + `ALGORITHM=INSTANT`. Le précédent existe
  (`20260507200001_bank_account_journal_link.sql`, `AFTER qr_iban`), mais la position n'apporte rien.
- **Retenu** : `ADD COLUMN description VARCHAR(60) NULL, ALGORITHM=INSTANT, LOCK=NONE` en fin de table ; dans le
  CSV, la colonne reste juste après `bank_name` (C-26-2-6), c'est le sérialiseur qui fixe l'ordre.
- **Écarté** : garder `AFTER` (rien à gagner, un mode d'échec de plus à surveiller).
- **Réversibilité** : sans objet (position physique invisible).

### C-26-2-17 — L'ensemble des caractères refusés, fermé et nommé

- **Retenu** : `char::is_control` (Cc) plus U+2028, U+2029, U+200B à U+200F, U+202A à U+202E, U+2060 à U+2064,
  U+2066 à U+2069, U+FEFF — une fonction `is_forbidden_in_description`, testée par plage. Pas de normalisation
  NFC : un accent décomposé compte pour deux caractères (angle mort assumé ; les navigateurs saisissent en
  précomposé).
- **Écarté** : une dépendance de catégories Unicode (toute la catégorie Cf) — une crate pour un champ ; NFC à
  l'enregistrement — modifierait silencieusement la saisie.

### C-26-2-18 — La garde d'inventaire compte un jeton par fichier

- **Contexte** : R3/F1 (P1) — la garde « expressions de gabarit » était ambiguë et sa liste d'exclusions citait des
  lignes de script.
- **Retenu** : décompte du jeton `bankName` dans **tout** fichier `.svelte` et `.ts` non-test de `frontend/src`,
  contre une table fermée `{fichier → nombre, motif}` sans numéro de ligne ; cas synthétiques par forme ; types en
  ligne remplacés par `BankAccountOptionSource & { id: number }` exporté de `format.ts` (tranché, plus « au choix »).
  Angle mort nommé : un libellé construit sans le jeton.
- **Écarté** : une analyse syntaxique Svelte (lourde, fragile aux versions) ; une liste par ligne (se périme à
  chaque retouche).

### C-26-2-19 — Le changement de contrat du `PUT` est figé et documenté

- **Contexte** : R4/F2 (P1) — `PUT /bank-accounts/{id}` est ouvert aux clés d'API ; un corps 0.13.0 efface la
  description.
- **Retenu** : C-26-2-8 maintenu (remplacement complet), mais figé par un test et la mutation M13, annoncé au
  CHANGELOG `### Modifié` (« Changement de contrat pour une intégration par clé d'API ») et dans
  `docs/api-external.md` (« Depuis la v0.13.1 »).
- **Écarté** : conserver une description absente (`COALESCE`) — rendrait impossible l'effacement par un `PUT`
  qui omet le champ et romprait la symétrie avec `qrIban` ; c'est précisément M13.

### C-26-2-20 — La liste des colonnes est une macro du dépôt

- **Retenu** : `macro_rules! columns` privé à `repositories/bank_accounts.rs`, `concat!` à chaque `SELECT`, sur
  le précédent de `repositories/accounts.rs:21-42` ; la route de `kesh-api` relit par `find_by_id_in_tx` (fonction
  publique neuve). Contrôles : `grep -rnE "SELECT id, company_id, bank_name" crates/*/src` vide ;
  `grep -rnF "id, company_id, bank_name, iban" crates/*/src` → la seule ligne de la macro.
- **Écarté** : `const &str` + `format!` à chaque appel (une allocation par requête, ce que le code review 14-3a a
  retiré d'`accounts.rs`) ; `#[macro_export]` (exporterait la liste hors de `kesh-db`).

### C-26-2-21 — La borne 60 a une source par langage, et le squash la vérifie

- **Retenu** : `BANK_ACCOUNT_DESCRIPTION_MAX_CHARS` (Rust, entité) et `BANK_ACCOUNT_DESCRIPTION_MAX` (TS,
  `format.ts`) ; un test de `kesh-db` confronte la constante au `varchar(…)` du squash. Le nombre reste en toutes
  lettres dans la migration, les FTL, le manuel, le CHANGELOG et `api-external.md` — greper `\b60\b`.
- **Écarté** : générer la constante TS depuis Rust (outillage neuf pour un entier).

### C-26-2-22 — La description survit à un changement de banque à l'onboarding

- **Contexte** : F11 (P1) — `upsert_primary_in_tx` réécrit banque, IBAN et QR-IBAN sans toucher la description.
- **Retenu** : assumé (conséquence de C-26-2-2 : l'onboarding ne connaît pas la description) ; dit au manuel. Le
  cas suppose de refaire l'étape 6 de l'onboarding, rare.
- **Écarté** : effacer la description dans l'upsert — perdrait une saisie sur une simple correction d'IBAN.

### C-26-2-23 — `details.bankName` porte le nom complet, et le CHANGELOG le dit

- **Retenu** : la clé JSON reste `bankName` (C-26-2-9) ; sa valeur devient « UBS — Personnel » ; annoncé au
  CHANGELOG `### Modifié`. Aucun lecteur dans le dépôt ; route fermée aux clés d'API.

### C-26-2-24 — Une sauvegarde 0.13.1 refusée par une 0.13.0 : attendu

- **Contexte** : F7 (P1) — `check_schema_compat` refuse une colonne source inconnue.
- **Retenu** : écrit en AC1 comme comportement attendu ; il l'est doublement, la 26-1 portant
  `kesh_version_min_required` à `0.13.1`. Le retour arrière passe par la sauvegarde de la base prise avant la mise
  à jour (consigne déjà au CHANGELOG 0.13.0).

### C-26-2-25 — Version de migration `20261011000002`, après la 26-1 (arbitrage de l'orchestrateur)

- **Contexte** : la 26-1 réservait aussi `20261011000001` — collision sqlx.
- **Retenu** (orchestrateur, 2026-10-10) : la 26-1 garde `20261011000001` et fusionne la première ; la 26-2a prend
  `20261011000002` et se développe (ou se rebase avant son gate final) sur `origin/main` contenant la 26-1. Elle
  recompte alors les cinq compteurs P5 (80 / 80 / 10 / 70 / 0 attendus, la commande fait foi), porte
  `migrations_upgrade_path.rs` à `total` 80 / soustracteur 46 (frontière 34) avec sa ligne de généalogie,
  régénère le squash, et **ajoute** sa ligne à `crates/kesh-db/migrations.sha384` (registre des checksums, garde
  P8 outillé par `published_migrations_keep_their_checksums`) — fichier que la fiche ne nommait pas avant cette
  remédiation. Remplace C-26-2-14.
