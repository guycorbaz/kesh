# Prompt — revue de code P1 de la Story 32-9b (les outils de lecture des contacts et des pièces), trois lentilles, Opus

Tu relis le développement de la **partie b de la Story 32-9** de Kesh (comptabilité suisse, Rust/Axum, MariaDB) :
*le service des pièces importées `services::pieces` (`lister_tout` pour l'API, `lister` et `PiecePublique` pour le
MCP — sans empreinte, sans chemin de fichier, sans rien de ce qui est lu du QR-code) et la migration de
`list_imported` ; trois outils MCP — `kesh_contacts` et `kesh_contact` (`contacts:read`), `kesh_pieces`
(`documents:read`, métadonnées seules) — et le registre à dix-neuf outils ; les pièges du client C et d'une seconde
pièce P2 dans `kesh_seed::pieges` ; le masquage des tiers éprouvé sous portées combinées (#781, point 3) ; la lecture de
chaque champ des projections (`PiecePublique`, `factures::detail_sans_fichier`) ; la garde `mcp_sans_fichier` étendue
aux imports groupés coupés ou imbriqués ; la matrice du manuel.* Un défaut ici fait sortir une donnée personnelle sous
une portée qui ne l'ouvre pas, le contenu d'une pièce (IBAN, créancier, référence lus du QR), un texte non enveloppé,
ou une donnée d'une autre société.

Worktree `/home/gcorbaz/devel/kesh-32-9`, branche `story/32-9b`. Le diff à relire, **aplati** (un seul diff, pas la
suite des commits) : `git -C /home/gcorbaz/devel/kesh-32-9 diff origin/main...HEAD`. Langue du rapport : français.
Modèle : **Opus**. Contexte frais : ne t'appuie sur aucune passe antérieure.

⚠️ **Seule la partie b est en revue.** La partie a (socle `mcp/sortie.rs`, seize outils, `detail_sans_fichier`) est
fusionnée et revue (deux passes) ; ses LOW restants sont tracés dans #781. Ne relis la partie a que là où la 32-9b la
touche ou s'y appuie. Les prompts standard sont la 32-10a (développée en parallèle, ailleurs : son absence n'est pas
un finding) ; le contenu des pièces viendra en 0.17.0 (spike, E3) — **aucune ressource MCP** n'est attendue.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/32-9-outils-de-lecture.md` — *Ce que le code dit*, *La conception, en une
page*, *L'inventaire clos des champs libres rendus* (partie **32-9b**), *Les jeux d'arguments* (lignes 32-9b), *Les
pièges et la largeur des colonnes* (lignes 32-9b), *Les tailles mesurées* (`kesh_contacts`, `kesh_pieces`), *Partie
B* (AC-b1 à AC-b7), *Tests et mutations prévus* (Mb1 à Mb7), *Ce que les gardes ne voient pas*, *Dev Notes*, *À
recontrôler à l'étape 0*, et le **Dev Agent Record** de la partie b (étape 0, livré, tailles, mutations, gates, écarts)
— puis le registre `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-329.md`, choix **C-32-9-60 à
C-32-9-63**. Le Dev Agent Record déclare des gates, des mutations et un E2E : **ne les prends pas pour argent
comptant**, recoupe-les avec les journaux `/home/gcorbaz/devel/kesh-gate-logs/329b-*` (lecture seule). Aucun contrôle
visible ne change : pas de capture attendue.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp`, ni ailleurs ; **aucune redirection** (`>`, `>>`,
  `tee`, `2>`) : compare par des commandes enchaînées (pipe), lis les PDF vers la sortie standard seulement. Un fichier
  créé par erreur se **déclare** au rapport ; il ne se supprime pas (supprimer, c'est écrire ; aucun `rm`).
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `merge`, `worktree`, `add`, `fetch`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python`/`python3`, ni `perl`, ni `ruby`, ni
  `node`/`npm`/`npx`, ni `bash -c`/`sh -c` d'un script, ni `jq` ; ni **`scripts/prepare-release.sh`**, ni
  `scripts/regen-test-schema.sh`, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`,
  ni aucun script de `kesh-gate-logs` ou d'un scratchpad ; ni `make`, ni `latexmk`, ni `cargo` (aucune sous-commande,
  `cargo fmt` et `cargo tree` compris), ni `rustc`, ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni `curl`, ni
  `sed -i`.
- **Aucune commande `gh`**, pas même en lecture.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée.
- **Données réelles** : aucune ; les fixtures sont synthétiques (`PIEGE_329_*`, `P329*`). Ne lis rien sous
  `~/kesh-test/`.

## Les trois axes obligatoires, pour chaque lentille

1. **Tout ce qui sort par `/mcp`, énuméré.** Pour les **trois outils neufs** (et pour tout outil de la 32-9a dont la
   32-9b change la sortie), dresse depuis le code la liste **close** des champs de sa structure `sortie!` et de ses
   sous-structures, avec leur type ; pour chaque champ texte de la base (`Contact`, `ImportedSupplierInvoice` via
   `PiecePublique`), dis s'il sort enveloppé (`TexteNonFiable`), masquable (`Tiers`), retypé (`Code`), ou non rendu —
   et confronte cette liste à l'inventaire de la fiche (partie 32-9b) et à la constante `INVENTAIRE` de
   `tests/mcp_enveloppe_e2e.rs`. Pour `ImportedSupplierInvoice`, énumère **chaque** champ de l'entité et dis par quel
   chemin il pourrait atteindre `src/mcp/` (la déstructuration de `From<ImportedSupplierInvoice> for PiecePublique`).
   Énumère aussi ce qui sort **hors** des structures : le résumé (`Resume`, déposé à l'écran sous `required` — la
   recherche `search` n'y entre-t-elle que par sa présence ?), les textes d'erreur (un `Validation` du service recopie
   le statut reçu : atteint-il l'agent ?), les descriptions et schémas de `tools/list`. Un champ de chaîne qui sortirait
   brut, ou une donnée lue du QR qui sortirait, est un CRITICAL.
2. **Chaque motif greppé sur le dépôt réel.** Les gardes amendées de `crates/kesh-api/tests/mcp_gardes_lexicales.rs` —
   `mcp_sans_fichier` (motif ligne à ligne **et** `imports_groupes_de_fs` sur texte aplati), `outils_et_politique_en_leur_lieu`
   (dix-neuf `impl McpTool`), `sorties_sans_serialisation_parallele` — et les listes de `services_motif.rs`
   (`HANDLERS_MIGRES` avec `list_imported`, `SITES_NON_RESOLUS` avec `routes/imported_supplier_invoices.rs` à 9 / 12 et
   huit gestionnaires, le compte de `services/` à quinze). Pour **chaque** garde : greppe toi-même son motif sur son
   périmètre, confronte la sortie à la liste écrite, cherche une forme **plausible** que le motif ne voit pas (alias
   `use … as`, chemin qualifié, coupure de ligne, macro, `use crate::…::{fs}`), et dis si elle est écrite comme angle
   mort.
3. **Chaque dépendance lue à la source.** Toute affirmation du code ou du Dev Agent Record sur `serde` (dérivation de
   `Serialize` sur `ContactType`, `Salutation`, `Language` — `rename_all = "UPPERCASE"` de `Language` —, `Deserialize`
   d'une énumération sans `rename_all` pour `contactType`, `snake_case` de `StatutPiece`), sur le dépôt des contacts
   (`contacts::create` compose `address` et ignore `NewContact.address` ; `client_number_columns`), sur le dépôt des
   pièces (`imported_supplier_invoices::create`, `list_by_status`, l'ordre `created_at DESC, id DESC`, la contrainte
   « tout ou rien » `20261015000001_imported_documents_without_qr.sql:36-50`) se vérifie à la ligne dans
   `crates/kesh-db` ou dans `~/.cargo/registry/src/*/<crate>-<version>/` (ligne citée) ; sinon, dis-le.

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention.

1. **`crates/kesh-api/src/services/pieces.rs`** — l'exigence précède-t-elle toute lecture et toute validation ? La
   société est-elle lue avant la validation, comme l'ancien gestionnaire ? `PiecePublique` peut-elle porter, par un
   détour, un champ du QR, l'empreinte ou le chemin ? `lister` écrête-t-il `limit` et `offset` sans panique
   (`usize::try_from`, `i64::MAX`) ? Le `total` est-il celui du statut entier ?
2. **`crates/kesh-api/src/routes/imported_supplier_invoices.rs`** — `list_imported` rend-il exactement la même réponse
   qu'avant (statut absent, vide, hors domaine, message d'erreur, ordre, rôle `Consultation` refusé) ? Le rôle API
   exigé par le service (`Comptable`) égale-t-il le RBAC de la route (`lib.rs`, bloc `comptable_routes`) ?
3. **`crates/kesh-api/src/mcp/outils/{contacts,pieces,commun}.rs`, `mcp/sortie.rs`, `mcp/registre.rs`** — chaque
   projection est-elle exhaustive (aucun `..`) ; les arguments (`deny_unknown_fields`, `statut` requis, `limit`,
   `offset`, énumérations) refusent-ils ce qu'ils doivent ; les schémas promettent-ils exactement ce que les structures
   d'arguments acceptent (noms, énumérations, `required`, `maximum`, `default`) ? `champ_enum!` sur `ContactType`,
   `Salutation`, `Language` : un `Serialize` écrit à la main y rendrait-il une chaîne de la base ? Le `pays` (deux
   caractères) est-il enveloppé ? L'ordre des dix-neuf outils.
4. **`crates/kesh-seed/src/pieges.rs`** — `semer` passe-t-il par les dépôts seuls ? Les textes tiennent-ils dans leurs
   colonnes ; les numéros 18 à 39 sont-ils uniques ; P2 respecte-t-elle « tout ou rien » et `UNIQUE (company_id,
   file_hash)` ? La fonctionnalité `pieges` reste-t-elle hors du binaire de production ?
5. **Les tests** (`mcp_enveloppe_e2e.rs`, `mcp_outils_e2e.rs`, `services_rollout.rs`, `services_motif.rs`,
   `mcp_gardes_lexicales.rs`, `mcp_transport_e2e.rs`, `mcp_matrice_manuel.rs`, `tests/common/mcp_pieges.rs`) — un test
   peut-il passer **à vide** : `pieces_sans_donnees_du_qr` parcourt-il une sortie qui contient réellement P2 ;
   `coordonnees_masquees_sous_portees_combinees` voit-il des outils des trois portées et des pièges sortir ; les tests
   « chaque champ » ont-ils des valeurs voisines **distinctes** (sans quoi une transposition passe) ; `imports_groupes_de_fs`
   a-t-il des témoins rouges et verts réalistes ?

## Lentille E — Edge cases

1. **Le masquage sous portées combinées (#781, point 3)** : sous `books:read` + `invoices:read` + `documents:read`
   sans `contacts:read`, une donnée du carnet (courriel, adresse, téléphone, IDE, numéro de client, conditions) ou une
   coordonnée de tiers peut-elle sortir par un autre outil — un libellé que Kesh compose, l'adresse d'envoi d'une
   facture (`emailed_to`), un QR de pièce (créancier, IBAN), le nom de fichier d'une pièce ? Les conditions de paiement
   d'une facture recopiées de celles du contact ?
2. **Les pièces** : un `original_filename` ou un `mime_type` qui porte une injection, des caractères cachés, `"}` ; une
   pièce `discarded` ; une pièce d'une autre société (cloisonnement) ; `limit` à 0, 66, `i64::MAX`, `offset` au-delà du
   total ; un statut en casse différente (`TO_COMPLETE`) ; `qrLu` pour une pièce sans QR.
3. **Les contacts** : un contact archivé (`includeArchived`), un contact d'une autre société par `kesh_contact`
   (indiscernable d'un inexistant ?), `contactType` en minuscules, `sortBy` inconnu, `search` très long (aucune borne :
   faut-il en poser une ?), la sortie d'un contact `Personne` sans prénom.
4. **Les tailles** : contacts 1 038 o par élément pour 971 à la fiche (+ 7 %), maximum 22 → 23 196 o, marge de
   356 octets sous l'assertion de 23 552 — un contact ordinaire un peu plus long fait-il refuser l'appel au maximum, et
   est-ce dit dans la description et au manuel ? `plafond_au_maximum` éprouve-t-il les deux outils neufs ?
5. **Les mutations** (`329b-mutations.log`, `329b-mutation-*.out`) : chacune tuée par le **test** visé et par la
   raison visée ? Une mutation plausible **non jouée** survivrait-elle — `PiecePublique` qui gagne `reference_value`
   rendu en `Code`, `qr_lu` toujours `true`, `kesh_contacts` qui ne transmet pas `includeArchived`, le `statut` de
   `kesh_pieces` ignoré au profit de `to_complete`, `lister` sans l'exigence (seul `lister_tout` l'appelle) ? Raisonne
   sur le code, n'exécute rien.

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC-b1 à AC-b7, rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve. En
particulier :

- **AC-b1** : l'ordre (exigence, société, validation, lecture), les bornes 35 / 65, `PiecePublique`, les tests R7
  (refus avant lecture et avant validation, contrôle positif, lecture seule, tranche, société lue d'abord).
- **AC-b2** : les trois outils, leurs arguments et bornes contre la fiche ; dix-neuf outils dans l'ordre ;
  `tools_list_filtre` amendé (`contacts:read` → `[kesh_contacts, kesh_contact, kesh_lire_envoi]`).
- **AC-b3** : `INVENTAIRE` (lignes 32-9b) contre l'inventaire de la fiche ; `CHAMPS` (numéros 18 à 39) contre le
  tableau des largeurs — et l'écart déclaré **C-32-9-60** (`contacts.address` non posable : composée par le dépôt) ;
  `pieces_sans_donnees_du_qr`.
- **AC-b4, AC-b6** : `TEMOINS`, les propriétés par outil ; aucune ressource, aucun fichier ouvert.
- **AC-b5** : la matrice et le paragraphe « Les contacts et les pièces » du manuel d'administration, phrase par phrase
  contre le code — **et contre le PDF aplati** (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '
  | grep -oE ".{60}Les contacts et les pièces.{600}"`) ; l'entrée du `CHANGELOG.md`. Un texte publié promet-il encore
  que `documents:read` ne sert aucun outil (`grep -n "documents:read" README.md website/*.html docs/api-external.md
  docs/manual/fr/*.tex CHANGELOG.md`) ?
- **Les choix C-32-9-60 à C-32-9-63** : chacun est-il justifié, et l'un d'eux contredit-il un AC, une décision de Guy ou
  la fiche sans le dire ? Les écarts déclarés au Dev Agent Record (dont un `cargo fmt` qui a réécrit trois fichiers de
  test, hors de l'outil d'édition).

## Rapport

Écris ton rapport dans **un seul** fichier : `/home/gcorbaz/devel/kesh-gate-logs/329b-review-p1-B.md` (lentille B),
`/home/gcorbaz/devel/kesh-gate-logs/329b-review-p1-E.md` (lentille E) ou
`/home/gcorbaz/devel/kesh-gate-logs/329b-review-p1-A.md` (lentille A), selon la lentille qui t'est confiée — par
l'outil d'écriture de fichier, **jamais** par une redirection. Contenu :

1. **Les findings**, chacun : identifiant (`B1`, `E1`, `A1`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier et
   ligne, la preuve (sortie de commande copiée), la correction proposée, et s'il est **né du développement** de la 32-9b
   ou **antérieur**.
2. **L'énumération de tout ce qui sort par `/mcp`** (axe 1) pour les outils neufs — même si elle ne rend aucun finding.
3. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
4. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
5. Les fichiers créés par erreur, s'il y en a.
