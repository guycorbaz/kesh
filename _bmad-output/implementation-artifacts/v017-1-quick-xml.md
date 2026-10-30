# Story v017-1 : release 0.17.0 — quick-xml 0.39 → 0.42, les deux dénis de service de l'import CAMT.053

Status: ready-for-dev

<!-- Fiche créée le 2026-10-30 (Opus 5.5, autonomie), worktree `kesh-0171`, branche `story/v017-1-quick-xml`,
     partie de `c5101b07` (tête de la candidate 0.16.1-rc1). Issue : #789 (P1, sécurité), en `refs` seulement.
     Registre : `v017-1-quick-xml-choix-autonomes.md` (C-v017-1-1 à C-v017-1-11). -->

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
  start.attributes()`, contrôle par défaut), appelé sur **chaque** `Start` nommé `Amt` (bras `"Amt"` à `mod.rs:168`, appel
  `attr_string(&start, b"Ccy")?` à `mod.rs:179`, avant tout test de contexte) ; il s'arrête au premier `Ccy`, si bien qu'un `<Amt>` dont les milliers d'attributs précèdent `Ccy` — ou qui
  n'en a pas — parcourt tout. *Prémisse de l'issue précisée, non contredite.*
- **Qui peut le déclencher** : un utilisateur authentifié de rôle Comptable ou plus (routes `preview` et `create`,
  `crates/kesh-api/src/lib.rs:791-802`), dans la limite `KESH_BANK_IMPORT_MAX_MB` (défaut 10, borne [1, 100],
  `config.rs:236-238`). **Le parseur tourne sur un fil de l'exécuteur Tokio** (`routes/bank_imports.rs:1045` et `:1249`,
  `parse_camt053(&fields.file_bytes)` sans `spawn_blocking`) : la boucle quadratique bloque un fil de travail du serveur
  entier, pas seulement la requête. Hors périmètre (C-v017-1-8, #820).
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
et tracé par **#819** (C-v017-1-7), sa correction devant traiter la clé de doublon de ligne. Le même bras
`_ => {}` jette aussi les sections **CDATA** (`Event::CData`) : un `<Ustrd>` ou un `<AddtlNtryInf>` en CDATA est lu
vide. Défaut de même nature (texte perdu en silence), figé par AC9 et à verser à #819 (F-5 de la validation P1).

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
| 0.42.0 (#990) | `BytesText::new()` et `escape()` échappent `\r` en référence de caractère (touche pain.001 si un texte porte `\r`). Établi dans la source, ci-dessous. |

### Établi dans la source 0.42.0 (validation P1, 2026-10-30)

Source téléchargée hors du dépôt (`kesh-gate-logs/v017-1-src/quick-xml-0.42.0/`, `static.crates.io`), lue sans rien
exécuter. Chemins relatifs à ce dossier.

- **Profondeur (AC7)** — `src/name.rs:729-733` : `pub fn push(&mut self, start: &BytesStart) -> Result<(), NamespaceError> {
  self.nesting_level = self.nesting_level.checked_add(1).ok_or(NamespaceError::TooDeeplyNested(u16::MAX as usize))?;`,
  **avant** la boucle des attributs, donc pour **toute** balise ouvrante ; `push` est appelé sur chaque `Start` et
  chaque `Empty` (`src/reader/ns_reader.rs:82-87`, `self.ns_resolver.push(&e)?;`). En 0.39.4, `src/name.rs:668`
  `self.nesting_level += 1;` sur un `u16` (`:492`) : **panique** au 65 536e niveau sous le profil de test (le
  `[profile.dev]` du dépôt, `Cargo.toml:37-46`, ne désactive pas `overflow-checks`), débordement silencieux en release.
- **Plafond (M2)** — `src/name.rs:541` `pub const DEFAULT_MAX_NAMESPACE_BINDINGS: usize = 128;` ; réglage
  `src/name.rs:770` `pub fn set_max_namespace_bindings(&mut self, limit: usize) -> &mut Self` ; accès
  `src/reader/ns_reader.rs:131` `pub fn resolver_mut(&mut self) -> &mut NamespaceResolver`. Le contrôle est dans `add`
  (`src/name.rs:659-668` et `:707-709`, `self.bindings.len().saturating_sub(BUILTIN_NAMESPACE_BINDINGS) >=
  self.max_namespace_bindings` → `TooManyBindings`) : il compte les liaisons **en portée**, tout niveau confondu.
- **`escape::unescape` survit** — `src/escape.rs:236` `pub fn unescape(raw: &str) -> Result<Cow<'_, str>, EscapeError>`.
- **Texte brut (AC9, M3)** — `BytesText` : `src/events/mod.rs:601` `pub fn into_inner(self) -> Cow<'i, str>` (contenu
  tel que lu, **sans** normalisation), `:625` `pub fn xml10_content(&self) -> Cow<'i, str>` (normalise les fins de
  ligne, `normalize_xml10_eols`), `:656` `pub fn xml_content(&self, version: XmlVersion) -> Cow<'i, str>`, `:697`
  `impl<'i> Deref for BytesText<'i>`. L'équivalent de l'ancien `decode()` (0.39.4 `src/events/mod.rs:616-617`,
  `self.decoder.decode_cow(&self.content)`, sans normalisation) est **`into_inner()` / `Deref`**, non `xml_content`.
- **Valeur d'attribut** — 0.42 `src/events/attributes.rs:264` `unescape_value` est `#[deprecated = "use
  Self::normalized_value()"]` (`:263`) ; `:75` `pub fn normalized_value(&self, version: XmlVersion)`, dont la
  documentation dit (`:85`) : « The characters `\t`, `\r`, `\n` are replaced with whitespace characters (`0x20`) ».
  0.39.4 `unescape_value` (`src/events/attributes.rs:56-58`, `:104-116`) décodait et dés-échappait **sans** cette
  normalisation : un `Ccy` portant une tabulation ou un saut de ligne **littéral** se lirait autrement (écart admis,
  C-v017-1-11 ; un tel `Ccy` n'est pas un code de devise et faisait déjà échouer la validation de la devise).
- **Doublons d'attributs** — `src/events/attributes.rs:1036` `check_for_duplicates`, `:1061`
  `check_for_duplicates_hashed` ; l'erreur `AttrError::Duplicated` est rendue **au second** attribut, pendant
  l'itération (`:1050`, `:1088`), comme en 0.39. `attr_string` s'arrêtant au premier `Ccy`, `<Amt Ccy="CHF"
  Ccy="EUR">` se lit `CHF` sous les deux versions.
- **Références d'entité** — toujours émises à part : `src/reader/mod.rs:317`
  `Ok(Event::GeneralRef(BytesRef::wrap(&bytes[1..bytes.len() - 1])))` ; #819 vit en 0.42.
- **Validation UTF-8 hors texte (F-1/R-5)** — le lecteur à tampon valide chaque morceau lu, y compris commentaires,
  CDATA et instructions : `src/reader/buffered_reader.rs:297` `return Ok((bang_type, std::str::from_utf8(&buf[start..])?));`
  (sections `<!…>`), `:240`, `:588`. En 0.39.4, seul `t.decode()` (`camt053/mod.rs:235`, sur les `Event::Text`)
  contrôlait l'UTF-8 ; commentaires, CDATA, PI et attributs non lus passaient. Divergence **admise** (C-v017-1-11).
- **`\r` dans pain.001 (#990)** — `src/escape.rs:109` `matches!(ch, b'<' | b'>' | b'&' | b'\'' | b'\"' | b'\r')`, et
  `src/events/mod.rs:586-587` `pub fn new(content: &'i str) -> Self { Self::from_escaped(escape(content)) }` ; en 0.39.4
  `src/escape.rs:106` n'échappe que `< > & ' "`. Seul `\r` change.

Sites de `kesh-import` à adapter en 0.42 (inventaire clos, `grep -rn "quick_xml" crates/` → 8 lignes, dont 2 de
commentaire) : `mod.rs:32-34` (imports), `:82`, `:146` et `:196` (`local_name().as_ref()`), `:85` (`Namespace(b)` devient
`&str`), `:89-91` (`Unknown(prefix)` devient `String`), `:234-238` (`t.decode()` retiré, puis
`quick_xml::escape::unescape` — remplacer `t.decode()` par `t.into_inner()` ou `&*t`, **pas** par `xml_content`),
`:686-690` (`fn local_name` sur des octets), `:692-707` (`attr.key.local_name().as_ref()
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
  sur `quick-xml` qu'elle porterait est déclaré au rapport (issue à demander à l'orchestrateur, non traité ici).
- **AC3** — `cargo publish --dry-run -p kesh-import` est vert (la CI le rejoue), et l'invariant « zéro dépendance
  workspace interne » de `kesh-import` (`ci.yml:140-147`) tient.
- **AC4** — Aucun appel déprécié : `cargo clippy --workspace --all-targets -- -D warnings` vert, **sans** `#[allow(deprecated)]`
  ajouté (`git diff c5101b07 -- crates | grep -c "allow(deprecated)"` rend 0).

### Les entrées hostiles (tests en mémoire, sans réseau ni fichier réel)

Les trois tests vivent dans `crates/kesh-import/tests/camt053_hostile.rs` (fichier neuf) ; chaque document est **construit
en mémoire** à partir d'un en-tête v04 synthétique minimal valide (`<Document xmlns="NS_V04"><BkToCstmrStmt>…`). Chaque
test exécute `parse_camt053` dans un fil (`std::thread::spawn`) et attend son résultat par un canal avec
`recv_timeout(Duration::from_secs(10))` : `Err(RecvTimeoutError::Timeout)` fait échouer le test (« le parseur n'a pas
rendu la main en 10 s »), `Err(RecvTimeoutError::Disconnected)` le fait échouer sous un **autre** message (« le fil du
parseur est mort — panique »), sans bloquer la suite. Ils sont **rouges sous 0.39.4** et verts sous 0.42
(C-v017-1-4). Le journal rouge du commit A (`$GATE_LOGS/v017-1-rouge-0.39.txt`) doit montrer la **cause** attendue de
chaque rouge : dépassement de délai pour AC5, `Ok` au lieu d'`Err` pour AC6 (a)(b), fil mort par débordement
(`attempt to add with overflow`, 0.39.4 `src/name.rs:668`) pour AC7 ; un rouge d'une autre cause (document mal formé
par ailleurs, `Err` précoce) invalide le test et se corrige avant le commit A. Sous `--test-threads=1` (CI), le fil
d'AC5 continue de tourner après l'échec et ralentit les tests suivants du binaire : à l'état rouge seulement.

- **AC5 — RUSTSEC-2026-0194 (attributs).** Un `<Amt>` **à balise fermante** (`<Amt a0="" … >1.00</Amt>` : un `<Amt …/>`
  vide est un `Event::Empty`, que le parseur ne traite pas, `mod.rs:145` ne lit que `Event::Start`), placé dans un
  `<Ntry>` d'un `<Stmt>`, portant **200 000** attributs distincts
  (`a0="" a1="" …`, environ 2,4 Mo, sous le plafond de 10 MiB) **sans** `Ccy` : `parse_camt053` rend la main en moins de
  10 s, en `Ok` ou en `Err` (le résultat n'est pas asserté, seul le délai l'est — la fixture ne cherche pas à être un
  relevé valide). *Rouge attendu sous 0.39 : 200 000² / 2 comparaisons ; l'avis mesure ~6 s pour 80 000 en release, le
  test tourne en debug.*
- **AC6 — RUSTSEC-2026-0195 (déclarations d'espace de noms).** (a) Un `<Stmt>` portant **10 000** déclarations
  `xmlns:p0="urn:x:0" … xmlns:p9999="urn:x:9999"` : `parse_camt053` rend `Err(CamtError::MalformedXml(_))` en moins de
  10 s. (b) La même construction sur `<Document>` lui-même (en plus de `xmlns` par défaut) : `Err(MalformedXml(_))`.
  (c) **Pas de faux positif** : un relevé v04 synthétique valide dont `<Document>` porte **cinq** déclarations
  (constante nommée, très en deçà de `DEFAULT_MAX_NAMESPACE_BINDINGS` = 128)
  (`xmlns`, `xmlns:xsi` et trois préfixes inutilisés) — l'ordre de grandeur des fichiers bancaires réels — se lit `Ok`, à
  l'identique du même relevé sans les préfixes inutilisés (`assert_eq!` sur les `Vec<ImportedStatement>`). *Rouge
  attendu sous 0.39 pour (a) et (b) : le document se lit sans erreur.*
- **AC7 — imbrication profonde (#977, couverte par le choix de 0.42).** Un document dont `<BkToCstmrStmt>` contient
  **70 000** éléments `<X>` nus imbriqués (au-delà de `u16::MAX`), refermés : `parse_camt053` rend
  `Err(CamtError::MalformedXml(_))` en moins de 10 s, **sans paniquer** (le canal ne rend pas `Disconnected`). La
  profondeur est comptée sur toute balise ouvrante (0.42 `src/name.rs:729-733`, établi ci-dessus) : aucune déclaration
  n'est nécessaire dans la fixture.
- **AC8 — L'erreur reste celle que l'écran sait dire.** Les refus d'AC6 et d'AC7 sont des `CamtError::MalformedXml`, que
  `map_camt_error` (`routes/bank_imports.rs:880-884`) traduit en `BANK_IMPORT_MALFORMED_XML` — **aucun code d'erreur ni
  aucune clé i18n neuve**. Un test d'intégration de `kesh-api` (dans le binaire de test des imports bancaires existant,
  `crates/kesh-api/tests/bank_imports_e2e.rs`) poste le document d'AC6 (a) sur `POST /api/v1/bank-imports/preview` et
  asserte ce que le test voisin `post_import_rejects_truncated_xml` (`bank_imports_e2e.rs:620-638`) asserte pour un XML
  tronqué : statut `400`, `error.code == "BANK_IMPORT_MALFORMED_XML"`, `error.details.kind == "MALFORMED_XML"`. Le même
  test poste aussi la fixture `v04_latin1_hors_texte.xml` d'AC9 et asserte les mêmes trois valeurs (C-v017-1-11).

### Les données de la 0.16 — rien ne change de sens (contrainte du 2026-10-30)

- **AC9 — Caractérisation de la sortie du parseur, établie AVANT la montée.** Un test
  `crates/kesh-import/tests/camt053_golden.rs` (fichier neuf) parse **chaque** fichier CAMT.053 du dépôt — les 14 de
  `crates/kesh-import/tests/fixtures/camt053/`, les 4 de `frontend/tests/e2e/fixtures/camt053_*.xml` (lus par
  `std::fs::read` à l'exécution, non par `include_bytes!`), et les quatre fixtures neuves ci-dessous — et compare le
  résultat à un fichier de référence `crates/kesh-import/tests/fixtures/camt053_golden/<nom>.json` (dossier **frère** de
  `camt053/`, pour ne pas fausser le décompte `ls camt053/` que citent `camt053_tests.rs:3` et le `README.md` des
  fixtures) :
  - pour un `Ok(stmts)`, `serde_json::to_string_pretty(&stmts)` **entier** (toutes les valeurs de `ImportedStatement`
    et `ImportedTransaction`, `Serialize` dérivé, `types.rs:28-90`) ;
  - pour un `Err`, le **nom du variant** et, sauf pour `MalformedXml`, sa charge (`UnsupportedVersion`,
    `MissingRequiredField`, `InvalidAmount`, `InvalidDate` sont rédigés par Kesh) ; pour `MalformedXml`, le variant seul,
    **sauf** quand le message vient de Kesh (préfixes `fin de fichier inattendue`, `élément racine <Document>
    introuvable`, `préfixe namespace non lié`), qui se compare entier (C-v017-1-3).

  **Les références sont produites sous `quick-xml` 0.39.4, dans un commit qui précède celui de la montée**, et ce commit
  ne touche aucune ligne de `crates/*/src` ni de `Cargo.*` (`git show --stat <sha>` le montre, cité au Change Log). Le
  commit de la montée ne modifie **aucun** fichier de référence (contrôle mécanique au § *Gates*). Un mode de
  régénération (`KESH_GOLDEN_REGEN=1`) produit les références ; il est **verrouillé dans le code du test** : sous
  `KESH_GOLDEN_REGEN=1`, le test lit `Cargo.lock` à la racine du workspace (`env!("CARGO_MANIFEST_DIR")/../../Cargo.lock`)
  et **panique** si la version résolue de `quick-xml` n'est pas `0.39.x`, ou si la variable `CI` est définie. Hors de
  ce mode, une référence manquante fait rougir le test. Le même verrou garde la régénération de la référence pain.001
  d'AC12 (dans son test, non dans `crates/kesh-payment/examples/gen_golden.rs`, qui reste tel quel et ne produit que
  `pain001_sample.xml`, inchangé par AC12).

  Fixtures neuves (synthétiques, `README.md` du dossier complété) :
  - `v04_characterization.xml` : BOM UTF-8 en tête ; fins de ligne CRLF ; un `<Ustrd>` dont le texte contient un saut
    de ligne **intérieur** ; `xmlns:xsi` + `xsi:schemaLocation` sur `<Document>` ; commentaires ; une section CDATA dans
    un `<AddtlNtryInf>` ; noms accentués en UTF-8 (`Müller`, `Genève`) ; **des références d'entité** dans `<Nm>`,
    `<Ustrd>` et `<NtryRef>` (`&amp;`, `&lt;`, `&#xE9;`) — leur sortie **actuelle, fautive**, est figée telle quelle et
    commentée dans le test (« comportement de 0.16, défaut tracé par #819 ; ne pas corriger ici ») ; un
    `<Amt Ccy="C&#72;F">` dans un `Ntry/Amt` (référence de caractère dans la seule valeur d'attribut lue : 0.39 lit
    `CHF`, une lecture brute lirait `C&#72;F` — tue M4) ;
  - `v04_latin1.xml` : déclaré `encoding="ISO-8859-1"`, un nom accentué encodé en Latin-1 (attendu : `MalformedXml`
    sous 0.39 comme sous 0.42 — à confirmer par la référence) ;
  - `v04_latin1_hors_texte.xml` : relevé valide **dont tout le texte est ASCII**, mais portant un octet `0xE9`
    (Latin-1) dans un **commentaire**, dans une section **CDATA** d'`<AddtlNtryInf>` et dans la valeur de
    `xsi:schemaLocation`. Sous 0.39.4, aucun de ces octets n'est décodé (`mod.rs:235` ne décode que les `Event::Text`,
    `mod.rs:246` jette le reste) : la référence attendue est `Ok(…)`. Sous 0.42, le lecteur refuse le fichier
    (`buffered_reader.rs:297`) : **divergence admise** (C-v017-1-11), seule exception à la règle d'égalité ci-dessous.
    Le test compare alors l'issue 0.42 à `Err(MalformedXml)` **et** garde la référence 0.39 dans le dépôt, nommée
    `v04_latin1_hors_texte.0.39.json`, comme trace de ce qui a changé ; l'exception est **nominative** (ce seul fichier),
    écrite en dur dans le test ;
  - `v04_duplicate_attr.xml` : un `<Amt Ccy="CHF" Ccy="EUR">`. Attendu `CHF` sous les deux versions (le doublon n'est
    signalé qu'au second attribut, et `attr_string` s'arrête au premier `Ccy` — établi ci-dessus) ; la référence le
    fige.

  Classes de divergence après la montée :
  - **interdite** — un `Ok` dont une valeur diffère, ou un `Ok` devenu `Err` sur un document UTF-8 bien formé : la
    montée est fautive, le code d'adaptation se corrige jusqu'à l'égalité (par exemple `into_inner()` plutôt que
    `xml_content`, qui normalise les fins de ligne) ;
  - **admise** — la seule de C-v017-1-11 : un document **non UTF-8** (octet invalide hors du texte lu), `Ok` sous 0.39,
    `Err(MalformedXml)` sous 0.42 ; nominative (`v04_latin1_hors_texte.xml`) ;
  - **à rapporter** — toute autre différence (par exemple un document mal formé que 0.39 tolérait et que 0.42 refuse) :
    elle **arrête le développement** et se rapporte à l'orchestrateur, avec la référence en cause ; elle ne se règle
    jamais en régénérant la référence.
- **AC10 — L'empreinte d'un fichier ne change pas.** Un test unitaire dans `crates/kesh-api/src/routes/bank_imports.rs`
  (module de tests existant, ou neuf s'il n'y en a pas) asserte `compute_sha256_hex` sur les octets de
  `frontend/tests/e2e/fixtures/camt053_v04_minimal.xml` (via `include_bytes!`) **égal à une constante hexadécimale écrite
  en dur**, relevée sous 0.39.4 dans le commit d'AC9. Et `git diff c5101b07 -- crates/kesh-api/src/routes/bank_imports.rs
  crates/kesh-api/src/routes/bank_import_fingerprint.rs` ne touche ni `compute_sha256_hex`, ni `preview_fingerprint`, ni
  `DOMAIN_TAG` (diff limité au test ajouté). *Ce test ne peut pas rougir par la montée — l'empreinte est prise sur les
  octets — : il fixe l'invariant contre une modification future, et c'est écrit dans son commentaire.*
- **AC11 — Un relevé importé ne passe pas pour neuf.** Un test d'intégration **neuf** dans
  `crates/kesh-api/tests/bank_imports_e2e.rs` importe `read_fixture("v04_minimal.xml")` (helper `:183`), puis reposte
  **le même fichier** en aperçu, et asserte **les deux** : `warnings.duplicateFile` présent (forme assertée par
  `post_preview_returns_duplicate_file_warning`, `:736`, `:766-770`) **et** `warnings.duplicateLines` de longueur égale au
  nombre de transactions du fichier (forme assertée par `post_preview_returns_duplicate_lines_warning`, `:878`, `:911`).
  Aucun test existant ne fait les deux sur le même fichier : `:736` n'asserte que le doublon de fichier, `:878` prévisualise
  un **autre** fichier (`v04_overlap.xml`). *Import et aperçu tournant dans le même binaire, ce test ne voit pas une
  dérive entre versions : il fixe la logique `DuplicateKey` ; la preuve inter-versions est AC9 (même sortie du parseur →
  mêmes clés), et AC10 pour le hash. Son commentaire le dit.*
- **AC12 — pain.001 inchangé octet pour octet.** Le test `golden_file_matches` (`pain001/mod.rs:399-433`) reste vert
  **sans modifier** `tests/fixtures/pain001_sample.xml`. Un second test de référence, **produit sous 0.39.4 dans le commit
  d'AC9**, génère un lot dont les noms et la référence non structurée portent `&`, `<`, `>`, `"`, `'` et `é`
  (`tests/fixtures/pain001_special_chars.xml`), et reste égal après la montée. Un texte contenant `\r` n'entre pas dans
  la référence : 0.42 l'échappe en `&#13;` (#990), écart assumé et écrit dans le commentaire du test (C-v017-1-9).

### Ce qui ne change pas

- **AC13** — Aucune migration, aucun changement de version de crate, aucun texte visible ni clé i18n, aucune ligne de
  manuel : le diff **de cette story** (`git diff <base de la branche> --stat -- crates/kesh-db/migrations crates/kesh-i18n
  docs/manual frontend/src` vide, et aucune ligne `^version` de `crates/*/Cargo.toml` dans ce diff — la v016-2 porte le
  passage à 0.17.0, que la réunion des branches apportera). Les 15 tests de `camt053_tests.rs` et les 9 de `pain001/mod.rs`
  passent sans modification de leurs assertions.
- **AC14** — `CHANGELOG.md` : sous un en-tête `## [0.17.0]` (que l'orchestrateur fusionnera), section **Sécurité** :
  « L'import d'un relevé CAMT.053 ne peut plus être ralenti des heures ni épuiser la mémoire du serveur par un fichier
  forgé : la bibliothèque XML `quick-xml` passe de 0.39 à 0.42 (avis RUSTSEC-2026-0194 et RUSTSEC-2026-0195, #789). Un
  relevé se lit exactement comme avant : l'empreinte d'un fichier déjà importé et la détection des doublons ne changent
  pas. Seule exception : un fichier qui n'est pas en UTF-8 — ce qu'exige la norme ISO 20022 —, accepté jusqu'ici quand
  les octets fautifs se trouvaient hors du texte lu (commentaire, CDATA, attribut ignoré), est désormais refusé comme
  XML mal formé. » (formulation libre, ces quatre faits obligatoires, lien d'issue au format du fichier).

## Tâches

0. **Étape 0 — recontrôles** (déclarés au Change Log, ligne lue citée) : relire les numéros de ligne de cette fiche sur
   la tête de branche ; relire, dans la source 0.42.0 résolue par le lock après la montée, les lignes citées au
   § *Établi dans la source 0.42.0* (elles ont été lues sur l'archive de `static.crates.io` ; un écart se déclare).
1. **Commit A — caractérisation, sous 0.39.4** (aucune ligne de `src` ni de `Cargo.*`) : les quatre fixtures neuves et le
   `README.md` (son catalogue et son décompte recomptés par `ls tests/fixtures/camt053/*.xml | wc -l`, comme le
   commentaire `camt053_tests.rs:3`) ; `camt053_golden.rs` et les références `camt053_golden/*.json` ; la constante d'AC10 et son test ; la référence
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
| *(pas de M1)* — revenir à 0.39 ne compile plus après le commit B : le rouge sous 0.39 est celui du **commit A**, journal `v017-1-rouge-0.39.txt` | AC5, AC6 (a)(b), AC7 |
| M2 — dans `parse`, appeler `reader.resolver_mut().set_max_namespace_bindings(usize::MAX)` (0.42 `src/name.rs:770`, `src/reader/ns_reader.rs:131`) | AC6 (a)(b) |
| M3 — dans le traitement de `Event::Text`, remplacer `into_inner()` par `xml10_content()` | AC9 (`v04_characterization`, saut de ligne CRLF intérieur) |
| M4 — dans `attr_string`, remplacer `normalized_value` par la valeur brute sans dés-échappement | AC9 (`v04_characterization`, `Ccy="C&#72;F"`) |
| M5 — dans `compute_sha256_hex`, hacher `&bytes[1..]` | AC10 |
| M6 — dans `pain001`, remplacer `BytesText::new(text)` par `BytesText::from_escaped(text)` | AC12 (référence à caractères spéciaux) |

Une mutation qui survit se déclare au Change Log, avec la raison ; elle ne se tait pas.

## Gates

Gate complet (`scripts/test-fast.sh` sous verrou, `flock "$GATE_LOGS/.gate.lock"`) et E2E complet
(`docs/testing.md` § *Prérequis Playwright local*, à la lettre, dossiers propres au worktree) au **dernier commit de
code** ; frontend (`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`) même sans changement frontend, la CI les
joue. La suite E2E est le seul chemin qui fait traverser un CAMT.053 réel la frontière HTTP (specs `bank-import*`).
Ne déclarer que ce qui a tourné, chiffres recomptés.

**Contrôle des références (mécanique, au gate de fin de story et avant tout push)** — le sha du commit A s'écrit au
Change Log dès sa création, puis :

```sh
git diff --stat <sha-commit-A> HEAD -- crates/kesh-import/tests/fixtures/camt053_golden \
  crates/kesh-payment/tests/fixtures/pain001_special_chars.xml crates/kesh-payment/tests/fixtures/pain001_sample.xml
git show --stat <sha-commit-A> -- crates/*/src Cargo.toml Cargo.lock crates/*/Cargo.toml
```

Les deux sorties sont **vides** ; elles sont copiées au Change Log. Les lentilles de revue les rejouent.

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
- **Les entités et les sections CDATA dans le texte** restent mal lues (C-v017-1-7, #819) — figé, non corrigé.
- **Un fichier non UTF-8** dont les octets fautifs étaient hors du texte lu (commentaire, CDATA, attribut ignoré)
  s'importait en 0.16 et est refusé en 0.17 (`BANK_IMPORT_MALFORMED_XML`) — divergence admise (C-v017-1-11), prouvée
  par `v04_latin1_hors_texte.xml` et AC8. Un tel fichier déjà importé n'est pas touché en base ; c'est sa
  **réimportation** qui serait refusée au lieu d'être signalée en doublon (le parse précède la recherche du hash,
  `routes/bank_imports.rs:1036-1045`).
- **Un `Ccy` portant une tabulation ou un saut de ligne littéral** serait normalisé en espace par 0.42
  (`normalized_value`) ; aucune fixture ne le fige, une telle valeur n'étant pas un code de devise.
- **L'amplification mémoire d'un fichier valide** (un nœud de texte copié par le dés-échappement puis `trim` ; des
  millions de `<Ntry>` alimentant `Vec<ImportedTransaction>`) n'est bornée que par `KESH_BANK_IMPORT_MAX_MB` (jusqu'à
  100, `lib.rs:793-799`). Aucun autre lecteur XML dans le dépôt (`Cargo.lock` : seul `quick-xml`).

## Dev Notes

### Fichiers touchés (prévus)

`crates/kesh-import/Cargo.toml`, `crates/kesh-payment/Cargo.toml`, `Cargo.lock` ; `crates/kesh-import/src/camt053/mod.rs`
(et `v04.rs`/`v08.rs` si l'import change) ; `crates/kesh-payment/src/pain001/mod.rs` (si la compilation l'exige, plus le
test d'AC12) ; tests neufs `crates/kesh-import/tests/camt053_golden.rs`, `camt053_hostile.rs`, fixtures et références
sous `crates/kesh-import/tests/fixtures/` (`camt053_golden/`, quatre `.xml` et le `README.md` dans `camt053/`),
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
- Les lignes de la source 0.42.0 citées au § *Établi dans la source 0.42.0*, contre la source que résout le lock
  après la montée (lues le 2026-10-30 sur l'archive `static.crates.io` ; elles levaient les quatre inconnues que la
  version initiale de cette liste portait : profondeur, nom du plafond, texte brut, `unescape`).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-10-30 — Fiche créée (Opus 5.5, autonomie) ; registre C-v017-1-1 à C-v017-1-10. Prémisses de #789 confirmées au
  code ; précisées : RUSTSEC-2026-0195 atteint tout le document (tout `Start`, pas seulement `<Document>`) ;
  RUSTSEC-2026-0194 atteint Kesh par `attr_string` sur `<Amt>` (`NamespaceResolver::push` itère sans contrôle en 0.39.4,
  `name.rs:671`). Défaut voisin relevé (entités ignorées, `mod.rs:246`), hors périmètre, issue demandée.
- 2026-10-30 — **Validation P1** (Sonnet, deux lentilles en contexte frais ; rapports
  `kesh-gate-logs/v017-1-quick-xml-validate-p1-{R,F}.md`). Décompte : **R** 0 CRITICAL, 0 HIGH, 2 MEDIUM, 5 LOW ;
  **F** 0 CRITICAL, 0 HIGH, 2 MEDIUM, 5 LOW ; F-1 et R-5 désignent le même défaut, soit **4 MEDIUM distincts et
  10 LOW** comptés par l'orchestrateur. Remédiation par l'agent de fiche (Opus 5.5), prémisses vérifiées une à une ; la
  source `quick-xml` 0.42.0 a été téléchargée hors du dépôt (`kesh-gate-logs/v017-1-src/`, permission de
  l'orchestrateur) et lue sans rien exécuter. Traitement :
  - **R-1 (MEDIUM), appliqué** — prémisse vérifiée : `bank_imports_e2e.rs:736` n'asserte que `duplicateFile`, `:878`
    prévisualise `v04_overlap.xml`. AC11 devient un test neuf, même fichier, deux assertions ; la portée (logique de clé,
    non dérive entre versions) est écrite.
  - **R-2 (MEDIUM), appliqué** — prémisse vérifiée : après le commit B le code est écrit pour l'API 0.42 (`into_inner`,
    noms en `&str`), il ne compile plus sous 0.39. M1 retirée ; le rouge sous 0.39 est celui du commit A, dont la cause
    doit être journalisée.
  - **F-1 = R-5 (MEDIUM / LOW), appliqué selon la décision de l'orchestrateur (divergence admise)** — prémisse vérifiée
    dans les deux sources : 0.39.4 ne décode que les `Event::Text` (`camt053/mod.rs:235`) ; 0.42.0 valide toute section
    lue (`src/reader/buffered_reader.rs:297`). Fixture `v04_latin1_hors_texte.xml` (référence 0.39 consignée), refus
    `BANK_IMPORT_MALFORMED_XML` asserté par AC8, phrase au CHANGELOG (AC14), ligne au § *Ce que les gardes ne voient
    pas*, choix C-v017-1-11, classes de divergence d'AC9.
  - **F-2 (MEDIUM), appliqué** — verrou dans le code du test (`KESH_GOLDEN_REGEN` refusé si le lock ne résout pas
    `quick-xml` 0.39.x ou si `CI` est défini) et contrôle mécanique au § *Gates* (deux commandes dont la sortie vide se
    copie au Change Log). `gen_golden.rs` existe (`crates/kesh-payment/examples/`) mais ne produit que
    `pain001_sample.xml`, inchangé : il n'est pas modifié, et le diff de contrôle couvre ce fichier.
  - **R-3 (LOW), appliqué** — confirmé dans la source : 0.42 `src/name.rs:729-733` (profondeur comptée sur toute balise),
    0.39.4 `src/name.rs:668` (débordement `u16`). AC7 simplifié (fixture nue, `Err(MalformedXml)` asserté) ; distinction
    `Timeout` / `Disconnected` posée pour les trois tests hostiles.
  - **R-4 (LOW), appliqué** — `<Amt Ccy="C&#72;F">` ajouté à `v04_characterization.xml` ; M4 tuée par AC9.
  - **R-6 (LOW), rejeté** — prémisse fausse : 0.42 `src/escape.rs:109` échappe `\r`, et `BytesText::new` vaut
    `Self::from_escaped(escape(content))` (`src/events/mod.rs:586-587`) ; la ligne du tableau était juste, elle cite
    désormais la source.
  - **R-7 (LOW), appliqué** — cote d'`attr_string` corrigée (`:168` le bras, `:179` l'appel) ; signatures de `BytesText`
    établies (`into_inner` `:601`, `xml10_content` `:625`, `xml_content` `:656`) ; `escape::unescape` survit
    (`src/escape.rs:236`).
  - **F-3 (LOW), appliqué** — `<Amt>` à balise fermante (un `Empty` n'est pas lu, `mod.rs:145`) ; cause de chaque rouge
    du commit A exigée au journal ; constante nommée pour les cinq déclarations ; fil survivant noté.
  - **F-4 (LOW), appliqué** — trois classes de divergence dans AC9 (interdite, admise, à rapporter) ; le cas
    `v04_duplicate_attr.xml` est établi identique sous les deux versions (`src/events/attributes.rs:1050`, `:1088`).
  - **F-5 (LOW), appliqué** — #819 et #820 cités à la place de « issue demandée » ; CDATA ignoré (même bras
    `mod.rs:246`) relevé, figé par AC9, à verser à #819 (demandé à l'orchestrateur).
  - **F-6 (LOW), appliqué** — amplification mémoire et absence d'autre lecteur XML écrites au § *Ce que les gardes ne
    voient pas*.
  - **F-7 (LOW), appliqué** — AC13 mesuré sur le diff de la story ; C-v017-1-1 nuancé (#977 : panique en debug,
    débordement silencieux en release ; risque ajouté par 0.42 = #963, gardé par AC9).
  - **Ajout de l'agent, mécanique** : les références vont dans `tests/fixtures/camt053_golden/`, dossier frère, pour ne
    pas fausser le décompte `ls camt053/` de `camt053_tests.rs:3` et du `README.md` des fixtures, que le commit A recompte.
