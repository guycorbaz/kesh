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
  (comportement existant `explicit_zero_debit_or_credit_accepted`). Une seule erreur de ce code
  refuse **tout** le fichier, sans import partiel possible (C-26-1-15, *corps réécrit en P3*).
- **Effet sur les profils existants** : un débit négatif sous le défaut, aujourd'hui inversé en
  silence en encaissement, refuse désormais l'import. C'est voulu : c'est précisément le
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
  `\n` — mais la position est posée **avant** la lecture (cf. C-26-1-16).
- **Retenu** : le réglage compte les **lignes non vides avant les données, ligne de titres
  comprise** — c'est le comportement actuel, documenté au lieu d'être changé. Fichier UBS :
  **9** (huit lignes d'informations + les titres ; la ligne vide ne compte pas). Le libellé, le
  texte d'aide et le manuel le disent. Les numéros de ligne des erreurs : sémantique du lecteur,
  fixée par test et documentée (C-26-1-16 — un enregistrement qui suit une ligne vide est rapporté
  au numéro de cette ligne vide).
- **Écarté** : compter les lignes physiques (il faudrait lire le fichier deux fois ou
  renoncer au saut par le lecteur, et la ligne vide d'UBS serait comptée par les uns et pas par
  les autres) ; détecter la ligne de titres (écarté par le PL).
- **Réversible** : changer la sémantique plus tard changerait le sens des profils enregistrés —
  coûteux.

### C-26-1-8 — Garde contre le surcomptage : une ligne ignorée qui porte une date est refusée

> ⚠️ **Amendé par C-26-1-16** (validation P2, R2-6/F2-8) : le numéro donné par `HeaderSkipsData { line }` suit la
> sémantique du lecteur, non le numéro physique ; et par **C-26-1-17** : angle mort d'une ligne avalée non datée.

- **Contexte** : compter **trop peu** de lignes est bruyant (la ligne de titres arrive en
  données et rend une erreur `INVALID_DATE`). Compter **trop** est muet : la ou les premières
  transactions disparaissent sans un mot. Avec un préambule de neuf lignes et une ligne vide
  que l'on peut compter ou non (C-26-1-7), l'erreur d'une unité est probable.
- **Retenu** : quand `header_row_count ≥ 1`, si la **dernière ligne ignorée** porte, à l'index
  `column_mapping.date`, une valeur que `date_format` lit comme une date, le parseur refuse
  l'import : nouvelle variante `CsvError::HeaderSkipsData { line }` → `422
  BANK_CSV_HEADER_SKIPS_DATA`, message traduit qui donne le numéro de la ligne (sémantique C-26-1-16). Ce
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

> ⚠️ **Amendé en validation P2** (F2-2, arbitrage (5)) : l'écran ne lit pas `messageI18nKey` (site non résolu pour
> `i18n-keys.test.ts`) mais une table TS littérale code → clé, `CSV_LINE_ERROR_LABELS`, famille déclarée dans
> `MOTIFS_DYNAMIQUES`.

- **Contexte** (F1 HIGH, R-2) : `csv_line_error_to_payload` émet `format!("{:?}").to_uppercase()` (`INVALIDDATE`) ;
  le panneau n'affiche que ce code, `messageI18nKey` n'est lu nulle part — le signalement de #620 serait illisible.
- **Retenu** (arbitrage de l'orchestrateur, forme tranchée ici) : une méthode `CsvLineErrorCode::as_str()` à `match`
  exhaustif, testée égale à la sérialisation serde pour chaque variant ; plutôt que `serde_json::to_value` au site
  d'appel, parce qu'une `&'static str` ne peut pas échouer et qu'un variant neuf sans graphie ne compile pas. Les
  panneaux affichent le message traduit par une **table TS littérale** `CSV_LINE_ERROR_LABELS` (code → clé), famille
  déclarée dans `MOTIFS_DYNAMIQUES` et site de gabarit dans `SITES_GABARIT_ATTENDUS` — jamais par
  `messageI18nKey` lu à l'exécution, site non résolu pour `i18n-keys.test.ts` (*corps réécrit en P3*). Changement de graphie des cinq codes existants au CHANGELOG
  (`### Modifié`) ; `docs/api-external.md` ne les documente pas.
- **Écarté** : garder le `Debug` et documenter `AMOUNTSIGNMISMATCH` (code illisible, divergent du serde déjà déclaré).
- **Réversible** : non une fois publié (graphie d'API) ; aucun consommateur connu aujourd'hui.

### C-26-1-15 — `AMOUNT_SIGN_MISMATCH` refuse l'import entier, sans confirmation possible

> ⚠️ **Précisé en validation P2** (R2-1 = F2-1, arbitrage (1)) : la décision porte sur un compteur **propre au signe**
> (`sign_total`), indépendant du plafond de 100 erreurs collectées ; et par **C-26-1-23** (extourne, deux messages,
> liste des lignes à l'écran).

- **Contexte** (F4) : en erreur de ligne confirmable, un fichier UBS sous le mauvais réglage s'importerait par la case
  « lignes valides quand même » réduit à ses crédits — dépenses absentes, solde faux, en silence : #620 par une autre
  porte.
- **Retenu** (arbitrage de l'orchestrateur ; *corps réécrit en P3*) : la décision vit dans `kesh-import` —
  `parse_csv_collect` tient un compteur `sign_total` et un vecteur `sign_errors` **propres au signe**, indépendants
  du plafond général, et rend `HardFailure(CsvError::AmountSignMismatch { errors, total_errors, truncated,
  whole_debit_column })` dès que `sign_total > 0` ; `kesh-api` → `422 BANK_CSV_AMOUNT_SIGN_MISMATCH`, en preview et en
  import, `confirmPartialImport` sans effet ; deux messages selon `whole_debit_column` (C-26-1-23, C-26-1-25) ; l'écran
  liste les lignes en cause et n'offre pas la case.
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

### Validation P2 de la 26-1 (2026-10-10) — choix de la remédiation

Rapports : `/home/gcorbaz/devel/kesh-gate-logs/26-1-validate-p2-{R,F}.md` (Opus). Arbitrages (0) à (5) de
l'orchestrateur repris tels quels.

### C-26-1-23 — L'extourne isolée bloque aussi ; le message dit quoi faire selon le cas

- **Contexte** (F2-5) : un crédit écrit en négatif (encaissement contre-passé) ou, sous `negative`, un débit écrit en
  positif a un sens non ambigu ; le refus non confirmable de C-26-1-15 bloque pourtant tout le relevé.
- **Retenu** (arbitrage de l'orchestrateur : sécurité d'abord) : le refus reste **bloquant** même pour une ligne
  isolée. La conséquence est **écrite** (fiche, manuel, CHANGELOG) : un tel relevé ne s'importe pas tel quel dans cette
  version. Le relevé réel de Guy n'en contient aucune (structure relevée le 2026-10-10, sans chiffre). Le refus porte
  `whole_debit_column` (définition fermée par C-26-1-25 : au moins **deux** débits lus et non nuls, **tous** en
  défaut de signe — `d ≥ 2` et `e = d`) : s'il est vrai, le message dit « réglez le sens des débits du profil » ;
  sinon (dont le cas d'un seul débit, en défaut) il dit qu'il s'agit d'une ou de quelques lignes (une extourne, par
  exemple) et l'écran **liste** les lignes en cause (arbitrage (2)). *(Corps aligné sur C-26-1-25 le 2026-10-11,
  finding F4-1 de la validation P4 ciblée.)*
- **Écarté** : rendre l'extourne isolée confirmable, ou bloquer seulement au-delà d'une majorité (seuil arbitraire ;
  une ligne de mauvais signe peut aussi être un mauvais réglage sur un fichier à un seul débit).
- **Réversible** : oui — un assouplissement ultérieur (story dédiée, issue si le cas se présente) n'enlève rien.

### C-26-1-24 — La ligne `migrations.sha384` suit le fichier jusqu'au merge

- **Contexte** (F2-12) : le registre des checksums dit qu'une migration inscrite « ne se modifie jamais » ; la boucle
  de revue de code peut pourtant demander de retoucher la migration avant le merge.
- **Retenu** : tant que la branche n'est pas fusionnée, une retouche de la migration est **délibérée** (bases du
  worktree reconstruites, P8), la ligne `sha384` est recalculée **dans le même commit**, et le Change Log le dit.
  L'immuabilité vaut à partir de la publication (merge, puis release).
- **Réversible** : sans objet.

### Validation P3 de la 26-1 (2026-10-11) — choix de la remédiation

Rapports : `/home/gcorbaz/devel/kesh-gate-logs/26-1-validate-p3-{R,F}.md` (Sonnet). Arbitrages (1) et (2) de
l'orchestrateur repris tels quels.

### C-26-1-25 — `whole_debit_column` : définition fermée, et jamais vrai pour un seul débit

- **Contexte** (R3-2 = F3-2, F3-4) : la définition « toutes les valeurs non nulles de la colonne Débit » ne disait ni
  quelles lignes comptent, ni d'où l'information sort de `parse_row` ; et un fichier à un seul débit en défaut recevait
  le conseil de changer le réglage — qui, suivi, importerait un remboursement comme une dépense, en silence.
- **Retenu** (arbitrage de l'orchestrateur) : parmi les lignes dont le montant Débit a été **lu**, `d` = débits non
  nuls, `e` = ceux en défaut de signe ; `whole_debit_column ⇔ d ≥ 2 et e = d`. Les lignes en échec avant le montant et
  un débit illisible sont hors des deux comptes ; une erreur de signe de la colonne Crédit compte dans `sign_total`
  seulement. L'information sort de `parse_row` par un type de retour interne `RowParse { result, debit:
  DebitObservation }` — rien n'est ajouté à `CsvLineError` (public, sérialisé). Cas limites fixés par test.
- **Écarté** : un champ « colonne » dans `CsvLineError` (changement d'API) ; relire les cellules dans la boucle
  (duplication de `parse_row`) ; `d ≥ 1` (le cas du remboursement isolé).
- **Réversible** : oui.

### C-26-1-26 — `migrations_upgrade_path.rs` réutilise `common::migrations_before`

- **Contexte** (R3-5) : la remédiation P2 faisait écrire une fonction locale `index_of_migration`, doublon de
  `common::migrations_before` (règle DRY du dépôt).
- **Retenu** : `#[allow(dead_code)] mod common;` (l'attribut, sur la seule déclaration, couvre
  `common::apply_migrations_up_to`, inutilisée dans ce fichier) et appel qualifié `common::migrations_before(…)` ; la
  fonction locale `apply_migrations_up_to`, qui porte la garde de frontière, reste.
- **Écarté** : la fonction locale (doublon) ; supprimer la locale au profit de celle de `common` (perte de la garde de
  frontière et de son message).

### C-26-1-27 — Le manuel : quatre sections à la 26-1, la procédure d'import à la 26-3

- **Contexte** (R3-1 = F3-1) : la remédiation P2 (arbitrage (4), F2-13) faisait réécrire par la 26-1 la § « Import avec
  un profil existant », que la 26-3 réécrit déjà (AC 2.1 de sa fiche) d'après l'écran réel, autrement.
- **Retenu** (arbitrage de l'orchestrateur) : la 26-1 ne la touche pas ; elle tient « Profils banque », la sous-section
  « Le relevé CSV d'UBS », « Rejet partiel » et « Détection des doublons » (que la 26-3 ne touche pas — vérifié en
  lecture dans sa branche le 2026-10-11). Le refus de signe dit « réglez le sens des débits du profil, puis redéposez le
  fichier », et la sous-section UBS renvoie à la procédure de la 26-3.
- **Réversible** : oui.

### Développement de la 26-1 (2026-10-11) — choix de l'implémentation

### C-26-1-28 — Le plafond des lignes à ignorer est une constante publique

- **Retenu** : `kesh_import::MAX_HEADER_ROW_COUNT = 20`, lue par `CsvProfile::validate` et par son message
  (`header_row_count 21 > 20`) ; le `CHECK` de la base porte la même borne, écrite en dur (SQL).
- **Écarté** : un `20` littéral dans `validate` (deux sites à tenir alignés dans le même fichier).
- **Réversible** : oui.

### C-26-1-29 — Garde de surcomptage : la longueur se lit par `record.get(date)`

- **Contexte** : AC5 écrit « au moins `date + 1` champs » ; `rec.len() >= date + 1` déclenche `clippy::int_plus_one`.
- **Retenu** : `record.get(date)` rend `None` pour une ligne trop courte — même sémantique, sans comparaison. La
  mutation M13 (« `> date + 1` ») s'exécute en **insérant** la comparaison fautive ; le test de limite la voit.
- **Réversible** : oui.

### C-26-1-30 — Les erreurs de signe ne passent pas par le vecteur général

- **Retenu** : une erreur `AMOUNT_SIGN_MISMATCH` alimente seulement `sign_total`/`sign_errors` (AC3), jamais le
  vecteur général ni `total_errors` ; quand `sign_total > 0`, ce dernier n'est de toute façon pas rapporté.
  Accumulateur interne `RowAccumulator` (transactions, deux vecteurs, `d`, `e`).
- **Écarté** : alimenter les deux (une erreur de signe compterait deux fois et occuperait le plafond général).
- **Réversible** : oui.

### C-26-1-31 — Textes allemands, italiens et anglais rédigés par l'agent

- **Retenu** : les sept clés neuves et la clé modifiée sont traduites dans les trois autres locales (registre suisse :
  « gross », « Belastungen », « Administration → Bankprofile »…). À relire par un locuteur à la recette.
- **Réversible** : oui (texte seul).

### C-26-1-32 — CHANGELOG : rubrique « Précisé »

- **Retenu** : la fiche (AC18, R3-9) demande que les lignes vides soient consignées « en *Précisé* » : la rubrique
  `### Précisé` est créée dans la section `0.13.1`, à côté de `Corrigé` et `Modifié` (pas de rubrique de Keep a
  Changelog pour une clarification sans changement).
- **Réversible** : oui (prepare-release ne lit que l'en-tête de section).

### C-26-1-33 — Sous-section « Le relevé CSV d'UBS » placée juste après « Profils banque »

- **Retenu** : `\subsubsection` à la suite de « Profils banque », donc **immédiatement avant** « Import avec un
  profil existant » (à la 26-3). Le conflit textuel au rebase est probable sur la ligne frontière : **union**.
- **Écarté** : la placer après « Rejet partiel » (hors de la sous-section CSV, lecture moins naturelle).
- **Réversible** : oui.

### C-26-1-34 — E2E : « aucun import de plus » mesuré sur le total de la société

- **Retenu** : `GET /api/v1/bank-imports?limit=1` (champ `total`, sans filtre de compte) avant et après le dépôt
  refusé — plus strict qu'un compte par compte bancaire, et sans dépendre de l'identifiant du compte.
- **Réversible** : oui.

### C-26-1-35 — AC17 : la répartition débit/crédit se relit par le lecteur `csv`

- **Retenu** : pour savoir de quelle colonne vient chaque transaction, le test relit les lignes de données avec le
  crate `csv` (mêmes réglages que le parseur, 9 enregistrements sautés) et les apparie dans l'ordre ; aucun nombre en
  dur, aucune valeur imprimée.
- **Réversible** : oui.

### C-26-1-36 — M19 jouée sous ses deux formes ; la forme `is_sign_negative()` est équivalente

- **Contexte** : AC19 prescrit M19 sous « `<= ZERO` **ou** `is_sign_negative()` ». La seconde forme est restée verte :
  `rust_decimal` ne garde pas le bit de signe de `-0.00` lu par `parse_amount`, si bien que la mutation ne change
  aucun comportement.
- **Retenu** : mutation **équivalente**, figée par le test `negative_zero_has_no_sign_bit_after_parse` (commit
  `f23ad162`), qui rougirait si la bibliothèque se mettait à garder ce signe ; la forme `<= ZERO` (et `>= ZERO` sous
  `Negative`) est jouée sur les trois comparaisons et **rougit** chaque fois (`zeros_and_explicit_plus_are_accepted`,
  `explicit_zero_debit_or_credit_accepted`, `whole_debit_column_definition`).
- **Écarté** : reclasser M19 sans test, ou réécrire la comparaison pour rendre la mutation détectable (aucun
  comportement à protéger).
- **Réversible** : oui.

### C-26-1-37 — M26 attrapée par le garde-fou du squash, non par le test d'AC8

- **Contexte** : les tests d'AC8 montent le **squash** (`test-schema/`), que la mutation de la migration ne touche
  pas ; ce n'est donc pas eux qui peuvent la voir.
- **Retenu** : la mutation rougit sur `test_schema_guard::squash_matches_real_schema_structure` (le squash diverge du
  schéma réel : la contrainte relâchée n'y est plus) et sur `published_migrations_keep_their_checksums` ; c'est le
  canal prévu par la Story 22-5 pour toute dérive migration/squash. Compté comme « rougit ».
- **Écarté** : un test d'AC8 de plus sur le vrai `MIGRATOR` (il entrerait dans la liste fermée de
  `test_schema_guard.rs` pour un défaut déjà détecté).
- **Réversible** : oui.

### C-26-1-38 — Garde de complétude de `CsvLineErrorCode::all()` : comparée aux variants que serde tire de l'enum

- **Contexte** (revue de code P1, F1 = E-3) : la garde `all_is_complete` itérait sur `all()` elle-même ; un variant
  ajouté à l'enum, à `as_str()` et au `match` du test mais oublié dans `all()` la laissait verte. AC14-9 annonçait
  « ne compile pas ».
- **Retenu** : le test intercepte `Deserializer::deserialize_enum`, auquel la dérivation `serde::Deserialize` passe la
  liste des noms de variants **tirée de l'enum** (aucune liste à la main), et exige `all()` (par `as_str`) égale à
  cette liste, dans l'ordre. Éprouvé par mutation (`MutantVariant` ajouté à l'enum, à `as_str` et à `i18n_key`,
  oublié dans `all()` → `all_is_complete` rouge).
- **Écarté** : une garde **à la compilation** — sans macro procédurale (`strum`), Rust n'énumère pas les variants hors
  d'un `match`, et un `match` exhaustif ne contraint que la fonction qui le porte ; une fonction de rang à `match`
  exhaustif (proposée par la lentille A) se tient elle aussi à la main ; analyser le message d'erreur de serde
  (`expected one of …`), qui dépend d'un texte et non d'une API. AC14-9 récrit en conséquence (« fait rougir le
  test »), sans changer son intention.
- **Réversible** : oui.

### C-26-1-39 — Un débit nul rend le zéro sans signe

- **Contexte** (revue de code P1, E-4, LOW) : sous `Positive`, `-parsed` sur `0.00` produit un zéro à bit de signe
  (`Neg` de `rust_decimal`), égal à `ZERO` mais affichable « -0.00 » ; comportement antérieur à la story.
- **Retenu** : `parse_row` rend `Decimal::ZERO` pour un débit nul ; `zeros_and_explicit_plus_are_accepted` asserte
  l'absence de bit de signe sous les deux réglages ; mutation (normalisation retirée) rouge.
- **Écarté** : laisser tel quel (correctif d'une ligne, dans le code que la story réécrit) ; normaliser aussi le
  `-amount` de CAMT (`camt053/mod.rs`, hors périmètre de la story — recensé au Change Log).
- **Réversible** : oui.
- **Amendé en revue de code P2** (E2-2 = B-P2-1 = A-4, LOW, convergent) : `Decimal::ZERO` est d'échelle 0 — un débit
  `0.00` s'affichait « 0 CHF » à l'aperçu (sérialisation `serde-str`), alors qu'un crédit nul s'affiche « 0.00 », et
  sous `Negative` le code d'avant rendait déjà « 0.00 ». **Retenu** : `parsed.abs()` — bit de signe effacé, échelle
  du fichier gardée ; `zeros_and_explicit_plus_are_accepted` asserte en plus `to_string() == "0.00"` sous les deux
  réglages ; mutation (retour à `Decimal::ZERO`) rouge. **Écarté** : `parsed` seul (sans bit de signe aujourd'hui par
  le constat M19, mais sans garantie si la bibliothèque changeait) ; `Decimal::new(0, parsed.scale())` (équivalent,
  moins lisible). Le premier « Retenu » ci-dessus reste pour l'historique.

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

### C-26-2-3 — 60 caractères, espaces retirés, vide = absente, caractères de contrôle refusés *(ensemble refusé élargi par C-26-2-17 et C-26-2-28)*

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

### C-26-2-5 — Forme du libellé *(amendé par C-26-2-26 : aucun changement visible sans description)*

- **Retenu** : deux fonctions, et elles seules :
  - **nom du compte** : `UBS — Personnel` avec description, `UBS` sans — pour les listes qui montrent
    l'IBAN à part (écran *Comptes bancaires*, *Paramètres*, tuile de l'accueil) et pour le message
    serveur du refus de désignation ;
  - **libellé d'option** : nom du compte, `" — "`, IBAN complet — `UBS — Personnel — CH93…` avec,
    `UBS — CH93…` sans — pour tous les `<select>` et l'en-tête du formulaire de lien au plan comptable.
- **Constat** : l'affichage d'aujourd'hui est **la banque et l'IBAN complet** dans les sélecteurs, la
  banque seule dans les listes (l'IBAN complet ou raccourci en dessous) — et non « la fin de l'IBAN »
  comme le dit le commentaire de #622. Sans description, ces affichages sont conservés au caractère près,
  **sans exception** : le sélecteur de l'import bancaire, qui écrit `UBS (CH93…)`, garde sa parenthèse
  (paramètre `ibanStyle = 'parens'` de la fonction de libellé d'option, C-26-2-26). *(Corps réécrit en P3 de
  validation, L1 : la version d'origine annonçait une exception que C-26-2-26 a retirée.)*
- **Écarté** *(à l'origine ; renversé par C-26-2-26, qui garde la parenthèse par un paramètre de la même fonction)* :
  garder la parenthèse (une variante de la fonction pour un seul site) ; retirer l'IBAN des sélecteurs quand une description existe (deux homonymes
  redeviendraient indiscernables, C-26-2-4).
- **Réversibilité** : une ligne dans chaque fonction.

### C-26-2-6 — Place de la colonne *(remplacé pour la base par C-26-2-16 — colonne en fin de table ; la place dans le CSV reste)*

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

### C-26-2-8 — `PUT` en remplacement complet *(figé et documenté par C-26-2-19)*

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

### C-26-2-11 — Messages de validation serveur en français, comme leurs voisins *(le refus « caractère » couvre l'ensemble de C-26-2-17 / C-26-2-28)*

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
→ « UBS — CH… ») est accepté : il aligne ce menu sur les autres. *(Amendé par C-26-2-26, sur P2 R2-2/F-5 : la décision 4 du Project Lead s'applique à la lettre — le menu d'import garde « UBS (CH…) » sans description.)*

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

### C-26-2-20 — La liste des colonnes est une macro du dépôt *(amendé en P2 : `find_by_id_in_tx(tx, company_id, id)`, scopé par société, patron `projects.rs:59`)*

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

### C-26-2-22 — La préservation de la description par l'upsert d'onboarding (défense en profondeur) *(titre et corps réécrits en P3 de validation, L2-L3)*

- **Contexte** : F11 (P1) — `upsert_primary_in_tx` réécrit banque, IBAN et QR-IBAN sans toucher la description.
- **Retenu** : `upsert_primary_in_tx` ne touche pas la description (conséquence de C-26-2-2 : l'onboarding ne
  la connaît pas) ; **non dit au manuel** (phrase retirée en P2) — le cas est inatteignable, voir l'amendement.
- **Amendement P2** (R2-1 = F-1, arbitrage de l'orchestrateur) : le cas n'est pas rare, il est **inatteignable** —
  l'étape 6 exige `step_completed == 6` hors démo (`routes/onboarding.rs:581`, l. 1060), la description ne se
  saisit qu'à partir de l'étape 7 (`routes/bank_accounts.rs:222`), une restauration force l'étape à 8
  (`backup.rs:644`). La phrase du manuel est **retirée** ; le test de l'upsert et M5 restent comme défense en
  profondeur du dépôt.
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

## Story 26-2 — validation P2 remédiée (2026-10-10)

### C-26-2-26 — Sans description, chaque site garde exactement son affichage (arbitrage de l'orchestrateur)

- **Contexte** : P2 R2-2 (MEDIUM) = F-5 — le menu de l'import passait de « UBS (CH…) » à « UBS — CH… » sans
  description, contre la décision 4 du Project Lead (« sans elle, l'affichage reste celui d'aujourd'hui »).
- **Retenu** : la lettre de la décision. `bankAccountOptionLabel(a, ibanStyle: 'dash' | 'parens' = 'dash')` : le
  **nom** vient toujours de `bankAccountName` (source unique), seule la présentation de l'IBAN suit la forme du
  site ; D7 en `'parens'` (« UBS — Personnel (CH…) » avec description, « UBS (CH…) » sans). Amende C-26-2-5 et
  C-26-2-15 ; la ligne CHANGELOG `### Modifié` sur ce menu disparaît.
- **Écarté** : harmoniser (C-26-2-5 d'origine) — c'est revenir sur une décision écrite du Project Lead ; un libellé
  propre à D7 hors de `format.ts` — duplication.

### C-26-2-27 — `BankAccountPutFields` plutôt qu'un huitième paramètre

- **Contexte** : P2 F-3 — `update_for_company` a déjà sept paramètres ; un huitième déclenche
  `clippy::too_many_arguments`.
- **Retenu** : `pub struct BankAccountPutFields<'a> { journal_account_id: Option<i64>, description: Option<&'a str> }`
  remplace `new_journal_account_id` ; `NewBankAccount` inchangé (C-26-2-7).
- **Écarté** : `#[allow(clippy::too_many_arguments)]` (arbitrage de l'orchestrateur).

### C-26-2-28 — Ensemble refusé élargi, contrôle après `trim()`

- **Contexte** : P2 R2-6 / F-8 — U+061C, U+00AD et d'autres invisibles passaient ; l'ordre trim / contrôle n'était
  pas fixé.
- **Retenu** : ajout de U+00AD, U+034F, U+061C, U+180E, U+206A à U+206F, U+3164, U+FE00 à U+FE0F, U+FFF9 à U+FFFB,
  U+E0000 à U+E007F ; angle mort nommé (U+115F, U+1160, U+FFA0, U+E0100 à U+E01EF…) ; message « de contrôle ou de
  mise en page » ; contrôle sur la valeur **après** `trim()` (« Épargne\n » accepté, « Épar\ngne » refusé).
- **Écarté** : toute la catégorie Cf par une crate (dépendance pour un champ).

### C-26-2-29 — Libellé « optionnelle », pas de guillemets autour du nom dans le refus

- **Retenu** : « Description (optionnelle) » / « (optional) » / « (opzionale) » / « (optional) », vocabulaire du
  voisin « QR-IBAN (optionnel) » ; aide traduite dans les quatre locales dans la fiche. Le message du refus de
  désignation reste sans guillemets autour de `{ $bank }` (FTL et replis `format!` inchangés).
- **Écarté** : « facultative » (rompt avec le voisin) ; des guillemets (quatre FTL et deux replis à changer pour un
  gain de lecture incertain).

### C-26-2-30 — La 26-2b corrige le compte de la QR-facture au manuel

- **Contexte** : P2 F-2 (MEDIUM) — le manuel utilisateur décrit un champ « Compte bancaire de réception » à la
  création de facture, qui n'existe pas ; la section neuve le contredirait. Vérifié au code : le PDF prend
  `bank_accounts::find_primary` (`invoice_pdf_service.rs:222`) ; QR-IBAN → référence QRR, sinon IBAN **sans
  référence** (`Reference::None`, l. 321-331) — le manuel disait aussi, à tort, ISO 11649.
- **Retenu** (arbitrage de l'orchestrateur) : la 26-2b (AC12) corrige les l. 911, 921, 933 et 1033, PDF régénéré
  et contrôlé aplati.
- **Écarté** : une issue P3 séparée — la section neuve parlerait de la QR-facture à côté d'un texte faux.

### C-26-2-31 — 26-2a développée avant la fusion de la 26-1 (cas « commencée avant »)

- **Contexte** : au démarrage de `bmad-dev-story` (2026-10-10), `origin/main` = `9b724027`, sans la 26-1.
- **Retenu** : migration `20261011000002` écrite sur la branche seule ; compteurs portés 78 → 79, soustracteur
  44 → 45, sha384 à 79 lignes ; le rebase et la réfection des recomptes (80 / 46, squash régénéré, union des sha384,
  bases de dev reconstruites) reviennent à l'orchestrateur, procédure de la fiche (Dev Notes § Coordination, cas 2).
- **Écarté** : attendre la fusion de la 26-1 (bloquait le développement) ; écrire d'avance 80 / 46 (faux sur l'arbre
  testé, le gate aurait rougi).
- **Réversibilité** : totale — les fichiers en conflit se refont au rebase.

### C-26-2-32 — Le montage du rattrapage de lettrage crée son compte bancaire par SQL

- **Contexte** : gate complet sur `c9e12142` — `tests/support/lettering_documents.rs` appelait `bank_accounts::create`
  sur un schéma arrêté avant `20261010000001` ; le dépôt relit `description` → `1054 Unknown column`, 4 échecs.
- **Retenu** : `INSERT … RETURNING id` ne nommant que des colonnes anciennes (valide à toute borne de migration).
- **Écarté** : faire tolérer au dépôt l'absence de colonne (code de production plié pour un test) ; monter le
  montage sur le schéma complet (le test perdrait son objet : rejouer M1 sur des données antérieures).
- **Réversibilité** : une fonction de test.

### C-26-2-33 — `find_by_id_in_tx` rend `Result<BankAccount, DbError>`

- **Contexte** : AC2 prescrit « même nom, même contrat » que `projects.rs:59` (qui rend `Option`) **et** la signature
  `-> Result<BankAccount, DbError>`, `NotFound` si absente.
- **Retenu** : la signature explicite de l'AC (l'appelant unique, la relecture après `INSERT`, n'a pas de cas
  « absent » légitime). Testé : scope par société et id inconnu → `NotFound`.
- **Réversibilité** : une fonction, un appelant.

### C-26-2-34 — Garde des libellés en dur recomptée (61 → 62)

- **Contexte** : `bankAccountOptionLabel` porte le suffixe `Label` ; `i18n-libelle-en-dur.test.ts` le relève.
- **Retenu** : candidate **conforme** (gabarits interpolés de données, aucun mot d'interface) ; borne 61 → 62,
  `conforme` 47 → 48, généalogie nommée. **Écarté** : renommer la fonction pour échapper à la garde (le nom est celui
  de la fiche, et contourner une garde par le nom est le défaut qu'elle combat).

## Story 26-3 — Le manuel décrit les écrans réels ; garde de `kesh-restore.sh` (#616, #617, #619)

*(Entrées venues de `main` à la fusion du 2026-10-11, par union.)*

### C-26-3-1 — 26-3 (spec) : #616 traité comme « le formulaire du compte », non comme une ligne

- **Contexte** : #616 signale la case « Compte de banque » (`user-manual.tex:345`, `:1652`). La liste
  où elle figure (`§ Ajout d'un compte`) décrit d'autres champs que le formulaire n'a pas non plus :
  « Devise », les types « Bilan / Résultat », le libellé « Libellé » (le formulaire dit « Nom ») ; le
  paragraphe « Modification d'un compte » dit le compte parent modifiable et le numéro modifiable
  tant qu'aucune écriture n'est saisie, alors que le dialogue d'édition n'a pas de champ parent et
  désactive toujours le numéro. Même symptôme (le manuel décrit un écran qui n'existe pas), même liste.
- **Retenu** : inventaire **clos** des champs des deux dialogues (`accounts/+page.svelte`, création
  `:532`-`:590`, édition `:618`-`:660` sur `fe162504`) ; la liste du manuel est réécrite sur cet
  inventaire, et dit comment un compte devient compte bancaire (Administration → Comptes bancaires).
  Les deux procédures d'import (`Import dans Kesh`, `Import avec un profil existant`) sont réécrites
  sur l'écran réel (`/bank-import` : « Importer des relevés », prévisualisation, « Confirmer l'import »).
- **Écarté** : corriger la seule ligne citée (laisse trois champs fictifs à côté) ; réécrire aussi
  « Profils banque » (création d'un profil) — la Story 26-1 modifie les réglages du profil et réécrit
  cette sous-section : la toucher ici créerait un conflit certain, sans gain.
- **Réversible** : oui (texte).

### C-26-3-2 — 26-3 (spec) : #617, les libellés d'écran « CAMT.053 seul » corrigés avec le manuel

- **Contexte** : le grep du symptôme (« le CAMT.053 est la voie ») trouve, hors du manuel, la zone de
  dépôt de l'import : `bank-import-labels-drop-zone` vaut « Glissez votre fichier CAMT.053 ici… » dans
  les **quatre** catalogues, alors que l'écran accepte `.csv` (`accept=".xml,…,.csv,text/csv,.txt"`) et
  que le repli Svelte dit déjà « CAMT.053 ou CSV » ; l'`aria-label` en dur et le titre de page
  (`bank-import-labels-page-title` = « Import bancaire CAMT.053 ») disent de même.
- **Retenu** : les trois textes disent « CAMT.053 ou CSV » (titre : « Import de relevés bancaires »,
  aligné sur le menu « Importer des relevés »), dans les quatre locales, replis compris.
- **Écarté** : laisser l'écran à une story ultérieure — c'est exactement l'écart que #617 reproche au
  manuel, sur l'écran que l'utilisateur a devant lui.
- **Réversible** : oui. Coût : la story touche du code d'écran, donc E2E complet au dernier commit.

### C-26-3-3 — 26-3 (spec) : #619, une garde FERMÉE, réglable, avec un filet par l'image

- **Contexte** : la garde de `kesh-restore.sh` (`:76`) ne voit que les conteneurs étiquetés
  `com.docker.compose.service=kesh-api` ; un service renommé n'est pas vu, la boucle ne tourne pas, le
  rechargement part sous un Kesh en marche. Un renommage dans un compose laisse de plus l'ancien
  conteneur **orphelin** (Compose ne le retire qu'avec `--remove-orphans`) : un `kesh-api` arrêté
  peut coexister avec un `kesh` en marche.
- **Retenu**, trois mécanismes cumulés :
  1. **réglage `SAUVEGARDE_SERVICE`** (défaut `kesh-api`, un service de `docker-compose.prod.yml`),
     comme les quatre `SAUVEGARDE_*` existants ;
  2. **fermée** : si **aucun** conteneur de ce service n'existe sur l'hôte, refus (le script ne sait
     pas ce qu'il garde) — message qui nomme `SAUVEGARDE_SERVICE` et, pour un hôte où Kesh n'est pas
     encore installé, la commande qui crée le conteneur sans le démarrer (`docker compose create
     <service>`, état `created`, admis — **amendé par C-26-3-11** : `docker compose up --no-start`) ;
  3. **filet** : est aussi refusé tout conteneur actif dont l'image est `gcorbaz/kesh` (toute
     étiquette ou empreinte, préfixe `docker.io/` admis), quel que soit son service — c'est lui qui
     voit l'orphelin et le service renommé.
- **Écarté** : (a) réglage seul, garde ouverte sur zéro conteneur — un oubli du réglage redonne le
  défaut d'origine ; (b) garde fermée seule — l'orphelin `kesh-api` arrêté la satisfait pendant que
  `kesh` tourne ; (c) image seule — une image construite localement ou une étiquette déplacée
  (`docker ps` affiche alors un identifiant) passe ; (d) auto-détection du service par le compose —
  le script ne lit pas le compose et ne doit pas en dépendre (C-15-14-77) ; (e) option `--force` —
  la garde n'a pas d'issue de contournement, c'est sa raison d'être.
- **Angle mort écrit** : une image Kesh construite sous un autre nom, dans un service autre que
  `SAUVEGARDE_SERVICE`, n'est vue par rien — elle exige le réglage. Deux installations Kesh sur le
  même hôte se gênent davantage (refus, sens sûr ; déjà écrit en C-15-14-77).
- **Réversible** : oui.

### C-26-3-4 — 26-3 (spec) : #619, le réseau du dump reste un réglage explicite, documenté

- **Contexte** : `SAUVEGARDE_RESEAU` (défaut `frontend`) doit désigner un réseau Docker **existant**
  d'où la base est joignable. Le manuel n'en montre aucun exemple, et sa propre recette Traefik
  (`§ sec:traefik`) rattache `kesh-api` à `kesh-internal`, réseau créé par Compose sous le nom
  `<projet>_kesh-internal` : avec elle, le défaut `frontend` échoue.
- **Retenu** : le manuel dit la règle (le réseau de `kesh-api`, qui atteint déjà la base), donne la
  commande qui l'affiche (`docker inspect` sur le conteneur), et l'exemple d'un reverse proxy
  (`SAUVEGARDE_RESEAU=proxy`), pour la tâche planifiée comme pour le rechargement. Le listing de
  suppression d'une base illisible garde `--network frontend` (la recette le réécrit par `sed`) ; la
  prose dit de le remplacer par la valeur de `SAUVEGARDE_RESEAU`.
- **Écarté** : déduire le réseau du conteneur `kesh-api` dans le script — un second réglage implicite,
  et une dépendance au nom de conteneur ; changer le listing en `--network <réseau>` — casse la recette
  (`recette.sh:280`) pour un gain de forme.
- **Réversible** : oui.

### C-26-3-5 — 26-3 (spec) : gardes — G16 amendée, une garde de texte neuve

- **Retenu** : G16 (`crates/kesh-api/tests/configuration_transmise.rs`) lit la nouvelle forme de la
  garde (sa regex exige aujourd'hui `--filter label=com.docker.compose.service=kesh-api`), le réglage
  `SAUVEGARDE_SERVICE` (défaut = service de `docker-compose.prod.yml`, cité au manuel sous la forme
  `SAUVEGARDE_SERVICE (kesh-api`), la sortie sur zéro conteneur et le filet par l'image. Une garde de
  texte neuve, **G20** dans `textes_coherents.rs`, interdit « Compte de banque » (**forme du défaut,
  sensible à la casse — amendé par C-26-3-9**) et « toutes les
  banques suisses » dans les manuels français (les catalogues : valeurs de la zone de dépôt et du titre
  d'import — précisé en validation P2, R-L12), chaque négative adossée à une
  positive sur le même fichier.
- **Écarté** : ne compter que sur la recette — elle n'est pas dans le gate ; seule G16 l'est.
- ⚠️ **Numéro à vérifier au merge** : les Stories 26-1 et 26-2, spécifiées en parallèle, peuvent
  prendre G20 ; renuméroter la dernière mergée.
- **Réversible** : oui.

### C-26-3-6 — 26-3 (spec) : PDF régénérés — les deux manuels touchés seulement

- **Retenu** : `make -B -C docs/manual admin user` ; la brochure n'est pas touchée (aucune occurrence
  du symptôme, grep relevé à l'AC 3) et n'est pas régénérée, pour ne pas commiter un PDF qui ne
  diffère que par ses horodatages.
- **Écarté** : `make fr` (régénère la brochure sans motif). Les manuels DE/EN/IT n'existent pas
  (`docs/manual/{de,en,it}/` ne portent qu'un `README.md`).
- **Réversible** : oui.

### C-26-3-7 — 26-3 (spec) : un paragraphe voisin contradictoire corrigé au passage

- **Contexte** : sous le listing de téléchargement des scripts (`admin-manual.tex:1669`), le
  paragraphe dit « \keshpath{main} désigne la version courante du dépôt… remplacez \keshpath{main}
  par son étiquette », alors que le listing pointe déjà sur `v0.13.0` (G19 l'y oblige depuis la
  release 0.13.0). Même section que celle que la story réécrit, même nature (le manuel décrit autre
  chose que ce qu'il montre).
- **Retenu** : la phrase dit que l'URL désigne l'étiquette de la version installée, à garder égale à
  l'image du compose. Le numéro de version du listing **ne bouge pas** dans cette story : c'est la
  release v0.13.1 qui le portera (point 4-bis, G19).
- **Réversible** : oui.

### C-26-3-8 — 26-3 (spec) : CHANGELOG sous `## [0.13.1] — Non publié`

- **Retenu** : la story crée la section si elle n'existe pas (les 26-1 et 26-2 y écrivent aussi ;
  fusion par union au merge), entrées sous `### Corrigé`, plus une ligne « Action requise » pour
  l'exploitant Synology : re-télécharger `kesh-restore.sh` au tag de la version (`v0.13.1`), poser
  `SAUVEGARDE_SERVICE` si son service ne s'appelle pas `kesh-api`, `SAUVEGARDE_RESEAU` s'il n'est pas sur
  `frontend`, recréer le conteneur par `docker compose up --no-start` s'il a été supprimé (aligné sur
  l'AC 11.1 en validation P3, R-L7).
- **Amendé en validation P1 (R-9)** : au rebase, les entrées des trois stories se fusionnent par
  union sous **une seule** section `[0.13.1]` (`prepare-release.sh` refuse plus d'une section) ;
  contrôle `grep -c '^## \[0\.13\.1\] — ' CHANGELOG.md` = 1.
- **Réversible** : oui.

### C-26-3-9 — 26-3 (validation P1, R-1 = HIGH-2, R-5 = MEDIUM-1) : G20 ciblée ; lecture de `docker ps` à séparateur `|`

- **Contexte** : G20 prescrite « insensible à la casse » sur « Compte de banque » aurait rougi sur six
  phrases justes des manuels (« son propre compte de banque »). La lecture unique de `docker ps`, sans
  séparateur fixé, décalait les champs sur une étiquette de service vide (conteneur hors compose).
- **Retenu** : G20 cherche la **forme du défaut**, sensible à la casse, sur la source brute
  (`\textbf{Compte de banque}`, `« Compte de banque »`), avec « propre compte de banque » en positive ;
  elle lit les **quatre** catalogues (drop-zone avec « CSV », titre sans « CAMT.053 ») avec un
  anti-test-muet (> 1000 clés). Lecture `docker ps --format '{{.State}}|{{.Image}}|{{.Label …}}|{{.Names}}'`,
  `IFS='|' read -r ETAT IMAGE SERVICE NOM RESTE`, ligne vide ignorée, ligne incomplète ou à cinq champs
  = refus.
- **Écarté** : G20 limitée aux deux numéros de ligne (fragile) ; tabulation comme séparateur (IFS
  blanc : champs vides avalés).
- **Réversible** : oui.

### C-26-3-10 — 26-3 (validation P1, R-2 = HIGH-1, R-4, R-7, MEDIUM-5) : la recette ne recharge jamais dans 4-bis

> ⛔ **Remplacé par C-26-3-13** (validation P2) : la section 4-bis à vrais conteneurs est abandonnée.

- **Contexte** : R5/R6 laissaient le script recharger ; les oracles `$C` des sections 5, 5-bis, 7
  rougissaient, et une mutation faisait rougir tous les cas suivants. Le garde-fou d'étiquette cité
  (`recette.sh:31`) n'existait pas ; un Kesh étranger sur l'hôte aurait fait rougir la recette sans cause.
- **Retenu** : section « 4-bis » après la 4, chaque cas en `SAUVEGARDE_BASE=autre` — la garde qui passe
  s'arrête à l'étape 3 sur « recrée la base », sans écriture ; oracle de motif distinct par cas ; R6
  « rechargement abouti » supprimé (couvert par 6-9) ; R7 conteneur hors compose ; état de sortie
  prescrit et vérifié ; étiquette `gcorbaz/kesh:recette-synology-<8 hex>` construite, refusée si
  préexistante, retirée seule ; pré-vol « aucun `gcorbaz/kesh` actif ».
- **Écarté** : rejouer R6 après la section 9 (rechargement déjà couvert, et état final de la recette à
  remonter) ; restaurer C après chaque cas (oracle de plus, sans gain).
- **Réversible** : oui.

### C-26-3-11 — 26-3 (validation P1, MEDIUM-4) : une seule commande de recréation, `docker compose up --no-start`

- **Contexte** : la fiche donnait `docker compose create` au message et `up --no-start` en note
  technique ; le manuel ne disait pas qu'un `docker compose down` rend désormais le rechargement
  impossible sans recréer le conteneur.
- **Retenu** : `docker compose up --no-start <service>` partout (script, manuel — étape d'arrêt de la
  *Recovery* et *Stratégie 3-2-1* —, recette), avec l'angle mort : la commande lit le compose entier,
  `.env` doit être en place.
- **Écarté** : `create` (moins répandue dans les versions de Compose v2) ; les deux formes (deux
  consignes pour un geste).
- **Réversible** : oui.

### C-26-3-12 — 26-3 (validation P1, MEDIUM-3, R-8) : chemins d'écran et e-banking tiers

- **Retenu** : l'inventaire des renvois au chemin de l'écran d'import est fermé par un grep
  (`emph{Importer}`, `emph{Import bancaire}`, `Quotidien → Importer**`, `Nouvel import` : 7 lignes, 0
  attendu) — `user-manual.tex:1756`, `:1826`, `docs/user-guide/fr/getting-started.md:74` ajoutés —, et
  `docs/i18n-glossaire.md:151` suit la valeur d'écran (**amendé par C-26-3-16** : c'est la colonne
  « précédent » qui change, vers `audit-log-entity-bank-imports`). La sous-section « Récupération du fichier
  CAMT.053 » perd ses chemins de menus d'e-banking tiers (Multibanking/Klippa, PostFinance Yellownet),
  invérifiables, au profit d'une consigne générale.
- **Écarté** : garder les chemins tiers « à titre indicatif » — c'est la forme exacte de #617.
- **Hors inventaire, assumés** : `\section{Import bancaire}` (titre de chapitre), noms de fonctionnalité
  (manuel admin, brochure, README), type d'entité du journal d'audit, `docs/kesh-specifications.txt`.
- **Réversible** : oui.

### C-26-3-13 — 26-3 (validation P2, signal D5) : la garde se prouve contre un faux `docker`, sans conteneur

- **Contexte** : P2 (Opus, R 5M/12L, F 9M/8L) — presque tout né de la remédiation P1 (`f5c4a958`) et
  porté par la section 4-bis à vrais conteneurs ; **recyclage** : P1 MEDIUM-5 revient en P2 M1/M5 — la
  garde lit l'état Docker de **tout l'hôte**, que la recette ne maîtrise pas (`kesh-dev`, `kesh-api`
  d'autres compose du dépôt). Signal D5 déclaré.
- **Retenu** (décision de l'orchestrateur, **sans découpage, par simplification**) : la section neuve
  (« 0 », avant la mise en place) n'emploie **aucun** vrai conteneur. Un faux `docker` rend des sorties
  `docker ps --format` fabriquées (cas renommés **S1-S23** et complétés par C-26-3-18 ; à l'origine G1-G17 : service renommé, orphelin arrêté, `created`, `restarting`,
  `paused`, hors compose, ligne vide, ligne incomplète, cinq champs, deux projets, aucun conteneur, image
  Kesh sous un autre service, image par identifiant, image voisine, démon injoignable), journalise ses
  appels et **refuse** (code 97) toute autre sous-commande : aucune action sur une base n'est possible.
  `SAUVEGARDE_BASE=autre` reste, pour que « la garde passe » soit un message du script. Mutations aux
  cas rouges **exacts** (O, F, Z, I, R).
- **Écarté** : corriger 4-bis (pré-vol élargi, nom de service propre à la recette, `ecrire_compose`,
  `arreter_service`, nettoyage du hors-compose) — chaque correctif ajoutait une dépendance à l'hôte ;
  découper la story.
- **Conséquences** : plus de pré-vol, d'étiquette d'image, de `compose_factice` paramétré, d'`up
  --no-start` ni de nettoyage pour ces cas ; les sections réelles 1-9 inchangées ; leur dépendance à
  l'hôte (déjà vraie pour `kesh-api` avec la garde d'origine) est écrite en **prérequis** dans
  `docs/testing.md` — le message de la garde nomme le conteneur en cause.
- **Réversible** : oui.

### C-26-3-14 — 26-3 (remédiation P2) : le faux `docker` est une fonction exportée, pas un exécutable du PATH

- **Contexte** : la consigne « faux `docker` en tête de PATH » ne marche pas : `kesh-restore.sh:54`
  préfixe `PATH` par `/usr/local/bin:/usr/bin:/bin`, si bien que `/usr/bin/docker` passerait devant.
- **Retenu** : `docker() { … }; export -f docker` dans le **sous-shell de chaque cas** ; bash importe la
  fonction et la cherche **avant** le `PATH`. Anti-test-muet : chaque cas exige une ligne au journal du
  faux (journal vide = le vrai `docker` a répondu = rouge).
- **Écarté** : modifier `kesh-restore.sh` pour ne plus préfixer `PATH` (le préfixe protège la tâche
  planifiée DSM, au `PATH` minimal) ; un faux à chemin absolu (le script ne l'appellerait pas).
- **Réversible** : oui.

### C-26-3-15 — 26-3 (validation P2, F-M6) : la procédure d'import ne fait jamais changer le profil après l'aperçu

- **Contexte** : vérifié au code — l'aperçu est calculé au dépôt avec le profil alors choisi ; changer de
  profil ensuite ne le recalcule pas, et « Confirmer l'import » envoie le nouveau profil avec l'aperçu de
  l'ancien (`BankImportUpload.svelte:106`-`:116`, `:222`, `:73`-`:82`).
- **Retenu** : le manuel dit de choisir le profil **avant** de déposer ; sinon **Annuler**, choisir,
  **redéposer** (`reset()` garde le profil). Le défaut d'écran n'est **pas** corrigé ici (hors périmètre
  documentaire de la story) : **issue P2 à ouvrir par l'orchestrateur** (« l'aperçu d'import n'est pas
  recalculé quand on change de profil après le dépôt ; la confirmation importe avec un mapping non
  prévisualisé »).
- **Écarté** : corriger l'écran dans la 26-3 (change le comportement, demande ses propres tests) ;
  décrire « changer le profil puis confirmer » (chemin trompeur).
- **Réversible** : oui.

### C-26-3-16 — 26-3 (validation P2, F-M8) : glossaire — la colonne « précédent » suit une clé qui atteste encore

- **Retenu** : `docs/i18n-glossaire.md:151`, colonne « précédent » : `Import bancaire CAMT.053` →
  `audit-log-entity-bank-imports` (Import bancaire / Bankimport / Importazione bancaria / Bank import) ;
  les quatre termes ne changent pas. **Justification corrigée en validation P3 (R-L3)** : l'en-tête du
  glossaire dit que la colonne nomme la clé, mais ce bloc de lignes y porte des **valeurs d'écran**
  (`Carnet d'adresses`, `Réconciliation`…). On écrit donc, dans le style des voisines, la valeur **et** la
  clé : `Import bancaire` (`audit-log-entity-bank-imports`).
- **Écarté** : y écrire la nouvelle valeur du titre (« Import de relevés bancaires »), qui n'atteste pas
  « import bancaire » — une attestation fausse dans le document qui sert à n'en pas avoir.
- **Réversible** : oui.

### C-26-3-17 — 26-3 (validation P2, L6/L7) : cibles DE/EN/IT fixées ; titre et `aria-label` lus par i18n

- **Retenu** : sur le terme attesté `relevé (bancaire)` (glossaire `:105`, figé par `nav-bank-import`) —
  titre : `Import von Kontoauszügen` / `Bank statement import` / `Importazione di estratti conto` ; zone :
  `CAMT.053- oder CSV-Datei hier ablegen oder klicken zum Durchsuchen` / `Drop your CAMT.053 or CSV file
  here or click to browse` / `Trascinare il file CAMT.053 o CSV qui o fare clic per sfogliare`. Le
  `<title>` lit `bank-import-labels-page-title` et l'`aria-label` lit `bank-import-labels-drop-zone` (clés
  existantes, coût nul, tenus par G20).
- **Écarté** : « traductions de même sens » (à deviner) ; garder les deux textes en dur « assumés ».
- **Réversible** : oui.

### C-26-3-18 — 26-3 (validation P3, R-M2 = F-5/F-6, R-L1, R-L4 = F-7, F-4, R-L5) : le faux `docker` précisé, six cas neufs

- **Contexte** : P3 (Sonnet, R 2M/9L, F 4M/8L) — la simplification tient (17 cas, 5 mutations exacts),
  mais la fiche ne disait ni **où** la fonction est définie (définie au niveau du script, elle masquerait
  le vrai `docker` pour la mise en place et la `trap nettoyer`, muette), ni que `FAUX_*` doivent être
  exportées (sinon « unbound variable » sous `set -u`), ni comment jouer la section 0 seule ; aucun cas
  PASSE n'exerçait le scénario #619 réussi (service renommé, réglé, arrêté) ; les noms `G14`-`G17` des cas
  heurtaient ceux de gardes existantes.
- **Retenu** : fonction **définie et exportée dans le sous-shell de chaque cas**, `FAUX_*` exportées dans
  ce sous-shell, `FAUX_SORTIE` toujours un fichier (vide pour S5), `SAUVEGARDE_*` désarmées puis posées par
  cas, `2>&1` ; assertion `type -t docker` = `file` après la section 0 ; `RECETTE_SECTION=0` joue la
  section 0 seule (ni Docker ni `sqlx` — **amendé par C-26-3-21** : ordre de `recette.sh` imposé pour que ce soit vrai) ; cas renommés **S1-S23**, dont six neufs : S18 (#619 réussi), S19
  (+ orphelin arrêté), S20 (image Kesh d'un autre service à l'arrêt), S21 (`dead`, image nue), S22 (#619
  seul, en marche → FILET avant la sortie sur zéro), S23 (image nue d'un autre service → FILET) ; huit
  mutations (O, F, F′, Z, I, R, D, T) aux cas rouges recomptés sur 23 cas ; motifs `est « running »` et
  `est « paused »` ajoutés aux lignes 153 et 162 de la section 4 réelle (R-L6), seule retouche des sections
  réelles.
- **Écarté** : faire du faux un exécutable (masqué par `kesh-restore.sh:54`, C-26-3-14) ; porter la section
  0 dans le gate par un test Rust (R-L9) — non requis par les issues, **signalé au Project Lead**.
- **Réversible** : oui.

### C-26-3-19 — 26-3 (validation P3, F-9) : G14 lit `up`

- **Contexte** : `docker compose up --no-start kesh-api` devient la voie unique de recréation ; G14 ne lit
  pas `up` (verbes `exec|stop|start|restart|logs|pull|rm`).
- **Retenu** : `up` ajouté aux verbes de G14, options longues admises ; inventaire préalable (6 lignes
  nomment `kesh-api`, toutes hors sections Synology : vertes) ; mutation `up --no-start kesh` en section
  Synology → rouge.
- **Écarté** : une exigence textuelle dans G16 (f) seule — elle tiendrait la présence de la commande, pas
  l'existence du service nommé.
- **Réversible** : oui.

### C-26-3-20 — 26-3 (validation P3, F-12) : `<title>` au tiret long

- **Retenu** : `{i18nMsg('bank-import-labels-page-title', …)} — Kesh` — majorité des titres lus par clé
  (16 `} — Kesh` **dans un `<title>`** — 17 lignes au grep brut, dont une hors `<title>` — contre 8 `} - Kesh`
  sous `frontend/src/routes`, relevé P3, précisé en P4 ciblée, L3).
- **Écarté** : garder le tiret court d'origine (minorité, et sans la lecture par clé).
- **Réversible** : oui.

### C-26-3-21 — 26-3 (validation P4 ciblée, M1) : la section 0 tourne avant `openssl` et la trap qui appelle `docker`

- **Contexte** : P4 ciblée (Haiku) — `recette.sh:25`-`:37` exécute `mktemp`, `openssl` et pose
  `trap nettoyer EXIT` avant toute section ; `nettoyer` appelle le **vrai** `docker` (`rm -f`, `network rm`,
  `run mariadb:10.11`, image tirée si absente), erreurs vers `/dev/null`. « `RECETTE_SECTION=0` sans
  Docker » était vrai en résultat, faux en exécution. Vérifié par l'orchestrateur.
- **Retenu** : ordre imposé — `mktemp` et une trap **minimale** `rm -rf "$W"`, puis la section 0 (sortie
  en mode 0), puis `openssl`, garde `kesh-mariadb-dev`, `nettoyer` et `trap nettoyer EXIT`. Prérequis du
  mode 0 écrits exactement (bash, coreutils, gzip, grep, awk ; ni docker, ni openssl, ni sqlx, ni python3).
- **Écarté** : écrire le prérequis « tel quel » (docker appelé à la sortie) — la section 0 perdrait la
  propriété qui la justifie (aucune dépendance à l'hôte) ; conditionner la trap par
  `[ "${RECETTE_SECTION:-}" = 0 ] || trap …` — laisse `openssl` avant la section 0 et duplique le test.
- **Réversible** : oui.

### C-26-3-22 — 26-3 (dev) : la boucle lit la ligne entière, puis la découpe ; trois séparateurs exactement

- **Contexte** : l'AC 5.2 prescrit `while IFS='|' read -r ETAT IMAGE SERVICE NOM RESTE`. Mesuré au
  poste : sous cette forme, une ligne `|||` (quatre champs vides) est indiscernable d'une ligne vide
  — elle serait ignorée au lieu d'être refusée —, et une ligne `a|b|c|d|` (séparateur final) laisse
  `RESTE` vide : elle passerait pour lisible.
- **Retenu** : `while IFS= read -r LIGNE` (ligne vide → ignorée), puis
  `IFS='|' read -r ETAT IMAGE SERVICE NOM RESTE <<< "$LIGNE"` et un contrôle de **trois `|`
  exactement** (`SEPARATEURS=${LIGNE//[^|]/}`), en plus des champs `ETAT`, `IMAGE`, `NOM` non vides
  et de `RESTE` vide. Toujours dans le shell courant (`done <<< "$LISTE"`). La clause de G16 (AC 7.5)
  lit le `--format` à `|` et `IFS='|' read -r ETAT IMAGE SERVICE NOM RESTE` — elle tient cette forme.
  *(Revue de code P4, 2026-10-11 : elle exige aussi que le `done` fermant cette boucle soit
  `done <<< "$LISTE"`, lu par profondeur — un commentaire ou une autre boucle ne la satisfont pas.)*
- **Écarté** : la forme littérale de l'AC 5.2 (les deux trous ci-dessus).
- **Réversible** : oui.

### C-26-3-23 — 26-3 (dev) : renvois du manuel utilisateur par `\label` neufs ; la 26-2 non mergée

- **Contexte** : AC 1.2, 2.2 et 2.6 — la 26-2 (`sec:comptes-bancaires`) n'est pas mergée au
  développement (`main` = `9b724027`).
- **Retenu** : renvois au texte (*Administration → Comptes bancaires*) et `\label` neufs, posés sur
  des titres existants : `sec:ajout-compte`, `sec:solde-comptable`, `sec:import-dans-kesh`,
  `sec:import-csv` (AC 3.1), `sec:import-csv-profil`. À la fusion avec la 26-2, les renvois de
  l'AC 1.2 pourront porter `\S\ref{sec:comptes-bancaires}` — signalé à l'orchestrateur.
- **Écarté** : citer `sec:comptes-bancaires` avant qu'il existe (`??` au PDF).
- **Réversible** : oui.
