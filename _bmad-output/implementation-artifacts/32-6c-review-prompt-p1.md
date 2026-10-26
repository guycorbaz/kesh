# Prompt — revue de code P1 de la Story 32-6c (le journal du filtre d'adresse du serveur MCP), trois lentilles, Sonnet

Tu relis le développement de la **Story 32-6c** de Kesh (comptabilité suisse, Rust/Axum, MariaDB) : *chaque décision
`Admis` ou `Refuse` du filtre d'adresse de `/mcp` écrit **un** évènement au journal du serveur (`tracing`, cible
`kesh_api::mcp::reseau`), refus en `warn`, admission en `debug`, avec onze champs dans un ordre fixe — l'id de société,
l'état du filtre, le nombre de plages, le pair, son statut de mandataire, la **forme** des en-têtes de mandataire
(comptes et présences, **jamais** leurs valeurs), le client retenu (calculé même filtre coupé) et le verdict — ; et le
manuel d'administration dit comment lire ce journal.* C'est une **story de diagnostic** : aucune réponse envoyée au
client ne doit changer, aucune migration, aucun contrôle visible. Elle servira à Guy, à la mise à jour de son NAS
Synology derrière le proxy inversé de DSM, à savoir si le filtre filtre vraiment.

Worktree `/home/gcorbaz/devel/kesh-32-6c`, branche `story/32-6c`, partie de `c5287dec` (`main`). Le diff à relire,
**aplati** (un seul diff, pas la suite des commits) :
`git -C /home/gcorbaz/devel/kesh-32-6c diff origin/main...HEAD`. Langue du rapport : français. Modèle : **Sonnet**.
**Passe complète**, en contexte frais : ne t'appuie sur aucune passe antérieure.

⚠️ **Seule la 32-6c est en revue.** Le point d'entrée et ses gardes (32-6a), le journal d'audit des appels (32-6b), la
table des envois (32-8a) sont fusionnés et revus ; un défaut que la 32-6c ne touche pas est un finding **antérieur**, à
classer comme tel. La section *Le serveur MCP* du manuel est la 32-7 ; la description détaillée du fonctionnement de
MCP, la 32-11. La 32-8b se développe en parallèle dans un autre worktree : **n'y lis rien**, elle n'est pas en revue.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/32-6c-journal-filtre-adresse.md` — *Ce que le code dit* (faits 1 à 13),
*Critères d'acceptation* (AC-1 à AC-11, AC-T1 à AC-T6), *Tests et mutations prévus* (M1 à M21), *Ce que les gardes ne
voient pas*, *Points soumis à Guy* (ses réponses), *À recontrôler à l'étape 0*, et le **Dev Agent Record** (étape 0,
mutations, gates, décomptes) — puis le registre `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-326c.md`
(C-32-6c-1 à C-32-6c-13, et les choix du développement s'il y en a). Le Dev Agent Record déclare des gates, des
mutations, un E2E et une sortie réelle du fichier de log : **ne les prends pas pour argent comptant**, recoupe-les avec
les journaux `/home/gcorbaz/devel/kesh-gate-logs/326c-*` (lecture seule). Aucun contrôle visible ne change : pas de
capture attendue.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp`, ni ailleurs ; **aucune redirection** (`>`, `>>`,
  `tee`) : compare par des commandes enchaînées (pipe), lis les PDF vers la sortie standard seulement. Un fichier créé
  par erreur se **déclare** au rapport ; il ne se supprime pas (supprimer, c'est écrire).
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`, `fetch`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python`/`python3`, ni `perl`, ni `ruby`, ni
  `node`/`npm`/`npx`, ni `bash -c`/`sh -c` d'un script, ni `jq` ; ni **`scripts/prepare-release.sh`**, ni
  `scripts/regen-test-schema.sh`, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`,
  ni aucun script de `kesh-gate-logs` ou d'un scratchpad (dont les scripts de mutations et d'E2E de la 32-6c) ; ni
  `make`, ni `latexmk`, ni `cargo` (aucune sous-commande, `cargo fmt` compris), ni `rustc`, ni `docker`,
  ni `mysql`/`mariadb`, ni `sqlx`, ni `curl`, ni `sed -i`.
- **Aucune commande `gh`**, pas même en lecture.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée dans le
  rapport.
- **Données réelles** : aucune ; les fixtures sont synthétiques (adresses de documentation et de bouclage). Ne lis rien
  sous `~/kesh-test/`.

## Les trois axes obligatoires, pour chaque lentille

1. **Aucune valeur d'en-tête au journal.** L'évènement ne porte que des valeurs dérivées (AC-5, C-32-6c-6). Lis
   `journaliser_decision` dans `crates/kesh-api/src/mcp/reseau.rs` : un champ peut-il porter un texte venu du client
   (une conversion `to_str`, un `?en_tetes`, un `Debug` d'un type qui contient une valeur reçue, l'`Origin`) ? Le test
   `journal_du_filtre_sans_secret` (AC-T4) cherche-t-il **sur tous les évènements** de la capture, et ses pièges
   couvrent-ils chaque en-tête nommé par l'AC-5 ? Copie les sorties `grep -nF` (`grep -nF "to_str" …`,
   `grep -nF "PIEGE_" crates/kesh-api/tests/mcp_reseau_e2e.rs`).
2. **Réponses inchangées.** `decider` a été réordonné (le client calculé avant l'état du filtre, AC-3) : sa **décision**
   est-elle exactement celle de `c5287dec` pour tout réglage et toute requête (`git show c5287dec:crates/kesh-api/src/mcp/reseau.rs`
   comparé au diff) ? Le `debug!` d'entrée ignorée de `allowed_networks` part-il dans les mêmes cas qu'avant ? Les tests
   (a) à (s) de `mcp_reseau_e2e.rs` et ceux de `mcp_transport_e2e.rs` sont-ils **inchangés** dans le diff
   (`git diff origin/main...HEAD --stat`, puis le diff de ces fichiers) ?
3. **Le manuel contre le code, et le PDF aplati.** La sous-section `sec:mcp-journal-filtre` de
   `docs/manual/fr/admin-manual.tex` (dans § *Monitoring et logs*) et la phrase ajoutée à l'étape 4 de DSM
   (§ `sec:synology`) : chaque phrase est-elle vraie **du code livré** (niveaux, champs, ordre, noms des cas, messages
   cités des chemins sans décision — `transport.rs:87`, `services/serveur_mcp.rs:41`, `reseau.rs`), de la configuration
   (`docker-compose.yml`, `docker-compose.prod.yml`, `.env.example`, `crates/kesh-api/src/logging.rs`) et des
   dépendances (`tracing-appender-0.2.5/src/rolling.rs` pour le nom du fichier, `tracing-subscriber-0.3.23` pour le
   filtre de cible et les couleurs) ? La **ligne d'exemple** est-elle bien celle d'une sortie réelle du **fichier** de
   log (recoupe avec le journal cité au Dev Agent Record), avec des adresses de documentation ou de bouclage ? Le
   manuel conseille-t-il quoi que ce soit à DSM au-delà du renvoi (il ne doit **pas** prescrire `X-Forwarded-For` à
   DSM : décision de Guy) ? Puis le **PDF** :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oE ".{60}Diagnostiquer le filtre d'adresse.{200}"`,
   et de même pour `kesh_api::mcp::reseau=debug`, `kesh.*.log`, `peut forger`, `n'emploie pas X-Real-IP` — le PDF
   est-il celui du `.tex` livré ? Aucune autre phrase du manuel ne devient-elle fausse
   (`grep -nE 'X-Real-IP|RUST\\?_LOG|/mcp' docs/manual/fr/admin-manual.tex`) ? Le manuel utilisateur et la brochure
   sont-ils inchangés ?

Chaque dépendance invoquée se lit **à la source**, dans `~/.cargo/registry/src/*/<crate>-<version>/` (version lue dans
`Cargo.lock`, ligne citée).

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention.

1. **`crates/kesh-api/src/mcp/reseau.rs`** — `decider`, `ClientJournal`, `journaliser_decision`, la macro locale. Un
   chemin de `decider` rend-il `Admis` ou `Refuse` **sans** évènement, ou en émet-il deux ? `Ecartee` peut-il en émettre
   un ? Le pair écrit est-il normalisé ? `pair_mandataire` est-il calculé contre les mandataires et non contre les
   plages ? Les comptes `xff_lignes` et `xff_entrees` sont-ils faits sur les octets (une ligne non ASCII, une ligne
   vide, une virgule finale) ? Le coût par requête (une mise en forme par société décidée, même quand le niveau est
   coupé) est-il raisonnable ?
2. **Les doc-comments** (doc du module de `reseau.rs`, de `decider`, de `journaliser_decision`, ligne 2 de l'ordre des
   contrôles de `mcp/mod.rs`) : disent-ils vrai du code ?
3. **Les tests** (`decision_journalisee_par_cas`, `ecartee_sans_evenement_de_decision`, `serveur_desactive_aucun_evenement`,
   `journal_du_filtre_sans_secret`, `deux_societes_deux_evenements` dans `tests/mcp_reseau_e2e.rs` ;
   `filtre_cible_n_ouvre_que_le_filtre_mcp` dans `src/logging.rs`) — un test peut-il passer **à vide** : une liste
   d'évènements attendue vide qui serait vide parce que la capture ne voit rien (le témoin est-il dans la **même**
   capture ?), une égalité qui compare deux chaînes construites de la même façon, un cas dont la réponse n'est pas
   vérifiée, un filtrage par fragment qui laisserait passer un second évènement de décision ?

## Lentille E — Edge cases

1. **Chaque cas de l'AC-T1** (quatorze) : la ligne attendue est-elle celle que le code produit **pour la bonne
   raison** ? Un cas réel de production n'a-t-il pas d'équivalent (IPv6 derrière un mandataire IPv4, un mandataire
   déclaré en IPv4 mappée, plusieurs lignes `X-Real-IP`, un `Forwarded` vide) — et le journal y dit-il vrai ?
2. **Le filtre coupé** (AC-3) : le client calculé ne change-t-il jamais la décision (`client=aucun` filtre coupé reste
   admis) ? Le `debug!` d'entrée ignorée de `allowed_networks` reste-t-il absent filtre coupé ?
3. **Les chemins sans décision** (AC-7, AC-9 (viii)) : l'énumération du manuel est-elle complète au code de la branche
   (les trois `RouteAbsente` de `transport.rs`, la route exacte de `lib.rs`) ? Un chemin écrit-il un évènement de
   décision qu'il ne devrait pas ?
4. **La directive de journal** (AC-8, AC-T6) : `RUST_LOG=info,kesh_api::mcp::reseau=debug` passe-t-il par
   `build_log_filter` sans perdre la baseline `sqlx=warn` ? Une cible voisine (`kesh_api::mcp::reseau_x`, si elle
   existait) serait-elle ouverte aussi (préfixe, `directive.rs`) — et cela importe-t-il ?
5. **Les mutations** (`326c-mut-*.log` ou le journal de campagne) : chacune tuée par une **assertion nommée** et pour
   la raison visée (lis le journal, pas seulement le verdict), et le diff résiduel nul ? Une mutation plausible **non
   jouée** survivrait-elle — `pair` écrit avant normalisation, `x_real_ip` lu sur `forwarded`, l'évènement émis avant la
   boucle des plages avec un verdict provisoire, `client` écrit `Debug` (`Some(…)`) ? Raisonne sur le code, n'exécute
   rien.

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC-1 à AC-11 et AC-T1 à AC-T6, rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la
preuve. En particulier :

- **AC-1** : exactement onze champs, dans l'ordre, et le message fixe ; un seul jeu de champs pour les deux niveaux.
- **AC-6** : réponses octet pour octet (axe 2).
- **AC-9 à AC-11** : axe 3, point par point contre l'énumération de l'AC-9 (les tirets et les cas (i) à (ix)), avec les
  corrections de l'étape 0 (LOW R1, R2, R3, F2, F3, F4, F5 de la validation P3) — chacune est-elle appliquée, à la fiche
  **et** au manuel, avec la ligne lue citée au Change Log ?
- **CHANGELOG inchangé** (C-32-6c-10), `sprint-status.yaml` : `32-6c-journal-filtre-adresse: review`.
- **Le registre** : C-32-6c-11 annoté (non réécrit) ; les choix du développement, s'il y en a, justifiés ; aucun ne
  contredit une décision de Guy (refus en `warn` ; formes et présences seulement ; journal même filtre coupé ; aucun
  conseil `X-Forwarded-For` à DSM).
- **Les décomptes du Dev Agent Record** (tests ajoutés, mutations jouées, chiffres des gates, échecs E2E) : recompte-les
  depuis les journaux et le diff, périmètre déclaré.
- **Les commits** : `refs #723`, **aucun mot-clé de fermeture**
  (`git log origin/main..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` doit être vide).

## Rapport

Écris ton rapport dans **un seul** fichier : `/home/gcorbaz/devel/kesh-gate-logs/326c-review-p1-B.md` (lentille B),
`/home/gcorbaz/devel/kesh-gate-logs/326c-review-p1-E.md` (lentille E) ou
`/home/gcorbaz/devel/kesh-gate-logs/326c-review-p1-A.md` (lentille A), selon la lentille qui t'est confiée — par l'outil
d'écriture de fichier, **jamais** par une redirection. Contenu :

1. **Les findings**, chacun : identifiant (`B1`, `E1`, `A1`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier et
   ligne, la preuve (sortie de commande copiée, `grep -nF` pour toute présence ou absence), la correction proposée, et
   s'il est **né du développement** de la 32-6c ou **antérieur**.
2. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
   Les trois axes obligatoires y figurent nommément, le manuel **avec le PDF aplati**.
3. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
4. Les fichiers créés par erreur, s'il y en a.
