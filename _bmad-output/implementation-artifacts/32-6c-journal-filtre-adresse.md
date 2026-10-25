# Story 32.6c : le journal du filtre d'adresse du serveur MCP

Status: ready-for-dev

*Fiche rédigée le 2026-10-25, en autonomie (Opus 5.5), worktree `kesh-32-6c`, branche `story/32-6c` partie de
`origin/main` `c5287dec` (32-6a, 32-6b, 32-8b0 et 32-8a fusionnées). Registre : `epic-32-choix-autonomes-326c.md`
(C-32-6c-1 à C-32-6c-13 ; 11 à 13 ajoutés à la validation P1, 7 et 11 annotés à la validation P2). Clé courte : `326c`. Issue de l'epic : #723 (CR « Serveur MCP intégré à Kesh »), en `refs`
seulement. Les numéros de ligne cités sont ceux de `c5287dec`. Validations P1 et P2 remédiées le 2026-10-25 (Change Log) ;
réponses de Guy du même jour aux cinq points soumis : § *Points soumis à Guy*.*

⚠️ **Story de diagnostic, sans changement de comportement.** Elle ajoute au journal du serveur ce qu'il faut pour
vérifier, à la mise à jour du NAS, ce que le filtre d'adresse de `/mcp` voit derrière le proxy inversé de DSM. Aucune
réponse envoyée au client ne change ; aucune migration ; aucun contrôle visible.

## Story

En tant que Guy, qui mettra à jour Kesh sur son NAS Synology derrière le proxy inversé de DSM, je veux lire dans le
journal du serveur, pour chaque requête vers `/mcp`, l'adresse du pair TCP, la présence et la forme des en-têtes de
mandataire, le client que le filtre retient et son verdict — sans aucune clé, aucun en-tête d'authentification, aucun
corps ni aucune donnée comptable — afin de savoir si le filtre d'adresse filtre vraiment, et de régler les mandataires
de confiance en connaissance de cause.

## Origine

- **Guy, le 2026-10-25** : derrière le proxy inversé de DSM, le filtre d'adresse de `/mcp` ne filtre que si DSM pose
  `X-Forwarded-For` ; rien ne l'établit. « On vérifiera lors de la mise à jour : ajoute le nécessaire dans les logs au
  cas où. »
- **Fiche 32-6** (`32-6-endpoint-mcp.md`) : § *Ce que les gardes ne voient pas*, « Le filtre d'adresse voit le pair que
  voit Kesh, non le client réel » — **non vérifié sur le NAS** ; point 2 pour Guy : « quelle adresse voit-il pour un
  client (le journal du serveur la donnera après la 32-6a) ? ». **Le journal ne la donne pas** : fait 2 ci-dessous.
- **C-32-6-4** (`epic-32-choix-autonomes-326.md:57-64`) : « Le refus est au journal du serveur (`tracing`), jamais en
  base. » Et l'AC-b4 de la 32-6b (`32-6-endpoint-mcp.md:786-790`) : les refus du transport vont « au **journal du
  serveur** seulement ». **Pour le refus du filtre, ce journal n'existe pas** (fait 2) : la 32-6c tient la promesse
  écrite par C-32-6-4.
- **Décision de Guy du 2026-10-25** (relayée par l'orchestrateur) : la garde « 2 bis » (refuser un pair non déclaré
  qui envoie des en-têtes de mandataire) est écartée, la sécurité relevant de l'infrastructure
  (`32-6-endpoint-mcp.md:1359-1361`). La 32-6c **n'y revient pas** : elle observe, elle ne décide rien de plus.

## Ce que le code dit (relevé au code le 2026-10-25, `c5287dec`)

1. **La décision du filtre est une fonction pure, par société** : `mcp::reseau::decider(s, pair, en_tetes) -> Decision`
   (`crates/kesh-api/src/mcp/reseau.rs:104-140`), appelée **une fois par société activée** et par requête, sans court-
   circuit, par le transport (`transport.rs:91-94` : `.filter(|s| reseau::decider(s, pair, requete.headers()) ==
   Decision::Admis)`). Trois issues : `Admis`, `Refuse`, `Ecartee` (`reseau.rs:91-99`).
2. **Aujourd'hui, ni l'admission ni le refus ne laissent de trace.** `decider` n'émet que deux évènements, qui ne
   portent ni le pair ni le client : `tracing::error!` pour une entrée de `trusted_proxies` qui ne s'analyse pas
   (`reseau.rs:110-114`, issue `Ecartee`) et `tracing::debug!` pour une entrée ignorée de `allowed_networks`
   (`reseau.rs:129-132`). Le transport, quand aucune société ne sert le client, rend la route absente sans rien écrire
   (`transport.rs:95-97`). Le `warn!` de `reseau.rs:161-164` parle de `KESH_PUBLIC_BASE_URL` (au démarrage), non du
   filtre. `grep -n "tracing::" crates/kesh-api/src/mcp/reseau.rs crates/kesh-api/src/mcp/transport.rs` → 6 lignes :
   `reseau.rs:110`, `:129`, `:161`, `transport.rs:87`, `:121`, `:138` — aucune ne nomme une adresse.
   **Les autres chemins d'une requête `/mcp` sans décision** (validation P1, R3) — chacun rend la route absente sans
   évènement de décision : (a) une **erreur de la base** à la lecture des réglages (`list_enabled` rend `Err`,
   `kesh-db/src/repositories/mcp_server_settings.rs:96-100`, `:107`) : `transport.rs:87`, `error!` « serveur MCP :
   réglages illisibles, /mcp n'est pas servi », aucune société servie — **non** des lignes illisibles, qui relèvent
   de (b) *(corrigé à la validation P2, R1)* ; (b) **une** ligne illisible (une liste qui n'est pas
   un tableau de chaînes) : `services/serveur_mcp.rs:41`, `error!` « serveur MCP : ligne de réglages illisible, la
   société n'est pas servie », avec le `company_id` (exercé par `tests/mcp_transport_e2e.rs:355-370`) — la société est
   absente de `reglages`, donc aucun `decider` ne tourne pour elle ; (c) `ConnectInfo` absent : `transport.rs:78-80`,
   route absente sans rien écrire — **inatteignable en production** (`main.rs:399` :
   `into_make_service_with_connect_info::<SocketAddr>()`). `grep -n "tracing::" crates/kesh-api/src/services/serveur_mcp.rs`
   → `:41` seul. **En amont du transport** (validation P2, F3) : `/mcp` est monté par `route_service("/mcp", …)`
   (`lib.rs:1278`), sur ce chemin exact ; les chemins voisins (`/mcp/`, `/MCP`, `//mcp`, `/mcp/x`) partent au repli
   statique sans passer par le transport (`tests/mcp_transport_e2e.rs:405-418`, `chemins_voisins`) — aucune ligne,
   aucune erreur. Ces chemins sont hors de la story (elle n'ajoute rien à `transport.rs` ni à `serveur_mcp.rs`) ;
   le manuel les nomme (AC-7, AC-9 (viii)).
3. **Le filtre désactivé ne calcule pas le client** : `if !s.address_filter_enabled { return Decision::Admis; }`
   (`reseau.rs:119-121`) précède `client(...)` (`:122`). Le mandataire est lu **avant** l'état du filtre (`:105-118`,
   P7-H3 de la 32-6a).
4. **Le client** : `reseau::client(pair, en_tetes, mandataires) -> Option<IpAddr>` (`reseau.rs:63-87`). Pair hors des
   mandataires → `Some(pair)`, `X-Forwarded-For` non lu (`:65-67`) ; sinon toutes les lignes, coupées aux virgules, de
   droite à gauche ; une valeur non ASCII ou une entrée qui n'est pas une adresse nue → `None` (`:70`, `:72`). `None`
   donne `Refuse` filtre activé (`:122-124`). `X-Real-IP` et `Forwarded` ne sont **jamais** lus (doc du module,
   `:4-8` ; C-32-6-5).
5. **Le pair est normalisé deux fois** : `pair.ip().to_canonical()` au transport (`transport.rs:81`) et
   `pair.to_canonical()` dans `client` (`reseau.rs:64`).
6. **« Aucune plage » n'est pas « filtre désactivé »** : filtre activé et `allowed_networks` vide → **personne** n'est
   admis (AC-a8, `32-6-endpoint-mcp.md` ; test (o), `tests/mcp_reseau_e2e.rs:264-268`). Le filtre désactivé, c'est
   `address_filter_enabled = FALSE` seulement.
7. **Le serveur désactivé ne décide rien** : `serveur_mcp::reglages_actifs` ne rend que les sociétés `enabled = TRUE`
   (`services/serveur_mcp.rs:38-47`) ; sans société activée, `servies` est vide sans qu'aucun `decider` ne tourne
   (`transport.rs:84-97`).
8. **Le niveau de journal** : `RUST_LOG` si non vide, sinon `info` (`logging.rs:129-131`) ; `build_log_filter` passe la
   chaîne à `EnvFilter::new` et ajoute `sqlx=warn` sauf directive `sqlx` explicite (`logging.rs:104-120`). Les
   composes transmettent `RUST_LOG: ${RUST_LOG:-info}` (`docker-compose.yml:91`, `docker-compose.prod.yml:109`) ;
   `.env.example:245-253` le documente. Une directive de cible se compare au **préfixe** de la cible de l'évènement
   (`tracing-subscriber-0.3.23/src/filter/env/directive.rs:246` : `meta.target().starts_with(&target[..])`) ; la cible
   d'un évènement de `reseau.rs` est `kesh_api::mcp::reseau` (le test `n_prime_…` l'exige :
   `tests/mcp_reseau_e2e.rs:302`).
9. **La mise en forme** : le journal standard (`fmt::layer()`, `logging.rs:135`) cite les champs texte entre guillemets
   — `record_str` → `record_debug(field, &value)` hors `message` (`tracing-subscriber-0.3.23/src/fmt/format/mod.rs:1264-1273`)
   — et rend `%valeur` par `Display`, sans guillemets ; booléens et entiers passent par `record_debug` — `company_id`
   (`i64`) par `record_i64` (`tracing-core-0.1.36/src/field.rs:291`), les comptes (`usize`) par `record_u64` (`:296`),
   les booléens par `record_bool` (`:311`). **Le message vient en tête**, avant les champs : les macros rangent
   `message` en premier (`tracing-0.1.44/src/macros.rs:660` : `{ message = …format_args!(…), $($fields)* }`) ; la
   capture des tests écrit donc `<NIVEAU> <cible> message=serveur MCP : décision du filtre d'adresse company_id=…`
   (le message par `record_debug` d'un `Arguments`, sans guillemets). La capture écrit ` nom=valeur` sans
   guillemets pour tous les champs (`tests/common/capture_trace.rs:30-38`). **Un exemple de ligne au manuel se copie
   donc d'une sortie réelle**, non de la capture (C-32-6c-8), et **du fichier de log**, non de la sortie standard
   (fait 13, C-32-6c-11).
10. **La capture des tests** : `CaptureTrace::installer()` retient tous les évènements, tous niveaux, sous la forme
    `"<NIVEAU> <cible> champ=valeur…"` (`tests/common/capture_trace.rs:40-47`), par fil (`:9-14`) ; elle sert déjà à
    l'issue `Ecartee` (`tests/mcp_reseau_e2e.rs:289-307`) et à `jeton_jamais_journalise`
    (`tests/mcp_transport_e2e.rs:1299-1336`).
11. **Précédent d'adresse au journal** : la connexion journalise l'adresse du client en `info!` et `warn!`
    (`routes/auth.rs:190`, `:229`, `:239`, `:652` : `ip = %ip`), dont un `warn!` **par requête** une fois la limite de
    débit atteinte (`:190`).
12. **Le manuel** : DSM, étape 4 (`docs/manual/fr/admin-manual.tex:638-661`) fait poser `X-Forwarded-Proto` et
    `X-Real-IP` (`:652-655`) — **pas** `X-Forwarded-For` ; or `/mcp` ignore `X-Real-IP`. Journaux DSM : `:696`.
    § *Monitoring et logs* (`:3017`), § *Logs applicatifs* (`:3019-3027`, `docker compose logs`). **`RUST_LOG` est déjà
    décrit** au tableau *Journalisation* de § `sec:env-vars` (`:792` : « Niveau/directives (`trace`..`error`). Baseline
    `sqlx=warn` maintenue en `debug` (anti-fuite SQL). », défaut `info`) ; le `.tex` écrit `RUST\_LOG`, que
    `grep -n "RUST_LOG"` ne voit pas — le motif à rejouer est `grep -nE 'RUST\\?_LOG'`, qui rend `:792`, `:1641`, `:1643`,
    `:1885`, `:1924`. `:1641` dit que **les variables du terminal l'emportent sur le `.env`** (interpolation de Compose),
    dans le contexte d'une répétition sur une machine de développement, sans `sudo` — la 32-6c ne le transpose pas au
    NAS (validation P2, F1) ;
    `:621` fait appliquer une modification du `.env` par `sudo docker compose up -d`. **Aucune ligne ne décrit `/mcp`**
    (la section *Le serveur MCP* est à la 32-7, `backlog`). *(Fait corrigé à la validation P1, R1 : il affirmait
    qu'aucune ligne ne parlait de `RUST_LOG`.)*
13. **Où lire le journal sans couleurs** (validation P1, R2 = F2). La sortie standard (`fmt::layer()`, `logging.rs:135`)
    n'appelle pas `with_ansi(false)` : avec la fonctionnalité `ansi` (par défaut, `crates/kesh-api/Cargo.toml:36` ne
    coupe pas les défauts), elle émet des codes couleur sauf si `NO_COLOR` est non vide
    (`tracing-subscriber-0.3.23/src/fmt/fmt_layer.rs:743` : `let ansi = cfg!(feature = "ansi") &&
    env::var("NO_COLOR").map_or(true, |v| v.is_empty());` ; `:305-312` en est la documentation), ce que ni les composes ni le `Dockerfile` ne posent :
    `docker compose logs` porte des séquences d'échappement autour des noms de champs et des `=`, et un `grep` sur
    `champ=valeur` y échoue (le message, non stylé, se retrouve). **Le fichier de log, lui, est sans couleurs**
    (`logging.rs:143` : `.with_ansi(false) // jamais de codes couleur dans un fichier`) et **actif par défaut** dans
    les deux composes (`docker-compose.yml:96`, `docker-compose.prod.yml:119` : `/var/log/kesh/kesh.log`), monté sur
    `./log` par le compose du Synology (`docker-compose.prod.yml:165`, montage fixe ; installé sous le nom
    `docker-compose.yml`, `admin-manual.tex:574`), sur `${KESH_LOG_HOST_DIR:-./log}` par `docker-compose.yml:140`
    (validation P2, R5), au format `pretty` par défaut (`:99`, `:122`) — le même formateur que la sortie
    standard (`logging.rs:143-147`). ⚠️ **Le nom réel du fichier** : avec la rotation `daily` (défaut), le fichier
    s'appelle `kesh.<AAAA-MM-JJ>.log` (`tracing-appender-0.2.5/src/rolling.rs:648` :
    `format!("{}.{}.{}", filename, date, suffix)`, date `[year]-[month]-[day]`, `:564`) ; avec la rotation `never`,
    `kesh.log` (`:643` : `format!("{}.{}", filename, suffix)`). Par défaut donc **non** `kesh.log` — que le
    manuel (`admin-manual.tex:957`, `sudo tail -f ./log/kesh.log`) et `.env.example:289` écrivent à tort. Défaut
    préexistant, hors du périmètre (C-32-6c-13) ; la sous-section de la 32-6c écrit le nom réel.

## Critères d'acceptation

Tests dans `crates/kesh-api/tests/mcp_reseau_e2e.rs` (par `CaptureTrace`, pair simulé par `MockConnectInfo`, comme les
tests (a) à (s)) et dans `crates/kesh-api/src/logging.rs` (`mod tests`). Adresses d'exemple : plages de documentation
(`192.0.2.0/24`, `198.51.100.0/24`, `2001:db8::/32`) et adresses privées génériques ; aucune donnée réelle.

### A — L'évènement de décision

1. **AC-1 — un évènement, et un seul, par décision `Admis` ou `Refuse`.** `decider` émet, pour chaque société dont il
   rend `Admis` ou `Refuse`, **exactement un** évènement de cible `kesh_api::mcp::reseau`, de message fixe
   `serveur MCP : décision du filtre d'adresse`, avec ces champs, **dans cet ordre** (C-32-6c-2, C-32-6c-3) :

   | champ | valeur | forme |
   |---|---|---|
   | `company_id` | `s.company_id` | entier |
   | `filtre` | `s.address_filter_enabled` | booléen |
   | `plages` | `s.allowed_networks.len()` (entrées stockées, valides ou non) | entier |
   | `pair` | le pair TCP normalisé (`to_canonical`) | `%` (adresse) |
   | `pair_mandataire` | le pair appartient-il aux mandataires analysés de `s` | booléen |
   | `xff_lignes` | nombre de lignes `X-Forwarded-For` (0 : absent) | entier |
   | `xff_entrees` | nombre de morceaux séparés par des virgules, toutes lignes, comptés **sur les octets** (une ligne non ASCII compte ; une ligne vide compte 1) | entier |
   | `x_real_ip` | `X-Real-IP` présent | booléen |
   | `forwarded` | `Forwarded` présent | booléen |
   | `client` | le client que retient `reseau::client`, ou `aucun` | `%` |
   | `verdict` | `admis` ou `refuse` | texte |

   **Aucun autre champ.** Un seul **jeu de champs** (une fonction de `reseau.rs` appelée par `decider`), deux niveaux
   (AC-2) : un niveau `tracing` est une constante, il faut donc deux appels de macro (ou un `macro_rules!` local) ;
   l'égalité de l'AC-T1 tient les deux arêtes. Le **message précède les champs** dans la ligne écrite (fait 9) : la
   ligne attendue est `<NIVEAU> kesh_api::mcp::reseau message=serveur MCP : décision du filtre d'adresse
   company_id=… filtre=… … verdict=…`.
2. **AC-2 — les niveaux.** `Refuse` → **`WARN`** (visible au niveau par défaut, `info`) ; `Admis` → **`DEBUG`**
   (visible sous `RUST_LOG=…,kesh_api::mcp::reseau=debug`, AC-8). Les champs sont **les mêmes** aux deux niveaux
   (C-32-6c-4).
3. **AC-3 — le filtre désactivé journalise ce qu'il retiendrait.** `address_filter_enabled = FALSE` : la décision
   reste `Admis` dans tous les cas (fait 3 ; aucun test existant ne change), mais le client est **calculé** pour le
   journal — `client=aucun` si l'en-tête d'un mandataire déclaré est illisible — et l'évènement part en `DEBUG` avec
   `filtre=false`, `verdict=admis` (C-32-6c-5). C'est ce qui permet de vérifier le client vu **avant** d'activer le
   filtre.
4. **AC-4 — l'issue `Ecartee` n'a pas d'évènement de décision.** Une entrée invalide de `trusted_proxies` garde son
   `error!` actuel (`reseau.rs:110-114`, inchangé) et **n'émet pas** l'évènement de l'AC-1 : la décision a déjà le
   sien, et aucun client n'est calculé (le mandataire illisible pourrait être pris pour le client).
5. **AC-5 — rien que des valeurs dérivées.** L'évènement ne porte **aucune valeur d'en-tête** reçue (ni
   `X-Forwarded-For`, ni `X-Real-IP`, ni `Forwarded`, ni `Origin`, ni `Authorization`, ni `Cookie`), aucun corps, aucune
   clé, aucune donnée comptable : seulement des adresses **analysées** (`IpAddr`), des comptes, des booléens, l'id de
   société et deux mots fixes (C-32-6c-6). Garde : test `journal_du_filtre_sans_secret` (AC-T4).
6. **AC-6 — rien ne change pour le client.** Les réponses de `/mcp` sont **octet pour octet** celles de `c5287dec` :
   refusé = réponse de route absente, admis = suite des contrôles. Les tests (a) à (s) de `mcp_reseau_e2e.rs` et ceux de
   `mcp_transport_e2e.rs` passent **sans modification** ; les tests neufs comparent aussi la réponse à la route absente
   (`verdict(…)`, `mcp_reseau_e2e.rs:44-68` ; pour l'AC-T4, par `envoyer` et `requete`, `tests/common/mcp.rs:272`,
   `:304`). Le journal ne devient pas un oracle : il ne sort pas du serveur.
7. **AC-7 — le serveur désactivé n'écrit rien.** Sans société `enabled = TRUE`, aucune décision n'est prise (fait 7) :
   **aucun** évènement de décision. Le manuel dit ce que veut dire l'absence de ligne de décision pour une société,
   par l'énumération de l'AC-9 (viii) (fait 2 ; validation P1, R3 ; validation P2, R1 et F3).

### B — La directive de journal

8. **AC-8 — la directive ciblée n'ouvre que le filtre.** `build_log_filter("info,kesh_api::mcp::reseau=debug")` laisse
   passer un `debug!` de cible `kesh_api::mcp::reseau`, **pas** un `debug!` d'une autre cible de `kesh_api` ni un
   `info!` de `sqlx` (baseline `sqlx=warn` maintenue), et laisse passer le `warn!`. Test unitaire dans
   `logging.rs` (AC-T6) : la phrase du manuel est prouvée par le code qu'elle décrit, non par la documentation de la
   dépendance.

### C — Le manuel

9. **AC-9 — le manuel d'administration dit comment lire ce journal.** Une sous-section neuve
   `\subsection{Diagnostiquer le filtre d'adresse du serveur MCP}\label{sec:mcp-journal-filtre}`, dans § *Monitoring et
   logs*, après § *Logs applicatifs* (`admin-manual.tex:3019-3027`), « depuis la 0.16.0 », dit **tel que le code le
   fait** (C-32-6c-7) :
   - un refus s'écrit **toujours** (niveau `warn`) ; une admission seulement au niveau `debug`. ⚠️ **Le cas que la
     mise à jour du NAS doit établir — DSM vu comme un poste du réseau local, admis par les plages par défaut (cas 5 de
     l'AC-T1) — n'écrit rien au niveau par défaut** : pour le vérifier, passer le filtre en `debug` le temps du test
     (validation P1, F3) ;
   - **où poser `RUST_LOG`** : la variable est déjà décrite au tableau *Journalisation* (renvoi à § `sec:env-vars`,
     `admin-manual.tex:792`, sans la redécrire) ; pour la seule cible du filtre, **remplacer** la ligne
     `RUST_LOG=info` du `.env` de l'installation (`.env.example:253`) par `RUST_LOG=info,kesh_api::mcp::reseau=debug`,
     puis `sudo docker compose up -d` (comme `:621`) ; **vérifier l'effet** par la présence de lignes `DEBUG` de
     décision au journal après une requête admise ; **revenir à `RUST_LOG=info` ensuite** (même geste) ; ne pas
     employer `RUST_LOG=debug` tout court (tout Kesh passe en `debug`) (validation P2, F1, R4, F7) ;
   - **lire dans le fichier de log**, sans couleurs (fait 13) — une seule commande, juste dans la configuration par
     défaut : `sudo grep "filtre d'adresse" ./log/kesh.*.log`, dans le dossier de l'installation (rotation
     quotidienne : `kesh.<AAAA-MM-JJ>.log`) ; **ses limites, en une phrase** : avec la rotation `never` le fichier
     s'appelle `kesh.log`, avec `KESH_LOG_HOST_DIR` il est dans ce dossier-là, et le motif est développé par le shell
     de l'utilisateur avant `sudo`, si bien que le dossier doit lui être lisible (validation P2, F8, R5, R3). Format
     `pretty` par défaut (avec `KESH_LOG_FILE_FORMAT=json`, une ligne JSON par évènement, mêmes champs). **Aucune
     commande n'est prescrite pour la sortie standard** (`docker compose logs`), dont les codes couleur coupent les
     `grep` sur `champ=valeur` (fait 13) : le repli de la validation P1, éprouvé sur aucun NAS, est retiré
     (validation P2, F8, R3 ; C-32-6c-11 annoté) ;
   - **une ligne d'exemple copiée d'une sortie réelle** du binaire, **prise du fichier de log** (fait 9, fait 13),
     adresses de documentation ;
   - chaque champ en une ligne, et ce qu'il permet de trancher : un tableau de lecture au moins pour ces cas —
     (i) `pair` = une adresse de passerelle Docker, `pair_mandataire=false`, `xff_lignes=0` : le proxy ne pose pas
     `X-Forwarded-For`, Kesh voit tous les clients sous l'adresse du proxy, le filtre ne distingue personne — c'est le
     cas de la publication de port par le mandataire en espace utilisateur de Docker (la passerelle) (Guy,
     2026-10-25 : le filtre ne filtre rien et « le manuel le dit » ; la sécurité relève de l'infrastructure) ;
     (ii) `pair_mandataire=false`, `xff_lignes` ≥ 1 : l'en-tête est posé mais ignoré, le filtre ne distingue pas
     davantage — par exemple un Traefik non déclaré, qui pose l'en-tête (`pair` = l'adresse de son conteneur), **non
     éprouvé** (fiche 32-6, `32-6-endpoint-mcp.md:1017-1019`) (validation P2, F2 = R2) ; déclarer le pair parmi les
     mandataires de confiance, **avec cette réserve, écrite au manuel** (validation P1, R4 ; acceptée par Guy) : *si le
     mandataire déclaré est une adresse par laquelle passent tous les clients (la passerelle Docker), tout poste qui
     joint directement le port publié peut forger `X-Forwarded-For`, ou, sans en-tête, être vu comme la passerelle
     elle-même, admise par les plages par défaut (`reseau.rs:76-78`) ; le filtre est une défense en profondeur, non
     une barrière* (validation P2, R7) ;
     (iii) `pair_mandataire=true`, `client` = l'adresse du poste : le filtre voit le vrai client ;
     (iv) `client=aucun` : l'en-tête du mandataire porte une entrée qui n'est pas une adresse nue (port, `unknown`),
     **filtre activé** la requête est refusée (filtre coupé, elle est admise : (vi)) ;
     (v) `x_real_ip=true` ou `forwarded=true` : ces en-têtes ne sont **jamais** lus par `/mcp` ;
     (vi) `filtre=false` : la ligne dit ce que le filtre retiendrait, rien n'est refusé ;
     (vii) `filtre=true`, `plages=0` : personne n'est admis ;
     (viii) **aucune ligne de décision pour une société**, le filtre étant en `debug` (au niveau par défaut, une
     admission n'écrit rien) — **énumération des sorties de Kesh, non des causes extérieures** (validation P2, F3 et
     R1). Fait foi : les trois retours `RouteAbsente` du transport (`transport.rs:79`, `:88`, `:96`) et la route
     exacte `route_service("/mcp", …)` (`lib.rs:1278`). Au code de `c5287dec` :
     — **avec une erreur** du serveur MCP : « réglages illisibles, /mcp n'est pas servi » (`transport.rs:87`) : la
     base n'a pas pu être lue, aucune société n'est servie ; « ligne de réglages illisible, la société n'est pas
     servie » (`services/serveur_mcp.rs:41`) : les réglages de cette société (`company_id` donné) ne se décodent pas ;
     « une entrée de trusted_proxies ne s'analyse pas » (`reseau.rs:110-114`, AC-4) ;
     — **sans erreur** : le serveur MCP n'est pas activé pour cette société (fait 7 ; sans aucune ligne pour aucune
     société, il ne l'est pour aucune) ; ou la requête est arrivée sur un chemin voisin (`/mcp/`, `/MCP`), servi par le
     repli statique (fait 2) ; ou elle n'est pas parvenue à Kesh. `transport.rs:79` (`ConnectInfo` absent) est
     inatteignable en production (fait 2 (c)) ;
     (ix) `pair_mandataire=true`, `xff_lignes=0`, `client` = le pair : le mandataire est déclaré mais ne pose pas
     `X-Forwarded-For` (`reseau.rs:76-78`) — le diagnostic qui suit (ii) si le proxy n'envoie rien (validation P1, R6) ;
   - avec plusieurs sociétés activées, **une ligne par société** (`company_id`) : un refus pour l'une n'empêche pas
     l'autre de servir ;
   - **ce que le journal ne contient jamais** : aucune clé, aucun en-tête `Authorization`, aucun corps, aucune valeur
     d'en-tête reçue (des comptes et des présences seulement), aucune donnée comptable ; il contient en revanche
     l'**adresse** des clients (comme le journal de connexion) ;
   - en cas de requêtes refusées en nombre, chacune écrit une ligne (aucune limite de débit sur `/mcp`).
10. **AC-10 — DSM renvoie au diagnostic.** À l'étape 4 de § `sec:synology` (`admin-manual.tex:652-655`), une phrase :
    le serveur MCP n'emploie **pas** `X-Real-IP` ; ce que le filtre voit derrière le proxy de DSM se vérifie au journal
    (\S\ref{sec:mcp-journal-filtre}). **Aucune prescription** d'en-tête supplémentaire à DSM tant qu'elle n'a pas été
    vérifiée (C-32-6c-9 ; point 4 : Guy, le 2026-10-25, « non » au conseil d'en-tête avant vérification).
11. **AC-11 — PDF.** `admin-manual.pdf` régénéré (`make fr` dans `docs/manual/`, sous le verrou) et commité ; le PDF
    aplati (`pdftotext … - | tr '\n' ' ' | tr -s ' '`) contient « Diagnostiquer le filtre d'adresse », la directive
    `kesh_api::mcp::reseau=debug`, le nom `kesh.*.log`, la réserve de l'AC-9 (ii) (« peut forger ») et la phrase de
    l'AC-10. `user-manual` et brochure : inchangés. **CHANGELOG
    inchangé** (C-32-6c-10).

### T — Les tests

12. **AC-T1 — `decision_journalisee_par_cas`** : pour chaque cas, la requête, puis **la liste des évènements de
    décision** (évènements de la capture dont le texte contient `décision du filtre d'adresse`) **égale** à la liste
    attendue, construite par une fonction `attendu(niveau, champs…)` qui suit l'ordre de l'AC-1 — **le message
    d'abord, puis les onze champs** (fait 9) — égalité de chaîne complète, non `contains` (C-32-6c-3). Quatorze cas :

    | # | réglages | pair | en-têtes | verdict, niveau | champs notables |
    |---|---|---|---|---|---|
    | 1 | défauts (filtre, 7 plages) | `198.51.100.7` | — | refuse, WARN | `pair_mandataire=false xff_lignes=0 client=198.51.100.7` |
    | 2 | défauts | `192.168.1.20` | — | admis, DEBUG | `client=192.168.1.20` |
    | 3 | mandataire `172.18.0.1` | `172.18.0.1` | XFF `198.51.100.7` | refuse, WARN | `pair_mandataire=true xff_lignes=1 xff_entrees=1 client=198.51.100.7` |
    | 4 | mandataire `172.18.0.1` | `172.18.0.1` | XFF `192.168.1.20`, `X-Real-IP: 198.51.100.7` | admis, DEBUG | `x_real_ip=true client=192.168.1.20` |
    | 5 | défauts (aucun mandataire) | `172.18.0.1` | XFF `198.51.100.7, 192.0.2.10`, `Forwarded: for=192.0.2.10` | admis, DEBUG | `pair_mandataire=false xff_lignes=1 xff_entrees=2 forwarded=true client=172.18.0.1` — **le cas DSM non déclaré** |
    | 6 | mandataire `172.18.0.1` | `172.18.0.1` | XFF `unknown` | refuse, WARN | `client=aucun` |
    | 7 | mandataire `172.18.0.1` | `172.18.0.1` | deux lignes XFF : `192.0.2.10, 192.168.1.20` puis `198.51.100.7` | refuse, WARN | `xff_lignes=2 xff_entrees=3 client=198.51.100.7` |
    | 8 | filtre désactivé | `198.51.100.7` | — | admis, DEBUG | `filtre=false client=198.51.100.7` |
    | 9 | filtre désactivé, mandataire `172.18.0.1` | `172.18.0.1` | XFF `unknown` | admis, DEBUG | `filtre=false client=aucun` |
    | 10 | filtre activé, `allowed_networks = []` | `192.168.1.20` | — | refuse, WARN | `plages=0` |
    | 11 | défauts | `2001:db8::7` | — | refuse, WARN | `pair=2001:db8::7 client=2001:db8::7` |
    | 12 | défauts | `::ffff:198.51.100.7` | — | refuse, WARN | `pair=198.51.100.7` (normalisé) |
    | 13 | mandataire `172.18.0.1`, `allowed_networks = ["pas-un-réseau","192.0.2.0/24"]` | `172.18.0.1` | une ligne XFF **non ASCII** : octets `192.0.2.10, \xe9` | refuse, WARN | `plages=2 xff_lignes=1 xff_entrees=2 client=aucun` — `plages` compte l'entrée invalide ; la ligne non ASCII compte sur ses octets |
    | 14 | mandataire `172.18.0.1` | `172.18.0.1` | deux lignes XFF : **vide** (`""`), puis `192.0.2.10,` | refuse, WARN | `xff_lignes=2 xff_entrees=3 client=aucun` — une ligne vide compte 1, une virgule finale ouvre un morceau vide |

    Chaque cas compare aussi la réponse à la route absente (`verdict(…)`) : refusé ⇔ route absente (AC-6). La
    capture s'installe **par cas** (après la pose des réglages), de sorte qu'une liste attendue d'**un** élément prouve
    « un seul évènement » (AC-1). `verdict(…)` envoie aussi la requête de route absente, qui ne passe pas par
    `decider` : elle n'ajoute aucun évènement (à vérifier à l'étape 0, sinon filtrer sur la seule requête `/mcp`).
    Cas 13 et 14 (validation P1, F1) : `HeaderValue::from_bytes` admet l'octet `0xE9` et la valeur vide
    (`http-1.4.2/src/header/value.rs:557-559` : `b >= 32 && b != 127 || b == b'\t'`, appliqué octet par octet,
    `:216-220`) ; au cas 13, le client étant `None`, la boucle des plages ne
    tourne pas et le `debug!` d'entrée ignorée n'est pas émis (il n'est de toute façon pas un évènement de décision) ;
    au cas 14, la première ligne vide suffit à rendre `None` (`"".parse::<IpAddr>()` échoue, `reseau.rs:72`).
13. **AC-T2 — `ecartee_sans_evenement_de_decision`** : `trusted_proxies = ["pas-un-mandataire"]` → la liste des
    évènements de décision est **vide** ; témoin : l'`error!` `trusted_proxies` est présent dans la même capture
    (`exiger_ensemble`, comme `mcp_reseau_e2e.rs:302-306`).
14. **AC-T3 — `serveur_desactive_aucun_evenement`** : aucune société activée → réponse de route absente, **aucun**
    évènement de décision ; **témoin dans la même capture** : la société activée ensuite, la même requête produit
    l'évènement attendu (sans quoi le test passerait à vide).
15. **AC-T4 — `journal_du_filtre_sans_secret`** : deux requêtes, l'une admise (pair `192.168.1.20`) et l'autre refusée
    (mandataire `172.18.0.1`, XFF piégé), portant `Authorization: Bearer kesh_mcp_PIEGE_JETON_326C`,
    `Cookie: kesh_session=PIEGE_COOKIE_326C`, `X-Forwarded-For: PIEGE_XFF_326C` (sur la requête refusée),
    `X-Real-IP: PIEGE_XRI_326C`, `Forwarded: for=PIEGE_FWD_326C`, corps `{"PIEGE_CORPS_326C": …}` ; **`Origin:
    https://piege-origine-326c.example` sur la seule requête refusée**. Les deux requêtes partent en **`POST`** avec
    **`Content-Type: application/json`** (sans quoi l'admise s'arrête en 405 ou 415 avant l'authentification,
    `transport.rs:100`, `:115` ; validation P2, F6). Les requêtes passent par `envoyer` et `requete`
    (`tests/common/mcp.rs:272`, `:304`), **non** par `verdict(…)` : `verdict` fixe le corps `LISTE` et exige 401 d'une
    requête admise (`mcp_reseau_e2e.rs:44-68`). Assertions de réponse : la refusée égale la route absente ; l'admise,
    sans `Origin`, atteint l'authentification et rend **401** avec `TEXTE_NON_AUTHENTIFIE` (le jeton piégé est donc lu
    par `authentifier`, et la capture couvre aussi ce chemin) (validation P1, R8). Témoin : les deux évènements de
    décision existent ; puis `exiger_absent` pour chaque piège et pour `kesh_mcp_` — sur **tous** les évènements de la
    capture, non sur les seuls évènements de décision.
16. **AC-T5 — `deux_societes_deux_evenements`** : deux sociétés créées par `societe(pool, …)` (`tests/common/mcp.rs:64`)
    dans cet ordre — `a` puis `b`, `a < b` (identifiants croissants) — et activées, `a` aux défauts, `b` avec
    `allowed_networks = ["192.0.2.0/24"]` ; pair `192.0.2.10` → admis (par `b`, réponse 401 par `verdict(…)`) ; la
    liste des évènements de décision est **égale**, dans l'ordre, à `[refuse en WARN pour a, admis en DEBUG pour b]`.
    **Sans tri** : `list_enabled` rend les sociétés `ORDER BY company_id` (`kesh-db/src/repositories/mcp_server_settings.rs:103`)
    et `reglages_actifs` l'écrit comme contrat (« par `company_id` croissant (AC-a2) », `services/serveur_mcp.rs:31`),
    et le transport appelle `decider` dans cet ordre (`transport.rs:91-94`) (validation P1, F4, F5, R7). L'aide `livres`
    ne convient pas : elle ne rend pas l'id du leurre (`tests/common/mcp.rs:171`).
17. **AC-T6 — `filtre_cible_n_ouvre_que_le_filtre_mcp`** (`logging.rs`) : un abonné local (`with_default`) composé de
    `build_log_filter("info,kesh_api::mcp::reseau=debug")` et d'une couche de capture minimale ; évènements émis avec
    `target:` explicite — `debug!` de `kesh_api::mcp::reseau` (passe), `debug!` de `kesh_api::routes` (bloqué),
    `info!` de `sqlx::query` (bloqué), `warn!` de `kesh_api::mcp::reseau` (passe), `info!` de `kesh_api` (passe).

## Tâches

- [ ] **T-0 — étape 0** (§ *À recontrôler*), lignes lues citées au Change Log.
- [ ] **T-1 — l'évènement** (AC-1 à AC-5) : dans `mcp/reseau.rs`, une fonction privée `journaliser_decision(…)` (nom
  indicatif) appelée par `decider` à ses deux sorties `Admis`/`Refuse`, une seule liste de champs pour les deux niveaux
  (un `macro_rules!` local, ou deux appels dont l'égalité est tenue par AC-T1) ; `decider` calcule le client **avant**
  de consulter l'état du filtre, sans changer son résultat (AC-3) ; comptes d'en-têtes sur les octets ; mise à jour de la
  doc du module (`reseau.rs:1-23`), de celle de `decider` (`:101-103`) et de la ligne 2 de l'ordre des contrôles
  (`mcp/mod.rs:15`) : « un évènement par société décidée ».
- [ ] **T-2 — les tests** AC-T1 à AC-T5 dans `tests/mcp_reseau_e2e.rs` ; AC-T6 dans `logging.rs`.
- [ ] **T-3 — le manuel** (AC-9 à AC-11) : sous-section, phrase DSM, ligne d'exemple prise d'une sortie réelle — le
  **fichier de log** du backend local de l'E2E (`KESH_LOG_FILE_PATH` vers un dossier du worktree, format `pretty`),
  après une requête vers `/mcp` depuis `127.0.0.1` avec un réglage qui refuse — ; PDF régénéré sous le verrou ; contrôle du PDF aplati, sorties citées.
- [ ] **T-4 — mutations** (§ *Tests et mutations*), une à une, restauration par `git checkout --` + `touch`.
- [ ] **T-5 — gates** (§ *Gates*), Dev Agent Record, `sprint-status.yaml` → `review`, prompt de revue P1.

## Tests et mutations prévus

| # | mutation (dans `mcp/reseau.rs` sauf mention) | test qui doit rougir |
|---|---|---|
| M1 | évènement retiré à la sortie `Refuse` | AC-T1 (cas 1, 3, 6, 7, 10, 11, 12, 13, 14) |
| M2 | `Refuse` journalisé en `debug!` | AC-T1 (cas 1 : `WARN` attendu) |
| M3 | `Admis` journalisé en `info!` (ou `warn!`) | AC-T1 (cas 2 : `DEBUG` attendu) |
| M4 | retour anticipé du filtre désactivé gardé avant l'évènement (pas d'évènement) | AC-T1 (cas 8, 9) |
| M5 | `client` = le pair au lieu du client retenu | AC-T1 (cas 3, 4, 7) |
| M6 | `xff_entrees` compte les lignes | AC-T1 (cas 5 : 2 attendu ; cas 7 : 3 attendu) |
| M7 | `x_real_ip` toujours `false` | AC-T1 (cas 4) |
| M8 | `forwarded` lu sur `x-forwarded-for` (présence confondue) | AC-T1 (cas 3 : `forwarded=false` attendu, XFF présent) |
| M9 | `pair_mandataire` calculé contre `allowed_networks` | AC-T1 (cas 2 : `pair_mandataire=false` attendu, le pair étant dans les plages) |
| M10 | valeur brute de `X-Forwarded-For` ajoutée en champ | AC-T4 (`PIEGE_XFF_326C`) et AC-T1 (égalité) |
| M11 | `en_tetes = ?en_tetes` ajouté en champ | AC-T4 (`PIEGE_JETON_326C`, `PIEGE_COOKIE_326C`) |
| M12 | évènement émis aussi pour `Ecartee` (dans la branche `Ecartee` de la boucle des mandataires, `reseau.rs:106-117`) | AC-T2 |
| M13 | évènement émis dans la boucle des plages (un par entrée) | AC-T1 (cas 1 : un élément attendu, sept émis) |
| M14 | évènement émis une seconde fois au transport (`transport.rs`, après le filtre) | AC-T1 (un élément attendu) |
| M15 | `decider` du filtre désactivé rend `Refuse` quand `client` est `None` (le calcul anticipé change le verdict) | AC-T1 cas 9 (`admis` attendu, et la réponse n'est plus celle d'un admis) |
| M16 | `build_log_filter` remplace la chaîne par `debug` quand elle contient `::` (directive ciblée élargie) | AC-T6 (`kesh_api::routes` bloqué) |
| M17 | `plages` = nombre d'entrées **valides** de `allowed_networks` | AC-T1 (cas 13 : `plages=2` attendu, 1 émis) |
| M18 | `xff_entrees` compté sur le texte (`to_str().ok()`), lignes non ASCII ignorées | AC-T1 (cas 13 : `xff_entrees=2` attendu, 0 émis) |
| M19 | `xff_entrees` saute les morceaux vides (`split(…).filter(\|m\| !m.is_empty())`) | AC-T1 (cas 14 : `xff_entrees=3` attendu, 1 émis) |
| M20 | `transport.rs` émet un évènement de décision (`verdict=refuse`) quand `servies` est vide | AC-T3 (aucun évènement attendu, serveur désactivé) |
| M21 | une seule décision journalisée par requête (la première société ; ou émission déplacée au transport, une ligne par requête) | AC-T5 (deux éléments attendus) |

⚠️ **M9 et le choix des adresses** : `172.18.0.1` est dans `172.16.0.0/12` (plages par défaut, `tests/common/mcp.rs:162`),
si bien que, au cas 3, mandataire et plage donnent tous deux `true` ; c'est le **cas 2** (pair `192.168.1.20` dans les
plages, hors des mandataires) qui tue M9. Si l'étape 0 veut une mutation tuée par deux cas, ajouter un cas où le
mandataire est hors des plages (`192.0.2.1` déclaré, réglages par défaut). M15 : la mutation la plus probable du
développement (le calcul du client avancé) ; le cas 9 la tue seul, le test (d) ne voit que le pair public sans en-tête.
M17 à M21 (validation P1) : M17 à M19 gardent les deux propriétés de comptage de l'AC-1 (« valides ou non », « sur les
octets ») qu'aucun des douze premiers cas n'exerçait (F1) — M18 est la tentation naturelle, puisque `client` fait
`to_str().ok()` (`reseau.rs:70`) ; M19 est la mutation du cas 14, sans laquelle ce cas ne garderait rien de neuf
(C-32-6c-12) ; M20 et M21 donnent une mutation à l'AC-T3 et à l'AC-T5, qui n'en avaient pas (R7).

Tests existants qui doivent rester verts **sans modification** : tout `mcp_reseau_e2e.rs` (a) à (s), (n'),
`mcp_transport_e2e.rs` (dont `jeton_jamais_journalise`, qui voit désormais les évènements de décision et y cherche le
jeton), `mcp_journal_e2e.rs`, les tests unitaires de `reseau.rs` et de `logging.rs`.

## Gates

- Entre les passes : gate ciblé — `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo nextest run -E 'binary(mcp_reseau_e2e) | binary(mcp_transport_e2e) | binary(mcp_journal_e2e)'` et les tests de
  bibliothèque de `kesh-api` (`logging.rs`, `mcp/reseau.rs`).
- **Au dernier commit de code** : base remise à zéro (bases propres au worktree, **sans** redémarrer le conteneur,
  `docs/agents-orchestration.md` § 2), **gate complet** `scripts/test-fast.sh --ci` sous `flock …/.gate.lock` ; frontend
  `check`, `lint-i18n-ownership`, `test:unit`, `build` (aucun fichier du frontend ne change : le build sert l'E2E) ;
  **E2E complet** (`docs/testing.md` § *Les échecs attendus*). Aucune migration : pas d'exception `kesh-db`.
- **Recette visuelle : sans objet** — aucun contrôle visible n'est ajouté ni modifié.
- Revue de code : P1 complète (Sonnet ou Opus, jamais Haiku), lentilles B, E, A ; axes nommés : **le manuel contre le
  code et le PDF aplati**, **aucune valeur d'en-tête au journal**, **réponses inchangées**.

## Ce que les gardes ne voient pas

- **Ce que DSM fait réellement.** Les tests simulent un mandataire ; seul le NAS dira quel pair Kesh voit derrière le
  proxy de DSM, et si DSM pose `X-Forwarded-For`. C'est l'objet même de la story : le journal le dira à la mise à jour.
- **Un second site d'émission futur.** AC-T1 compare la liste complète des évènements de décision sur quatorze cas : un
  champ ajouté à **ce** site rougit ; un **autre** évènement, écrit ailleurs dans `mcp/` avec une valeur d'en-tête, ne
  rougit que si un test d'absence (AC-T4, `jeton_jamais_journalise`) passe par lui.
- **La forme d'une entrée illisible.** Le journal compte les entrées et dit `client=aucun`, sans la valeur (C-32-6c-6) :
  l'administrateur sait **qu'**une entrée n'est pas une adresse nue, non **laquelle** (port, `unknown`, crochets). Et
  tant que le pair n'est pas déclaré mandataire, l'en-tête n'est pas analysé : `xff_entrees` en donne le nombre, non la
  lisibilité.
- **L'amplification au niveau par défaut.** Chaque requête refusée écrit un `warn!` par société activée, sans limite
  de débit (décision de Guy : pas de limite en 0.16.0). Même classe que le `warn!` par requête de la connexion limitée
  (`routes/auth.rs:190`) et que l'`error!` d'une entrée de mandataire invalide (fiche 32-6, § *Ce que les gardes ne
  voient pas*, « amplification du journal du serveur »). Le fichier de log tourne chaque jour, sans borne de taille
  (§ `sec:logs-fichier`). Question 1.
- **Plusieurs sociétés** : un `warn` de refus pour une société n'empêche pas une autre de servir le même client ; le
  manuel le dit, le journal ne résume pas la requête.
- **L'adresse des clients** est une donnée personnelle de la personne qui se connecte ; elle est déjà au journal de
  connexion (fait 11), et le fichier de log est dans le périmètre de la sauvegarde de l'installation (§
  `sec:logs-fichier`).
- **Le temps de réponse** : l'évènement ajoute une mise en forme par décision, non mesurée ; l'oracle temporel de la
  fiche 32-6 (la décision lit la base, une route absente non) reste ce qu'il était.
- **La mise en forme réelle** (guillemets, ordre, couleurs de la sortie standard, format `json`) n'est vue d'aucun
  test : l'exemple du manuel se copie d'une sortie réelle, prise du fichier de log (faits 9 et 13), et la revue le relit.
  La commande de lecture du manuel (AC-9) n'est éprouvée sur aucun NAS ; ses limites y sont écrites.
- **Le manuel** : aucun gate ne le lit ; le prompt de revue nomme l'axe.

## Dev Notes

### Esquisse (indicative dans la forme, exigée dans le fond)

```rust
// mcp/reseau.rs — `decider`, après la lecture des mandataires (inchangée, `Ecartee` sans évènement)
let pair = pair.to_canonical();
let retenu = client(pair, en_tetes, &mandataires);          // calculé même filtre désactivé (AC-3)
let decision = if !s.address_filter_enabled {
    Decision::Admis
} else if let Some(c) = retenu {
    if admis_par_les_plages(s, c) { Decision::Admis } else { Decision::Refuse }
} else {
    Decision::Refuse
};
journaliser_decision(s, pair, contient(&mandataires, pair), en_tetes, retenu, decision);
decision
```

- Les comptes : `en_tetes.get_all("x-forwarded-for").iter()` — lignes ; morceaux par
  `v.as_bytes().split(|b| *b == b',').count()` sommés ; présence par `en_tetes.contains_key("x-real-ip")` et
  `"forwarded"`. **Aucune** de ces valeurs n'est convertie en texte.
- `client` : un petit type `Display` (« adresse » ou `aucun`) passé par `%`, pour une ligne sans guillemets.
- La boucle des plages et son `debug!` d'entrée ignorée (`reseau.rs:126-134`) restent ; seule leur place change si le
  calcul est réordonné. Le `debug!` d'entrée ignorée **n'est pas** un évènement de décision (filtrage des tests sur le
  message).

### Fichiers touchés (prévus) et modules

- `crates/kesh-api/src/mcp/reseau.rs`, `crates/kesh-api/src/mcp/mod.rs` (doc) — module **`mcp`** ;
  `crates/kesh-api/tests/mcp_reseau_e2e.rs` — rattaché au module `mcp` (convention de la fiche 32-6, `tests/mcp_*`) ;
- `crates/kesh-api/src/logging.rs` (`mod tests` seulement) — module **`logging`** ;
- `docs/manual/fr/admin-manual.tex` et `.pdf` — **manuel**.

**Trois modules** (≤ 5). `main.rs`, la configuration, les composes et `.env.example` ne changent pas : la directive
passe par `RUST_LOG`, déjà transmis (fait 8 ; C-32-6c-7).

### Le manuel (axe de revue)

- Vérifier l'AC-9 et l'AC-10 contre le code livré et contre le **PDF aplati** ; `grep -nE 'X-Real-IP|RUST\\?_LOG|/mcp'
  docs/manual/fr/admin-manual.tex` (le `.tex` écrit `RUST\_LOG`, fait 12) avant et après ; aucune autre phrase du manuel ne devient fausse (la Traefik,
  § `sec:traefik`, pose ses propres en-têtes : la sous-section en parle-t-elle juste ? Elle ne prescrit rien pour
  Traefik ; la 32-7 écrit la section *Le serveur MCP* et y renverra).
- **Pour la 32-7** (à reporter par l'orchestrateur dans sa fiche) : la section *Le serveur MCP* renvoie à
  \S\ref{sec:mcp-journal-filtre} pour le diagnostic, et reprend, avec ce que le filtre ne garantit pas, la réserve
  que la 32-6c écrit désormais elle-même à l'AC-9 (ii) : déclarer la passerelle Docker comme mandataire fait lire
  `X-Forwarded-For` à **toute** requête qui passe par elle, y compris celle d'un poste du réseau local qui joindrait le
  port publié directement, sans DSM, et qui pourrait alors forger l'en-tête. *(Annoté à la validation P1, R4 : ce
  point était reporté à la 32-7 ; la 32-6c pouvant paraître seule, il est au manuel dès la 32-6c.)*
- **Pour la 32-11** : la description détaillée du fonctionnement de MCP (décision de Guy du 2026-10-25, point 5) y
  est inscrite à l'epic ; la sous-section de la 32-6c reste un mode d'emploi du journal, non cette description.
- **Défaut préexistant, hors périmètre** (C-32-6c-13) : `admin-manual.tex:957` et `.env.example:289` nomment le fichier
  `./log/kesh.log`, que la rotation `daily` nomme `kesh.<AAAA-MM-JJ>.log` (fait 13). Issue à ouvrir par
  l'orchestrateur (P3, manuel qui décrit autre chose que le code) ; la 32-6c n'y touche pas.

### Références

- Fiche 32-6 : AC-a7, AC-a8, AC-a9, AC-b4 ; § *Ce que les gardes ne voient pas* ; § *Le manuel (axe de revue)* ;
  points 2 et 2 bis pour Guy. Registre 326 : C-32-6-4, C-32-6-5, C-32-6-6.
- Epic 32, § 32-6c ; § *Réponses de Guy (2026-10-25)* : pas de limite de débit en 0.16.0, le manuel dit ce que le
  filtre ne garantit pas.

## Points soumis à Guy

*Réponses de Guy du 2026-10-25, relayées par l'orchestrateur : 1 oui, 2 oui, 3 oui, 4 non, 5 oui — avec, au point 5,
« ajoute la description détaillée du fonctionnement de MCP », qui revient à la 32-11 (ci-dessous). Les questions
restent telles qu'elles ont été posées ; chaque réponse suit sa question.*

1. **Un refus du filtre au niveau `warn`, donc visible sans rien régler — d'accord ?** Chaque requête refusée écrit
   une ligne, sans limite de débit ; un robot qui frappe `/mcp` depuis l'extérieur remplirait le journal comme il le
   remplit déjà en frappant la page de connexion (`warn` par requête une fois la limite atteinte). *Ma recommandation :
   `warn`* — c'est le cas qui demande une action, et un refus d'un poste légitime doit se voir sans changer `RUST_LOG`.
   L'autre choix : tout en `debug`, le journal ne disant rien tant qu'on ne l'a pas demandé.
   → **Guy : oui** (refus en `warn`).
   *Ajout de l'orchestrateur, après la réponse — rien n'établit que Guy l'a vu (déplacé à la validation P2, F9 ;
   il figurait dans la question)* : complément de la validation P1 (F3), écrit au manuel (AC-9) : (a) avec plusieurs
   sociétés activées, un `warn` « refuse » part pour chaque société qui refuse, même quand une autre sert la requête ;
   (b) le cas que la mise à jour doit établir — DSM non déclaré, admis par les plages par défaut — n'écrit rien au
   niveau `info` : il faut le niveau `debug` le temps du test.
2. **Ne journaliser que la forme des en-têtes (nombre de lignes et d'entrées, présence), jamais leur valeur ?**
   *Ma recommandation : oui.* La valeur d'un en-tête est écrite par le client, donc par qui veut : la recopier ouvrirait
   le journal à des textes forgés. Le client **retenu**, lui, est une adresse analysée, et il suffit au diagnostic. Le
   coût : si une entrée est illisible, le journal dit `client=aucun` sans dire laquelle.
   → **Guy : oui** (formes et présences seulement, jamais les valeurs des en-têtes).
3. **Le filtre désactivé journalise-t-il quand même le client qu'il retiendrait ?** *Ma recommandation : oui* (en
   `debug`) : c'est ce qui te permet, à la mise à jour, de vérifier ce que Kesh voit derrière DSM **avant** d'activer le
   filtre, sans risquer de couper ton agent.
   → **Guy : oui** (journaliser aussi quand le filtre est coupé).
4. **Faut-il dès maintenant conseiller au manuel d'ajouter `X-Forwarded-For` aux en-têtes personnalisés de DSM ?**
   *Ma recommandation : non.* Ni la variable que DSM accepte ni son effet ne sont vérifiés ; la fiche fait dire au
   manuel que `/mcp` ignore `X-Real-IP` et où lire le diagnostic. Ce que la mise à jour du NAS établira, la 32-7 (ou un
   correctif du manuel) l'écrira.
   → **Guy : non** (pas de conseil `X-Forwarded-For` à DSM avant vérification) : l'AC-10 reste sans prescription.
5. **Le paragraphe du manuel avant la section *Le serveur MCP*** : la 32-6c l'écrit sous *Monitoring et logs*, la 32-7
   y renverra. *Ma recommandation : oui* — un diagnostic se cherche avec les journaux ; l'autre choix (attendre la
   32-7) laisserait la story sans son mode d'emploi.
   → **Guy : oui, « ajoute la description détaillée du fonctionnement de MCP »**. Cette description revient à la
   **32-11** (documentation de bout en bout) et y est inscrite à l'epic (§ 32-11 de `epic-32-serveur-mcp.md`) :
   manuels d'administration et utilisateur — ce qu'est MCP, le parcours d'une requête de l'agent jusqu'au journal, les
   portées, les outils, les prompts, la validation avant envoi, ce que Kesh garantit et ne garantit pas. **La 32-6c ne
   l'écrit pas** : sa sous-section reste le mode d'emploi du journal du filtre.

## À recontrôler à l'étape 0 du développement

- `reseau.rs:63-87` (`client`), `:104-140` (`decider`), `:110-114` (`error!`), `:129-132` (`debug!`) ; `transport.rs:81`,
  `:84-97` ; `mcp/mod.rs:15` — aucune story fusionnée entre-temps (32-8b, 32-10a) ne les a-t-elle changés ?
  `git log c5287dec..origin/main -- crates/kesh-api/src/mcp/reseau.rs crates/kesh-api/src/mcp/transport.rs`.
- `tests/common/capture_trace.rs:30-47` (format ` nom=valeur`, sans guillemets, `record_str`) et
  `tests/common/mcp.rs` (`reglages` `:137`, `RESEAUX_DEFAUT` `:162`, `activer` `:165`, `livres` `:171`, `societe`
  `:64`) ; `verdict(…)` (`mcp_reseau_e2e.rs:44-68`) : la requête de route absente n'émet-elle aucun évènement de
  `reseau` ? (sinon filtrer).
- `logging.rs:104-120`, `:129-131`, `:135` (sortie standard sans `with_ansi(false)`), `:143` (fichier sans couleurs) ;
  `tracing-subscriber` toujours en `0.3.23`, `tracing-appender` en `0.2.5` au `Cargo.lock` (faits 8, 9 et 13) ; les
  composes portent-ils toujours `KESH_LOG_FILE_PATH` par défaut (`docker-compose.yml:96`, `docker-compose.prod.yml:119`) ?
- `transport.rs:78-80`, `:87` et `services/serveur_mcp.rs:41` (les chemins sans décision du fait 2) : messages
  inchangés, puisque le manuel les cite (AC-7, AC-9 (viii)).
- Le manuel : `admin-manual.tex:638-661`, `:696`, `:3017-3027` ; la 32-7 a-t-elle écrit la section *Le serveur MCP*
  entre-temps ? Si oui, le renvoi de l'AC-10 s'ajoute aussi là, et la sous-section de l'AC-9 garde sa place.
- L'ordre des champs de l'AC-1 se retrouve-t-il dans la sortie réelle (fait 9) ? Le copier tel quel dans la ligne
  d'exemple du manuel.

## Dev Agent Record

### Agent Model Used

*(à remplir)*

### Completion Notes List

*(à remplir)*

### File List

*(à remplir)*

## Change Log

| Date | Étape | Résumé |
|---|---|---|
| 2026-10-25 | création | Fiche rédigée en autonomie (Opus 5.5) contre `c5287dec`, sur la demande de Guy du même jour (« ajoute le nécessaire dans les logs au cas où ») ; registre C-32-6c-1 à C-32-6c-10 ; epic annoté (§ 32-6c, après la 32-6) ; `sprint-status.yaml` : `32-6c-journal-filtre-adresse: ready-for-dev` ; prompt de validation P1 `326c-validate-prompt-p1.md` (Sonnet, lentilles R et F). Constat au relevé : C-32-6-4 et l'AC-b4 promettaient le refus « au journal du serveur », que le code n'écrit pas (fait 2). Décomptes (recomptés sur la fiche) : 17 AC (11 + 6 tests), 6 tâches (T-0 à T-5), 16 mutations (M1 à M16), 12 cas à l'AC-T1, 3 modules. |
| 2026-10-25 | validation P1 (remédiation) | Passe complète, lentilles R et F (Sonnet 5.5, contexte frais), rapports `kesh-gate-logs/326c-validate-p1-{R,F}.md` : R 0/0/4/6, F 0/0/1/4 ; **trend P1 : 0 CRITICAL / 0 HIGH / 5 MEDIUM / 10 LOW** (R2 et F2 sont un même constat, compté MEDIUM chez R et LOW chez F). Prémisse de chaque finding relue au code : **aucun finding réfuté**. Deux précisions : F2 cite `logging.rs:150` pour `with_ansi(false)`, la ligne est `:143` ; le tri par identifiants analysés que proposait R7 pour l'AC-T5 est remplacé par l'ordre contractuel de `list_enabled` (F4 : `ORDER BY company_id`, `mcp_server_settings.rs:103`). Remédiations (décisions de l'orchestrateur sur les MEDIUM) : **R1** fait 12 corrigé (`RUST\_LOG` décrit à `admin-manual.tex:792`, motif `grep -nE 'RUST\\?_LOG'`), l'AC-9 y renvoie et dit la précédence du terminal (`:1641`) ; **R2 = F2** fait 13 neuf — lecture dans le fichier de log (`logging.rs:143`, sans couleurs, actif par défaut), repli `sed` sur la sortie standard, ligne d'exemple prise du fichier, format global inchangé (C-32-6c-11) ; constat annexe : le fichier s'appelle `kesh.<AAAA-MM-JJ>.log` (`tracing-appender-0.2.5`), `admin-manual.tex:957` et `.env.example:289` disent `kesh.log` — défaut préexistant, hors périmètre, issue demandée (C-32-6c-13) ; **R3** fait 2 complété (`transport.rs:87`, `services/serveur_mcp.rs:41`, `transport.rs:78-80`), AC-7 et AC-9 (viii) reformulés ; **R4** réserve sur le mandataire déclaré écrite à l'AC-9 (ii), Dev Notes 32-7 annotées ; **F1** cas 13 et 14, mutations M17 et M18, plus M19 pour le cas 14 (C-32-6c-12). LOW intégrés : R5 (message en tête, AC-1, AC-T1, fait 9), R6 (AC-9 (iv) « filtre activé », cas (ix)), R7 (M12 reformulée, M20 pour l'AC-T3, M21 pour l'AC-T5), R8 (AC-T4 par `envoyer`/`requete`, `Origin` sur la seule requête refusée), R9 (fait 9 : `record_i64` `:291`, `record_u64` `:296`, `record_bool` `:311` ; `HeaderValue` lu en `http-1.4.2`), R10 (couvert par l'AC-9 : `up -d` comme `:621`, précédence du terminal), F3 (point 1 complété, AC-9), F4 (AC-1 « un seul jeu de champs » ; AC-T5 : l'ordre est un contrat), F5 (AC-T5 par `societe` ×2). Réponses de Guy du 2026-10-25 inscrites (§ *Points soumis à Guy* : 1 oui, 2 oui, 3 oui, 4 non, 5 oui) ; la description détaillée du fonctionnement de MCP inscrite à l'epic, § 32-11, et au § *Réponses de Guy (2026-10-25)*. Registre : C-32-6c-11 à 13 neufs, C-3 à C-7 et C-9 annotés. Symptômes grepés (`RUST_LOG`, `with_ansi`, `aucune ligne`, `douze`, `16 mutations`) : un résidu corrigé (Dev Notes, § *Le manuel* : `grep "RUST_LOG"` → `RUST\\?_LOG`). Décomptes recomptés à la source (`grep -c` sur la fiche et le registre) : 17 AC (11 + 6 tests), 6 tâches, **21 mutations** (M1 à M21), **14 cas** à l'AC-T1, **13 faits**, **13 choix** (C-32-6c-1 à 13), 3 modules. Passe suivante : P2, lentilles R et F, Opus, `326c-validate-prompt-p2.md`. |
| 2026-10-25 | validation P2 (remédiation) | Passe complète, lentilles R et F (Opus 5.5, contexte frais), rapports `kesh-gate-logs/326c-validate-p2-{R,F}.md` : R 0/0/2/5, F 0/0/3/7 ; **trend : P1 0/0/5/10 → P2 0/0/5/12** (sommes brutes des deux lentilles ; recouvrements : F2 = R2, F3 ⊃ R1, F1 ≈ R4 ≈ F7, F8 ⊃ R3 + R5, F4 = R6 (a)). ⚠️ **Signal D5 pour le Project Lead : recyclage.** Les dix-sept findings **naissent tous de la remédiation P1** (les deux lentilles le déclarent, aucun défaut d'origine neuf) et portent tous sur ce que la fiche fait écrire au manuel ; la sévérité ne décroît pas (5 MEDIUM → 5 MEDIUM). Ce n'est pas l'exception D5 (défauts distincts et d'origine) : c'est le correctif précédent qui produit le défaut suivant. Consigne de l'orchestrateur appliquée : **retirer plutôt qu'enrichir** — une phrase du manuel non éprouvée sur le NAS se retire ou se marque « non éprouvé ». Prémisses relues au code (`transport.rs:77-97`, `:100`, `:115`, `services/serveur_mcp.rs:31-47`, `mcp_server_settings.rs:96-107`, `lib.rs:1278`, `mcp_transport_e2e.rs:405-418`, `reseau.rs:76-78`, `:104-116`, `.env.example:250-253`, `docker-compose.yml:138-140`, `docker-compose.prod.yml:163-165`, `admin-manual.tex:572-576`, `:955-958`, `tracing-subscriber-0.3.23/src/fmt/fmt_layer.rs:741-743`, `tracing-appender-0.2.5/src/rolling.rs:636-650`) : **aucun finding réfuté**. Remédiations (décisions de l'orchestrateur) : **F1 + R4 + F7** AC-9 : remplacer la ligne `RUST_LOG=info` du `.env` par `RUST_LOG=info,kesh_api::mcp::reseau=debug`, `sudo docker compose up -d`, vérifier par les lignes `DEBUG` au journal ; la vérification `env \| grep` et la précédence du terminal sous `sudo` sont **retirées** ; fait 12 précisé (`:1641` vaut pour une machine de développement, non transposé) ; C-32-6c-7 annoté. **F2 = R2** AC-9 (i) ne cite plus que la publication de port en espace utilisateur de Docker ; Traefik non déclaré passe en exemple du cas (ii), marqué « non éprouvé » (fiche 32-6, `:1017-1019`). **F3 + R1** fait 2 (a) corrigé (`transport.rs:87` = erreur de la base, non lignes illisibles) et complété du chemin amont (`lib.rs:1278`, chemins voisins `/mcp/`, `/MCP` au repli statique) ; AC-9 (viii) réécrit en **énumération des sorties de Kesh**, la règle qui fait foi écrite (les trois `RouteAbsente` de `transport.rs:79`, `:88`, `:96` et la route exacte de `lib.rs:1278`), les causes extérieures déclarées hors énumération, et la condition de niveau (`debug`) ; « (pour la société attendue) » remplacé par « non activé pour cette société » ; AC-7 renvoie à (viii) au lieu d'en tenir une copie. **F8 + R5 + R3** une seule commande (`sudo grep "filtre d'adresse" ./log/kesh.*.log`), ses limites en une phrase (rotation `never`, `KESH_LOG_HOST_DIR`, motif développé avant `sudo`) ; le **repli `sed` est retiré** de l'AC-9, de T-3 et de § *Ce que les gardes ne voient pas* ; fait 13 précisé (montage par compose, nom sous rotation `never`) ; C-32-6c-11 annoté. LOW intégrés : F4 = R6 (a) (fait 13 et annotation C-11 : `fmt_layer.rs:743`), R6 (b) (AC-T5 : `serveur_mcp.rs:31`), F5 (M19 sans `split_terminator`), F6 (AC-T4 : `POST` et `Content-Type: application/json`), F9 (point 1 : le complément P1 déplacé **après** la réponse de Guy, comme ajout de l'orchestrateur), F10 (alternative `with_ansi(false)` notée à C-32-6c-11, à soumettre à Guy si le fichier ne suffit pas), R7 (réserve (ii) : tout poste qui joint le port publié, et le cas sans en-tête vu comme la passerelle, `reseau.rs:76-78`). Symptômes grepés par valeur (`env \| grep`, `Traefik`, `sed `, `kesh.*.log`, `pour la société attendue`, `toutes les sociétés`, `1641`, `split_terminator`, `305-312`, `serveur_mcp.rs:30`) sur la fiche, le registre et l'epic : restent seulement des occurrences légitimes — entrées passées du registre (C-7 et C-11 *Retenu*, annotés, non réécrits), transcription de la réponse de Guy à l'epic (`:627`), contexte de l'epic (`:34`, `:183`, `:412`), Dev Notes (« ne prescrit rien pour Traefik », toujours vrai). Décomptes recomptés à la source (`grep -c`) : 17 AC (11 + 6 tests), 6 tâches, 21 mutations, 14 cas, 13 faits, 13 choix, 3 modules — inchangés. Passe suivante : P3, lentilles R et F, Sonnet, `326c-validate-prompt-p3.md`, avec un axe obligatoire sur les régressions de cette remédiation. |
