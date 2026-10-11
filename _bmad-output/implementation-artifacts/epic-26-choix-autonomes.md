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
