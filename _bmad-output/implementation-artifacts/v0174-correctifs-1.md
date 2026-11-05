# Story v0174-correctifs-1 : l'import CAMT.053 lit les références d'entité et le CDATA (#819), sans dédoubler les lignes déjà importées

Status: review

*Fiche sœur de `v0174-correctifs.md` (l'index : contraintes communes, CHANGELOG, manuels, ordre), rédigée le 2026-11-05 en
autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-0174c`, branche `story/v0174-correctifs` partie de `7bd4d5c0`.
Release **0.17.4**, **sans migration**. Registre : `v0174-choix-autonomes-v0174c.md`, choix C-v0174c-3 à 7 et 21. Clé de statut
`v0174-correctifs-1`. Issue **#819** (bug, P2), en `refs` seulement. Numéros de ligne de `7bd4d5c0`.*

## Story

En tant que Comptable qui importe un relevé CAMT.053, je veux que le nom d'une contrepartie, un libellé ou une référence
portant `&`, `<`, une lettre codée (`&#xE9;`) ou une section CDATA soit lu en entier — « Dupont & Fils », non « Fils » —,
**sans** qu'un relevé ou une ligne déjà importés par une version antérieure cessent d'être reconnus comme doublons.

## Contrainte — production

⚠️ **Le Project Lead a importé des relevés réels en 0.17.x.** Aucun essai sur ses fichiers ni sur le NAS : fixtures
synthétiques seulement (consigne commune ; `memory/donnees-reelles-hors-depot-public.md`). **Aucune migration**, aucune
transaction réécrite (C-v0174c-2, C-v0174c-7). Modules : `kesh-import`, `kesh-core`, `kesh-api` (trois).

## Ce que le code dit (7bd4d5c0)

1. **La cause, confirmée.** `parse_with_version` (`crates/kesh-import/src/camt053/mod.rs:114-`) distribue chaque
   `Event::Text` à `handle_text` après `trim()` (`:233-248`) ; tout autre événement tombe dans `_ => {}` (`:249`) — dont
   `Event::GeneralRef` (référence d'entité, émise à part depuis quick-xml 0.39 et toujours en 0.42,
   `crates/kesh-import/Cargo.toml:14`, `quick-xml = "0.42"`) et `Event::CData`. Les champs simples sont posés par
   `= Some(txt.to_string())` (`:330` … `:483`) : le dernier fragment écrase les précédents ; `Ustrd` empile chaque fragment
   comme une partie (`t.ustrd_parts.push`, `:454`), jointes par une espace (`emit_transactions`, `:637`). Le CDATA est lu
   **vide** (commentaire de #819). Prémisse **confirmée** ; elle vaut pour **tous** les champs texte, montants et dates
   compris (un montant écrit en CDATA serait illisible).
2. **La caractérisation fige le défaut.** `crates/kesh-import/tests/camt053_golden.rs` compare la sortie aux références
   `tests/fixtures/camt053_golden/*.json`, produites sous quick-xml 0.39 (`:1-31`), régénération verrouillée (`regen_mode`,
   `:100-114`). La seule fixture à entités et CDATA lus est `v04_characterization.xml` (`:46` `REF &amp; Co &lt;1&gt;`,
   `:48` CDATA, `:68` `Dupont &amp; Fils, &#xE9;tablissement Müller`) ; sa référence porte `"reference": "1"`,
   `"details": ""` (Ntry 1) et `"counterparty_name": "tablissement Müller"` (Ntry 2). Recherche
   `/usr/bin/grep -lE '&[a-zA-Z#]|CDATA' crates/kesh-import/tests/fixtures/camt053/*.xml frontend/tests/e2e/fixtures/camt053_*.xml`
   → `v04_characterization.xml`, `v04_latin1_hors_texte.xml` (CDATA hors du texte lu, refusé depuis 0.42, divergence admise
   C-v017-1-11). L'en-tête de `camt053_tests.rs:3` compte **19** fixtures (`ls … | wc -l`) et `MIN_CRATE_FIXTURES = 19`
   (`camt053_golden.rs:56`) : **aucune fixture n'est ajoutée** à `tests/fixtures/camt053/` (elle demanderait une référence
   0.39 qu'on ne peut plus produire) ; les XML de cette story sont **écrits dans les tests**.
2 bis. **Le lecteur rogne chaque fragment** (trouvé à la validation P1, lentille R, R1 ; lu dans la source) : `parse` pose
   `cfg.trim_text(true)` (`camt053/mod.rs:65`), et quick-xml 0.42 rogne le début de **chaque** texte à chaque entrée dans l'état
   texte — y compris après une référence (`quick-xml-0.42.0/src/reader/mod.rs:308-315`, `InsideRef` → `InsideText` ;
   `:351-352`, `skip_whitespace` si `trim_text_start`) — et la fin de chaque fragment (`src/reader/state.rs:64-72`,
   `emit_text`, `trim_text_end`) ; la documentation de l'option le dit (`src/reader/mod.rs:170-184`, « Incorrect trimming around
   comments, PIs, and CDATA sections »). `Dupont &amp; Fils` arrive donc en `Text("Dupont")`, `GeneralRef(amp)`, `Text("Fils")` :
   accumuler sans changer la configuration donnerait « Dupont&Fils ».
3. **La clé d'un relevé déjà importé ne dépend pas de la lecture.** `create` (CAMT) calcule l'empreinte des **octets**
   (`compute_sha256_hex`, `routes/bank_imports.rs:1197`) et la cherche sous le verrou du compte
   (`find_by_company_and_hash`, `:1266`) ; l'aperçu de même (`:1041`, `:1086`). Une lecture corrigée ne change pas
   l'empreinte : **réimporter un fichier déjà importé reste refusé** (`422 BANK_IMPORT_DUPLICATE_FILE`), sauf confirmation.
4. **La clé d'une ligne en dépend, pour ses identifiants seulement.** `DuplicateKey` (`crates/kesh-core/src/bank_imports.rs:295-300`)
   = date de comptabilisation, montant normalisé, compte, et `reference_normalized` : la première valeur non vide parmi
   `reference` → `end_to_end_id` → `transaction_id`, `trim().to_lowercase()` (`normalize_reference_fallback`, `:373-384`).
   Le nom et le libellé n'en font pas partie (manuel `user-manual.tex:2706`). La clé d'une ligne **change** si l'un de ces
   identifiants porte une entité ou un CDATA — `REF &amp; Co &lt;1&gt;` : « 1 » en 0.17.3, « ref & co <1> » après. Les
   clés existantes viennent de la base (`existing_dedup_keys`, `routes/bank_imports.rs:1894-1914`), les neuves de la lecture
   (`detect_duplicate_lines_for_imported`, `:1919-`), appelée à la création CAMT (`:1337`), CSV (`:2369`) et par l'aperçu
   (`compute_duplicate_lines_warnings`, `:1712-1733`, appelée `:1097` et `:2118`). **Conséquence** : un *autre* fichier qui
   reprend une ligne déjà importée (export qui chevauche) ne la reconnaîtrait plus ; elle s'importerait deux fois.
5. **Ce qui lit le nom et le libellé** : les règles d'affectation (`CounterpartyContains`, `CounterpartyExact`,
   `ReferenceContains`, `crates/kesh-reconciliation/src/rules.rs:19`, `:49-`) et la désignation d'une facture par le nom
   (manuel `:2753`, `:2895`). Une règle « Contrepartie exacte » écrite sur un nom tronqué cessera de reconnaître les imports
   suivants (C-v0174c-7).

## Conception (C-v0174c-3 à 6)

- **Le lecteur de la lecture corrigée ne rogne plus** (C-v0174c-21) : en mode `Joined`, `trim_text(false)` ; le seul `trim()`
  est celui du vidage du tampon, qui rend pour un texte ordinaire exactement le fragment rogné d'aujourd'hui (texte intérieur,
  CRLF compris, intact ; un texte fait de blancs seuls, entre deux balises, se vide sans distribution). Le mode `Legacy0173`
  garde `trim_text(true)` — c'est ce qui le rend identique, bit pour bit, à la 0.17.3. La configuration du lecteur est donc
  posée par le mode : `parse` et `parse_legacy_0173` construisent chacune leur `NsReader` (`check_end_names = true` dans les
  deux). Un blanc entre deux balises produit, sans rognage, un `Text` de blancs que la boucle de détection de `<Document>`
  (`:74-104`) ignore déjà (`_ => {}`).
- **Accumuler, puis distribuer** (C-v0174c-3). Un tampon `pending: String` reçoit `Text` (traitement d'aujourd'hui :
  `into_inner`, `unescape`), `GeneralRef` (résolue) et `CData` (`into_inner`, sans normalisation des fins de ligne, comme le
  texte, Story v017-1 AC9). **Avant tout autre événement** — `Start`, `End`, `Empty`, `Comment`, `PI`, `Decl`, `DocType`,
  `Eof` — le tampon est vidé : `trim()`, puis `handle_text(s, &path, txt)` s'il est non vide (le `path` est encore celui de
  l'élément : le vidage précède le `push` d'un `Start` et le `pop` d'un `End`). Un élément sans entité ni CDATA donne
  exactement les appels d'aujourd'hui.
- **Résoudre** (C-v0174c-4) : `BytesRef::resolve_char_ref()` (`quick-xml-0.42.0/src/events/mod.rs:1724`) pour `&#…;` /
  `&#x…;`, `quick_xml::escape::resolve_predefined_entity` (`src/escape.rs:851`) pour un nom ; sinon `MalformedXml` au
  message rédigé par Kesh — « référence d'entité inconnue : » suivi du nom —, ajouté à `KESH_MESSAGES`
  (`camt053_golden.rs:61-67`, passage de `[&str; 5]` à `[&str; 6]`) ; le garde `inventory_of_kesh_messages_is_closed`
  (`:407-`) en compte les sites.
- **Le mode hérité** (C-v0174c-5) : la boucle porte un paramètre `TextMode { Joined, Legacy0173 }` ; `Legacy0173` est le
  comportement d'aujourd'hui, **à l'identique** (distribution fragment par fragment, `GeneralRef` et `CData` ignorés).
  `parse` (public, `kesh_import::parse_camt053`) lit en `Joined` ; `kesh_import::camt053::parse_legacy_0173(xml)`,
  `#[doc(hidden)]`, en `Legacy0173`, avec la même détection de version. Le type `ImportedStatement` ne change pas.
  **Chemin du mode** (validation P2, P9) : `parse_with_version` (`pub(crate)`) reçoit le `TextMode` ; les fonctions
  **publiques** `camt053::v04::parse` et `v08::parse` (`v04.rs:16-19`, qui reçoivent un lecteur fourni par l'appelant)
  **gardent leur signature** et appellent `parse_with_version(…, Joined)` ; leur doc dit désormais que le lecteur fourni ne
  doit pas rogner le texte (`trim_text(false)`), faute de quoi « Dupont&Fils » (C-v0174c-21). `parse_legacy_0173` ne passe
  pas par elles : elle construit son lecteur à `trim_text(true)`, détecte la version par l'URI comme `parse`, et appelle
  directement `parse_with_version(…, Legacy0173)`. Les deux détections d'URI partagent une fonction privée (DRY).
- **L'alias de clé** (C-v0174c-5). `kesh-core` reçoit une variante de `detect_duplicate_lines` qui prend, en plus des
  brouillons, un tableau `aliases: &[Option<DuplicateKey>]` aligné par index : une ligne neuve est doublon si sa clé **ou**
  son alias égale une clé existante (`DuplicateLine.key` porte la clé qui a égalé). `detect_duplicate_lines` reste et appelle
  la variante avec des alias vides (CSV, saisie manuelle inchangés). Dans `kesh-api`, aux **deux** sites CAMT — aperçu
  (`:1050`, `:1097`) et création (`:1278`, `:1337`) —, le fichier est relu par `parse_legacy_0173`, le même relevé choisi par
  IBAN (`select_statement_by_iban`) ; si la lecture héritée échoue, si sa sélection échoue, ou si les deux lectures n'ont pas
  le même nombre de transactions, **aucun alias** (jamais de `?`) (un tel fichier n'a pas pu être importé en 0.17.3) ; sinon, alias de la ligne *i* = la clé
  héritée de la ligne *i* **si elle diffère** de sa clé neuve. Les deux lectures se font hors du verrou du compte pour
  l'aperçu, sous le verrou pour la création (là où la lecture se fait aujourd'hui).
- **Les transactions déjà importées ne se réécrivent pas** (C-v0174c-7) : elles gardent leur texte tronqué.

## Acceptance Criteria

**AC-1.1 — Les entités.** Tests unitaires de `kesh-import` (`tests/camt053_tests.rs`, XML écrit dans le test, IBAN et noms
inventés) : `<Nm>Dupont &amp; Fils</Nm>` → `counterparty_name = "Dupont & Fils"` ; `&lt;`, `&gt;`, `&quot;`, `&apos;` ;
`&#xE9;tablissement` et `&#233;tablissement` → `établissement` ; **les espaces autour d'une entité sont gardés** :
`<Nm>A &amp; B</Nm>` → `"A & B"`, `<Nm>A&amp;B</Nm>` → `"A&B"`, `<Nm>  A &amp; B  </Nm>` → `"A & B"` (seuls les blancs
extérieurs tombent) ; `<NtryRef>REF &amp; Co &lt;1&gt;</NtryRef>` →
`reference = "REF & Co <1>"` ; une entité en tête (`&amp;Co` → `&Co`) et en queue ; `<Ustrd>A &amp; B</Ustrd>` → **une**
partie, `details = "A & B"`. **Rouges avant correctif.**

**AC-1.2 — Le CDATA.** `<AddtlNtryInf><![CDATA[Versement de Müller à Genève]]></AddtlNtryInf>` → `details` égal à ce texte ;
`<Ustrd>Facture <![CDATA[n° 12]]> octobre</Ustrd>` → `"Facture n° 12 octobre"` (les deux espaces autour du CDATA gardés) ;
`<Nm><![CDATA[ Dupont ]]></Nm>` → `"Dupont"` (rogné au vidage) ; un CDATA aux fins de ligne CRLF les garde.
**Rouges avant correctif.**

**AC-1.3 — Ce qui ne change pas.** Un texte coupé par un commentaire (`<Nm>A<!-- x -->B</Nm>`) se lit comme en 0.17.3 (C-v0174c-3,
écartée b) — test qui le fige. Une entité inconnue (`&foo;`) rend `Err(MalformedXml)` au message de Kesh. Les montants et
dates d'un fichier ordinaire sont inchangés (AC-1.4).

**AC-1.4 — La caractérisation double** (C-v0174c-6), dans `camt053_golden.rs` :
- le **mode hérité** (`parse_legacy_0173`) égale **toutes** les références existantes, `v04_characterization.json` comprise,
  et la divergence admise `v04_latin1_hors_texte` comme aujourd'hui ;
- le **mode corrigé** (`parse_camt053`) égale les mêmes références, **sauf** `v04_characterization`, comparée à
  `v04_characterization.0.17.4.json`, écrite à la main, identique à la référence 0.39 **à trois valeurs près** —
  attendues, après C-v0174c-21 : Ntry 1 `"reference": "REF & Co <1>"`, `"details": "Versement de Müller à Genève"` ; Ntry 2
  `"counterparty_name": "Dupont & Fils, établissement Müller"` (tout le reste, `"currency": "C\tF"` et le CRLF de `details`
  compris, inchangé). Ces valeurs sont **recalculées au développement** : la sortie réelle du mode corrigé est relevée par un
  test unitaire avant d'écrire le fichier ; tout écart avec la liste ci-dessus (une valeur de plus qui change, un blanc) se
  déclare au Change Log et se justifie au code, jamais ne s'aligne en silence. Le mode corrigé, sans rognage du lecteur, doit
  égaler **toutes** les autres références : c'est la preuve que C-v0174c-21 ne change rien aux fichiers ordinaires. L'exception est nominative (une constante, comme `UTF8_EXCEPTION`), et `every_reference_has_an_input`
  (`:260-`) la reconnaît. **La référence 0.39 n'est ni régénérée ni modifiée** (`git diff` vide sur
  `tests/fixtures/camt053_golden/*.json` hors le fichier neuf).
- `characterization_fixture_reaches_the_forms_it_freezes` (`:285-308`) : son dernier commentaire (« ne pas corriger ici »)
  est récrit ; il vérifie en plus les trois valeurs corrigées.
- Le commentaire d'en-tête de `v04_characterization.xml` (`:3`) dit « sortie FAUTIVE figée » : **le fichier XML ne se modifie
  pas** (son empreinte et sa référence 0.39 en dépendent) ; c'est `tests/fixtures/README.md:44` qui dit désormais les deux
  références.

**AC-1.5 — Un relevé importé avant le correctif reste reconnu** (le test que le prompt demande), dans
`crates/kesh-api/tests/bank_imports_e2e.rs`, XML synthétique écrit dans le test (CHF, IBAN du compte du montage, solde
cohérent), dont une ligne porte `<NtryRef>REF &amp; Co</NtryRef>` et une autre `<EndToEndId>E2E &amp; 7</EndToEndId>` sans
`NtryRef` :
- *(a) le montage « 0.17.3 »* : le relevé est enregistré **comme la 0.17.3 l'aurait fait** — en-tête et transactions issus de
  `parse_legacy_0173` (la preuve que cette lecture est celle de la 0.17.3 est AC-1.4), `file_hash` = SHA-256 des octets —, par
  le dépôt (`bank_imports::create_with_transactions`, `crates/kesh-db/src/repositories/bank_imports.rs:72`) ;
- *(b) le même fichier* : `POST /api/v1/bank-imports` rend `422 BANK_IMPORT_DUPLICATE_FILE` ; l'aperçu le signale ;
- *(c) un autre fichier* qui reprend les deux lignes (autre `<Stmt><Id>`, une ligne neuve en plus, soldes recalculés) :
  l'aperçu rend `duplicateLines` sur **les deux** lignes reprises ; `POST` (par défaut `skip`) enregistre **la seule ligne
  neuve** (une transaction en base de plus, `duplicate_lines_skipped: 2` à la trace) ; avec `confirmDuplicateLines=import`,
  trois.
- *(d) le témoin* : le même scénario, montage fait par `parse_camt053` (lecture corrigée) — les deux lignes sont reconnues
  par leur clé neuve, sans alias.

Le montage (a) appelle `parse_legacy_0173`, qui n'existe pas avant le correctif : AC-1.5 **ne compile pas avant** — il n'est
ni rouge ni vert ; sa preuve est sa mutation : (c) **rougit sous MA-819-4** (alias retiré). Pour constater (b) avant
correctif, le développeur peut jouer une variante dont le montage se fait par `parse_camt053` (lecture de la 0.17.3, à ce
moment-là) : verte, comme après (C-v0174c-5 : la clé du relevé ne dépend pas de la lecture).

**AC-1.6 — L'alias ne s'applique qu'au CAMT, se signale à l'aperçu et, à la création, s'écarte par défaut comme tout
doublon, tracé à l'audit (`duplicate_lines_skipped`).** `kesh-core` : la variante avec alias vides rend
exactement ce que rend `detect_duplicate_lines` (tests existants verts sans changement d'assertion) ; un alias égal à la clé
neuve n'est pas **compté deux fois** (la variante tolère un alias égal à la clé, sans `DuplicateLine` en double) ; un alias
qui égale une clé existante produit un `DuplicateLine` dont `key` est celle de l'alias. **Côté `kesh-api`** : si la
lecture héritée échoue, si **sa sélection par IBAN** échoue (`select_statement_by_iban` rend une erreur,
`routes/bank_imports.rs:932-`), ou si les deux lectures n'ont pas le même nombre de transactions, aucun alias n'est passé et
l'import suit son cours — **jamais** de `?` sur la lecture ou la sélection héritées. La règle « l'alias n'est passé que s'il
**diffère** de la clé neuve » est celle de l'appelant (`kesh-api`) et se teste là (une ligne sans entité ne reçoit aucun
alias). **Deux tests nommés** : (i) *lecture héritée en échec* — une date écrite en CDATA, lue vide par le mode hérité
(`MissingRequiredField` ou date invalide) : le relevé s'importe, sans alias ; (ii) *sélection héritée en échec* — l'IBAN du
relevé écrit avec une référence de caractère (`CH93&#48;…`, IBAN du compte du montage) : le mode hérité le lit tronqué,
`select_statement_by_iban` ne trouve aucun relevé, la lecture corrigée le sélectionne ; le relevé s'importe, sans alias.
(Un IBAN en CDATA ne convient pas pour (ii) : le mode hérité échouerait dès la lecture, `finalize_statement`.) Les
tests CSV de `bank_imports_e2e.rs` sont inchangés.

**AC-1.7 — Les transactions existantes ne bougent pas.** Après (c), les deux transactions du montage gardent leur texte
tronqué (`reference = "Co"` pour `REF &amp; Co` — dernier fragment après `trim`, à constater au montage —, nom et libellé
tels que lus en 0.17.3) : relu en base.

## Tests et mutations (P2)

| # | Test | Attendu |
|---|---|---|
| T1-1 | AC-1.1, AC-1.2 (unitaires `kesh-import`) | **rouges avant** |
| T1-2 | AC-1.3 | vert avant (commentaire, montants) ; entité inconnue rouge avant (lue « tronquée » sans erreur) |
| T1-3 | AC-1.4 (deux comparaisons) | la comparaison corrigée rouge avant sur `v04_characterization` seule |
| T1-4 | AC-1.5 (b, c, d), AC-1.7 | ne compilent pas avant (`parse_legacy_0173` absente) ; (c) rouge sous MA-819-4 |
| T1-5 | AC-1.6 (`kesh-core`, et le repli sans alias de `kesh-api`) | rouges avant (fonction absente) |

**Mutations nommées** (jouées sous le verrou de gate, restauration `git checkout -- <fichier>` puis `touch`) :
- **MA-819-1** — le bras `GeneralRef` ne fait rien : T1-1 (entités) rougit.
- **MA-819-2** — le bras `CData` ne fait rien : T1-1 (CDATA) rougit.
- **MA-819-3** — le tampon est vidé après chaque fragment (distribution d'aujourd'hui en mode corrigé) : T1-1 et T1-3
  rougissent.
- **MA-819-4** — l'alias n'est jamais passé (`aliases` vide aux sites CAMT) : T1-4 (c) rougit.
- **MA-819-5** — `parse_legacy_0173` lit en mode corrigé : la comparaison héritée de T1-3 rougit sur
  `v04_characterization`.
- **MA-819-6** — la variante de `kesh-core` ignore l'alias : T1-5 rougit.
- **MA-819-7** — le mode corrigé lit avec `trim_text(true)` (C-v0174c-21 défait) : les cas « espaces autour » de T1-1
  rougissent (« A&B » au lieu de « A & B »).
- *(Pas de mutation « mode hérité sans rognage du lecteur » : elle serait **équivalente** — la boucle héritée rogne déjà chaque
  fragment elle-même (`let txt = unescaped.trim()`, `camt053/mod.rs:242`) et saute un fragment vide ; le `trim_text(true)` du
  mode hérité est gardé pour l'identité de configuration avec la 0.17.3, non parce qu'un test le distinguerait.)*

## Tâches

1. T1-1, T1-2, T1-3, T1-5 rouges d'abord (constatés, notés au Dev Agent Record) ; T1-4 écrit.
2. `kesh-import` : configuration du lecteur par mode (C-v0174c-21), tampon, résolution, `TextMode`, `parse_legacy_0173` ; `KESH_MESSAGES` ; référence
   `v04_characterization.0.17.4.json` ; `README.md` des fixtures.
3. `kesh-core` : variante avec alias, documentée (`///`).
4. `kesh-api` : relecture héritée et alias aux deux sites CAMT ; commentaires de `bank_imports.rs` qui décrivent la clé.
5. Manuel utilisateur (fiche-index, § *Manuel*) et CHANGELOG (§ *CHANGELOG*).
6. Mutations MA-819-1 à 7 ; gates (fiche-index, § *Gates*).

## Dev Notes

- `kesh-import` est **publiable** (`cargo publish --dry-run -p kesh-import` en CI) : `parse_legacy_0173` est `#[doc(hidden)]`,
  documentée comme réservée à Kesh ; aucun type public ne change. L'erreur ajoutée est une variante existante
  (`MalformedXml`).
- La preuve de l'équivalence « mode hérité = 0.17.3 » est AC-1.4 sur **toutes** les fixtures ; ne pas la remplacer par un
  raisonnement.
- **Recette** : aucun écran ne change (le texte importé change) ; capture de l'aperçu d'un import synthétique dont un nom porte
  « & », avant/après (`kesh-gate-logs/v0174c/captures/`), et de l'avertissement de doublon d'AC-1.5 (c).
- Aucun relevé réel : pour décrire un fichier réel, jamais ; les XML des tests sont inventés.

## Change Log

- 2026-11-05 — fiche créée (Opus 5.5, autonomie). Prémisse de #819 **confirmée** et étendue (tous les champs texte, § 1) ;
  clé du relevé indépendante de la lecture (§ 3), clé de ligne dépendante pour ses seuls identifiants (§ 4) → alias de clé
  hérité (C-v0174c-5) ; caractérisation double (C-v0174c-6). Aucune migration.
- 2026-11-05 — **Validation P1 (Sonnet, R et F)** : sur cette fiche, R1 (HIGH), R6, R7 (LOW). Prémisse de R1 lue dans la source
  de quick-xml 0.42 (`reader/mod.rs:170-184`, `:308-315`, `:351-352` ; `reader/state.rs:64-72`) : confirmée — le lecteur rogne
  chaque fragment ; remède C-v0174c-21 (lecture corrigée sans rognage, mode hérité inchangé), cas « espaces autour », MA-819-7
  (une variante sur le mode hérité serait équivalente, dit), référence recalculée au développement. R6 : AC-1.5 non compilable avant, dit ; « jamais écartée en silence » reformulé.
  R7 : repli sans alias sur échec de la sélection héritée, testé.
- 2026-11-05 — **Validation P2 (Opus, lentille complète)** : sur cette fiche, LOW seulement — P4 (MA-819-1 à **7** aux tâches),
  P6 (deux tests nommés du repli sans alias, règle « alias différent » rattachée à `kesh-api`), P8 (**deux** sites CAMT, non
  trois), P9 (chemin du mode jusqu'à `parse_with_version`, signatures publiques de `v04::parse` / `v08::parse` inchangées).
  *(Entrée déplacée au développement de la section « File List », où elle avait été écrite, vers ce Change Log ; texte
  inchangé.)*
- 2026-11-05 — **Développement** (Opus 5.5, autonomie), commit `16039590`. Étape 0 : la référence de caractérisation de la
  lecture corrigée **recalculée** — `v04_characterization.0.17.4.json` écrit depuis la référence 0.39 à trois valeurs près,
  puis comparé à la sortie réelle de `parse_camt053` par `corrected_reading_matches_references_except_the_characterization` :
  **égalité, écart nul** (aucune autre valeur ne change, `"currency": "C\tF"` et le CRLF de `details` compris). Statut → review.
- 2026-11-06 — **Revue de code P1 (Sonnet, B, E, A)** : B 0/0/1/4, E 0/0/1/5, A 0/0/1/6 ; le MEDIUM unique, convergé
  (B-1 = E-4 = A-1 : la lecture corrigée d'un relevé v08 sans test ni mutation), remédié (`f3b5d5b1`) avec les LOW B-3, A-3,
  A-5, A-6, A-7 ; prémisses relues au code. Prochaine : revue P2 ciblée (Sonnet, `v0174-correctifs-review-prompt-p2-1.md`).

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`claude-opus-5-5`), autonomie, worktree `/home/gcorbaz/devel/kesh-0174c`.

### Completion Notes List

- **Rouges d'abord** (`kesh-gate-logs/v0174c/rouges.log`, `rouges-819-import.log`, `rouges-819-core.log`) : `camt053_tests`
  7 rouges / 24 (entités du nom, références de caractère, espaces autour, entité en tête et en queue, référence et `Ustrd`,
  CDATA, entité inconnue) — le commentaire et le témoin ordinaire verts, comme prévu ; la caractérisation corrigée rouge sur
  `v04_characterization` seule ; les trois tests d'alias de `kesh-core` ne compilaient pas. AC-1.5 ne compilait pas avant
  (dit par la fiche).
- **Lecture** (`camt053/mod.rs`) : `reader_for(mode)` (`trim_text(true)` en `Legacy0173` seulement, C-v0174c-21),
  `detect_version` partagé, `TextMode`, tampon `pending` vidé avant tout événement autre que `Text`/`GeneralRef`/`CData`
  (`flush_pending`) ; `resolve_char_ref` puis `resolve_xml_entity` (C-v0174c-22) ; `KESH_MESSAGES` 5 → 6.
  `parse_legacy_0173` `#[doc(hidden)]` ; `v04::parse`/`v08::parse` gardent leur signature (doc : lecteur sans rognage).
- **Alias** : `kesh_core::bank_imports::detect_duplicate_lines_with_aliases` (`detect_duplicate_lines` l'appelle à vide) ;
  `kesh-api` `legacy_0173_aliases` aux deux sites CAMT, `&[]` aux deux sites CSV (C-v0174c-23).
- **Tests neufs** : `camt053_tests.rs` 9 (`*_819`) ; `camt053_golden.rs` deux comparaisons (`legacy_reading_…`,
  `corrected_reading_…`) et `characterization_fixture_reaches_the_forms_it_freezes` étendu ; `kesh-core` 3 ;
  `kesh-api` 3 unitaires (`legacy_alias_tests`) et 5 E2E (`bank_imports_e2e.rs`, `*_819`). Ciblé : `cible-819-api.log`,
  8/8 verts.
- **Mutations** (`kesh-gate-logs/v0174c/mut/mutations.log`, `mut/MA-819-*.log`) : MA-819-1, -2, -3, -4, -5, -6, -7 **toutes
  tuées** (7/7). MA-819-4 rougit déjà au (b) d'AC-1.5 (l'aperçu du même fichier ne signale plus ses lignes), puis au (c).
- **Angle mort déclaré** : « les deux lectures n'ont pas le même nombre de lignes » n'a pas de montage (C-v0174c-23).
- **Faux doublon par alias** (limite assumée, index (10)) : non testé à part ; borné aux lignes dont la clé change.
- **Écart de méthode déclaré** : deux éditions de fichiers de test faites par script au lieu de l'outil Edit
  (`camt053_golden.rs`, trois remplacements à motif vérifié unique ; ajout par `cat >>` à `camt053_tests.rs`) — contraire à
  la consigne 14 ; le contenu a été relu et compilé. Le reste par Edit. *(Corrigé à la remédiation P1, A-5 : la rédaction
  précédente nommait aussi `supplier_invoice_draft_e2e.rs`, qui est de la fiche 2.)*
- **Remédiation de la revue P1** (commit `f3b5d5b1`) : test `v08_corrected_reading_resolves_entities_and_cdata_819`
  (camt.053.001.08, entité, référence de caractère, CDATA ; B-1 = E-4 = A-1) et **mutation MA-819-8** (`v08.rs` en
  `Legacy0173`) **tuée** (`mut/MA-819-8.log`, 1 échec sur 17 : le test neuf) ; `invalid_char_refs_are_malformed_819`
  (`&#0;`, `&#xD800;`, B-3, vert d'emblée : il fige la branche) ; le témoin (d) d'AC-1.5 asserte que les clés qui ont égalé
  sont les clés neuves (`|ref & co|`, `|e2e & 7|`, A-3) ; manuel et CHANGELOG disent les deux limites de la reconnaissance
  par alias (#915, A-6) ; README des fixtures (A-7). Ciblé : 11/11 (`*_819` de `kesh-import`). Total des mutations de la
  fiche : 8/8 tuées.
- Gates : au dernier commit de code de la story, fiche-index § *Dev Agent Record (gates)*.

### File List

- `crates/kesh-import/src/camt053/mod.rs`, `v04.rs`, `v08.rs`
- `crates/kesh-import/tests/camt053_tests.rs`, `camt053_golden.rs`, `fixtures/README.md`
- `crates/kesh-import/tests/fixtures/camt053_golden/v04_characterization.0.17.4.json` (neuf)
- `crates/kesh-core/src/bank_imports.rs`
- `crates/kesh-api/src/routes/bank_imports.rs`, `crates/kesh-api/tests/bank_imports_e2e.rs`
- `docs/manual/fr/user-manual.tex` (commit `16039590`) et son PDF (régénéré aux commits de documentation `3ba6e053`, puis
  après la remédiation P1), `CHANGELOG.md`
