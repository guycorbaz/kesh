# Prompt — revue de code P1 de la Story v017-1 (quick-xml 0.39 → 0.42), trois lentilles, Sonnet

Tu relis le développement de la **Story v017-1** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11),
release 0.17.0 : **#789** (P1, sécurité) — la montée de `quick-xml` de 0.39.4 à 0.42.0 dans `kesh-import` (lecture
CAMT.053) et `kesh-payment` (écriture pain.001), qui corrige RUSTSEC-2026-0194 (contrôle quadratique des attributs en
double) et RUSTSEC-2026-0195 (déclarations d'espace de noms non bornées). La contrainte du Project Lead du 2026-10-30
(`/home/gcorbaz/devel/kesh-gate-logs/prompts/v017-donnees-a-garder.md`, à lire) pèse sur toute la story : **un relevé
CAMT.053 déjà importé en 0.16 doit se lire exactement comme avant** — même sortie du parseur, même empreinte de fichier,
mêmes doublons. **Aucune migration, aucun texte visible, aucun manuel.**

Worktree `/home/gcorbaz/devel/kesh-0171`, branche `story/v017-1-quick-xml`, partie de `c5101b07`. Le diff à relire,
**aplati** : `git -C /home/gcorbaz/devel/kesh-0171 diff c5101b07...HEAD`. Trois commits de code, dans cet ordre
**imposé par la fiche** (§ *Gates*) :

- **A** `11ef8b69` — caractérisation sous 0.39 : `crates/kesh-import/tests/camt053_golden.rs`, `camt053_hostile.rs`,
  cinq fixtures neuves sous `tests/fixtures/camt053/`, les références `tests/fixtures/camt053_golden/*.json`,
  `crates/kesh-payment/tests/pain001_special_chars.rs` et sa référence ; aucun fichier sous `crates/*/src/`, aucun
  `Cargo.*` ;
- **B** `2d608d4a` — la montée : deux `Cargo.toml`, `Cargo.lock`, `crates/kesh-import/src/camt053/mod.rs` ;
- **C** `35f67172` — `crates/kesh-api/src/routes/bank_imports.rs` (test AC10), `crates/kesh-api/tests/bank_imports_e2e.rs`
  (AC8, AC11), `CHANGELOG.md` (AC14).

Puis des commits de comptes rendus (`_bmad-output/`). Langue du rapport : français. Modèle : **Sonnet**. Passe
**complète** (première passe d'une revue de code). Contexte frais : ne t'appuie sur aucune passe antérieure.

⚠️ **Les trois propriétés centrales.**
1. **Rien ne change de sens.** Sous 0.42, le parseur rend, pour chaque fichier CAMT.053 du dépôt, **exactement** ce
   qu'il rendait sous 0.39 (références produites au commit A), à la seule exception admise de C-v017-1-11 (un fichier
   non UTF-8 hors du texte lu, désormais refusé). Les adaptations de `mod.rs` (texte par `into_inner()`, valeur
   d'attribut par `quick_xml::escape::unescape(&attr.value)`) en sont la condition : une normalisation glissée
   (`xml10_content`, `normalized_value`) est un défaut, même si un test reste vert.
2. **Les deux avis sont fermés, et les tests le prouvent pour la bonne raison.** Les rouges du commit A (journal
   `kesh-gate-logs/v017-1-rouge-0.39.txt`) ont-ils la cause attendue (délai pour AC5, `Ok` au lieu d'`Err` pour AC6 (a)(b),
   fil mort par `attempt to add with overflow` pour AC7), et les tests verts sous 0.42 ne passent-ils pas à vide ?
3. **La caractérisation ne peut pas être régénérée pour masquer un écart.** Le verrou de `KESH_GOLDEN_REGEN`, la règle
   « après A, rien de la caractérisation ne bouge » (contrôles git du § *Gates* de la fiche) et le garde d'inventaire des
   messages `MalformedXml` tiennent-ils ?

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/v017-1-quick-xml.md` — *Ce que le code dit*, *Établi dans la source 0.42.0*,
les **AC1 à AC14**, *Tâches*, *Tests et mutations prévus* (M2 à M7-bis), *Gates*, *Ce que les gardes ne voient pas*, le
**Dev Agent Record** et le **Change Log** ; puis le registre `v017-1-quick-xml-choix-autonomes.md`, **C-v017-1-1 à 11**
(fiche) et **C-v017-1-12 et suivants** (choix du développement). Le Dev Agent Record déclare des rouges, des gates, des
mutations et un E2E : **ne les prends pas pour argent comptant**, recoupe-les avec les journaux hors dépôt (lecture
seule) : `/home/gcorbaz/devel/kesh-gate-logs/v017-1-rouge-0.39.txt`, `v017-1-golden-vert-0.39.txt`,
`v017-1-octets-fixtures.txt`, `v017-1-cargo-audit.txt`, `v017-1-publish-dry-run*.txt`, `v017-1-mutations.txt`,
`v017-1-gate/*.log`, et les scripts `v017-1-mut/run.sh`, `v017-1-mut/mutate.py`, `v017-1-gate/gate.sh` (à **lire**,
jamais à exécuter). La source de `quick-xml` 0.42.0 est lisible en
`/home/gcorbaz/devel/kesh-gate-logs/v017-1-src/quick-xml-0.42.0/` (lecture seule) ; celle de 0.39.4 en
`/home/gcorbaz/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/quick-xml-0.39.4/` (lecture seule).

⛔ Ne lis **rien** sous `~/kesh-test/` (relevés réels de Guy).

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp`, ni ailleurs ; **aucune redirection** (`>`, `>>`,
  `tee`, `2>`, `&>`, `2>&1`) : compare par des commandes enchaînées (pipe). Un fichier créé par erreur se **déclare** au
  rapport, il ne se supprime pas.
- **Aucun `rm`, sous aucune forme** (ni `rm -f`, ni `rm -rf`, ni `unlink`, ni `find -delete`) ; ni **`pkill`**, ni
  **`killall`**.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`, `fetch`, `merge`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python`/**`python3`**, ni `perl`, ni `ruby`,
  ni `node`/**`npm`**/`npx`, ni `bash -c`/`sh -c` d'un script, ni `jq` ; ni **`scripts/prepare-release.sh`**, ni
  `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni **aucun script du dépôt** ni de `kesh-gate-logs` (dont
  `run.sh`, `mutate.py`, `gate.sh`) ; ni **`make`**, ni `latexmk`, ni **`cargo`** (pas même `cargo tree`, `cargo audit`
  ou `cargo publish --dry-run` : leurs sorties sont dans les journaux), ni **`docker`**, ni `mysql`/`mariadb`, ni
  `sqlx`, ni **`curl`**, ni **`claude`** sous aucune forme, ni **`sed -i`**, ni `iconv` vers un fichier, ni `playwright`.
- **Aucune commande `gh`**, pas même en lecture. **Aucun accès réseau** : les avis RustSec se jugent sur la fiche et
  sur `~/.cargo/advisory-db/crates/quick-xml/RUSTSEC-2026-019{4,5}.md` (lecture).
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`, `cmp`, `od`, `head`, `tail`,
  `sha256sum`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- ⚠️ **Octets** : dans certains shells, `grep` est une fonction qui appelle `ugrep -I` et **ignore les fichiers
  binaires** — un `grep -c $'\xe9'` y rend 1 sans sortie sur un fichier qui porte l'octet. Pour les octets non ASCII,
  emploie `command grep` ou `od -c` (C-v017-1-18).
- ⛔ **Sur un PDF aplati, jamais de `grep -oE` ni de `grep -E` à quantificateur `{0,N}` ou `{N,M}`** : `grep -cF` ou
  `grep -oF` sur des **motifs fixes**, sous `timeout 60`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée dans le
  rapport.
- **Données réelles** : aucune ; les fixtures sont synthétiques (IBAN d'exemple public, noms inventés).

## Les axes obligatoires, pour chaque lentille

1. **Le manuel et les textes.** La fiche dit qu'aucun manuel ne bouge (AC13) : vérifie-le
   (`git diff --stat c5101b07...HEAD -- docs/manual` vide, PDF compris) **et** vérifie que le manuel ne dit rien que la
   montée rend faux — l'import CAMT.053 et son refus « XML mal formé », l'encodage d'un relevé (ISO-8859-1 / UTF-8), les
   doublons de fichier et de ligne : `grep -nF` sur `docs/manual/fr/*.tex` (`CAMT`, `ISO-8859`, `UTF-8`, `mal formé`,
   `doublon`), puis le **PDF aplati** sous `timeout 60` avec motifs fixes. Un fichier non UTF-8 désormais refusé
   (C-v017-1-11) : le manuel promet-il quelque part qu'un tel fichier s'importe ? Le `CHANGELOG.md` (AC14) : les quatre
   faits obligatoires, rien de plus large que les deux avis, liens au format du fichier, en-tête `## [0.17.0]`
   provisoire.
2. **La caractérisation (AC9) et ses gardes.** `camt053_golden.rs` : le verrou (`regen_mode`, lecture du `Cargo.lock`,
   `CI`) ; l'exception nominative et conditionnée de `v04_latin1_hors_texte` ; la représentation (C-v017-1-12) — un
   `MalformedXml` de Kesh peut-il être pris pour un relais, ou l'inverse ? `KESH_MESSAGES` et le garde
   `inventory_of_kesh_messages_is_closed` (retrait des commentaires qui respecte les chaînes, appariement des
   parenthèses, expression `,?`) ; les marqueurs de `characterization_fixture_reaches_the_forms_it_freezes`. Les
   références sont-elles **plausiblement** celles de 0.39 (lis `v04_characterization.json` : la sortie fautive des
   entités et du CDATA, #819, y est-elle figée telle quelle ?) ; les fixtures portent-elles les octets annoncés (`od -c`,
   `command grep`) ; `git ls-files --eol` sur `v04_characterization.xml` (CRLF conservé) ?
3. **Les tests hostiles (AC5 à AC7) et les mutations.** `camt053_hostile.rs` : `releve()`, la valeur par défaut du
   `Ntry/Amt` (C-v017-1-14), le témoin d'AC5, `parse_borne` (`Timeout` / `Disconnected`, `#[track_caller]`), les
   tailles. Chaque rouge du journal `v017-1-rouge-0.39.txt` a-t-il la cause attendue ? Chaque mutation M2 à M7-bis du
   journal `v017-1-mutations.txt` est-elle tuée **par le test visé** et pour la raison visée, M7-bis sans rouge ? Une
   mutation plausible non jouée survivrait-elle (par exemple : `trim` supprimé, `check_end_names` retiré, `unescape` du
   texte retiré) ?
4. **Les contrôles git du § *Gates*.** Rejoue les neuf commandes (quatre contrôles, cinq sondes) avec
   `A=11ef8b69` et `BASE=c5101b07` — elles sont en lecture seule — et compare aux sorties copiées au Change Log.

## Lentille B — Blind (le diff seul, sans la fiche)

1. **Le code de production** (`camt053/mod.rs`) : chaque site adapté à l'API 0.42 conserve-t-il la sémantique
   d'avant (lis la source 0.39.4 et 0.42.0 aux lignes en cause) — `local_name().as_ref()` comparé à `"Document"`,
   `Namespace(uri)`, `Unknown(prefix)` et son message, `into_inner()` contre l'ancien `decode()`, `unescape(&attr.value)`
   contre l'ancien `unescape_value()` ? Les doc-comments disent-ils vrai (la branche « nom non UTF-8 » est-elle vraiment
   inatteignable) ? `kesh-payment` compile sans changement : `BytesText::new`, `push_attribute`, `BytesDecl::new` ont-ils
   le même comportement d'échappement (hors `\r`, C-v017-1-9) ?
2. **Les tests** : `camt053_golden.rs`, `camt053_hostile.rs`, `pain001_special_chars.rs`, les tests de `kesh-api`
   (`post_preview_rejects_hostile_and_non_utf8_xml_as_malformed`, `post_preview_of_imported_file_flags_file_and_every_line`,
   `file_hash_is_sha256_of_received_bytes`) : un cas peut-il passer à vide ? la constante SHA-256 est-elle celle du
   fichier (`sha256sum frontend/tests/e2e/fixtures/camt053_v04_minimal.xml`) ?
3. **Les manifestes** : `git show 2d608d4a -- Cargo.lock` ne touche-t-il que `quick-xml` ? Aucune `features` ajoutée ?
   Aucun `#[allow(deprecated)]` ? Aucun mot-clé de fermeture (`closes`, `fixes`, `resolves`) dans les messages de
   commit (`git log c5101b07..HEAD --format=%B`) ?

## Lentille E — Edge cases

1. **Entrées CAMT.053** : un `Ccy` à `\r` ou `\n` littéral, à entité `&#9;`, vide ; un texte à `\r` seul (Mac) ou à CRLF
   en bord (rogné par `trim`) ; un `<Amt Ccy="…">` en `Event::Empty` ; un préfixe non lié sur `<Document>` sous 0.42 —
   le message de Kesh sort-il encore (ou `quick-xml` 0.42 refuse-t-il avant, ce qui changerait la référence d'un
   fichier) ? une déclaration d'encodage `UTF-16` ; un BOM UTF-16 ; un relevé à 129 déclarations réparties sur plusieurs
   niveaux (plafond **global** de 0.42) — un fichier bancaire réel peut-il l'atteindre ?
2. **Les chemins API** : en **création** (`POST /api/v1/bank-imports`), le doublon de fichier est cherché avant le parse
   (`routes/bank_imports.rs:1237` contre `:1249`), en aperçu après (`:1045` contre `:1081`) : les conséquences écrites
   au § *Ce que les gardes ne voient pas* et au CHANGELOG sont-elles exactes ? Le plafond d'upload
   (`KESH_BANK_IMPORT_MAX_MB`) et les tailles des documents hostiles.
3. **pain.001** : un nom qui contient `\r`, `\t`, un caractère hors BMP, `]]>` — la sortie 0.42 diffère-t-elle de 0.39
   ailleurs que pour `\r` (lis `src/escape.rs` des deux versions) ?

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC (**AC1 à AC14**), rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve.
En particulier : AC1 (`grep -c 'name = "quick-xml"' Cargo.lock`, manifestes) ; AC2 (journal `v017-1-cargo-audit.txt` :
ni 0194 ni 0195, autres avis déclarés) ; AC3 (journal du `publish --dry-run`, invariant `ci.yml:140-147`) ; AC4 ; AC5 à
AC7 (journaux rouge et vert) ; AC8 ; AC9 (inventaire des fichiers comparés, exception, verrou, garde, octets) ; AC10
(constante, diff limité au test : `git diff c5101b07...HEAD -- crates/kesh-api/src/routes/bank_imports.rs
crates/kesh-api/src/routes/bank_import_fingerprint.rs`) ; AC11 ; AC12 (`pain001_sample.xml` inchangé) ; AC13 ; AC14. Le
§ *À recontrôler à l'étape 0* et les quatre LOW de la validation P4 : chaque point est-il recontrôlé au Dev Agent
Record, ligne lue citée ? L'ordre des commits A, B, C et les contrôles du § *Gates* (sorties au Change Log).

Et : les choix C-v017-1-12 et suivants sont-ils justifiés, l'un contredit-il un AC, la fiche ou le registre ? Chaque
nombre du Dev Agent Record (tests, mutations, gates, E2E) se recompte-t-il depuis sa source, avec son périmètre ? La
contrainte du 2026-10-30 (`v017-donnees-a-garder.md`) : la story ajoute-t-elle une table, une migration, ou quoi que ce
soit qui empêche l'import d'une sauvegarde 0.16 ? Les **défauts observés** que le Dev Agent Record demande en issue
sont-ils réels (prémisse lue au code) ?

## Rapport

Écris ton rapport dans **un seul** fichier : `/home/gcorbaz/devel/kesh-gate-logs/v017-1-quick-xml-review-p1-B.md`
(lentille B), `/home/gcorbaz/devel/kesh-gate-logs/v017-1-quick-xml-review-p1-E.md` (lentille E) ou
`/home/gcorbaz/devel/kesh-gate-logs/v017-1-quick-xml-review-p1-A.md` (lentille A), selon la lentille qui t'est confiée
— par l'outil d'écriture de fichier, **jamais** par une redirection. Contenu :

1. **Les findings**, chacun : identifiant (`B1`, `E1`, `A1`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier
   et ligne, la preuve (sortie de commande copiée, `grep -nF` pour toute présence ou absence), la correction proposée,
   et s'il est **né du développement** de la v017-1 ou **antérieur** (un défaut antérieur se **signale**, il ne se
   corrige pas ici).
2. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
3. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
4. Les fichiers créés par erreur, et toute commande lancée hors de la liste permise, s'il y en a.
