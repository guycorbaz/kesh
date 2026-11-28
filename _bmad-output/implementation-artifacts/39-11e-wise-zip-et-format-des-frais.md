# Story 39-11e : le zip de Wise tel que téléchargé, et le bon format des frais

Status: done

*Fiche rédigée le 2026-11-22, branche `story/39-11e` partie d'`epic/39-wise` (`b3826e2d`), numéros de ligne relevés à
ce commit. Epic 39, **phase 1 KISS** (amende G1 et tranche Q-39-11). **Dépend de** : 39-11a et 39-11a2 (lecteur),
39-11b (route `POST /api/v1/wise-imports`), toutes `done`. **Suivie par** : 39-11d (l'écran envoie le zip et affiche
les refus neufs ; fiche amendée le même jour, voir § *Raccords*). **Aucune migration**, aucune route neuve.
Besoin du responsable de l'epic (22.11), pour ses comptes 2025.*

## Story

En tant que **Comptable**, je veux donner à Kesh le zip que Wise me fait télécharger, **tel quel**, par l'écran ou
par l'API, et que Kesh refuse clairement un relevé exporté sans les frais séparés, afin d'importer mes trois soldes
en un geste et de ne jamais compter des frais dans le montant d'une ligne.

## Constats sur les relevés réels (métadonnées seules)

Source : les zips téléchargés de Wise par le responsable de l'epic (`data-not-committed/2025/11-wise/`, lus en
place, rien recopié ; mesure par un script hors dépôt, `python3 -I`, comptes seuls). Recherche :
`kesh-gate-logs/epic-39-40-kiss/wise-conversion-partielle-recherche.md`, « Mesure contre les relevés réels », § 3.

- **Le zip CSV contient un fichier par solde**, à la racine, sans dossier : `statement_<n>_<DEVISE>_<AAAA-MM-JJ>_
  <AAAA-MM-JJ>.csv` (`<n>` = un numéro de solde Wise, `<DEVISE>` = `CHF`, `EUR`, `USD`), trois entrées, ordre
  quelconque dans le répertoire du zip (il change d'un téléchargement à l'autre), méthode de compression ordinaire.
  Les zips CAMT et PDF ont la même forme (`.xml`, `.pdf`) : un zip de la mauvaise forme ne porte **aucun** `.csv`.
- **Avec ou sans « Afficher les transactions dont les frais sont indiqués séparément »** (libellé de l'écran
  d'export de Wise en français) : même en-tête (23 colonnes), mêmes soldes d'ouverture et de clôture, solde courant
  continu dans les deux. « Avec » ajoute une ligne `FEE-<id>` par opération à frais et met le montant **hors frais**
  sur la ligne d'origine ; « sans » n'a **aucune** ligne `FEE-` et met le montant **frais compris**, les frais restant
  lisibles dans *Total fees*. Les deux passent aujourd'hui tous les contrôles du lecteur : un relevé « sans »
  s'importerait, avec des montants frais compris et aucune écriture de frais possible (39-11c).
- **Mesure du critère** (2025 avec et sans, 2026 avec ; neuf fichiers) :

  | Relevés | lignes `FEE-` | lignes non `FEE-` à *Total fees* ≠ 0 | dont sans leur `FEE-<id>` |
  |---|---|---|---|
  | « avec », 2025 et 2026 (six fichiers) | 111 | 111 | **0** |
  | « sans », 2025 (trois fichiers) | **0** | 50 | 50 |

  Dans les fichiers « avec », chaque ligne `FEE-<id>` a sa ligne d'origine dans le fichier et vaut l'opposé de son
  *Total fees* (111 sur 111) ; les lignes `FEE-` y portent elles-mêmes *Total fees* = `0`.

## Décisions

- **D1 — Le zip est une forme d'envoi, non un format.** Un champ `file` dont les octets commencent par la signature
  locale du zip (`PK\x03\x04`, ou `PK\x05\x06` d'une archive vide) est lu comme une archive, **quel que soit son nom
  ou son type annoncé** ; tout autre champ `file` reste un CSV, comme aujourd'hui. Les CSV extraits rejoignent la
  liste des fichiers du formulaire et suivent **exactement** le chemin de 39-11b (lecteur, devise en double, devise
  sans compte, verrous, doublons, continuité, empreinte, compte rendu). Un zip et des CSV dans la même requête sont
  admis ; un même solde donné deux fois (dans le zip et à part) est refusé par la garde existante
  `WISE_DUPLICATE_CURRENCY_FILE`.
- **D2 — Tout ou rien, comme les fichiers à part.** Un CSV illisible du zip (ou au mauvais format des frais) refuse
  **toute** la requête, en nommant le fichier et l'archive, et rien n'est écrit. Justification : la route de 39-11b
  est déjà tout ou rien pour plusieurs fichiers (une seule transaction de base, § AC-4 de 39-11b), et les soldes d'un
  même compte Wise ne sont pas indépendants — une conversion porte ses deux moitiés sur deux soldes, un frais de
  conversion sur un troisième ; importer CHF et EUR sans USD ferait paraître à la revue (39-11c) des moitiés de
  conversion manquantes qui ne le sont pas. Le rapport de *Documents à traiter* (`crates/kesh-api/src/inbox_import.rs`,
  en-tête du module, Story 34-9) traite chaque fichier à part parce que des pièces sont **indépendantes** ; il en garde
  la leçon qui vaut ici : **chaque fichier est nommé**, au compte rendu d'un import réussi (une ligne par fichier, son
  archive comprise) comme au refus (le fichier et l'archive). Ce qui se corrige d'un geste (réexporter le zip entier)
  ne gagne rien à un import partiel.
- **D3 — Quelles entrées sont lues.** Seules les entrées **fichier, à la racine** de l'archive, dont le nom finit par
  `.csv` (sans casse). Une entrée dans un dossier (y compris un dossier de métadonnées qu'ajoutent certains outils de
  compression) ou d'une autre extension (`.xml`, `.pdf`, un `.zip` imbriqué : jamais ouvert) est **ignorée** et
  **nommée** au compte rendu (`ignoredEntries`). Une archive sans aucune entrée retenue est refusée (`NO_CSV` : le
  message dit d'exporter le relevé **au format CSV**, le cas réel d'un zip CAMT ou PDF). Ordre des CSV retenus : par
  nom (ordre déterministe ; celui du répertoire du zip ne l'est pas).
- **D4 — Bornes (zip bomb) et chemins.** Rien n'est écrit sur disque : tout se décompresse en mémoire.
  - **nombre d'entrées** de l'archive (répertoire central, `ZipArchive::len()`, lu avant toute décompression) :
    au plus **64** (`WISE_ZIP_MAX_ENTRIES` ; trois aujourd'hui, une par solde, la marge couvre un compte à nombreuses
    devises) — au-delà, `TOO_MANY_ENTRIES` ;
  - **taille décompressée cumulée** de toutes les entrées retenues de **toutes** les archives de la requête : au plus
    la limite du corps de la route (`bank_import_max_mib` Mio, `crates/kesh-api/src/lib.rs:977-980`) — ce qui ne
    s'enverrait pas décompressé ne passe pas compressé —, comptée **sur les octets réellement produits**, par une
    lecture à travers `take(reste + 1)`, **jamais** sur la taille que déclare l'en-tête (déclarative) — au-delà,
    `TOO_LARGE` ;
  - **chemin** : une entrée dont le nom est absolu, sort de la racine par `..` (`enclosed_name()` rend `None` ; un
    `..` qui reste dans l'archive, `a/../x.csv`, ou un `./x.csv` passent `enclosed_name` mais contiennent un `/` : entrée
    « dans un dossier », ignorée par D3), contient une barre oblique inverse `\`, ou contient un deux-points `:` (lettre
    de lecteur, `C:x.csv` : `enclosed_name()` ne la refuse que sous Windows, d'où un contrôle explicite du `:`) refuse
    **toute l'archive** (`UNSAFE_PATH`) — rien n'étant écrit sur disque, ce n'est pas une évasion, mais un tel nom ne
    vient pas de Wise ;
  - entrée **chiffrée** → `ENCRYPTED` ; méthode de compression autre que *stored* ou *deflate* (les seules du crate
    `zip` tel qu'il est configuré, `crates/kesh-api/Cargo.toml:66`) → `UNSUPPORTED_COMPRESSION` ; archive illisible
    (répertoire central faux, CRC faux à la lecture) → `UNREADABLE`. Ces deux métadonnées se lisent par
    `ZipArchive::by_index_raw(i)` (sans décompresser), **jamais** par `by_index(i)`, qui rend une erreur avant de
    construire l'entrée quand elle est chiffrée ou de méthode non gérée (`zip-2.6.1/src/read.rs:1113-1114`,
    `Decompressor::new`) et ferait dire `UNREADABLE` à la place ; seules les entrées retenues se rouvrent ensuite par
    `by_index(i)` pour être lues.
  - **Portée et ordre des contrôles** (la `reason` rendue en dépend) : `UNREADABLE` (ouverture de l'archive), puis
    `TOO_MANY_ENTRIES`, puis **entrée par entrée dans l'ordre du répertoire** : le chemin (`UNSAFE_PATH`) sur **toutes**
    les entrées, retenues ou non ; le chiffrement (`ENCRYPTED`) et la méthode (`UNSUPPORTED_COMPRESSION`) sur les
    seules entrées **retenues** (D3 : un `.pdf` chiffré est ignoré, jamais ouvert) ; puis la lecture des retenues, par
    nom (`UNREADABLE` sur un CRC faux, `TOO_LARGE`) ; enfin `NO_CSV`. La première faute trouvée l'emporte.
  - Angle mort assumé, le même que celui de la sauvegarde (`crates/kesh-api/src/admin_backup/import.rs:27-31`) :
    `ZipArchive::new` décode tout le répertoire central avant qu'on puisse compter les entrées ; sa mémoire n'est
    bornée que par la taille reçue (limite du corps). Deux entrées de même nom : le crate n'en garde qu'une, sans le
    dire.
- **D5 — Le bon format des frais, au lecteur.** Une ligne « à frais » est une ligne non `FEE-` dont *Total fees* est
  non nul ; elle « a sa `FEE-` » quand le fichier porte une ligne `FEE-<id>` de même identifiant (`FEE-` + son
  *TransferWise ID*). Un relevé est refusé `WISE_CSV_FEES_NOT_SEPARATED` quand **(a)** il ne porte **aucune** ligne
  `FEE-` et au moins **une** ligne à frais, **ou (b)** au moins **deux** lignes à frais n'ont pas leur `FEE-` (règle (b)
  ajoutée le 2026-11-22, L5 de la validation P1 : une ligne `FEE-` isolée ne suffit plus à faire passer un relevé
  « sans »). Critère **par fichier, avec une tolérance d'une ligne** dans un fichier qui porte des `FEE-` : la ligne
  `FEE-` est écrite une milliseconde avant sa ligne d'origine, si bien qu'une origine posée dans la toute première
  milliseconde de la période exportée peut avoir sa `FEE-` hors période — cela ne peut arriver qu'à **une** ligne, la
  première. Un fichier « sans » n'a jamais de ligne `FEE-` (0 sur trois fichiers réels, 50 lignes à frais) : la règle
  (a) le reconnaît dès qu'il porte un frais. **Limite, écrite au manuel (L6)** : un bon fichier dont la **seule** ligne
  à frais tombe à la borne (sa `FEE-` hors période) est refusé par (a) — cas rare, bruyant et réparable : réexporter
  en élargissant la période d'un jour. Le choix inverse (tolérer aussi ce cas) laisserait passer **en silence** un
  relevé « sans » à une seule opération à frais, avec un montant faux : un refus se voit, une écriture fausse non.
  - **Relevé sans aucun frais sur la période** (aucune ligne `FEE-`, *Total fees* nul ou vide partout) : **accepté**.
    Les deux exports sont alors identiques ligne pour ligne (rien à séparer, aucun montant frais compris) : il n'y a ni
    moyen ni raison de les distinguer. Écrit au manuel et testé.
  - **Fichier mixte** (des lignes `FEE-`, et **une** ligne à frais sans sa `FEE-<id>`) : **accepté** (jamais observé ;
    la forme d'un bon fichier coupé à la borne de période). Deux lignes à frais ou plus sans leur `FEE-`, même avec
    d'autres lignes `FEE-` : **refusé** (b).
  - Colonne *Total fees* absente (colonne facultative du lecteur, `OPTIONAL_COLUMNS`, `crates/kesh-import/src/wise.rs:91`) :
    rien à juger, accepté.
  - Le contrôle vient **après** les contrôles existants du lecteur (`parse_wise_csv`, `wise.rs:451-509`, contrôles de
    fichier `:491-494` : un fichier
    cassé garde son refus d'aujourd'hui) ; il vaut pour **tout** relevé Wise, envoyé à part ou dans un zip.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-11e-1` | `POST /api/v1/wise-imports` accepte, dans un champ `file`, le zip de Wise tel que téléchargé ; ses CSV vont chacun au solde de leur devise, comme des fichiers à part. | AC-1, AC-3 |
| `FR-39-11e-2` | Seules les entrées `.csv` à la racine sont lues ; nombre d'entrées, taille décompressée et chemins bornés ; archive refusée par un code fermé. | AC-2 |
| `FR-39-11e-3` | Un relevé exporté sans les frais séparés est refusé par un code neuf, message aux quatre locales qui dit de réexporter en cochant l'option ; un relevé sans frais sur la période est accepté. | AC-4 |
| `FR-39-11e-4` | Le compte rendu de la réponse `201` nomme chaque fichier, son archive, ses lignes lues, importées et ignorées comme doublons, ses soldes, et les entrées ignorées ; chaque import créé se relit ensuite par l'API (sans l'archive ni les entrées ignorées). | AC-3, AC-5 |
| `NFR-39-11e-1` | DRY : lecture bornée d'une entrée de zip partagée avec la sauvegarde ; chemin d'import de 39-11b inchangé ; aucune migration, aucune route neuve. | AC-6 |
| `NFR-39-11e-2` | `docs/api-external.md` (exemple `curl`) et manuel utilisateur (`.tex`) disent comment exporter et ce que Kesh accepte. | AC-5, AC-7 |

## Critères d'acceptation

1. **AC-1 — Le zip à la route.** `POST /api/v1/wise-imports` (handler `wise_imports::create`,
   `crates/kesh-api/src/routes/wise_imports.rs:361`) : après `parse_form` (`:254`) et le `404
   PROVIDER_ACCOUNT_NOT_FOUND` (`:375-379`), chaque `UploadedFile` dont les octets portent la signature du zip (D1) est
   remplacé, **à sa place dans l'ordre du formulaire**, par ses CSV retenus (D3, par nom), chacun avec `filename` = le
   nom de l'entrée (rogné à 255 octets à la frontière d'un caractère, `truncate_bytes`, comme aujourd'hui `:269`) et
   `archive` = le nom du champ zip ; la suite du handler est **inchangée** et reçoit cette liste. L'empreinte d'un
   fichier reste celle de ses **octets CSV** ‖ devise (`file_currency_hash`, `:397`) : un relevé importé à part puis
   dans son zip (ou l'inverse) est refusé `422 BANK_IMPORT_DUPLICATE_FILE`, comme un réimport. La décompression
   (synchrone) s'exécute en `tokio::task::spawn_blocking`, comme la lecture de la sauvegarde. Les trois CSV fictifs
   de 39-11a (`crates/kesh-import/tests/fixtures/wise-releve-{chf,eur,usd}-fictif.csv`) zippés **dans le test** (aucun
   zip binaire commité) donnent le **même** compte rendu, aux champs neufs près, que les trois fichiers envoyés à part.
2. **AC-2 — Refus d'archive : `422 WISE_ZIP_INVALID`**, rien d'écrit, `details: { filename, reason, entry? }`
   (`filename` = l'archive ; `entry` = l'entrée en cause quand il y en a une), `reason` ∈ `UNREADABLE`,
   `TOO_MANY_ENTRIES` (avec `max`), `TOO_LARGE` (avec `maxBytes`), `UNSAFE_PATH`, `ENCRYPTED`,
   `UNSUPPORTED_COMPRESSION`, `NO_CSV` (D3, D4). Place dans l'ordre des refus : après le `404`, **avant** `422
   WISE_CSV_INVALID` ; plusieurs archives : la première refusée, dans l'ordre du formulaire. Message : `NO_CSV` a le
   sien (« L'archive « { $filename } » ne contient aucun relevé CSV : exportez de Wise le relevé au format CSV. Rien
   n'a été importé. »), les autres raisons un message commun (« L'archive « { $filename } » n'est pas un zip de
   relevés Wise que Kesh peut lire. Rien n'a été importé. ») ; quatre locales. Tests unitaires (archives fabriquées
   par `zip::ZipWriter` dans le test) : une entrée par raison ; dossier et `.pdf` ignorés et nommés ; `.CSV` en
   majuscules retenu ; `../x.csv`, `/x.csv`, `a\x.csv` refusés ; **zip bomb** — une entrée déclarée de 10 octets qui
   en produit plus que la borne (ou une entrée hautement compressible au-delà de la borne) refusée `TOO_LARGE` sans
   allouer au-delà de borne + 1 ; borne cumulée sur deux archives d'une même requête ; 65 entrées refusées, 64
   admises.
3. **AC-3 — Compte rendu.** La réponse `201` de 39-11b (`WiseImportResponse`, `:118-123`) gagne, sans rien retirer :
   `FileReport.rows` (lignes lues du fichier, = `inserted + duplicates`), `FileReport.archive` (`string | null`), et
   `ignoredEntries: [{ archive, entry }]` (toujours présent, vide sans zip). La trace `wise_import.created` porte ces
   mêmes champs (sa clé `files` sérialise `FileReport`, `:570-574` ; ajouter `ignoredEntries`). **Noms bornés** :
   chaque nom d'entrée ignorée (`ignoredEntries[].entry`) et `details.entry` d'un refus sont rognés à 255 octets à la
   frontière d'un caractère par le même `truncate_bytes` que le nom retenu, et leurs caractères de contrôle remplacés
   par `?` — un nom de zip peut faire 65 535 octets, 64 entrées tiendraient plusieurs Mio dans la réponse et l'audit. « Soldes contrôlés » :
   un fichier accepté a eu son solde courant contrôlé ligne à ligne (le lecteur refuse sinon,
   `WISE_CSV_RUNNING_BALANCE_BREAK`) — dit par la documentation, sans champ neuf. **Ce qui se relit** : le compte
   rendu complet n'est rendu **qu'à l'envoi**, par la réponse `201` ; perdue, un nouvel envoi du même zip est refusé
   `BANK_IMPORT_DUPLICATE_FILE`. Ensuite, chaque import créé se relit par `GET /api/v1/bank-imports/{importId}` (clé
   `read` admise) : il rend `BankImportResponse` (`crates/kesh-api/src/routes/bank_imports.rs:295` : `filename` = le
   nom de l'**entrée** CSV, période, soldes d'ouverture et de clôture, `transactionCount`, `importedAt`) et ses
   transactions — **ni** le nom de l'archive, **ni** `rows`, **ni** `ignoredEntries`, qui n'ont pas de colonne
   (l'étendre demanderait une migration, hors périmètre). La trace `wise_import.created`, qui porte tout, n'est lisible
   qu'à l'écran (le journal d'audit est fermé aux clés, `docs/api-external.md` § 4). La documentation dit ces trois
   points. Les tests de 39-11b gardent leurs assertions ; une
   assertion qui compare la réponse ou la trace **entière** gagne les champs neufs, déclarée au Change Log.
4. **AC-4 — Le bon format des frais.** Au lecteur (`crates/kesh-import/src/wise.rs`) : variante neuve
   `WiseCsvError::FeesNotSeparated { lines: Vec<usize>, count: usize }` (`lines` = les lignes à frais **sans leur
   `FEE-`**, au plus `MAX_CSV_LINE_ERRORS`, `crates/kesh-import/src/error.rs:49` ; `count` = leur nombre), code
   `WISE_CSV_FEES_NOT_SEPARATED` (`WiseCsvError::code`, `:326`), contrôle après `check_running_balance` (`:494`),
   règle D5. À l'API : refus **propre** `422 WISE_CSV_FEES_NOT_SEPARATED` (et non `WISE_CSV_INVALID`, pour que le
   message soit une consigne), à la place de `WISE_CSV_INVALID` dans l'ordre des refus (le premier fichier refusé, dans
   l'ordre de la liste développée), `details: { filename, archive | null, lines, count }`. Message FR : « Le relevé
   « { $filename } » a été exporté sans l'option « Afficher les transactions dont les frais sont indiqués
   séparément » : ses montants comprennent les frais. Exportez-le de nouveau depuis Wise en cochant cette option. Rien
   n'a été importé. » ; DE, IT, EN **décrivent** l'option (« l'option qui affiche les frais comme des transactions
   séparées ») sans en citer un libellé entre guillemets, faute de l'avoir relevé dans ces langues — choix au Change
   Log. `WISE_CSV_INVALID` du même fichier (forme cassée) passe avant. Tests (lecteur, entrées fabriquées à partir des
   fixtures fictives) : relevé « sans » (lignes `FEE-` retirées, montant de l'origine frais compris, *Total fees*
   gardé, solde courant recalculé) refusé, lignes et nombre justes ; les trois fixtures acceptées ; relevé sans aucun
   frais accepté ; fichier mixte à **une** ligne sans `FEE-` accepté ; fichier à `FEE-` dont **deux** lignes à frais
   n'ont pas la leur refusé (b) ; relevé sans `FEE-` à une seule ligne à frais refusé (a, limite écrite) ; *Total fees* `0`,
   `0.00` ou vide = nul ; colonne *Total fees* absente acceptée. Inventaire **avant** d'écrire le contrôle : toute entrée de test existante (`crates/kesh-import/tests/
   wise_csv.rs`, tests de `wise.rs`, `crates/kesh-api/tests/wise_import_e2e.rs`, tests de `wise_imports.rs`) qui
   porte une ligne à *Total fees* non nul sans aucune ligne `FEE-` — elle deviendrait rouge : chaque cas modifié
   (ajout de la ligne `FEE-` ou *Total fees* mis à zéro) est nommé au Change Log, avec la raison ; aucune attente
   d'un autre test n'est changée.
5. **AC-5 — `docs/api-external.md`**, § « Le compte Wise et l'import de ses relevés » (`:1007-1011`) : le champ `file`
   admet le zip (D1 à D4 en une phrase chacune, les bornes nommées), l'ordre des refus complété (`WISE_ZIP_INVALID`
   après le `404`, `WISE_CSV_FEES_NOT_SEPARATED` au rang de `WISE_CSV_INVALID`), la réponse avec `rows`, `archive`,
   `ignoredEntries`, et ce qui se relit (AC-3 : le compte rendu à l'envoi seulement ; `GET /bank-imports/{id}` sans
   archive ni entrées ignorées ; la trace à l'écran). **Exemple `curl`** (clé `read-write`, nom de compte et fichier fictifs,
   `curl -F "account=…" -F "file=@…zip"`, réponse abrégée). Deux lignes neuves à la table des erreurs (`:1315-1316`,
   même forme). Pas d'outil MCP.
6. **AC-6 — DRY et non-régression.** La lecture bornée d'une entrée (`lire_borne`,
   `crates/kesh-api/src/admin_backup/import.rs:151-164`, `take(max + 1)`) est **extraite** en une fonction partagée
   (par exemple `crate::zip_read::read_bounded(entry, max) -> io::Result<Option<Vec<u8>>>`, `None` au-delà de la
   borne) que `lire_borne` appelle, **messages et erreurs de la sauvegarde inchangés** (tests de `admin_backup`
   verts sans retouche) ; la lecture des CSV d'une archive vit dans un module de `kesh-api` (le crate `zip` y est
   déjà, `Cargo.toml:66` ; `kesh-import`, publiable et sans dépendance, n'en prend pas). Le chemin de 39-11b, de la
   liste des fichiers au `COMMIT`, n'est pas modifié ; aucune assertion des tests PayPal ni des imports CAMT/CSV ne
   change. Aucune route neuve : `audit_route_registry.rs` et la phrase du manuel d'administration inchangés.
7. **AC-7 — Manuel utilisateur** (`docs/manual/fr/user-manual.tex`, `.tex` seul, PDF à la release ; DE, IT, EN « à
   traduire » selon la règle du dépôt) : sous-section neuve « Exporter les relevés de Wise »
   (`\label{sec:wise-export}`) dans § *Import bancaire* (`\label{sec:import-bancaire}`, `:3012`), que 39-11d citera :
   choisir la période, le format **CSV**, **cocher** l'option des frais séparés (et pourquoi : sinon les montants
   comprennent les frais et Kesh refuse le relevé) ; le zip se donne **tel quel**, sans le décompresser ; un relevé
   sans frais sur la période passe ; la limite de D5 (un refus alors que l'option est cochée : élargir la période
   d'un jour) ; ce que Kesh ignore dans un zip ; un refus n'importe rien. Gardes de
   `crates/kesh-api/tests/textes_coherents.rs` lues avant d'écrire (aucun numéro de compte du plan). Contrôle local du
   PDF permis, aplati (`pdftotext … | tr '\n' ' ' | tr -s ' '`), PDF non commité.

## Tâches

*Ordre des commits (L9 de la validation P1 : la story n'est pas scindée)* : T1 (D5, livrable seul, protège déjà
l'import à plat) **d'abord, dans un commit séparé**, puis T2 et T3 (zip). Si une passe de revue de code stagne, c'est
la couture d'un découpage.

- [x] T0 — Relire au code les lignes citées ; inventaire de l'AC-4 (entrées de test à *Total fees* sans `FEE-`),
  écrit au Change Log avant tout changement.
- [x] T1 — Lecteur : `FeesNotSeparated` et son contrôle, tests (AC-4).
- [x] T2 — `read_bounded` extrait, `lire_borne` réécrite dessus (AC-6) ; module de lecture des CSV d'une archive,
  bornes et refus, tests unitaires (AC-2).
- [x] T3 — Route : développement des zips, `rows`/`archive`/`ignoredEntries`, trace, erreurs `WISE_ZIP_INVALID` et
  `WISE_CSV_FEES_NOT_SEPARATED`, quatre locales ; tests HTTP (AC-1, AC-3, AC-4).
- [x] T4 — `docs/api-external.md`, manuel utilisateur (AC-5, AC-7) ; *Note de version*.

## Stratégie de tests

- **Lecteur** (`crates/kesh-import/tests/wise_csv.rs`) : cas de l'AC-4.
- **Archive** (tests unitaires du module neuf) : cas de l'AC-2 ; archives fabriquées en mémoire par `zip::ZipWriter`.
- **HTTP** (`crates/kesh-api/tests/wise_import_e2e.rs`, `#[sqlx::test]` sur le squash, montage de 39-11b) : zip des
  trois fixtures → trois entrées, même compte rendu qu'à part (AC-1) ; zip + CSV de même devise → `422
  WISE_DUPLICATE_CURRENCY_FILE` ; CSV à part puis même CSV dans un zip → `422 BANK_IMPORT_DUPLICATE_FILE` ; zip à un
  CSV « sans » → `422 WISE_CSV_FEES_NOT_SEPARATED`, `details.archive`, rien d'écrit (aucune entrée `bank_imports`,
  aucune transaction) ; zip sans CSV (`.pdf` seuls) → `422 WISE_ZIP_INVALID` `NO_CSV` ; ordre des refus (`404` avant
  une archive illisible, archive illisible avant un CSV illisible) ; clé `read-write` admise, clé `read` → `403`
  (comme 39-11b) ; `ignoredEntries` et trace `wise_import.created`.
- **Sondes négatives** (mutations à jouer et consigner) : borne cumulée remplacée par la taille déclarée ; critère D5
  passé « par ligne », seuil 1 (le fichier mixte à une ligne doit rougir) ; seuil de (b) passé à 3 (le refus à deux
  lignes doit rougir) ; règle (a) retirée (le relevé « sans » à une ligne doit rougir) ; `enclosed_name` ignoré ; contrôle du `:` retiré ; métadonnées lues par `by_index` au lieu de
  `by_index_raw` (`ENCRYPTED` doit rougir) ; rognage des noms ignorés retiré.
- **Relevés réels, test ignoré** (comme `real_statements_are_read`) : un test `#[ignore]` lit les zips du dossier que
  nomme une variable d'environnement (aucun chemin écrit dans le dépôt, rien recopié) ; les zips « avec » s'importent
  (lecteur et archive), le zip « sans » est refusé `WISE_CSV_FEES_NOT_SEPARATED`.
- **Aucune donnée réelle** : fixtures fictives de 39-11a et entrées dérivées dans les tests ; aucun zip réel ni nom de
  fichier réel commité (le gabarit `statement_<n>_<DEVISE>_…` des tests prend un `<n>` inventé).
- Gate ciblé entre les passes ; gate complet, `scripts/gate-ci.sh` et E2E complet au dernier commit de code (aucun
  écran touché, mais l'E2E complet reste la règle au dernier commit de code).

## Modules touchés (4)

`kesh-import` (lecteur), `kesh-api` (route, module d'archive, `admin_backup/import.rs` pour l'extraction de
`read_bounded`, erreurs), `kesh-i18n` (quatre locales), docs (`docs/api-external.md`, `user-manual.tex`).

## Raccords

- **39-11d** (amendée le 2026-11-22) : le bouton d'import accepte aussi le zip (`accept=".csv,text/csv,.zip,
  application/zip"`), traduit `WISE_ZIP_INVALID` et `WISE_CSV_FEES_NOT_SEPARATED` par leur code, affiche `archive`
  et `ignoredEntries` au compte rendu ; son manuel renvoie à `sec:wise-export` au lieu de dire « le zip n'est pas lu ».
- **39-14** (raccord avec le cœur `provider_imports.rs` de la 38-12a) : cette story touche la **lecture** du
  formulaire et la forme de `FileReport`, non le cœur d'écriture ; au raccord, `rows`, `archive` et `ignoredEntries`
  sont à garder dans la construction du compte rendu (point 3 et point 7 de la liste « À raccorder » de 39-11b).

## Note de version

*Ajouté* — Le zip de relevés que Wise fait télécharger s'importe tel quel, sans le décompresser : Kesh en lit les
relevés CSV, un par solde, et dit ce qu'il a ignoré. Un relevé exporté sans l'option « frais indiqués séparément »
est refusé avec la marche à suivre, pour que les frais ne se perdent jamais dans le montant d'une ligne.
*Migration* — aucune.

## Notes de développement

- `zip` 2.6.1 (`Cargo.lock`), `default-features = false, features = ["deflate"]` : `ZipArchive::new`, `len()`,
  `by_index_raw(i)` pour les métadonnées (`ZipFile::{name, enclosed_name, is_dir, encrypted, compression}`, sans
  décompresser), puis `by_index(i)` pour lire les seules entrées retenues (D4) ; une erreur de CRC ne paraît qu'en
  fin de lecture d'une entrée (lire jusqu'au bout avant de l'accepter).
- Les octets d'un champ `file` sont un `Bytes` : `Cursor` dessus pour `ZipArchive`, sans copie.
- La limite du corps de la route (`DefaultBodyLimit`, `lib.rs:977-980`) borne déjà la taille **compressée** reçue.
- Le contrôle D5 ne lit que `is_fee` et `total_fees` de `WiseRow` (`wise.rs:133`, `:169`), déjà lus.

## Dev Agent Record

*Développé le 2026-11-22 (Claude Opus 5.5), branche `story/39-11e`. Commits : `e18be4cb` (D5, seul, d'abord, L9),
`3174200e` (zip), `ac3e1fe4` (assertion `rows` avec doublons) — dernier commit de code `ac3e1fe4`.*

### Ce qui est livré

- **D5** (`crates/kesh-import/src/wise.rs`) : `check_fees_separated`, après `check_running_balance` ; règles (a) et (b) de
  D5 ; `WiseCsvError::FeesNotSeparated { lines, count }` (lignes triées, au plus `MAX_CSV_LINE_ERRORS`), code
  `WISE_CSV_FEES_NOT_SEPARATED`. À l'API, `AppError::WiseCsvFeesNotSeparated` (`422`, `details: { filename, archive,
  lines, count }`) rendu par `csv_error` à la place de `WISE_CSV_INVALID`.
- **Archive** (`crates/kesh-api/src/zip_read.rs`, neuf, `pub`) : `read_bounded` (partagée ; `lire_borne` de
  `admin_backup/import.rs` réécrite dessus, messages inchangés), `looks_like_zip`, `extract_csv_entries` (D3, D4 : 
  métadonnées par `by_index_raw`, lecture des retenues par `by_index`, ordre des contrôles de la fiche, budget cumulé).
- **Route** (`routes/wise_imports.rs`) : `expand_archives` en `spawn_blocking` après le `404`, `WISE_ZIP_MAX_ENTRIES` = 64,
  budget = `bank_import_max_mib` Mio ; `entry_name` (caractères de contrôle → `?`, 255 octets) pour les noms retenus,
  ignorés et `details.entry` ; `FileReport.{archive, rows}`, `WiseImportResponse.ignoredEntries`, `IgnoredEntry` ; trace
  `wise_import.created` avec `ignoredEntries`. `AppError::WiseZipInvalid` (`422 WISE_ZIP_INVALID`, `max`/`maxBytes`).
- **i18n** : trois clés (`error-wise-csv-fees-not-separated`, `error-wise-zip-no-csv`, `error-wise-zip-invalid`) aux
  quatre locales, replis de `errors.rs` égaux au fr-CH. Choix (AC-4) : DE, IT, EN décrivent l'option sans en citer de
  libellé.
- **Docs** : `docs/api-external.md` (ordre des refus, réponse, paragraphe du zip, exemple `curl`, deux lignes à la table
  des erreurs) ; manuel `sec:wise-export` (`.tex` seul ; PDF contrôlé dans une copie hors dépôt, `latexmk -xelatex` vert,
  texte aplati relu ; aucun PDF commité). DE, IT, EN : à traduire.

### Inventaire de l'AC-4 (T0)

Entrées de test portant un *Total fees* non nul, relevées au commit `a9d5130d` : les trois fixtures (chaque ligne à
frais a sa `FEE-`) et `wise_csv.rs:450` (`Total fees` = « 0,35 », refusé avant par le lecteur, `AMOUNT`). Aucune autre
(`ligne()` de l'E2E et `wise_line()` laissent la colonne vide). **Aucune entrée existante modifiée, aucune attente changée** ;
les tests de 39-11b gardent leurs assertions (aucune ne compare la réponse ou la trace entière).

### Tests ajoutés (recomptés, `#[test]` + `#[sqlx::test]`, de `a9d5130d` à `ac3e1fe4`)

- `crates/kesh-import/tests/wise_csv.rs` : 34 → 42 (+8, D5).
- `crates/kesh-api/src/zip_read.rs` : 0 → 12 (archive, AC-2).
- `crates/kesh-api/tests/wise_import_e2e.rs` : 8 → 14 (+6 : D5 à plat, zip tel quel, à part puis zip et même devise,
  refus et clés, noms bornés ; `real_zips_are_read`, ignoré) ; plus une assertion `rows` dans
  `import_continuite_et_doublons_de_lignes`.
- Total : **26 tests neufs**, dont 1 ignoré.

### Relevés réels (test ignoré `real_zips_are_read`)

Joué le 2026-11-22 sur les zips du responsable de l'epic, hors dépôt, chemins passés par `KESH_WISE_REAL_ZIPS_ACCEPTED`,
`…_FEES_NOT_SEPARATED`, `…_NO_CSV` (rien recopié, seuls comptes et codes imprimés ; journal
`kesh-gate-logs/s39-11e/zips-reels.log`) : les **2** zips CSV « avec » s'ouvrent en 3 relevés chacun, tous lus ; le zip
CSV « sans » s'ouvre en 3 relevés, **les 3 refusés `WISE_CSV_FEES_NOT_SEPARATED`** ; les **7** zips CAMT et PDF refusés
`NO_CSV`. Vert.

### Sondes par mutation (hors dépôt, `kesh-gate-logs/s39-11e/mutations.sh`, `mutations.log`)

Copie `git archive` de `ac3e1fe4`, témoins verts d'abord (lecteur 41, archive 12, route 24 tests). **18 mutations, 18
tuées**, chacune par un test nommé en échec sur une assertion (`panicked at`) : D5 par ligne, seuil 3, règle (a) retirée,
contrôle retiré ; borne non cumulée, borne infinie, `enclosed_name` ignoré, `:` non contrôlé, métadonnées par `by_index`,
`.csv` sensible à la casse, dossier retenu, tri retiré, 65 entrées admises ; nom non rogné, caractères de contrôle gardés,
archive oubliée au refus, trace sans `ignoredEntries`, `rows` = insérées. (Un premier passage, qui recompilait tous les
binaires de test à chaque mutation, a été arrêté après le témoin et la première mutation, et relancé à portée étroite.)

### Gates (au dernier commit de code `ac3e1fe4`)

- Ciblés en cours : `binary(wise_csv)`, `binary(wise_import_e2e)`, `wise_imports`, `zip_read`, `admin_backup_e2e`,
  `admin_full_import_e2e` : 61 puis 166 passés.
- **Gate backend** (`scripts/test-fast.sh --ci`, base remise à zéro, `kesh-mariadb-dev6` par `heavy.sh`) : fmt, clippy
  workspace `-D warnings`, nextest **6498 passés, 17 ignorés** (journal `kesh-gate-logs/s39-11e/gate-backend.log`).
- **Frontend** : non joué — aucun fichier de `frontend/` touché.
- **E2E** : aucun — aucune spec ajoutée (règle de la nuit du 22.11). `gate-ci.sh` et E2E complet : à la fin de l'epic.

### Écarts et points relevés

- La limite de D5 (bon fichier à une seule ligne à frais coupée par la borne) est écrite au manuel, non levée.
- Limite de D5 relevée à la revue P1 (L3) : un relevé « sans » qui porte une ligne `FEE-` isolée et une seule ligne à frais sans
  sa `FEE-` passe (`fee_origins` non vide, `missing.len() < 2`) ; écrite au manuel et à l'API, issue `enhancement` P3 à ouvrir.
- Nom d'archive (`archive`) et `ignoredEntries` ne se relisent pas par `GET /bank-imports/{id}` (AC-3 ; une migration
  serait nécessaire).
- Écran (bouton qui accepte le zip, traduction des refus) : 39-11d.

## Change Log

- 2026-11-22 — fiche rédigée (Claude Opus 5.5), `drafted` ; constats mesurés sur les relevés réels (métadonnées
  seules). Validation P1 à lancer (`kesh-gate-logs/s39-11e/validation-p1-prompt.md`).
- 2026-11-22 — validation P1 (Claude Sonnet 5.5, contexte frais, rapport `kesh-gate-logs/s39-11e/validation-p1-rapport.md`) :
  **0 CRITICAL, 0 HIGH, 0 MEDIUM, 10 LOW**. Appliqués à la fiche : L1 (métadonnées par `by_index_raw`, D4 et Notes) ;
  L2 (lettre de lecteur refusée par un contrôle explicite du `:`, `..` refusé seulement s'il sort de la racine) ; L3
  (portée et ordre des contrôles, D4) ; L4 (noms d'entrées rognés à 255 octets, caractères de contrôle remplacés,
  AC-3) ; L5 (refus **aussi** dès deux lignes à frais sans leur `FEE-`, règle (b), D5, AC-4, sondes) ; L6 (limite
  écrite, D5 et AC-7 : un bon fichier à une seule ligne à frais coupée par la borne est refusé, réexporter sur une
  période élargie — préféré à une tolérance qui laisserait passer un montant faux en silence) ; L7 (ce que `GET /bank-imports/{id}` rend, exactement ; l'étendre
  demanderait une migration, non fait) ; L8 (39-11d : phrase contradictoire retirée, Change Log corrigé). L9 : pas de
  scission ; D5 développé en premier, dans un commit séparé (ordre écrit aux Tâches). L10 : hors périmètre (noms dans
  le fichier d'epic), rien ici. Citation `wise.rs:451-499` corrigée en `:451-509`. Aucune remédiation ne touchant du
  code et la passe ne rendant rien au-dessus de LOW, la boucle se clôt : `ready-for-dev`.
- 2026-11-22 — développement (Claude Opus 5.5) : D5 d'abord (`e18be4cb`), puis le zip (`3174200e`, `ac3e1fe4`) ; 26 tests
  neufs (1 ignoré), 18 mutations tuées, relevés réels vérifiés par le test ignoré, gate backend vert (6498 / 17 ignorés).
  `review` ; prompt de la revue P1 : `kesh-gate-logs/s39-11e/revue-p1-prompt.md`.
- 2026-11-22 — revue de code P1 (Claude Sonnet 5.5, contexte frais, rapport `kesh-gate-logs/s39-11e/revue-p1-rapport.md`) :
  **0 CRITICAL, 0 HIGH, 0 MEDIUM, 3 LOW** ; 107 tests rejoués, décomptes recomptés conformes. Tendance : validation P1 0/0/0/10
  (Sonnet) -> revue P1 0/0/0/3 (Sonnet). Les trois LOW sont documentaires, appliqués sans toucher la production : L1 (la
  borne de 64 compte toutes les entrées, dossiers compris : manuel et `docs/api-external.md`), L2 (deux entrées de même nom :
  la seconde est ignorée, écrit au manuel et à l'API), L3 (relevé « sans » avec une `FEE-` isolée et une seule ligne à frais :
  limite écrite au manuel, à l'API et ici ; issue `enhancement` P3 à ouvrir). Aucune passe de plus : aucune ligne de code de
  production touchée, aucun finding au-dessus de LOW. Dernier commit de code `ac3e1fe4`, gate backend vert (6498). Story `done`.
