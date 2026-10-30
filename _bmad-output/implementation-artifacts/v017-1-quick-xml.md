# Story v017-1 : release 0.17.0 — quick-xml 0.39 → 0.42, les deux dénis de service de l'import CAMT.053

Status: ready-for-dev

<!-- Fiche créée le 2026-10-30 (Opus 5.5, autonomie), worktree `kesh-0171`, branche `story/v017-1-quick-xml`,
     partie de `c5101b07` (tête de la candidate 0.16.1-rc1). Issue : #789 (P1, sécurité), en `refs` seulement.
     Registre : `v017-1-quick-xml-choix-autonomes.md` (C-v017-1-1 à C-v017-1-10). -->

## Story

En tant qu'**Administrateur d'une installation Kesh qui tient des livres réels**,
je veux que **l'import d'un relevé CAMT.053 ne puisse plus monopoliser un cœur ni épuiser la mémoire du serveur**,
afin qu'**un fichier forgé ne rende pas Kesh indisponible — sans qu'un relevé déjà importé en 0.16 change de sens**.

## Origine

- **#789** (P1, `bug`) : `cargo audit` du 2026-10-27 signale deux avis de sévérité 7,5 sur `quick-xml` 0.39.4 —
  **RUSTSEC-2026-0194** (temps quadratique du contrôle des attributs en double) et **RUSTSEC-2026-0195** (allocation non
  bornée des déclarations d'espace de noms dans `NsReader`). Correctif amont : `quick-xml >= 0.41.0`. Décision de Guy du
  2026-10-27 : traiter après la release 0.16.0.
- **Contrainte du Project Lead du 2026-10-30** (`kesh-gate-logs/prompts/v017-donnees-a-garder.md`, et message de
  l'orchestrateur) : l'installation du NAS (0.16.1-rc1) porte des données que Guy garde. **Un relevé CAMT.053 déjà
  importé en 0.16.x doit rester lisible et inchangé après la montée, et l'empreinte anti-doublon d'un fichier ne doit pas
  changer** — sinon un fichier réimporté passerait pour neuf. AC9 à AC11 en font des critères décidables.

## Ce que le code dit (relevé le 2026-10-30, `c5101b07`)

### Qui tire `quick-xml` — prémisse confirmée

`cargo tree -i quick-xml --workspace -e normal,build,dev` (lu le 2026-10-30) : **une seule version, 0.39.4**, tirée
**directement** par deux crates du workspace, et par aucune dépendance tierce :

```
quick-xml v0.39.4
├── kesh-import v0.16.1     (crates/kesh-import/Cargo.toml:14  quick-xml = "0.39")
└── kesh-payment v0.16.1    (crates/kesh-payment/Cargo.toml:9  quick-xml = "0.39")
```

`Cargo.lock:3058-3059` : `name = "quick-xml"`, `version = "0.39.4"`. `cargo info quick-xml` : dernière version
**0.42.0**. Les avis (`~/.cargo/advisory-db/crates/quick-xml/RUSTSEC-2026-019{4,5}.md`, lus) : `patched = [">= 0.41.0"]`.

### Exposition — confirmée, et précisée

- **Lecture CAMT.053** (`crates/kesh-import/src/camt053/`) : `mod.rs:62` `NsReader::from_reader(xml)`, puis
  `read_resolved_event_into` (`:75`) jusqu'à `<Document>`, puis `read_event_into` (`:129`) dans `parse_with_version`.
  **Les deux** appels passent par `NamespaceResolver::push` sur chaque `Start`/`Empty` (un `NsReader` résout toujours) :
  **RUSTSEC-2026-0195 atteint donc tout le document, pas seulement `<Document>`**. L'issue citait `v04.rs` ; `v04.rs` et
  `v08.rs` ne sont que des enveloppes de `parse_with_version` (`v04.rs`, `v08.rs` : `super::parse_with_version(reader, …)`).
- **RUSTSEC-2026-0194** : dans 0.39.4, `NamespaceResolver::push` itère les attributs **sans** contrôle
  (`quick-xml-0.39.4/src/name.rs:671` : `for a in start.attributes().with_checks(false)`), contrairement à ce que dit
  l'avis pour `NsReader`. Le chemin quadratique **réel** de Kesh est `attr_string` (`mod.rs:692-707`, `for attr in
  start.attributes()`, contrôle par défaut), appelé sur **chaque** `Start` nommé `Amt` (`mod.rs:168-177`, avant tout test de
  contexte) ; il s'arrête au premier `Ccy`, si bien qu'un `<Amt>` dont les milliers d'attributs précèdent `Ccy` — ou qui
  n'en a pas — parcourt tout. *Prémisse de l'issue précisée, non contredite.*
- **Qui peut le déclencher** : un utilisateur authentifié de rôle Comptable ou plus (routes `preview` et `create`,
  `crates/kesh-api/src/lib.rs:791-802`), dans la limite `KESH_BANK_IMPORT_MAX_MB` (défaut 10, borne [1, 100],
  `config.rs:236-238`). **Le parseur tourne sur un fil de l'exécuteur Tokio** (`routes/bank_imports.rs:1045` et `:1249`,
  `parse_camt053(&fields.file_bytes)` sans `spawn_blocking`) : la boucle quadratique bloque un fil de travail du serveur
  entier, pas seulement la requête. Hors périmètre (C-v017-1-8, issue demandée).
- **pain.001** (`crates/kesh-payment/src/pain001/mod.rs`) : **écriture seule** (`Writer`, `BytesStart::new`,
  `BytesText::new`, `BytesEnd::new`, `BytesDecl::new`, `push_attribute`). Aucun des deux avis ne l'atteint. Il monte quand
  même (C-v017-1-2), pour qu'une seule version de `quick-xml` reste dans l'arbre. Le fichier est **régénéré à la demande**
  (`kesh-db/src/repositories/payment_batches.rs:806-886`), non stocké.

### API publique de `kesh-import` — prémisse confirmée

`kesh_import::camt053::v04::parse` et `v08::parse` sont `pub` et prennent `&mut quick_xml::NsReader<R>` : **un type
`quick-xml` est dans l'API publique**. Personne hors du crate ne les appelle (`grep -rn "v04::\|v08::\|camt053::" crates
--include=*.rs`, hors `kesh-import/src` : aucun résultat). `kesh-import` n'est pas publié sur crates.io (`cargo search
kesh-import` : vide) ; la CI fait `cargo publish --dry-run -p kesh-import` (`.github/workflows/ci.yml:126-127`). La
montée de `quick-xml` change donc l'API publique de `kesh-import` ; la version 0.17.0 portée par la v016-2 (changement
mineur en 0.x) suffit au sens de semver. Signatures gardées (C-v017-1-6).

### L'empreinte anti-doublon ne dépend pas du parseur — confirmée

- **Doublon de fichier** : `file_hash = compute_sha256_hex(&fields.file_bytes)` (`routes/bank_imports.rs:1036`, `:1192`,
  `:1950`, `:2029` ; fonction `:1582-1585`), SHA-256 des **octets reçus**, comparé par
  `bank_imports::find_by_company_and_hash` (`kesh-db/src/repositories/bank_imports.rs:294-322`). Indépendant de
  `quick-xml`.
- **Empreinte d'aperçu** (`routes/bank_import_fingerprint.rs`, en-tête) : tableau haché de `DOMAIN_TAG`, `company_id`,
  compte, IBAN normalisé, **hash du fichier**, format, profil (CSV) ou `null` (CAMT). Aucun champ issu du parseur.
- **Doublon de ligne** : `DuplicateKey { booking_date, amount, reference_normalized, bank_account_id }`
  (`kesh-core/src/bank_imports.rs:287-292`, construite `:318-333`) — **elle dépend de la sortie du parseur** (date,
  montant, `NtryRef` / `EndToEndId` / `AcctSvcrRef`). C'est le seul point par lequel la montée pourrait faire passer une
  ligne déjà importée pour neuve : d'où la caractérisation AC9.

### Un défaut voisin, préexistant, que la montée ne corrige pas — signalé

`quick-xml` 0.39.4 émet les références d'entité dans le texte comme des événements séparés
(`src/reader/mod.rs:289` : `Ok(Event::GeneralRef(BytesRef::wrap(…)))`), et le parseur les ignore (`mod.rs:246`,
`_ => {}`). Un `<Nm>Dupont &amp; Fils</Nm>` produit `Text("Dupont ")`, `GeneralRef("amp")`, `Text(" Fils")` : la
référence disparaît, et pour les champs affectés par `= Some(txt)` le dernier fragment **écrase** le premier
(`handle_txdtls_text`, `Nm`, `EndToEndId`… : `mod.rs:423-470`). Ce défaut vit en 0.42 comme en 0.39 ; le corriger
**changerait** la sortie du parseur pour les fichiers qui portent des entités — exactement ce que la contrainte du
2026-10-30 interdit à cette story. **Hors périmètre** ; caractérisé tel quel par AC9 (fixture `v04_characterization.xml`)
et **issue demandée** (C-v017-1-7), sa correction devant traiter la clé de doublon de ligne.

### Ce qui change d'une version à l'autre (Changelog amont, lu le 2026-10-30)

| Version | Changement qui touche Kesh |
|---|---|
| 0.40.0 (#371) | `Attribute::unescape_value` **déprécié** (→ `normalized_value`), et il **applique désormais la normalisation des valeurs d'attribut** ; `clippy -D warnings` refuse l'appel déprécié (`mod.rs:698`). |
| 0.40.0 (#956) | MSRV 1.79 (le dépôt est en Rust 1.97, édition 2024). |
| 0.41.0 (#969) | Contrôle des doublons d'attributs en `O(N)` — **corrige RUSTSEC-2026-0194**. |
| 0.41.0 (#970) | `NamespaceResolver::push` refuse plus de 256 déclarations par élément — **corrige RUSTSEC-2026-0195**. |
| 0.42.0 (#963) | Le lecteur **valide l'UTF-8** de toute l'entrée (`Error::Encoding`) ; `QName`/`LocalName`/`Namespace`/`Prefix` enveloppent `&str` (`AsRef<[u8]>` retiré) ; `ResolveResult::Unknown` porte un `String` ; **`BytesText::decode()` retiré** ; événements en `Cow<str>`, `Deref<Target = str>`. MSRV 1.86, édition 2024. |
| 0.42.0 (#980) | Plafond **global** de liaisons d'espace de noms en portée, 128 par défaut (remplace le plafond par élément ; `TooManyDeclarations` → `TooManyBindings`). |
| 0.42.0 (#977) | `NamespaceResolver::push` rend `TooDeeplyNested` au-delà de `u16::MAX` niveaux, **au lieu de déborder**. |
| 0.42.0 (#990) | `BytesText::new()` et `escape()` échappent `\r` en référence de caractère (touche pain.001 si un texte porte `\r`). |

Sites de `kesh-import` à adapter en 0.42 (inventaire clos, `grep -rn "quick_xml" crates/` → 8 lignes, dont 2 de
commentaire) : `mod.rs:32-34` (imports), `:82`, `:146` et `:196` (`local_name().as_ref()`), `:85` (`Namespace(b)` devient
`&str`), `:89-91` (`Unknown(prefix)` devient `String`), `:234-238` (`t.decode()` retiré, puis
`quick_xml::escape::unescape`), `:686-690` (`fn local_name` sur des octets), `:692-707` (`attr.key.local_name().as_ref()
== name`, `unescape_value`). `v04.rs:9`, `v08.rs:15` (import `NsReader`, signature inchangée). `kesh-payment` :
`pain001/mod.rs:14-15`, appels `:104-119`, `:152-158`, `:218-239` — aucun changement d'API listé pour `Writer`,
`write_event`, `BytesStart::new`, `push_attribute` (tuple `(&str, &str)`), `BytesDecl::new`, `BytesText::new` ; à
confirmer à la compilation.

## Critères d'acceptation

### La dépendance

- **AC1** — `crates/kesh-import/Cargo.toml` et `crates/kesh-payment/Cargo.toml` déclarent `quick-xml = "0.42"` ;
  `Cargo.lock` ne porte **qu'une** entrée `name = "quick-xml"`, en `0.42.x` (`grep -c 'name = "quick-xml"' Cargo.lock`
  rend 1) ; `cargo tree -i quick-xml --workspace` ne montre que `kesh-import` et `kesh-payment`. Aucune fonctionnalité
  (`features`) n'est ajoutée à `quick-xml` (C-v017-1-1, C-v017-1-2).
- **AC2** — `cargo audit` (base RustSec à jour au jour du développement) ne signale plus **ni RUSTSEC-2026-0194 ni
  RUSTSEC-2026-0195** ; sa sortie complète est versée au journal (`$GATE_LOGS/v017-1-cargo-audit.txt`), et tout autre avis
  sur `quick-xml` qu'elle porterait est déclaré au rapport (issue demandée, non traité ici).
- **AC3** — `cargo publish --dry-run -p kesh-import` est vert (la CI le rejoue), et l'invariant « zéro dépendance
  workspace interne » de `kesh-import` (`ci.yml:140-147`) tient.
- **AC4** — Aucun appel déprécié : `cargo clippy --workspace --all-targets -- -D warnings` vert, **sans** `#[allow(deprecated)]`
  ajouté (`git diff c5101b07 -- crates | grep -c "allow(deprecated)"` rend 0).

### Les entrées hostiles (tests en mémoire, sans réseau ni fichier réel)

Les trois tests vivent dans `crates/kesh-import/tests/camt053_hostile.rs` (fichier neuf) ; chaque document est **construit
en mémoire** à partir d'un en-tête v04 synthétique minimal valide (`<Document xmlns="NS_V04"><BkToCstmrStmt>…`). Chaque
test exécute `parse_camt053` dans un fil (`std::thread::spawn`) et attend son résultat par un canal avec
`recv_timeout(Duration::from_secs(10))` : un dépassement fait échouer le test (« le parseur n'a pas rendu la main en
10 s »), sans bloquer la suite. Ils sont **rouges sous 0.39.4** et verts sous 0.42 (C-v017-1-4).

- **AC5 — RUSTSEC-2026-0194 (attributs).** Un `<Amt>` placé dans un `<Ntry>` portant **200 000** attributs distincts
  (`a0="" a1="" …`, environ 2,4 Mo, sous le plafond de 10 MiB) **sans** `Ccy` : `parse_camt053` rend la main en moins de
  10 s, en `Ok` ou en `Err` (le résultat n'est pas asserté, seul le délai l'est — la fixture ne cherche pas à être un
  relevé valide). *Rouge attendu sous 0.39 : 200 000² / 2 comparaisons ; l'avis mesure ~6 s pour 80 000 en release, le
  test tourne en debug.*
- **AC6 — RUSTSEC-2026-0195 (déclarations d'espace de noms).** (a) Un `<Stmt>` portant **10 000** déclarations
  `xmlns:p0="urn:x:0" … xmlns:p9999="urn:x:9999"` : `parse_camt053` rend `Err(CamtError::MalformedXml(_))` en moins de
  10 s. (b) La même construction sur `<Document>` lui-même (en plus de `xmlns` par défaut) : `Err(MalformedXml(_))`.
  (c) **Pas de faux positif** : un relevé v04 synthétique valide dont `<Document>` porte **cinq** déclarations
  (`xmlns`, `xmlns:xsi` et trois préfixes inutilisés) — l'ordre de grandeur des fichiers bancaires réels — se lit `Ok`, à
  l'identique du même relevé sans les préfixes inutilisés (`assert_eq!` sur les `Vec<ImportedStatement>`). *Rouge
  attendu sous 0.39 pour (a) et (b) : le document se lit sans erreur.*
- **AC7 — imbrication profonde (#977, couverte par le choix de 0.42).** Un document dont `<BkToCstmrStmt>` contient
  **70 000** éléments `<X>` imbriqués (au-delà de `u16::MAX`), refermés : `parse_camt053` rend la main en moins de 10 s
  **sans paniquer** (le fil ne se termine pas en `panic` : `JoinHandle::join` rend `Ok`). L'issue attendue est
  `Err(MalformedXml(_))` si `quick-xml` 0.42 rend `TooDeeplyNested` sur un élément **sans** déclaration ; **à établir dans
  la source 0.42 à l'étape 0** (`NamespaceResolver::push`) — si la profondeur n'est comptée qu'aux éléments qui déclarent,
  la fixture porte une déclaration `xmlns:pN` à chaque niveau, et l'AC asserte `Err(MalformedXml(_))`. Ce choix se cite
  au Change Log, ligne de la source lue.
- **AC8 — L'erreur reste celle que l'écran sait dire.** Les refus d'AC6 et d'AC7 sont des `CamtError::MalformedXml`, que
  `map_camt_error` (`routes/bank_imports.rs:880-884`) traduit en `BANK_IMPORT_MALFORMED_XML` — **aucun code d'erreur ni
  aucune clé i18n neuve**. Un test d'intégration de `kesh-api` (dans le binaire de test des imports bancaires existant,
  à identifier à l'étape 0 par `grep -rln "BANK_IMPORT_MALFORMED_XML" crates/kesh-api/tests`) poste le document d'AC6 (a)
  sur `POST /api/v1/bank-imports/preview` et asserte le statut et le code rendus — les mêmes qu'un XML tronqué
  aujourd'hui (les relever à l'étape 0, ne pas les supposer).

### Les données de la 0.16 — rien ne change de sens (contrainte du 2026-10-30)

- **AC9 — Caractérisation de la sortie du parseur, établie AVANT la montée.** Un test
  `crates/kesh-import/tests/camt053_golden.rs` (fichier neuf) parse **chaque** fichier CAMT.053 du dépôt — les 14 de
  `crates/kesh-import/tests/fixtures/camt053/`, les 4 de `frontend/tests/e2e/fixtures/camt053_*.xml`, et les trois
  fixtures neuves ci-dessous — et compare le résultat à un fichier de référence
  `crates/kesh-import/tests/fixtures/camt053/golden/<nom>.json` :
  - pour un `Ok(stmts)`, `serde_json::to_string_pretty(&stmts)` **entier** (toutes les valeurs de `ImportedStatement`
    et `ImportedTransaction`, `Serialize` dérivé, `types.rs:28-90`) ;
  - pour un `Err`, le **nom du variant** et, sauf pour `MalformedXml`, sa charge (`UnsupportedVersion`,
    `MissingRequiredField`, `InvalidAmount`, `InvalidDate` sont rédigés par Kesh) ; pour `MalformedXml`, le variant seul,
    **sauf** quand le message vient de Kesh (préfixes `fin de fichier inattendue`, `élément racine <Document>
    introuvable`, `préfixe namespace non lié`), qui se compare entier (C-v017-1-3).

  **Les références sont produites sous `quick-xml` 0.39.4, dans un commit qui précède celui de la montée**, et ce commit
  ne touche aucune ligne de `crates/*/src` ni de `Cargo.*` (`git show --stat <sha>` le montre, cité au Change Log). Le
  commit de la montée ne modifie **aucun** fichier `golden/*.json` (`git diff <sha-golden> HEAD --stat --
  crates/kesh-import/tests/fixtures/camt053/golden` vide). Un mode de régénération (`KESH_GOLDEN_REGEN=1`) est permis
  pour produire les références ; il n'est jamais actif en CI, et le test rougit sur une référence manquante.

  Fixtures neuves (synthétiques, `README.md` du dossier complété) :
  - `v04_characterization.xml` : BOM UTF-8 en tête ; fins de ligne CRLF ; un `<Ustrd>` dont le texte contient un saut
    de ligne **intérieur** ; `xmlns:xsi` + `xsi:schemaLocation` sur `<Document>` ; commentaires ; une section CDATA dans
    un `<AddtlNtryInf>` ; noms accentués en UTF-8 (`Müller`, `Genève`) ; **des références d'entité** dans `<Nm>`,
    `<Ustrd>` et `<NtryRef>` (`&amp;`, `&lt;`, `&#xE9;`) — leur sortie **actuelle, fautive**, est figée telle quelle et
    commentée dans le test (« comportement de 0.16, défaut tracé par l'issue <N> ; ne pas corriger ici ») ;
  - `v04_latin1.xml` : déclaré `encoding="ISO-8859-1"`, un nom accentué encodé en Latin-1 (attendu : `MalformedXml`
    sous 0.39 comme sous 0.42 — à confirmer par la référence) ;
  - `v04_duplicate_attr.xml` : un `<Amt Ccy="CHF" Ccy="EUR">` (le comportement de 0.39 — `Ok` avec `CHF`, ou erreur —
    est celui que la référence fige).

  Si, après la montée, une référence diffère, **la montée est fautive** : le code d'adaptation se corrige jusqu'à
  l'égalité (par exemple, lire le texte brut et l'unescaper comme avant plutôt que par `xml_content`, qui normalise les
  fins de ligne). Une différence jugée inévitable **arrête le développement** et se rapporte à l'orchestrateur, avec la
  référence en cause ; elle ne se règle pas en régénérant la référence.
- **AC10 — L'empreinte d'un fichier ne change pas.** Un test unitaire dans `crates/kesh-api/src/routes/bank_imports.rs`
  (module de tests existant, ou neuf s'il n'y en a pas) asserte `compute_sha256_hex` sur les octets de
  `frontend/tests/e2e/fixtures/camt053_v04_minimal.xml` (via `include_bytes!`) **égal à une constante hexadécimale écrite
  en dur**, relevée sous 0.39.4 dans le commit d'AC9. Et `git diff c5101b07 -- crates/kesh-api/src/routes/bank_imports.rs
  crates/kesh-api/src/routes/bank_import_fingerprint.rs` ne touche ni `compute_sha256_hex`, ni `preview_fingerprint`, ni
  `DOMAIN_TAG` (diff limité au test ajouté). *Ce test ne peut pas rougir par la montée — l'empreinte est prise sur les
  octets — : il fixe l'invariant contre une modification future, et c'est écrit dans son commentaire.*
- **AC11 — Un relevé importé en 0.16 ne passe pas pour neuf.** Un test d'intégration `kesh-api` (même binaire qu'AC8)
  importe `camt053_v04_minimal.xml`, puis le reposte en aperçu : l'aperçu signale le **doublon de fichier**
  (`duplicateFile` ou l'équivalent relevé à l'étape 0) **et** les **doublons de ligne** pour toutes ses transactions.
  S'il existe déjà un test qui établit exactement cela (Story 8-3, `grep -rn "duplicate" crates/kesh-api/tests` à
  l'étape 0), il suffit de le citer au Change Log avec son nom et de vérifier qu'il tourne au gate ; sinon il s'écrit.
  *Avec AC9, il couvre la chaîne : même octets → même hash ; même sortie du parseur → mêmes `DuplicateKey`.*
- **AC12 — pain.001 inchangé octet pour octet.** Le test `golden_file_matches` (`pain001/mod.rs:399-433`) reste vert
  **sans modifier** `tests/fixtures/pain001_sample.xml`. Un second test de référence, **produit sous 0.39.4 dans le commit
  d'AC9**, génère un lot dont les noms et la référence non structurée portent `&`, `<`, `>`, `"`, `'` et `é`
  (`tests/fixtures/pain001_special_chars.xml`), et reste égal après la montée. Un texte contenant `\r` n'entre pas dans
  la référence : 0.42 l'échappe en `&#13;` (#990), écart assumé et écrit dans le commentaire du test (C-v017-1-9).

### Ce qui ne change pas

- **AC13** — Aucune migration, aucun changement de version de crate, aucun texte visible ni clé i18n, aucune ligne de
  manuel (`git diff c5101b07 --stat -- crates/kesh-db/migrations crates/kesh-i18n docs/manual frontend/src` vide ;
  `grep -n '^version' crates/*/Cargo.toml` inchangé). Les 15 tests de `camt053_tests.rs` et les 9 de `pain001/mod.rs`
  passent sans modification de leurs assertions.
- **AC14** — `CHANGELOG.md` : sous un en-tête `## [0.17.0]` (que l'orchestrateur fusionnera), section **Sécurité** :
  « L'import d'un relevé CAMT.053 ne peut plus être ralenti des heures ni épuiser la mémoire du serveur par un fichier
  forgé : la bibliothèque XML `quick-xml` passe de 0.39 à 0.42 (avis RUSTSEC-2026-0194 et RUSTSEC-2026-0195, #789). Un
  relevé se lit exactement comme avant : l'empreinte d'un fichier déjà importé et la détection des doublons ne changent
  pas. » (formulation libre, ces trois faits obligatoires, lien d'issue au format du fichier).

## Tâches

0. **Étape 0 — recontrôles** (déclarés au Change Log, ligne lue citée) : relire les numéros de ligne de cette fiche sur
   la tête de branche ; obtenir la source de `quick-xml` 0.42 (`cargo fetch` après la montée, ou registre) et y lire
   `NamespaceResolver::push` (profondeur, AC7), le traitement de `BytesText` (comment retrouver le texte **brut** que
   rendait `decode()`), la survie de `quick_xml::escape::unescape`, et `Attribute::normalized_value` ; relever les codes
   et statuts d'un XML tronqué en aperçu (AC8) et le test de doublon existant (AC11).
1. **Commit A — caractérisation, sous 0.39.4** (aucune ligne de `src` ni de `Cargo.*`) : les trois fixtures neuves et le
   `README.md` ; `camt053_golden.rs` et les références `golden/*.json` ; la constante d'AC10 et son test ; la référence
   pain.001 d'AC12 ; **les trois tests hostiles d'AC5 à AC7**, rouges (consigner la sortie rouge au journal :
   `$GATE_LOGS/v017-1-rouge-0.39.txt`). Gate ciblé : `binary(camt053_golden)` vert, `binary(camt053_hostile)` rouge
   attendu sur ses trois tests.
2. **Commit B — la montée** : les deux `Cargo.toml`, `Cargo.lock` (par `cargo update -p quick-xml` après l'édition des
   manifestes, et rien d'autre : `git diff Cargo.lock` ne montre que `quick-xml` et, s'il y a lieu, ses propres
   dépendances) ; adaptation de `kesh-import` (inventaire ci-dessus) et de `kesh-payment` si la compilation l'exige.
   `fn local_name` : le cas « non UTF-8 » devient inatteignable (le lecteur valide l'UTF-8) ; le retirer ou le garder,
   mais sans laisser de commentaire faux. Aucun réglage des plafonds de `quick-xml` (C-v017-1-5).
3. **Commit C — AC8, AC11, CHANGELOG** ; `cargo audit` (AC2) ; `cargo publish --dry-run -p kesh-import` (AC3).
4. Mutations (ci-dessous), gate complet, E2E complet au dernier commit de code.

## Tests et mutations prévus

| Mutation (une à la fois, restaurée par `git checkout -- <f> && touch <f>`) | Doit faire rougir |
|---|---|
| M1 — dans `Cargo.toml` de `kesh-import`, revenir à `quick-xml = "0.39"` (et le lock) | AC5, AC6 (a)(b), AC7 (au minimum AC6) |
| M2 — dans `parse`, appeler `reader.resolver_mut().set_max_namespace_bindings(usize::MAX)` (nom exact relevé à l'étape 0) | AC6 (a)(b) |
| M3 — dans le traitement de `Event::Text`, remplacer le texte brut par sa forme normalisée (`xml_content`) ou inversement | AC9 (`v04_characterization`, saut de ligne CRLF intérieur) |
| M4 — dans `attr_string`, remplacer `normalized_value` par la valeur brute sans unescape | AC9 si une fixture porte une entité dans `Ccy` — sinon la mutation **survit** et c'est déclaré (l'ajouter à `v04_characterization.xml` pour la tuer est préférable) |
| M5 — dans `compute_sha256_hex`, hacher `&bytes[1..]` | AC10 |
| M6 — dans `pain001`, remplacer `BytesText::new(text)` par `BytesText::from_escaped(text)` | AC12 (référence à caractères spéciaux) |

Une mutation qui survit se déclare au Change Log, avec la raison ; elle ne se tait pas.

## Gates

Gate complet (`scripts/test-fast.sh` sous verrou, `flock "$GATE_LOGS/.gate.lock"`) et E2E complet
(`docs/testing.md` § *Prérequis Playwright local*, à la lettre, dossiers propres au worktree) au **dernier commit de
code** ; frontend (`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`) même sans changement frontend, la CI les
joue. La suite E2E est le seul chemin qui fait traverser un CAMT.053 réel la frontière HTTP (specs `bank-import*`).
Ne déclarer que ce qui a tourné, chiffres recomptés.

**Contrôle local facultatif sur un relevé réel (orchestrateur ou Guy, jamais l'agent)** : si un CAMT.053 réel est
disponible hors dépôt, l'avant/après se compare en local par un condensé de la sortie du parseur écrit dans `$GATE_LOGS`
seulement, et se rapporte « conforme / non conforme », sans aucun chiffre (`docs/agents-orchestration.md` § 8,
C-v017-1-10). `~/kesh-test/` ne s'ouvre pas par un agent.

## Ce que les gardes ne voient pas

- **La consommation de mémoire** n'est pas mesurée : AC6 asserte le **refus** (qui borne l'allocation par construction
  en amont), pas un plafond d'octets.
- **Le délai d'AC5** dépend de la machine ; 10 s pour 200 000 attributs en debug laisse une marge de plus d'un ordre de
  grandeur sous un parseur linéaire, mais un runner très chargé pourrait le dépasser : un rouge de délai isolé se
  rejoue avant d'être diagnostiqué.
- **Le blocage de l'exécuteur Tokio** par un parse long (même linéaire, sur 100 MiB) reste entier (C-v017-1-8).
- **Les entités dans le texte** restent mal lues (C-v017-1-7) — figé, non corrigé.

## Dev Notes

### Fichiers touchés (prévus)

`crates/kesh-import/Cargo.toml`, `crates/kesh-payment/Cargo.toml`, `Cargo.lock` ; `crates/kesh-import/src/camt053/mod.rs`
(et `v04.rs`/`v08.rs` si l'import change) ; `crates/kesh-payment/src/pain001/mod.rs` (si la compilation l'exige, plus le
test d'AC12) ; tests neufs `crates/kesh-import/tests/camt053_golden.rs`, `camt053_hostile.rs`, fixtures et références
sous `crates/kesh-import/tests/fixtures/camt053/` (`golden/`, trois `.xml`, `README.md`),
`crates/kesh-payment/tests/fixtures/pain001_special_chars.xml` ; tests `kesh-api` (AC8, AC10, AC11) ; `CHANGELOG.md`.
Trois crates de code (`kesh-import`, `kesh-payment`, `kesh-api` en test seulement) : sous le seuil de découpage.

### Le manuel

Aucun texte visible ne change ; ni `docs/manual/` ni les PDF ne bougent (AC13). Aucun manuel ne nomme `quick-xml` ni les
avis (`grep -rln "quick-xml\|quick_xml\|RUSTSEC" docs website README.md` : aucun résultat, relevé le 2026-10-30).

### Références

- Avis : `~/.cargo/advisory-db/crates/quick-xml/RUSTSEC-2026-0194.md`, `RUSTSEC-2026-0195.md`.
- Changelog amont : `https://github.com/tafia/quick-xml/blob/master/Changelog.md` (0.40.0 du 2026-05-11, 0.41.0 du
  2026-06-29, 0.42.0 du 2026-08-22).
- Contrainte des données : `kesh-gate-logs/prompts/v017-donnees-a-garder.md`.

## À recontrôler à l'étape 0 du développement

- Les numéros de ligne cités (relevés sur `c5101b07`).
- AC7 : comptage de profondeur dans `NamespaceResolver::push` de 0.42.
- Le nom exact du réglage de plafond en 0.42 (`set_max_namespace_bindings`, #980) pour M2.
- Le moyen, en 0.42, de retrouver le texte brut d'un `BytesText` égal à l'ancien `decode()` (AC9 en décide).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-10-30 — Fiche créée (Opus 5.5, autonomie) ; registre C-v017-1-1 à C-v017-1-10. Prémisses de #789 confirmées au
  code ; précisées : RUSTSEC-2026-0195 atteint tout le document (tout `Start`, pas seulement `<Document>`) ;
  RUSTSEC-2026-0194 atteint Kesh par `attr_string` sur `<Amt>` (`NamespaceResolver::push` itère sans contrôle en 0.39.4,
  `name.rs:671`). Défaut voisin relevé (entités ignorées, `mod.rs:246`), hors périmètre, issue demandée.
