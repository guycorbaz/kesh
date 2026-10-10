# Epic 26 — registre des choix faits en autonomie

Registre de l'Epic 26 (recette de la v0.13.0, cible v0.13.1 ; plan :
`_bmad-output/planning-artifacts/epic-26-recette-v0-13.md`). Même règle que le registre de
l'Epic 15 : chaque entrée est écrite **au moment où le choix est fait**, avec le contexte,
l'option retenue, les options écartées et le coût d'un retour en arrière. Numérotation
`C-26-<story>-k`. Mise à jour **par union**, jamais par écrasement (consigne 7 des agents) :
un doublon de numéro se renumérote, il ne s'écrase pas.

---

## Story 26-1 — Importer le CSV d'UBS tel quel (#620, #621)

Fiche : `26-1-import-csv-ubs-natif.md`. Choix faits à la création de la fiche (2026-10-10).

### C-26-1-1 — Fixture synthétique, contrôle du fichier réel hors dépôt

- **Contexte** : l'epic demande « le fichier d'UBS réel, anonymisé, versionné comme fixture ».
  Le dépôt est public ; le fichier réel porte l'IBAN, des noms, des adresses et les soldes de
  Guy. Anonymiser des centaines de lignes à la main laisse toujours un reste (une référence, un nom dans
  `Description3`), et rien ne le détecte.
- **Retenu** : (a) une fixture **synthétique**, construite de toutes pièces, qui reproduit
  **exactement la structure** du fichier UBS (BOM, préambule de huit lignes clé/valeur, ligne
  vide, titres identiques, débits négatifs, colonnes vides, champs entre guillemets contenant
  des `;`, point-virgule final, dates décroissantes, LF) — contenu fixé ligne à ligne dans la
  fiche ; (b) un test **`#[ignore]`** versionné qui lit le chemin du vrai fichier dans une
  variable d'environnement, vérifie l'invariant *somme des montants = solde final − solde
  initial* et *nombre = « Nombre de transactions »* **en lisant le préambule dans le test**,
  et n'imprime que « conforme » ou le nom de l'assertion en échec. Aucun chiffre personnel n'entre
  dans le dépôt — pas même un agrégat (amendé en validation P2, arbitrage (0) : le Dev Agent
  Record n'écrit que « conforme / non conforme »).
- **Écarté** : le fichier réel anonymisé (risque de fuite résiduelle invisible) ; un contrôle
  manuel non outillé (non rejouable).
- **Réversible** : oui — une fixture plus grande peut s'ajouter sans rien changer au reste.

### C-26-1-2 — Le réglage du sens : `debit_sign`, en mode débit/crédit séparés seulement

- **Contexte** : décision du Project Lead — le sens des montants est un réglage explicite du
  profil, jamais déduit. La fiche devait dire s'il vaut aussi en mode colonne unique.
- **Retenu** : un réglage `debit_sign ∈ {positive, negative}` (« la banque écrit ses débits
  sans signe » / « avec un signe moins »), **défaut `positive` = comportement actuel**, qui ne
  s'applique **qu'en mode débit/crédit séparés**. En mode colonne unique, la validation refuse
  `negative` (le réglage n'y a pas de sens défini) ; le formulaire n'affiche le choix qu'en mode
  séparé et envoie `positive` sinon.
- **Pourquoi pas le mode colonne unique** : la convention « actuelle » n'y est pas la même
  (colonne unique : débits déjà négatifs, signe gardé ; colonnes séparées : débits positifs,
  signe inversé). Un réglage unique « comment la banque écrit ses débits » aurait donc un
  **défaut qui dépend du mode** : il faudrait remplir la colonne par une migration qui écrit
  des données (`UPDATE … JSON_EXTRACT(column_mapping, '$.amount')`), donc un triage P7, un rejeu
  après restauration, et une sauvegarde antérieure restaurée à la valeur par défaut **fausse**
  pour tous les profils à colonne unique — un faux muet. Aucune banque connue n'écrit ses
  débits en positif dans une colonne unique, et personne ne l'a demandé. Ce n'est donc pas
  « simple » au sens de la consigne : écarté.
- **Écarté** : un booléen `invert_sign` valable dans les deux modes (même défaut dépendant du
  mode, et un nom qui décrit l'algorithme et non la banque, contre la décision du PL) ; la
  « valeur absolue, sens donné par la colonne » de la description initiale de #620 (remplacée
  par la décision du PL).
- **Réversible** : oui — étendre au mode colonne unique plus tard est additif (nouvelle valeur
  ou levée du refus), au prix de la migration décrite ci-dessus.

### C-26-1-3 — Colonne dédiée, et non un champ du JSON `column_mapping`

- **Contexte** : `ColumnMapping` est désérialisé avec `deny_unknown_fields`. Un champ ajouté au
  JSON rend le profil **illisible par une 0.13.0** (`parse_column_mapping` → `DbError::Invariant`
  → 500 sur la **liste entière** des profils et sur tout import CSV), et il échappe à toute
  contrainte de base.
- **Retenu** : colonne `debit_sign VARCHAR(10) NOT NULL DEFAULT 'positive'` placée
  `AFTER header_row_count`, avec `CHECK (debit_sign IN ('positive','negative'))`. Une 0.13.0
  la lit sans erreur (requêtes à colonnes explicites) ; la restauration d'une sauvegarde
  antérieure la remplit par son `DEFAULT` (`is_required()` faux), qui est **la sémantique exacte
  des profils d'avant** ; l'export `.keshbackup` (générique, par `information_schema`) la porte
  sans code ; l'export CSV des tables la gagne en une colonne.
- **Écarté** : le champ JSON (illisible par une 0.13.0, non contraint en base, et un défaut
  serde à maintenir des deux côtés).
- **Réversible** : coûteux après publication (une colonne ne se retire pas sans migration
  breaking) ; aucun coût avant.

### C-26-1-4 — Bump `kesh_version_min_required` à 0.13.1, et des dix crates avec

- **Contexte** : la colonne dédiée n'est pas breaking au sens **mécanique** de P3 (ni `DROP
  COLUMN`, ni changement de type). Mais une 0.13.0 qui démarrerait contre une base 0.13.1
  ignorerait `debit_sign = 'negative'` et **inverserait en silence chaque débit** d'un profil
  dont les lignes d'en-tête tiennent dans 5 — exactement le P1 #620, revenu par un retour en
  arrière. C'est le « downgrade silencieux corrupteur » que la politique de migration existe
  pour empêcher (P1 : « ne peut plus consommer **correctement** »).
- **Retenu** : la migration se termine par
  `UPDATE _kesh_version SET kesh_version_min_required = '0.13.1' WHERE id = 1;` et, **dans le
  même commit** (P2-bis), les dix `crates/*/Cargo.toml` passent à `0.13.1` (patron de la
  15-1a-i, `0724904c`). Le CHANGELOG ouvre `## [0.13.1] — Non publié`
  (`scripts/prepare-release.sh` sait finir une release dont les crates sont déjà bumpés,
  #566). Le statement `_kesh_version` est trié en `EXEMPT_MIGRATIONS` (`Durable`, patron des
  bumps précédents ; compteur 17 → 18).
- **Écarté** : `-- breaking-skip-bump` (P4) — la justification serait fausse : le binaire
  antérieur lit la colonne, mais pas correctement ; aucun bump (le retour en arrière rouvrirait
  le P1 en silence).
- **Coût assumé** : une base passée en 0.13.1 refuse ensuite une 0.13.0, et une sauvegarde
  faite en 0.13.1 ne s'importe plus dans une 0.13.0 — à écrire au CHANGELOG. Guy est seul
  utilisateur, base neuve du 2026-10-10.
- **Réversible** : non une fois publié (comme tout bump) ; libre jusqu'au tag.

### C-26-1-5 — Un montant de signe contraire à sa colonne est une erreur de ligne

> ⚠️ **Amendé par C-26-1-15** (validation P1) : l'erreur n'est plus confirmable — elle refuse l'import entier.

- **Contexte** : #620 demande qu'un montant de signe contraire à sa colonne soit signalé plutôt
  que deviné ; le PL a confirmé que ce signalement « reste souhaitable » sous le réglage.
- **Retenu** : nouveau code de ligne `AMOUNT_SIGN_MISMATCH` (clé
  `bank-import-csv-errors-amount-sign-mismatch`), levé en mode séparé quand : la colonne Débit
  porte une valeur **négative** sous `positive`, ou **positive** sous `negative` ; la colonne
  Crédit porte une valeur **négative** (quel que soit le réglage). Zéro est accepté partout
  (comportement existant `explicit_zero_debit_or_credit_accepted`). L'erreur suit le chemin des
  autres erreurs de ligne (`PartialFailure`, import partiel sur confirmation).
- **Effet sur les profils existants** : un débit négatif sous le défaut, aujourd'hui inversé en
  silence en encaissement, devient une erreur de ligne. C'est voulu : c'est précisément le
  défaut #620 qui devient visible pour qui n'a pas réglé son profil.
- **Écarté** : un simple avertissement (l'import passerait faux) ; accepter les deux signes en
  colonne Débit (retour à la déduction, contre la décision du PL).
- **Réversible** : oui.

### C-26-1-6 — Plafond des lignes à ignorer porté de 5 à 20

- **Retenu** : `header_row_count ≤ 20` dans `CsvProfile::validate`, dans la contrainte
  `chk_bank_profiles_header_row_count` (supprimée puis recréée par la migration), dans le
  formulaire (`max="20"`) et dans les libellés des quatre locales. UBS demande 9 ; 20 laisse de
  la marge pour d'autres préambules sans faire du réglage un saut arbitraire de fichier.
  `TINYINT UNSIGNED` suffit.
- **Écarté** : 255 (aucune banque connue n'en approche, et un plafond large aggrave l'erreur
  de comptage décrite en C-26-1-8) ; supprimer le plafond.
- **Réversible** : oui (relever encore la borne est une migration de contrainte).

### C-26-1-7 — Les lignes vides ne comptent pas

> ⚠️ **Amendé par C-26-1-16** (validation P1) : les numéros de ligne ne sont pas physiques pour un enregistrement
> qui suit une ligne vide.

- **Contexte** : le lecteur `csv` (csv-core 0.1.13, `reader.rs` l. 94) ignore les lignes **vides**
  (sans espace) avant de produire un enregistrement ; le saut d'en-tête compte des
  **enregistrements**. Les numéros de ligne rapportés (`Position::line`) comptent, eux, chaque
  `\n` : ils sont **physiques**, lignes vides comprises.
- **Retenu** : le réglage compte les **lignes non vides avant les données, ligne de titres
  comprise** — c'est le comportement actuel, documenté au lieu d'être changé. Fichier UBS :
  **9** (huit lignes d'informations + les titres ; la ligne vide ne compte pas). Le libellé, le
  texte d'aide et le manuel le disent. Les numéros de ligne des erreurs restent physiques (ceux
  d'un éditeur de texte), et un test le fixe sur la fixture.
- **Écarté** : compter les lignes physiques (il faudrait lire le fichier deux fois ou
  renoncer au saut par le lecteur, et la ligne vide d'UBS serait comptée par les uns et pas par
  les autres) ; détecter la ligne de titres (écarté par le PL).
- **Réversible** : changer la sémantique plus tard changerait le sens des profils enregistrés —
  coûteux.

### C-26-1-8 — Garde contre le surcomptage : une ligne ignorée qui porte une date est refusée

- **Contexte** : compter **trop peu** de lignes est bruyant (la ligne de titres arrive en
  données et rend une erreur `INVALID_DATE`). Compter **trop** est muet : la ou les premières
  transactions disparaissent sans un mot. Avec un préambule de neuf lignes et une ligne vide
  que l'on peut compter ou non (C-26-1-7), l'erreur d'une unité est probable.
- **Retenu** : quand `header_row_count ≥ 1`, si la **dernière ligne ignorée** porte, à l'index
  `column_mapping.date`, une valeur que `date_format` lit comme une date, le parseur refuse
  l'import : nouvelle variante `CsvError::HeaderSkipsData { line }` → `422
  BANK_CSV_HEADER_SKIPS_DATA`, message traduit qui donne le numéro de ligne physique. Ce
  n'est **pas** une détection de la ligne de titres : le nombre reste celui du profil, la garde
  ne fait que refuser un nombre manifestement trop grand.
- **Écarté** : rien faire (perte muette) ; avertir sans refuser (l'import passe amputé).
- **Réversible** : oui.

### C-26-1-9 — `debitSign` obligatoire dans la requête API

- **Contexte** : `PUT /api/v1/bank-profiles/{id}` est un remplacement complet, et la requête est
  en `deny_unknown_fields`.
- **Retenu** : `debitSign` est **requis** (pas de `#[serde(default)]`). Un défaut ferait qu'un
  client qui l'omet remette `positive` en silence sur un profil UBS — le P1 rouvert par un
  `PUT`. Les charges utiles de test existantes (`bank_profiles_e2e.rs`, Playwright) sont
  complétées.
- **Écarté** : champ facultatif à défaut `positive`.
- **Réversible** : oui (rendre facultatif plus tard est compatible).

### C-26-1-10 — Pas de profil UBS prêt à l'emploi

- **Contexte** : #621 propose un profil UBS proposé à la création.
- **Retenu** : hors périmètre — le PL a décidé que la v0.13.1 « ne livre que les deux
  réglages ». Le manuel donne en revanche les valeurs exactes d'un profil UBS (tableau), ce qui
  en tient lieu.
- **Réversible** : oui (story ultérieure, avec la lecture du préambule).

### C-26-1-11 — Pas de découpage

- **Retenu** : une seule story. Cinq modules de premier niveau : `kesh-import`, `kesh-db`,
  `kesh-api`, `kesh-i18n`, `frontend` — à la limite de la règle de splitting préventif, sans la
  dépasser ; s'y ajoutent la documentation (manuel FR, CHANGELOG, audit d'idempotence) et le
  bump mécanique des dix `Cargo.toml` (P2-bis), qui ne portent pas de logique. Les deux réglages
  partagent la migration, le DTO, le formulaire et la fixture : les séparer créerait deux
  migrations et deux bumps pour un même fichier UBS, que ni l'un ni l'autre n'importe seul.
- **Découpage de repli** si la validation lève le signal D5 sur un recyclage : 26-1a (sens des
  montants + garde de signe, C-26-1-2 à 5, 9 ; `closes #620`) puis 26-1b (plafond, comptage,
  garde de surcomptage, C-26-1-6 à 8 ; `refs #621`), la fixture et l'E2E UBS dans la seconde.

### C-26-1-12 — Une seule fixture pour les trois niveaux de tests

- **Retenu** : `crates/kesh-import/tests/fixtures/csv/ubs_synthetique.csv`, lue par les tests
  du parseur (`include_bytes!` sous `CARGO_MANIFEST_DIR`), par le test d'intégration de route
  (`../kesh-import/tests/fixtures/csv/…`) et par la spec Playwright (chemin relatif depuis
  `frontend/tests/e2e/`). Pas de copie (DRY) ; la provenance s'écrit au `README.md` des
  fixtures.
- **Écarté** : une copie sous `frontend/tests/e2e/fixtures/` (deux fichiers qui dériveraient).
- **Réversible** : oui.

### C-26-1-13 — Mapping UBS recommandé : date de comptabilisation

- **Retenu** : au manuel et dans les tests, `date = 2` (« Date de comptabilisation »),
  `debit_credit_split = (5, 6)`, `reference = 9` (« No de transaction »), `details = 10`
  (« Description1 »), séparateur `;`, décimale `.`, format `%Y-%m-%d`, lignes à ignorer 9,
  débits « avec un signe moins ». La date de comptabilisation est celle que porte l'écriture ;
  la date de transaction (colonne 0) est celle du paiement par carte et peut précéder la
  comptabilisation de plusieurs jours.
- **Réversible** : oui (ce n'est qu'une recommandation de saisie).

### Validation P1 de la 26-1 (2026-10-10) — choix de la remédiation

Rapports : `/home/gcorbaz/devel/kesh-gate-logs/26-1-validate-p1-{R,F}.md` (Sonnet). Arbitrages (1) à (4) de
l'orchestrateur repris tels quels ; les autres tranchés à la remédiation.

### C-26-1-14 — Le code d'erreur de ligne sort sous sa graphie canonique, et l'écran en affiche le message

- **Contexte** (F1 HIGH, R-2) : `csv_line_error_to_payload` émet `format!("{:?}").to_uppercase()` (`INVALIDDATE`) ;
  le panneau n'affiche que ce code, `messageI18nKey` n'est lu nulle part — le signalement de #620 serait illisible.
- **Retenu** (arbitrage de l'orchestrateur, forme tranchée ici) : une méthode `CsvLineErrorCode::as_str()` à `match`
  exhaustif, testée égale à la sérialisation serde pour chaque variant ; plutôt que `serde_json::to_value` au site
  d'appel, parce qu'une `&'static str` ne peut pas échouer et qu'un variant neuf sans graphie ne compile pas. Les
  panneaux affichent `i18nMsg(messageI18nKey, code)`. Changement de graphie des cinq codes existants au CHANGELOG
  (`### Modifié`) ; `docs/api-external.md` ne les documente pas.
- **Écarté** : garder le `Debug` et documenter `AMOUNTSIGNMISMATCH` (code illisible, divergent du serde déjà déclaré).
- **Réversible** : non une fois publié (graphie d'API) ; aucun consommateur connu aujourd'hui.

### C-26-1-15 — `AMOUNT_SIGN_MISMATCH` refuse l'import entier, sans confirmation possible

- **Contexte** (F4) : en erreur de ligne confirmable, un fichier UBS sous le mauvais réglage s'importerait par la case
  « lignes valides quand même » réduit à ses crédits — dépenses absentes, solde faux, en silence : #620 par une autre
  porte.
- **Retenu** (arbitrage de l'orchestrateur) : la décision vit dans `kesh-import` — `parse_csv_collect` collecte, puis
  rend `HardFailure(CsvError::AmountSignMismatch { errors, total_errors, truncated })` dès qu'une ligne porte ce code ;
  `kesh-api` → `422 BANK_CSV_AMOUNT_SIGN_MISMATCH`, message qui dit de régler le sens des débits, preview et import,
  `confirmPartialImport` sans effet ; l'écran n'offre pas la case.
- **Écarté** : refus dans la route seulement (un autre appelant de `kesh-import` hériterait de l'erreur confirmable) ;
  avertissement « majorité des lignes » (seuil arbitraire, une seule ligne inversée est déjà une erreur comptable).
- **Réversible** : oui.

### C-26-1-16 — Numéros de ligne : la sémantique du lecteur, documentée et fixée par test

- **Contexte** (F3, R-1) : `csv` 1.4.0 pose la position avant la lecture ; un enregistrement précédé d'une ligne vide
  est rapporté au numéro de cette ligne vide (titres UBS : 9, non 10).
- **Retenu** : ne pas recalculer les positions (il faudrait relire le fichier hors du lecteur) ; documenter (doc-comment,
  manuel) et fixer par test (AC14-4 attend 9). Les transactions UBS, non précédées d'une ligne vide, portent leur
  numéro exact.
- **Écarté** : un recalcul maison des numéros (complexité, risque de divergence avec le lecteur).
- **Réversible** : oui.

### C-26-1-17 — Angle mort de la garde de surcomptage, écrit

- **Contexte** (R-12) : la garde ne voit qu'une ligne avalée dont la date se lit.
- **Retenu** : l'écrire (doc-comment de `HeaderSkipsData`, manuel) et le fixer par un test qui documente sans promettre.
  Sur un relevé, une transaction sans date est rare.
- **Écarté** : examiner toutes les lignes ignorées (ce serait une détection de la ligne de titres, écartée par le PL).

### C-26-1-18 — Horodatages de migration attribués

- **Retenu** (arbitrage de l'orchestrateur, F5) : 26-1 `20261011000001`, 26-2 `20261011000002` ; la 26-1 fusionne en
  premier, la 26-2 recompte après rebase (compteurs 80/46, audit, sha384).

### C-26-1-19 — Aucun `;` ni `--` dans un littéral de la migration

- **Retenu** (R-7) : le `COMMENT` de la colonne s'écrit avec des points ; le découpeur de `post_restore.rs` ne gère pas
  les apostrophes.

### C-26-1-20 — Le test de migration d'AC9 vit dans `migrations_upgrade_path.rs`

- **Retenu** (F7) : fichier déjà inscrit à `ALLOWED_REAL_MIGRATOR_FILES` ; un fichier neuf aurait élargi la liste
  gardée par `test_schema_guard`.

### C-26-1-21 — Les statuts réels de la route des profils, décrits sans être changés

- **Contexte** (F2) : la fiche annonçait des 400 et un code `BANK_CSV_PROFILE_VALIDATION` qui n'existe pas.
- **Retenu** : `validate` → `422 BANK_CSV_PROFILE_INVALID` ; désérialisation (`debitSign` absent ou hors domaine) →
  `422` axum, corps texte brut — comportement actuel de toutes les erreurs de désérialisation de cette route, décrit
  tel quel. Pas d'extracteur à rejet géré : `impl_validation_extractor!` est local à `bank_accounts.rs`, le partager
  toucherait d'autres routes ; le formulaire envoie toujours le champ. Lecture de la ligne d'en-têtes par
  `headers()?.position()` (F12) consignée au même passage.
- **Écarté** : un `JsonRejection` propre aux profils (élargissement pour un cas que seul un client d'API atteint).
- **Réversible** : oui (story d'hygiène des rejets de désérialisation, toutes routes).

### C-26-1-22 — Deux scénarios E2E indépendants

- **Retenu** (R-5, F8) : noms et motifs de nom de fichier distincts ; la même fixture téléversée sous un autre nom par
  `setInputFiles({ name, mimeType, buffer })` — aucune copie versionnée, aucune dépendance d'ordre.
