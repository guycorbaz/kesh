# Prompt — revue de code P1 de la Story 32-9a (les outils de lecture des livres et des factures), trois lentilles, Opus

Tu relis le développement de la **partie a de la Story 32-9** de Kesh (comptabilité suisse, Rust/Axum, MariaDB) :
*le socle des sorties des outils MCP (`mcp/sortie.rs` : l'enveloppe `{"nonFiable": …}`, le trait scellé `ChampSur`,
les macros `sortie!` et `champ_enum!`, `Code`, `Tiers`, `Resume`, la fabrique unique `donnees`) ; `ContexteMcp` qui
porte la `Trace` et `afficher` ; quinze outils de lecture (`books:read` : exercices, plan comptable, écritures, détail
d'une écriture, grand livre, balance, bilan, compte de résultat, postes ouverts ; `invoices:read` : factures, facture,
échéancier, balance âgée, factures fournisseur, facture fournisseur) ; le service `exercices::lister` et son
gestionnaire ; `factures_fournisseur::detail_sans_fichier` ; trois erreurs de demande à texte fixe ; les fixtures
piégées `kesh_seed::pieges` (fonctionnalité de test) ; la matrice portée × outil comparée au manuel ; les gardes
lexicales neuves.* C'est **la story qui ouvre les livres réels à un agent** : un défaut ici fait sortir un texte non
enveloppé (injection), une coordonnée de tiers non masquée, une donnée d'une autre société, ou une écriture.

Worktree `/home/gcorbaz/devel/kesh-32-9`, branche `story/32-9a`. Le diff à relire, **aplati** (un seul diff, pas la
suite des commits) : `git -C /home/gcorbaz/devel/kesh-32-9 diff origin/main...HEAD`. Langue du rapport : français.
Modèle : **Opus**. Contexte frais : ne t'appuie sur aucune passe antérieure.

⚠️ **Seule la partie a est en revue.** Les outils des contacts et des pièces (`kesh_contacts`, `kesh_contact`,
`kesh_pieces`, portées `contacts:read` et `documents:read`) sont la **32-9b** : leur absence n'est pas un finding, et
une clé `contacts:read` ou `documents:read` seule ne liste que `kesh_lire_envoi`. Les prompts standard (forme de
l'enveloppe nommée) sont la 32-10a ; la section *Le serveur MCP* du manuel et la configuration de Claude Code sont la
32-7 ; le parcours avec un agent réel est la 32-11.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/32-9-outils-de-lecture.md` — *Ce que le code dit* (faits 1 à 12), *La
conception, en une page*, *L'inventaire clos des champs libres rendus* (partie 32-9a), *Les jeux d'arguments*, *Les
pièges et la largeur des colonnes*, *Les tailles mesurées*, *Partie A* (AC-a1 à AC-a24), *Tests et mutations prévus*
(Ma1 à Ma30), *Ce que les gardes ne voient pas*, *Dev Notes*, et le **Dev Agent Record** de la partie a (étape 0,
mutations, gates, tailles) — puis le registre `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-329.md`
(C-32-9-1 à 32, et les choix du développement **C-32-9-40 à C-32-9-48**). Le Dev Agent Record déclare des gates, des
mutations et un E2E : **ne les prends pas pour argent comptant**, recoupe-les avec les journaux
`/home/gcorbaz/devel/kesh-gate-logs/329a-*` (lecture seule). Aucun contrôle visible ne change : pas de capture
attendue.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp`, ni ailleurs ; **aucune redirection** (`>`, `>>`,
  `tee`) : compare par des commandes enchaînées (pipe), lis les PDF vers la sortie standard seulement. Un fichier créé
  par erreur se **déclare** au rapport ; il ne se supprime pas (supprimer, c'est écrire).
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

1. **Tout ce qui sort par `/mcp`, énuméré.** Pour chacun des seize outils du registre, dresse depuis le code la liste
   **close** des champs de sa structure `sortie!` (et de ses sous-structures), avec leur type ; pour chaque champ de
   type texte de la base (`String` d'une entité de `kesh-db` ou d'un rapport de `kesh-report`), dis s'il sort
   enveloppé (`TexteNonFiable`), masquable (`Tiers`), retypé (`Code`), ou s'il n'est pas rendu — et confronte cette
   liste à l'inventaire de la fiche et à la constante `INVENTAIRE` de `tests/mcp_enveloppe_e2e.rs`. Énumère aussi ce
   qui sort **hors** des structures : le résumé (`Resume`, déposé à l'écran sous `required`), les textes d'erreur, les
   descriptions et schémas de `tools/list`, les `instructions` d'`initialize`. Un champ de chaîne qui sortirait brut est
   un CRITICAL.
2. **Chaque motif greppé sur le dépôt réel.** Les gardes de `crates/kesh-api/tests/mcp_gardes_lexicales.rs` — neuves
   (`champ_sur_en_un_lieu`, `trace_confinee_dans_outil`, `sorties_sans_serialisation_parallele`, `mcp_sans_fichier`) et
   amendées (`outils_et_politique_en_leur_lieu`, `contexte_mcp_construit_par_le_repartiteur_seul`,
   `reponses_par_rendre_seul`, `prompts_scelles`) — et les listes de `services_motif.rs` (`HANDLERS_MIGRES`,
   `SITES_NON_RESOLUS`, le compte de `services/`). Pour **chaque** garde : greppe toi-même son motif sur son périmètre
   (`grep -rnE '<motif>' crates/kesh-api/src`, lignes de commentaire écartées à la main), confronte la sortie à la
   liste écrite, cherche une forme **plausible** que le motif ne voit pas (alias `use … as`, chemin qualifié, coupure
   de ligne, macro), et dis si elle est écrite comme angle mort.
3. **Chaque dépendance lue à la source.** Toute affirmation du code ou du Dev Agent Record sur `serde` (ordre des clés
   sans `preserve_order`, `serialize_unit_variant` en JSON, `rename_all` sur un `r#type`), `serde_json`
   (`to_string_pretty`, échappement), `rust_decimal` avec `serde-str` (forme d'un montant), `chrono` (forme d'une date),
   le résolveur 3 de Cargo (une fonctionnalité de dépendance de développement n'entre pas dans `cargo build --release
   -p kesh-api`), `sqlx` et MariaDB (`CHECKSUM TABLE` et les colonnes générées, `CHARACTER_MAXIMUM_LENGTH`), se
   vérifie dans `~/.cargo/registry/src/*/<crate>-<version>/` (ligne citée) ou dans la documentation que tu peux lire ;
   sinon, dis-le.

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention.

1. **`crates/kesh-api/src/mcp/sortie.rs`** — le sceau tient-il ? Une implémentation de `ChampSur` peut-elle s'écrire
   hors de `sortie.rs` sans que rien ne rougisse ? `sortie!` admet-il un champ `String` par un détour (type alias,
   `Box<str>`, `Cow`, `Option<String>`, tuple, un type générique) ? `champ_enum!` admet-il une énumération dont le
   `Serialize` est écrit à la main et rend une chaîne de la base ? `TexteNonFiable` peut-il être lu, cloné, affiché ou
   converti ? `Code::depuis_str` et `Code::parmi` : une chaîne inconnue peut-elle sortir brute ? `Tiers` masque-t-il
   **aussi** l'absence ? `texte_et_valeur` : le plafond porte-t-il sur ce qui part réellement (sous `exempt`, et sous
   `required` après le second `rendre_visible` du dépôt) ? Le résumé peut-il porter un texte lu ?
2. **`crates/kesh-api/src/mcp/outils/*.rs`** — chaque projection est-elle exhaustive (aucun `..`) ; un champ texte
   d'une entité est-il rendu par une autre voie qu'une structure `sortie!` ; un outil lit-il hors de son service ; les
   arguments (`deny_unknown_fields`, `limit`, `offset`, énumérations) refusent-ils ce qu'ils doivent ? Les schémas
   promettent-ils exactement ce que les structures d'arguments acceptent (noms, énumérations, `required`,
   `maximum`, `default`) ?
3. **`crates/kesh-api/src/mcp/{outil,repartiteur,textes,registre}.rs`** — la `Trace` peut-elle sortir de `ContexteMcp` ?
   `afficher` écrit-il la trace une fois par rapport ? Les trois variantes d'erreur et leurs issues de journal ; la
   liste blanche de `depuis_app_error` — un bras recopie-t-il un champ ? L'ordre de `tools/list` et le registre.
4. **`crates/kesh-api/src/services/{exercices,factures_fournisseur,postes_ouverts,ecritures,factures,mod}.rs`** et
   `routes/fiscal_years.rs` — `exercices::lister` exige-t-il avant toute lecture, sous la société du contexte ? Le
   gestionnaire migré a-t-il le même comportement (tri, borne, réponse) ? `detail_sans_fichier` ne transmet-il aucun
   chemin ? Les `pub use` exposent-ils un type qui porte un chemin de fichier (`storage_path`, `pdf_storage_path`) ou
   un secret ?
5. **`crates/kesh-seed/src/pieges.rs`**, `Cargo.toml` de `kesh-seed` et de `kesh-api` — la fonctionnalité `pieges`
   peut-elle entrer dans le binaire de production ? `semer` passe-t-il par les dépôts de production seulement (aucun
   SQL à la main) ? Les textes tiennent-ils dans leurs colonnes ?
6. **Les tests** (`mcp_outils_e2e.rs`, `mcp_enveloppe_e2e.rs`, `mcp_matrice_manuel.rs`, `tests/common/mcp_pieges.rs`,
   les amendements de `mcp_transport_e2e.rs`, `mcp_envois_e2e.rs`, `mcp_traduction.rs`, `services_rollout.rs`,
   `services_motif.rs`) — un test peut-il passer **à vide** : un attendu tiré de la même source que l'observé (le
   registre comparé à lui-même), un parcours qui ne rencontre aucun piège, une égalité vraie parce que les deux côtés
   sont des erreurs, une somme de contrôle qui ne voit pas une écriture, un témoin positif absent ?

## Lentille E — Edge cases

1. **L'enveloppe (AC-a1, AC-a15)** : un libellé qui contient `"}`, `\u0000`, U+2028, une séquence `\u{202E}` déjà
   écrite en clair, une chaîne de 1 000 caractères à sa largeur, un emoji à sélecteur ; une clé d'objet où un piège
   apparaîtrait ; un piège dans un numéro de compte de huit octets ; le second `rendre_visible` du dépôt (32-8b)
   change-t-il un texte déjà rendu visible (idempotence) ?
2. **Les tiers (AC-a6, AC-a16)** : sous `invoices:read` seule, l'IBAN, le QR-IBAN ou l'adresse d'envoi peuvent-ils
   sortir par un autre champ (libellé de l'écriture de règlement d'un lot de paiement, référence, description,
   `kesh_ecriture` de l'achat, échéancier) ? Les noms non masqués sont-ils bien ceux que la fiche et le manuel
   annoncent ?
3. **La société (AC-a20)** : chaque outil qui prend un identifiant (détail **et** liste : `accountId`, `contactId`,
   `fiscalYearId`, `id`) — un identifiant d'une autre société rend-il le même corps qu'un inexistant ? Un rapport sur
   l'exercice d'une autre société (`ReportFiscalYearNotFound`) recopie-t-il une date ? Le grand livre filtre-t-il son
   `accountId` par société ?
4. **Les tailles (AC-a21, C-32-9-44)** : les maximums tiennent-ils sous le plafond avec des textes plus longs que la
   fixture (le refus est-il dit et éprouvé) ? Le bilan d'une société de plus de 60 comptes à solde non nul ; la
   balance d'un plan de 300 comptes ; un `offset` au-delà du total ; `limit` à `i64::MAX` ; un `offset` de
   l'échéancier au-delà de 1 000 000.
5. **Les erreurs (AC-a22)** et le délai : un outil à plusieurs instructions (`ecritures::detail`, `echeances` en
   parallèle, les rapports) sous `max_statement_time` par instruction et le délai de 30 s de Kesh (#759) ; un pool de
   trois connexions et `tokio::join!` de l'échéancier.
6. **Les mutations** (`329a-mutations.log`, `329a-mutations.out`) : chacune tuée par le **test** visé et par la raison
   visée ? Une mutation plausible **non jouée** survivrait-elle — un `Tiers::selon` qui teste `InvoicesRead`, un outil
   qui rend `Option<TexteNonFiable>` toujours `None`, un `Code::parmi` insensible à la casse, une tranche de la balance
   décalée d'un, un `afficher` appelé avant l'exigence ? Raisonne sur le code, n'exécute rien.

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC-a1 à AC-a24, rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve. En
particulier :

- **AC-a9** : l'ordre des seize outils au registre ; `tools_list_filtre` (32-6) et `e1_rien_ne_part_sous_cle_requise`
  amendés ; `HORS_REGISTRE`.
- **AC-a10, AC-a11** : le tableau des outils (portée, service, arguments, pagination) contre le code ; les champs
  rendus de chaque structure arrêtés au Dev Agent Record.
- **AC-a13, AC-a14** : `INVENTAIRE` contre l'inventaire de la fiche (les lignes `NF` et `Tiers`), chaque exemption
  justifiée ; `CHAMPS` contre le tableau des largeurs (lignes de la 32-9a).
- **AC-a23** : la sous-section `sec:mcp-outils` du manuel d'administration, phrase par phrase contre le code — la
  matrice, l'enveloppe, le masquage des seules coordonnées, les noms non masqués et la liste des libellés que Kesh
  compose (`supplier_invoices.rs:512`, `:941`, `credit_notes.rs:590`, `routes/reconciliation.rs:2734`,
  `invoice_format.rs:259`), l'IBAN non porté par le contact (`contacts.rs:380-381`), l'absence d'écriture et la trace
  des rapports, ce qui n'est pas servi, les tailles (24 Kio, ≈ 10 000 jetons **estimés**) — **et contre le PDF
  aplati** (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oE ".{80}nonFiable.{120}"`).
  L'entrée `### Ajouté` du `CHANGELOG.md`. Aucun texte publié (README, `website/`, `docs/api-external.md`, manuel
  utilisateur, brochure) ne promet-il un outil qui n'existe pas, ou ne dit-il le contraire (`grep -n "MCP" README.md
  website/*.html docs/api-external.md docs/manual/fr/*.tex`) ?
- **Les choix C-32-9-40 à C-32-9-48** : chacun est-il justifié, et l'un d'eux contredit-il un AC, une décision de Guy
  (Q1 tranchée le 2026-10-25, Q11, Q12) ou la fiche sans le dire ? En particulier C-32-9-44 (bornes de la balance âgée
  révisées) et C-32-9-45 (la mesure de `aucun_outil_n_ecrit`).

## Rapport

Écris ton rapport dans **un seul** fichier : `/home/gcorbaz/devel/kesh-gate-logs/329a-review-p1-B.md` (lentille B),
`/home/gcorbaz/devel/kesh-gate-logs/329a-review-p1-E.md` (lentille E) ou
`/home/gcorbaz/devel/kesh-gate-logs/329a-review-p1-A.md` (lentille A), selon la lentille qui t'est confiée — par l'outil
d'écriture de fichier, **jamais** par une redirection. Contenu :

1. **Les findings**, chacun : identifiant (`B1`, `E1`, `A1`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier et
   ligne, la preuve (sortie de commande copiée), la correction proposée, et s'il est **né du développement** de la 32-9a
   ou **antérieur**.
2. **L'énumération de tout ce qui sort par `/mcp`** (axe 1), outil par outil — même si elle ne rend aucun finding.
3. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
4. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
5. Les fichiers créés par erreur, s'il y en a.
